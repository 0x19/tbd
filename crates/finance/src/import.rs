//! Importing transactions the provider already gave us.
//!
//! The prototype (`prototype/bank/`) pulled real Enable Banking responses and
//! saved them verbatim. This reads those files rather than calling the API,
//! which matters for three reasons: the ASPSP allows only a handful of fetches
//! a day, a re-import must be free, and the schema should be proven against
//! real data before any sync worker depends on it.
//!
//! The same path is what the sync worker will use later, so whatever is right
//! here — the dedup key, the field mapping, the sign convention — is right
//! there too.

use std::{collections::HashMap, path::Path};

use serde_json::Value;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use tbd_db::{DbError, map_err};
use uuid::Uuid;

/// What one import did.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Imported {
    /// Accounts created or already present.
    pub accounts: usize,
    /// Balance snapshots recorded.
    pub balances: usize,
    /// Rows inserted.
    pub inserted: usize,
    /// Rows already present, recognised by their dedup key.
    pub duplicates: usize,
    /// Rows skipped because they could not be mapped, with the reason logged.
    pub skipped: usize,
}

/// The dedup key for one transaction.
///
/// `entry_reference` when the provider gives one. Measured on Erste: present
/// and unique on all 2,657 rows pulled (`prototype/bank/FINDINGS.md`), so for
/// that bank this is the whole story.
///
/// Otherwise a content hash over length-prefixed, labelled fields, so no two
/// different tuples can produce the same bytes by running together. The label
/// and the length both go in; concatenating raw values would let
/// `("ab", "c")` and `("a", "bc")` collide.
///
/// `ordinal` distinguishes genuinely identical rows: two £4.50 coffees on one
/// day are two transactions, not one seen twice.
#[must_use]
pub fn dedup_key(entry_reference: Option<&str>, fields: &[(&str, &str)], ordinal: u32) -> Vec<u8> {
    if let Some(reference) = entry_reference.filter(|r| !r.trim().is_empty()) {
        let mut hasher = Sha256::new();
        hasher.update(b"entry_reference");
        hasher.update(
            u32::try_from(reference.len())
                .unwrap_or(u32::MAX)
                .to_le_bytes(),
        );
        hasher.update(reference.as_bytes());
        return hasher.finalize().to_vec();
    }
    let mut hasher = Sha256::new();
    for (label, value) in fields {
        let normalised = normalise(value);
        hasher.update(label.as_bytes());
        hasher.update(u32::try_from(label.len()).unwrap_or(u32::MAX).to_le_bytes());
        hasher.update(normalised.as_bytes());
        hasher.update(
            u32::try_from(normalised.len())
                .unwrap_or(u32::MAX)
                .to_le_bytes(),
        );
    }
    hasher.update(b"ordinal");
    hasher.update(ordinal.to_le_bytes());
    hasher.finalize().to_vec()
}

/// Uppercase, collapse whitespace, drop everything else.
///
/// Must stay stable across a provider's formatting drift: a bank that starts
/// trimming trailing spaces, or switches to title case, must not make every
/// historical row look new and re-import the lot.
#[must_use]
pub fn normalise(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut space = false;
    for c in value.chars() {
        if c.is_ascii_alphanumeric() {
            if space && !out.is_empty() {
                out.push(' ');
            }
            space = false;
            out.push(c.to_ascii_uppercase());
        } else {
            space = true;
        }
    }
    out
}

/// One account as the session recorded it.
struct SeenAccount {
    uid: String,
    iban: Option<String>,
    currency: String,
    name: String,
}

fn accounts_in(session: &Value) -> Vec<SeenAccount> {
    session
        .get("accounts")
        .and_then(Value::as_array)
        .map(|rows| {
            rows.iter()
                .filter_map(|a| {
                    Some(SeenAccount {
                        uid: a.get("uid")?.as_str()?.to_owned(),
                        iban: a
                            .get("account_id")
                            .and_then(|i| i.get("iban"))
                            .and_then(Value::as_str)
                            .map(str::to_owned),
                        currency: a.get("currency")?.as_str()?.to_owned(),
                        name: a
                            .get("name")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_owned(),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

/// A provider transaction, mapped onto the columns the schema wants.
struct Row {
    amount_minor: i64,
    currency: String,
    credit_debit: String,
    booking: Option<String>,
    value: Option<String>,
    counterparty: Option<String>,
    counterparty_iban: Option<String>,
    raw_amount: String,
}

/// Read a provider transaction into [`Row`].
///
/// `own_iban` is the account being read. It decides which side is the
/// counterparty, and that is not a detail: measured on 2,912 real Erste rows,
/// the obvious rule -- creditor on a debit, debtor on a credit -- records *our
/// own* IBAN as the counterparty on 120 of them. Those 120 are transfers
/// between accounts we hold, and counting them as income or spend is the same
/// euros twice.
///
/// Returns `None` when the row cannot be mapped at all; the caller counts it
/// as skipped rather than guessing. Nothing here invents a value: a missing
/// amount is a skip, not a zero.
fn row(value: &Value, own_iban: Option<&str>) -> Option<Row> {
    let amount = value.get("transaction_amount")?;
    let raw = amount.get("amount")?.as_str()?;
    let currency = amount.get("currency")?.as_str()?.to_owned();

    // Amounts arrive as strings and are always positive; the direction is
    // carried separately. Parsed as minor units without going through a float,
    // because a float cannot hold every decimal exactly and money must.
    let minor = minor_units(raw)?;
    let credit_debit = value
        .get("credit_debit_indicator")
        .and_then(Value::as_str)
        .unwrap_or("DBIT")
        .to_owned();
    let signed = if credit_debit == "DBIT" {
        -minor
    } else {
        minor
    };

    let booking = value
        .get("booking_date")
        .and_then(Value::as_str)
        .map(str::to_owned);
    // 164 of 2,912 rows carry a value date that differs from the booking date.
    // It is what interest and FX settle on, so losing it loses reconciliation.
    let value_date = value
        .get("value_date")
        .and_then(Value::as_str)
        .map(str::to_owned);

    // The counterparty is whichever side is not us. Falling back to the
    // direction only when our own IBAN is unknown.
    let side_iban = |side: &str| {
        value
            .get(side)
            .and_then(|a| a.get("iban"))
            .and_then(Value::as_str)
            .map(str::to_owned)
    };
    let debtor_iban = side_iban("debtor_account");
    let creditor_iban = side_iban("creditor_account");
    let debtor_is_us = own_iban.is_some_and(|own| debtor_iban.as_deref() == Some(own));
    let creditor_is_us = own_iban.is_some_and(|own| creditor_iban.as_deref() == Some(own));

    let take_creditor = if debtor_is_us && !creditor_is_us {
        true
    } else if creditor_is_us && !debtor_is_us {
        false
    } else {
        // Both ours (an internal transfer) or neither identifiable: fall back
        // to the direction, which at least names the other party consistently.
        credit_debit == "DBIT"
    };

    let party = if take_creditor {
        value.get("creditor")
    } else {
        value.get("debtor")
    };
    let counterparty = party
        .and_then(|p| p.get("name"))
        .and_then(Value::as_str)
        .map(str::to_owned);
    let counterparty_iban = if take_creditor {
        creditor_iban
    } else {
        debtor_iban
    };

    Some(Row {
        amount_minor: signed,
        currency,
        credit_debit,
        booking,
        value: value_date,
        counterparty,
        counterparty_iban,
        raw_amount: raw.to_owned(),
    })
}

/// `"6.22"` -> `622`. No float anywhere on the path.
#[must_use]
pub fn minor_units(raw: &str) -> Option<i64> {
    let (whole, frac) = raw.split_once('.').unwrap_or((raw, ""));
    let whole: i64 = whole.parse().ok()?;
    let frac = format!("{frac:0<2}");
    let cents: i64 = frac.get(..2)?.parse().ok()?;
    let magnitude = whole.abs().checked_mul(100)?.checked_add(cents)?;
    Some(if raw.starts_with('-') {
        -magnitude
    } else {
        magnitude
    })
}

/// `remittance_information` is an array of strings on Erste; joined for the
/// single column, because what reconciliation reads is the text as a whole.
fn remittance(value: &Value) -> Option<String> {
    let parts = value.get("remittance_information")?.as_array()?;
    let joined = parts
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>()
        .join(" | ");
    (!joined.is_empty()).then_some(joined)
}

/// Import one prototype directory into a party's accounts.
///
/// Idempotent: a second run inserts nothing, because the dedup key is a unique
/// index. That is the property worth testing, and the reason to run it twice
/// before trusting it.
///
/// # Errors
/// The directory cannot be read, or the database rejects a row.
pub async fn from_prototype(
    pool: &PgPool,
    dir: &Path,
    profile: &str,
    party_id: Uuid,
) -> Result<Imported, DbError> {
    let mut report = Imported::default();
    let session = read_json(&dir.join("sessions").join(format!("{profile}.json")))?;

    // uid -> our account id, so a transaction file can find its account.
    let mut accounts: HashMap<String, Uuid> = HashMap::new();
    for seen in accounts_in(&session) {
        accounts.insert(
            seen.uid.clone(),
            upsert_account(pool, party_id, &seen).await?,
        );
        report.accounts += 1;
    }

    let seen_accounts = accounts_in(&session);
    // The file index is the order the *pull* iterated, which is the live
    // session's order -- and that is not the order the link-time session
    // recorded. Using the wrong one silently files every transaction against
    // the wrong account: on the real data it put 255 EUR rows on a USD
    // account. Prefer the live session when it was saved.
    let uids = pull_order(dir, profile, &seen_accounts);
    let by_uid: HashMap<&str, &SeenAccount> =
        seen_accounts.iter().map(|a| (a.uid.as_str(), a)).collect();
    let mut seen_keys: HashMap<Vec<u8>, u32> = HashMap::new();

    // Balances first: a snapshot is what a later reconciliation checks the
    // transactions against, and recording it is free.
    for (index, uid) in uids.iter().enumerate() {
        let Some(&account_id) = accounts.get(uid) else {
            continue;
        };
        report.balances += import_balances(pool, dir, profile, index, account_id).await?;
    }

    for path in transaction_files(&dir.join("raw"), profile)? {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        let index: usize = name
            .trim_start_matches(&format!("transactions-{profile}-"))
            .split('-')
            .next()
            .and_then(|i| i.parse().ok())
            .unwrap_or(usize::MAX);
        let Some(uid) = uids.get(index) else {
            continue;
        };
        let Some(&account_id) = accounts.get(uid) else {
            continue;
        };
        let Some(account) = by_uid.get(uid.as_str()) else {
            continue;
        };
        let own_iban = account.iban.as_deref();

        let body = read_json(&path)?;
        let Some(rows) = body.get("transactions").and_then(Value::as_array) else {
            continue;
        };

        // The check that would have caught the ordering bug on the first run.
        // An account holds one currency; a file whose rows disagree with it is
        // filed against the wrong account, and importing it anyway would be
        // worse than importing nothing.
        if let Some(found) = rows
            .iter()
            .filter_map(|t| t.get("transaction_amount")?.get("currency")?.as_str())
            .find(|c| *c != account.currency)
        {
            return Err(DbError::Invalid {
                field: "currency",
                reason: format!(
                    "{}: rows are {found} but account {} ({}) is {} -- the file is \
                     attributed to the wrong account",
                    path.display(),
                    account.uid,
                    account.iban.as_deref().unwrap_or("no iban"),
                    account.currency
                ),
            });
        }

        for t in rows {
            let Some(parsed) = row(t, own_iban) else {
                report.skipped += 1;
                continue;
            };
            let key = next_key(t, &parsed, &mut seen_keys);
            let affected = insert_row(pool, party_id, account_id, t, &parsed, &key).await?;
            if affected == 0 {
                report.duplicates += 1;
            } else {
                report.inserted += 1;
            }
        }
    }

    Ok(report)
}

/// Record the balance snapshot the pull saved for one account.
///
/// Missing files are not an error: an account with no balance file simply has
/// no snapshot, and saying so is better than inventing a zero.
async fn import_balances(
    pool: &PgPool,
    dir: &Path,
    profile: &str,
    index: usize,
    account_id: Uuid,
) -> Result<usize, DbError> {
    let raw = dir.join("raw");
    let Some(path) = std::fs::read_dir(&raw).ok().and_then(|entries| {
        let prefix = format!("balances-{profile}-{index}");
        entries.filter_map(Result::ok).map(|e| e.path()).find(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with(&prefix))
        })
    }) else {
        return Ok(0);
    };

    let body = read_json(&path)?;
    let Some(rows) = body.get("balances").and_then(Value::as_array) else {
        return Ok(0);
    };

    let mut recorded = 0;
    for b in rows {
        let Some(amount) = b.get("balance_amount") else {
            continue;
        };
        let (Some(raw_amount), Some(currency)) = (
            amount.get("amount").and_then(Value::as_str),
            amount.get("currency").and_then(Value::as_str),
        ) else {
            continue;
        };
        let Some(minor) = minor_units(raw_amount) else {
            continue;
        };
        let balance_type = b
            .get("balance_type")
            .and_then(Value::as_str)
            .unwrap_or("UNKNOWN");
        sqlx::query(
            "insert into finance.balances
                (id, account_id, balance_type, amount_minor, currency, raw)
             values ($1, $2, $3, $4, $5, $6)
             on conflict (account_id, balance_type, observed_at) do nothing",
        )
        .bind(Uuid::new_v4())
        .bind(account_id)
        .bind(balance_type)
        .bind(minor)
        .bind(currency)
        .bind(b)
        .execute(pool)
        .await
        .map_err(map_err)?;
        recorded += 1;
    }
    Ok(recorded)
}

/// The account uids in the order the pull iterated them.
///
/// `POST /sessions` and `GET /sessions/{id}` return the same accounts in
/// *different* orders, and the pull used the live one. Where its response was
/// saved, that is the authority; otherwise fall back to the link-time order and
/// let the currency check below catch a mismatch.
fn pull_order(dir: &Path, profile: &str, link_time: &[SeenAccount]) -> Vec<String> {
    let live = dir.join("raw").join(format!("session-live-{profile}.json"));
    if let Ok(value) = read_json(&live)
        && let Some(uids) = value.get("accounts").and_then(Value::as_array)
    {
        let ordered: Vec<String> = uids
            .iter()
            .filter_map(|u| u.as_str().map(str::to_owned))
            .collect();
        if !ordered.is_empty() {
            return ordered;
        }
    }
    link_time.iter().map(|a| a.uid.clone()).collect()
}

/// The dedup key for this row, with the ordinal that keeps a genuine repeat
/// from collapsing into the row before it.
fn next_key(raw: &Value, parsed: &Row, seen: &mut HashMap<Vec<u8>, u32>) -> Vec<u8> {
    let entry_reference = raw.get("entry_reference").and_then(Value::as_str);
    let remittance = remittance(raw).unwrap_or_default();
    let fields = [
        ("booking", parsed.booking.as_deref().unwrap_or_default()),
        ("amount", parsed.raw_amount.as_str()),
        ("currency", parsed.currency.as_str()),
        ("credit_debit", parsed.credit_debit.as_str()),
        (
            "counterparty",
            parsed.counterparty.as_deref().unwrap_or_default(),
        ),
        ("remittance", remittance.as_str()),
    ];
    let base = dedup_key(entry_reference, &fields, 0);
    let ordinal = seen.entry(base.clone()).or_insert(0);
    let key = if *ordinal == 0 {
        base
    } else {
        dedup_key(entry_reference, &fields, *ordinal)
    };
    *ordinal += 1;
    key
}

/// Insert one row, or recognise it as already present.
async fn insert_row(
    pool: &PgPool,
    party_id: Uuid,
    account_id: Uuid,
    raw: &Value,
    parsed: &Row,
    key: &[u8],
) -> Result<u64, DbError> {
    let status = match raw.get("status").and_then(Value::as_str) {
        Some("BOOK") => "booked",
        _ => "pending",
    };
    Ok(sqlx::query(
        "insert into finance.bank_transactions
            (id, party_id, account_id, status, entry_reference, dedup_key,
             amount_minor, currency, scale, credit_debit, booking_date, value_date,
             counterparty_name, counterparty_iban, remittance, reference_number, raw)
         values ($1, $2, $3, $4, $5, $6, $7, $8, 2, $9, $10::date, $11::date,
                 $12, $13, $14, $15, $16)
         on conflict (account_id, dedup_key) do nothing",
    )
    .bind(Uuid::new_v4())
    .bind(party_id)
    .bind(account_id)
    .bind(status)
    .bind(raw.get("entry_reference").and_then(Value::as_str))
    .bind(key)
    .bind(parsed.amount_minor)
    .bind(&parsed.currency)
    .bind(&parsed.credit_debit)
    .bind(parsed.booking.as_deref())
    .bind(parsed.value.as_deref())
    .bind(parsed.counterparty.as_deref())
    .bind(parsed.counterparty_iban.as_deref())
    .bind(remittance(raw).as_deref())
    .bind(raw.get("reference_number").and_then(Value::as_str))
    .bind(raw)
    .execute(pool)
    .await
    .map_err(map_err)?
    .rows_affected())
}

/// Read and parse a JSON file, naming it when either fails.
fn read_json(path: &Path) -> Result<Value, DbError> {
    let bytes = std::fs::read(path).map_err(|e| DbError::Invalid {
        field: "file",
        reason: format!("{}: {e}", path.display()),
    })?;
    serde_json::from_slice(&bytes).map_err(|e| DbError::Invalid {
        field: "file",
        reason: format!("{}: {e}", path.display()),
    })
}

/// Create the account, or find the one already there.
///
/// Keyed on `(party, iban, currency)`, because one IBAN is several accounts:
/// Erste exposes the company's in EUR, GBP, USD and HRK.
async fn upsert_account(
    pool: &PgPool,
    party_id: Uuid,
    seen: &SeenAccount,
) -> Result<Uuid, DbError> {
    let id = Uuid::new_v4();
    let existing: Option<(Uuid,)> = sqlx::query_as(
        "insert into finance.accounts (id, party_id, provider, provider_uid, iban, currency, name)
         values ($1, $2, 'enablebanking', $3, $4, $5, $6)
         on conflict (party_id, iban, currency) where iban is not null
         do update set name = excluded.name
         returning id",
    )
    .bind(id)
    .bind(party_id)
    .bind(&seen.uid)
    .bind(seen.iban.as_deref())
    .bind(&seen.currency)
    .bind(&seen.name)
    .fetch_optional(pool)
    .await
    .map_err(map_err)?;
    Ok(existing.map_or(id, |(found,)| found))
}

/// Files named `transactions-<profile>-<index>[-<ccy>]-p<n>.json`, in order.
fn transaction_files(raw: &Path, profile: &str) -> Result<Vec<std::path::PathBuf>, DbError> {
    let prefix = format!("transactions-{profile}-");
    let mut files: Vec<_> = std::fs::read_dir(raw)
        .map_err(|e| DbError::Invalid {
            field: "raw",
            reason: format!("{}: {e}", raw.display()),
        })?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with(&prefix))
        })
        .collect();
    files.sort();
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::{dedup_key, minor_units, normalise};

    #[test]
    fn amounts_never_go_through_a_float() {
        assert_eq!(minor_units("6.22"), Some(622));
        assert_eq!(minor_units("14500.82"), Some(1_450_082));
        assert_eq!(minor_units("0.01"), Some(1));
        assert_eq!(minor_units("100"), Some(10_000));
        assert_eq!(minor_units("-4.50"), Some(-450));
        // The classic float failure: 0.1 + 0.2 never appears because no float
        // is ever constructed.
        assert_eq!(minor_units("0.10"), Some(10));
        assert_eq!(minor_units("0.20"), Some(20));
        assert_eq!(minor_units("not a number"), None);
    }

    #[test]
    fn normalisation_survives_a_providers_formatting_drift() {
        // The same counterparty, spelled four ways, must normalise alike --
        // otherwise a bank that changes its casing re-imports every row.
        for spelling in [
            "TENDERLY D.O.O.",
            "Tenderly d.o.o.",
            "  tenderly   D.O.O.  ",
            "Tenderly, d.o.o.",
        ] {
            assert_eq!(normalise(spelling), "TENDERLY D O O", "{spelling}");
        }
    }

    #[test]
    fn an_entry_reference_is_the_key_when_there_is_one() {
        let fields = [("amount", "1.00")];
        let a = dedup_key(Some("406G50610R916031D"), &fields, 0);
        let b = dedup_key(Some("406G50610R916031D"), &[("amount", "999.00")], 0);
        assert_eq!(a, b, "the reference alone decides when present");

        let other = dedup_key(Some("DIFFERENT"), &fields, 0);
        assert_ne!(a, other);
    }

    #[test]
    fn fields_cannot_run_together_into_the_same_key() {
        // Without length prefixes these two would hash identically.
        let a = dedup_key(None, &[("x", "ab"), ("y", "c")], 0);
        let b = dedup_key(None, &[("x", "a"), ("y", "bc")], 0);
        assert_ne!(a, b, "length prefixes are what keep these apart");
    }

    #[test]
    fn two_identical_transactions_on_one_day_stay_two() {
        let fields = [("amount", "4.50"), ("counterparty", "A CAFE")];
        assert_ne!(
            dedup_key(None, &fields, 0),
            dedup_key(None, &fields, 1),
            "the ordinal is what keeps a genuine repeat from collapsing"
        );
    }

    #[test]
    fn a_blank_entry_reference_falls_through_to_the_content_hash() {
        let fields = [("amount", "1.00")];
        assert_eq!(
            dedup_key(Some("   "), &fields, 0),
            dedup_key(None, &fields, 0)
        );
    }
}
