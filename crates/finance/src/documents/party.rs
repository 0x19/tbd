//! Whose a document is.
//!
//! A mailbox belongs to a party, but what arrives in it does not: the
//! company's address gets the owner's motorcycle parts, the personal one
//! gets the company's Anthropic invoices. So the party is decided from
//! evidence, in order, and the decision is recorded beside the fields:
//!
//! 1. `declared`: a person said so, through `UpdateDocument`. Final.
//! 2. `payment`: exactly one candidate party has a debit of the document's
//!    amount within a few days of its date. The account that paid is the
//!    party that bought.
//! 3. `text`: the document names a candidate. An organisation's name or tax
//!    id anywhere in it wins over a person's, because an invoice to the
//!    company carries the contact person's name as well.
//! 4. `mailbox`: nothing better; the connector's party stands.
//!
//! The candidates are the parties the mailbox's owner may see, so a document
//! never moves to a party nobody who linked the mailbox can reach.

use chrono::Days;
use serde_json::Value;
use sqlx::PgPool;
use tbd_db::map_err;
use uuid::Uuid;

use crate::connectors::store::StoreError;

/// How the party was decided.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum By {
    /// A person set it.
    Declared,
    /// The account that paid it.
    Payment,
    /// The document names the party.
    Text,
    /// The mailbox it arrived in.
    Mailbox,
}

impl By {
    /// The wire word, the same place the fields put theirs.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Declared => "declared",
            Self::Payment => "payment",
            Self::Text => "text",
            Self::Mailbox => "mailbox",
        }
    }
}

/// The party a document belongs to, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decision {
    /// The party.
    pub party_id: Uuid,
    /// The evidence.
    pub by: By,
}

/// A party the document could belong to, with what its documents say.
#[derive(Debug, sqlx::FromRow)]
struct Candidate {
    id: Uuid,
    kind: String,
    display_name: String,
    legal_name: Option<String>,
    oib: Option<String>,
}

/// What the decision reads off the row.
#[derive(Debug, sqlx::FromRow)]
struct Stored {
    party_id: Uuid,
    total_minor: Option<i64>,
    currency: Option<String>,
    doc_date: Option<chrono::NaiveDate>,
    text: Option<String>,
    extracted: Value,
}

/// Days before the document's date a payment may fall (a card charge is
/// booked before the receipt is dated) and after it (a transfer on the due
/// date).
const BEFORE: u64 = 3;
const AFTER: u64 = 10;

/// Decide the party for `id` from what is stored. `None` when a person has
/// declared it, when the document does not exist, or when there is only
/// one party it could belong to.
///
/// # Errors
/// The database.
pub async fn decide(pool: &PgPool, id: Uuid) -> Result<Option<Decision>, StoreError> {
    let row = sqlx::query_as::<_, Stored>(
        "select party_id, total_minor, currency, doc_date, text, extracted
           from finance.documents where id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(map_err)?;
    let Some(Stored {
        party_id: current,
        total_minor: total,
        currency,
        doc_date: date,
        text,
        extracted,
    }) = row
    else {
        return Ok(None);
    };
    if extracted.get("party").and_then(Value::as_str) == Some("declared") {
        return Ok(None);
    }
    let candidates = candidates(pool, current).await?;
    if candidates.len() < 2 {
        return Ok(None);
    }
    let ids: Vec<Uuid> = candidates.iter().map(|c| c.id).collect();

    if let (Some(total), Some(date)) = (total, date) {
        let paid: Vec<(Uuid,)> = sqlx::query_as(
            "select distinct party_id from finance.bank_transactions
              where party_id = any($1)
                and credit_debit = 'DBIT'
                and abs(amount_minor) = $2
                and ($3::text is null or currency = $3)
                and booking_date between $4 and $5",
        )
        .bind(&ids)
        .bind(total)
        .bind(currency.as_deref().filter(|c| *c == "EUR"))
        .bind(date.checked_sub_days(Days::new(BEFORE)).unwrap_or(date))
        .bind(date.checked_add_days(Days::new(AFTER)).unwrap_or(date))
        .fetch_all(pool)
        .await
        .map_err(map_err)?;
        if let [(party,)] = paid.as_slice() {
            return Ok(Some(Decision {
                party_id: *party,
                by: By::Payment,
            }));
        }
    }

    if let Some(text) = text.as_deref().filter(|t| !t.trim().is_empty())
        && let Some(party) = named_in(text, &candidates)
    {
        return Ok(Some(Decision {
            party_id: party,
            by: By::Text,
        }));
    }

    Ok(Some(Decision {
        party_id: current,
        by: By::Mailbox,
    }))
}

/// Decide and write: the party (when it changes and no copy of the same
/// bytes already sits there) and the record of how.
///
/// # Errors
/// The database.
pub async fn assign(pool: &PgPool, id: Uuid) -> Result<Option<Decision>, StoreError> {
    let Some(decision) = decide(pool, id).await? else {
        return Ok(None);
    };
    // The same receipt can already be a document of the target party (both
    // mailboxes got it); the unique key says so, and this copy stays put.
    let moved = sqlx::query(
        "update finance.documents d
            set party_id = $2,
                extracted = coalesce(d.extracted, '{}'::jsonb) || jsonb_build_object('party', $3::text)
          where d.id = $1
            and not exists (select 1 from finance.documents o
                             where o.party_id = $2 and o.sha256 = d.sha256 and o.id <> d.id)",
    )
    .bind(id)
    .bind(decision.party_id)
    .bind(decision.by.as_str())
    .execute(pool)
    .await
    .map_err(map_err)?
    .rows_affected();
    if moved == 0 {
        sqlx::query(
            "update finance.documents
                set extracted = coalesce(extracted, '{}'::jsonb) || jsonb_build_object('party', 'mailbox'::text)
              where id = $1",
        )
        .bind(id)
        .execute(pool)
        .await
        .map_err(map_err)?;
        return Ok(None);
    }
    Ok(Some(decision))
}

/// The parties an owner of `party` may see, `party` included.
async fn candidates(pool: &PgPool, party: Uuid) -> Result<Vec<Candidate>, StoreError> {
    Ok(sqlx::query_as::<_, Candidate>(
        "select distinct p.id, p.kind, p.display_name, o.legal_name, o.oib
           from public.party_access owner
           join public.party_access theirs on theirs.user_id = owner.user_id
           join public.parties p on p.id = theirs.party_id
           left join public.orgs o on o.id = p.id
          where owner.party_id = $1 and owner.capability = 'own'
            and (theirs.expires_at is null or theirs.expires_at > now())
          order by p.kind, p.display_name",
    )
    .bind(party)
    .fetch_all(pool)
    .await
    .map_err(map_err)?)
}

/// Upper-case ASCII words: Croatian letters folded to their base, anything
/// else a separator. `import::normalise` alone drops a `Ć` as if it were
/// punctuation, and `VESIĆ` would never equal `VESIC`.
fn fold(s: &str) -> String {
    let folded: String = s
        .chars()
        .map(|c| match c {
            'č' | 'ć' => 'c',
            'Č' | 'Ć' => 'C',
            'đ' => 'd',
            'Đ' => 'D',
            'š' => 's',
            'Š' => 'S',
            'ž' => 'z',
            'Ž' => 'Z',
            other => other,
        })
        .collect();
    crate::import::normalise(&folded)
}

/// The candidate the text names: an organisation first (by name or tax id),
/// then a person (by name, either word order).
fn named_in(text: &str, candidates: &[Candidate]) -> Option<Uuid> {
    let haystack = format!(" {} ", fold(text));
    let has = |needle: &str| {
        let n = fold(needle);
        !n.is_empty() && haystack.contains(&format!(" {n} "))
    };
    let orgs = candidates.iter().filter(|c| c.kind == "org");
    for c in orgs {
        if c.oib.as_deref().is_some_and(|o| !o.is_empty() && has(o)) {
            return Some(c.id);
        }
        let names = [c.legal_name.as_deref(), Some(c.display_name.as_str())];
        if names.into_iter().flatten().map(bare_name).any(|n| has(&n)) {
            return Some(c.id);
        }
    }
    let people = candidates.iter().filter(|c| c.kind != "org");
    for c in people {
        let name = fold(&c.display_name);
        let mut words: Vec<&str> = name.split(' ').filter(|w| !w.is_empty()).collect();
        if words.is_empty() {
            continue;
        }
        if has(&words.join(" ")) {
            return Some(c.id);
        }
        words.reverse();
        if has(&words.join(" ")) {
            return Some(c.id);
        }
    }
    None
}

/// `Inorbit d.o.o.` → `Inorbit`: the legal form is written a dozen ways and
/// sometimes not at all.
fn bare_name(name: &str) -> String {
    let n = fold(name);
    let words: Vec<&str> = n.split(' ').filter(|w| !w.is_empty()).collect();
    let mut end = words.len();
    while end > 1
        && matches!(
            words[end - 1],
            "D" | "O" | "J" | "DD" | "DOO" | "JDOO" | "OBRT"
        )
    {
        end -= 1;
    }
    words[..end].join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cand(kind: &str, display: &str, legal: Option<&str>, oib: Option<&str>) -> Candidate {
        Candidate {
            id: Uuid::new_v4(),
            kind: kind.into(),
            display_name: display.into(),
            legal_name: legal.map(str::to_owned),
            oib: oib.map(str::to_owned),
        }
    }

    #[test]
    fn the_company_wins_when_both_are_named_and_the_person_alone_otherwise() {
        let org = cand(
            "org",
            "Inorbit d.o.o.",
            Some("Inorbit d.o.o."),
            Some("38846238650"),
        );
        let me = cand("person", "Nevio Vesic", None, None);
        let cs = [org, me];
        assert_eq!(
            named_in("Bill to\nINORBIT d. o. o.\nAttn: Nevio Vesić", &cs),
            Some(cs[0].id)
        );
        assert_eq!(named_in("OIB kupca: 38846238650", &cs), Some(cs[0].id));
        assert_eq!(
            named_in("Kupac: VESIĆ NEVIO, Benčani 15A", &cs),
            Some(cs[1].id)
        );
        assert_eq!(named_in("Receipt for order #1", &cs), None);
        assert_eq!(
            named_in("Inorbital Systems Ltd", &cs),
            None,
            "a longer word is not the name"
        );
    }

    #[test]
    fn legal_forms_are_stripped() {
        assert_eq!(bare_name("Inorbit d.o.o."), "INORBIT");
        assert_eq!(bare_name("Spes Fiume j.d.o.o."), "SPES FIUME");
        assert_eq!(bare_name("Erste banka d.d."), "ERSTE BANKA");
        assert_eq!(bare_name("Tenderly"), "TENDERLY");
    }
}
