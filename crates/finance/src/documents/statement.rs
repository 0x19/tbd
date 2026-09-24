//! A bank statement read into bank transactions. Erste mails the company's
//! "IZVOD PROMETA PO RAČUNU" every month; before the bank feed existed, and
//! for whatever the feed's 90-day window never covered, the statement is
//! the record. Its entries become provider-shaped rows and go through the
//! one ingest path (`crate::import::ingest_pages`), so a statement row is a
//! bank row like any other: categorised, matched to invoices by the number
//! the payer wrote, shown on the accountant's month. An entry the feed
//! already holds (same day, same signed amount on the account) is skipped,
//! so a month covered by both is counted once.

use std::collections::HashMap;

use chrono::NaiveDate;
use regex::Regex;
use serde_json::{Value, json};
use sqlx::PgPool;
use tbd_db::map_err;
use uuid::Uuid;

use crate::{
    connectors::store::StoreError,
    import::{Imported, ProviderAccount, SeenKeys, ingest_pages},
};

/// One line of the statement.
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub value_date: NaiveDate,
    pub booking_date: NaiveDate,
    /// The other side, as the bank printed it.
    pub counterparty: String,
    pub counterparty_iban: Option<String>,
    /// "Opis plaćanja": what the payer wrote, the invoice number included.
    pub description: String,
    /// The recipient's reference model and number ("HR68 8486-...", or "HR99").
    pub reference_number: String,
    /// "Referenca plaćanja": the bank's id of the entry, unique per statement.
    pub payment_reference: String,
    /// Positive minor units; `credit` says the direction.
    pub amount_minor: i64,
    pub credit: bool,
}

/// The statement: whose account, which month, what moved.
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Statement {
    pub iban: String,
    pub currency: String,
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub entries: Vec<Entry>,
    /// Days whose entries could not be told apart as credits and debits
    /// from the day's totals; their entries are left out.
    pub unresolved_days: usize,
    /// Entries that no day's totals followed (a statement laid out without
    /// them, as the personal account's is): their direction is unknown, so
    /// they are left out and counted here.
    pub undirected: usize,
}

/// Whether the text is one of these statements at all.
#[must_use]
pub fn is_statement(text: &str) -> bool {
    text.trim_start().starts_with("IZVOD PROMETA PO RAČUNU")
        && text.contains("Oznaka valute:")
        && text.contains("IBAN:")
}

fn re(pattern: &str) -> Regex {
    Regex::new(pattern).unwrap_or_else(|_| unreachable!("a literal pattern"))
}

fn date(s: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(s.trim_end_matches('.'), "%d.%m.%Y").ok()
}

/// "1.234,56" as minor units.
fn minor(s: &str) -> Option<i64> {
    let (whole, frac) = s.rsplit_once(',')?;
    let whole: i64 = whole.replace('.', "").parse().ok()?;
    let frac: i64 = frac.parse().ok()?;
    (frac < 100).then_some(whole * 100 + frac)
}

/// The account and the period from the head of the statement.
fn header(text: &str) -> Option<(String, String, NaiveDate, NaiveDate)> {
    let iban = re(r"IBAN:\s*([A-Z]{2}\d{2}[A-Z0-9]{11,30})").captures(text)?[1].to_owned();
    let currency = re(r"Oznaka valute:\s*([A-Z]{3})").captures(text)?[1].to_owned();
    let period = re(r"Za razdoblje[^:]*:\s*(\d\d\.\d\d\.\d{4})\.?\s+do\s+(\d\d\.\d\d\.\d{4})")
        .captures(text)?;
    Some((iban, currency, date(&period[1])?, date(&period[2])?))
}

/// Parse the statement; `None` when it is not one.
#[must_use]
pub fn parse(text: &str) -> Option<Statement> {
    if !is_statement(text) {
        return None;
    }
    let (iban, currency, from, to) = header(text)?;
    let value_line = re(r"^\d\d\.\d\d\.\d{4}\.?$");
    let booking_line = re(r"^\d\d\.\d\d\.\d{4}\.?\s+\S");
    let day_totals = re(r"^S\s*t\s*a\s*n\s*j\s*e\s+([\d.]+,\d\d)\s+([\d.]+,\d\d)");
    let lines: Vec<&str> = text.lines().map(str::trim).collect();
    let mut entries = Vec::new();
    let mut day: Vec<Entry> = Vec::new();
    let mut unresolved_days = 0;
    let mut i = lines
        .iter()
        .position(|l| l.starts_with("Početno stanje"))
        .map_or(0, |p| p + 1);
    while i < lines.len() {
        let line = lines[i];
        if line.starts_with("Stranica") {
            // A page break repeats the column headings; skip to their end.
            i = lines[i..]
                .iter()
                .position(|l| l.starts_with("Referenca plaćanja"))
                .map_or(lines.len(), |p| i + p + 1);
            continue;
        }
        if line.starts_with("Stanje na dan") {
            if let Some(c) = lines.get(i + 1).and_then(|l| day_totals.captures(l)) {
                if resolve(
                    &mut day,
                    minor(&c[1]).unwrap_or(0),
                    minor(&c[2]).unwrap_or(0),
                ) {
                    entries.append(&mut day);
                } else {
                    unresolved_days += 1;
                    day.clear();
                }
            }
            i += 1;
            continue;
        }
        let starts_entry = |at: usize| {
            lines.get(at).is_some_and(|l| value_line.is_match(l))
                && lines.get(at + 1).is_some_and(|l| booking_line.is_match(l))
        };
        if starts_entry(i) {
            let start = i;
            i += 2;
            while i < lines.len()
                && !starts_entry(i)
                && !lines[i].starts_with("Stanje na dan")
                && !lines[i].starts_with("Stranica")
            {
                i += 1;
            }
            if let Some(entry) = entry(&lines[start..i]) {
                day.push(entry);
            }
            continue;
        }
        i += 1;
    }
    let undirected = day.len();
    Some(Statement {
        iban,
        currency,
        from,
        to,
        entries,
        unresolved_days,
        undirected,
    })
}

/// One entry from its lines: the value date, then the booking date with the
/// start of the counterparty, then the rest down to the amount.
fn entry(lines: &[&str]) -> Option<Entry> {
    let value_date = date(lines.first()?)?;
    let toks: Vec<&str> = lines[1..]
        .iter()
        .flat_map(|l| l.split_whitespace())
        .collect();
    let booking_date = date(toks.first()?)?;
    let amount_re = re(r"^\d{1,3}(?:\.\d{3})*,\d\d$");
    let amount_at = toks.iter().rposition(|t| amount_re.is_match(t))?;
    let amount_minor = minor(toks[amount_at])?;
    let payment_reference = toks.get(amount_at.checked_sub(1)?)?.to_string();
    let ordinal_re = re(r"^\d{1,4}$");
    let marker = (1..amount_at.saturating_sub(1))
        .find(|&j| ordinal_re.is_match(toks[j]) && toks.get(j + 1) == Some(&"-"))?;
    let iban_re = re(r"^[A-Z]{2}\d{2}[A-Z0-9]{11,30}$");
    let head = &toks[1..marker];
    let counterparty_iban = head
        .iter()
        .find(|t| iban_re.is_match(t))
        .map(|t| (*t).to_owned());
    let counterparty = head
        .iter()
        .filter(|t| !iban_re.is_match(t))
        .copied()
        .collect::<Vec<_>>()
        .join(" ");
    let tail = &toks[marker + 2..amount_at - 1];
    let model_re = re(r"^HR\d\d$");
    let code_re = re(r"^[A-Z]{4}$");
    let cut = (0..tail.len())
        .find(|&j| {
            model_re.is_match(tail[j])
                || (code_re.is_match(tail[j])
                    && tail.get(j + 1).is_some_and(|n| model_re.is_match(n)))
        })
        .unwrap_or(tail.len());
    let description = tail[..cut].join(" ");
    let mut groups: Vec<Vec<&str>> = Vec::new();
    for t in &tail[cut..] {
        if model_re.is_match(t) {
            groups.push(vec![t]);
        } else if let Some(g) = groups.last_mut() {
            g.push(t);
        }
    }
    let reference_number = groups.last().map(|g| g.join(" ")).unwrap_or_default();
    Some(Entry {
        value_date,
        booking_date,
        counterparty,
        counterparty_iban,
        description,
        reference_number,
        payment_reference,
        amount_minor,
        credit: false,
    })
}

/// Mark the day's entries as credits or debits from the day's two totals.
/// All one way is plain; a mixed day is the subset whose sum is the credit
/// total, found over the day's few entries. `false` when no subset fits.
fn resolve(day: &mut [Entry], debits: i64, credits: i64) -> bool {
    if credits == 0 {
        return true;
    }
    if debits == 0 {
        for e in day.iter_mut() {
            e.credit = true;
        }
        return true;
    }
    if day.len() > 60 {
        return false;
    }
    // Reachable sums with the entries that make them, one mask per sum.
    let mut reach: HashMap<i64, u64> = HashMap::from([(0, 0)]);
    for (k, e) in day.iter().enumerate() {
        let mut next = reach.clone();
        for (&sum, &mask) in &reach {
            next.entry(sum + e.amount_minor).or_insert(mask | (1 << k));
        }
        reach = next;
    }
    let Some(&mask) = reach.get(&credits) else {
        return false;
    };
    let debit_sum: i64 = day
        .iter()
        .enumerate()
        .filter(|(k, _)| mask & (1 << k) == 0)
        .map(|(_, e)| e.amount_minor)
        .sum();
    if debit_sum != debits {
        return false;
    }
    for (k, e) in day.iter_mut().enumerate() {
        e.credit = mask & (1 << k) != 0;
    }
    true
}

/// The entries as the provider would have sent them, so the one ingest path
/// applies: `entry_reference` is the bank's own payment reference, prefixed
/// so it can never collide with a feed id.
#[must_use]
pub fn provider_rows(st: &Statement) -> Vec<Value> {
    st.entries
        .iter()
        .map(|e| {
            let us = json!({ "iban": st.iban });
            let them = json!({ "iban": e.counterparty_iban });
            let (debtor, debtor_account, creditor, creditor_account) = if e.credit {
                (json!({ "name": e.counterparty }), them, Value::Null, us)
            } else {
                (Value::Null, us, json!({ "name": e.counterparty }), them)
            };
            let model = e
                .reference_number
                .split_whitespace()
                .next()
                .unwrap_or("HR99");
            json!({
                "entry_reference": format!("izvod:{}", e.payment_reference),
                "status": "BOOK",
                "booking_date": e.booking_date.to_string(),
                "value_date": e.value_date.to_string(),
                "transaction_amount": {
                    "amount": format!("{}.{:02}", e.amount_minor / 100, e.amount_minor % 100),
                    "currency": st.currency,
                },
                "credit_debit_indicator": if e.credit { "CRDT" } else { "DBIT" },
                "debtor": debtor,
                "debtor_account": debtor_account,
                "creditor": creditor,
                "creditor_account": creditor_account,
                "remittance_information": [model, e.description],
                "reference_number": e.reference_number,
                "source": "statement",
            })
        })
        .collect()
}

/// What ingesting a statement did.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Ingested {
    pub imported: Imported,
    /// Entries the feed already held for that day and amount.
    pub already_in_feed: usize,
    pub unresolved_days: usize,
    /// Entries left out because nothing said which way they went.
    pub undirected: usize,
}

/// Read the statement's entries into the account it names, when that
/// account is known here. `None` for a statement of an account we do not
/// keep, or text that is not a statement.
///
/// # Errors
/// The database.
pub async fn ingest(
    pool: &PgPool,
    text: &str,
    source: &str,
) -> Result<Option<Ingested>, StoreError> {
    let Some(st) = parse(text) else {
        return Ok(None);
    };
    let Some((account_id, party_id, name)) = sqlx::query_as::<_, (Uuid, Uuid, String)>(
        "select id, party_id, name from finance.accounts where iban = $1 and currency = $2 limit 1",
    )
    .bind(&st.iban)
    .bind(&st.currency)
    .fetch_optional(pool)
    .await
    .map_err(map_err)?
    else {
        return Ok(None);
    };
    // What the feed already holds on those days: one statement entry per
    // feed row of the same day and signed amount is the same money.
    let mut held: HashMap<(NaiveDate, i64), u32> = HashMap::new();
    for (day, amount) in sqlx::query_as::<_, (NaiveDate, i64)>(
        "select booking_date, amount_minor from finance.bank_transactions
          where account_id = $1 and booking_date between $2 and $3 and entry_reference not like 'izvod:%'",
    )
    .bind(account_id)
    .bind(st.from)
    .bind(st.to)
    .fetch_all(pool)
    .await
    .map_err(map_err)?
    {
        *held.entry((day, amount)).or_default() += 1;
    }
    let mut fresh = st.clone();
    let mut already_in_feed = 0;
    fresh.entries.retain(|e| {
        let signed = if e.credit {
            e.amount_minor
        } else {
            -e.amount_minor
        };
        match held.get_mut(&(e.booking_date, signed)) {
            Some(n) if *n > 0 => {
                *n -= 1;
                already_in_feed += 1;
                false
            }
            _ => true,
        }
    });
    let account = ProviderAccount {
        uid: format!("statement:{}", st.iban),
        iban: Some(st.iban.clone()),
        currency: st.currency.clone(),
        name,
    };
    let pages = [json!({ "transactions": provider_rows(&fresh) })];
    let imported = ingest_pages(
        pool,
        party_id,
        account_id,
        &account,
        &pages,
        &format!("statement {source}"),
        &mut SeenKeys::new(),
    )
    .await
    .map_err(StoreError::Db)?;
    Ok(Some(Ingested {
        imported,
        already_in_feed,
        unresolved_days: st.unresolved_days,
        undirected: st.undirected,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    const JULY: &str = "IZVOD PROMETA PO RAČUNU

Datum i vrijeme izdavanja: 01.08.2026. 04:39
Za razdoblje (po datumu obrade): 01.07.2026. do 31.07.2026.

ERSTE&STEIERMÄRKISCHE BANK D.D.
OIB: 23057039320
INORBIT d.o.o.

Naziv klijenta: INORBIT d.o.o.
OIB: 38846238650

IBAN: HR9224020061100925189
Broj računa: 1100925189

Oznaka valute: EUR
Broj izvoda: 107-126

Referenca plaćanja Isplata Uplata

Početno stanje: 9.803,42

01.07.2026.
01.07.2026. HRVATSKI ZAVOD ZA ZDRAVSTVENO
OSIGURANJE-OBVEZNO ZDRAV.
OSIGURANJE
HR6510010051550100001
1 - Placa za 6 / 2026
HLTI HR99
HR68 8486-38846238650-26182
2026-64860923-11315093333
246,37

01.07.2026.
01.07.2026. Vesic Nevio
HR3924020063202676456 5 - Placa za 6 / 2026
SALA HR67 38846238650-26182-0
HR69 40002-38846238650-100
2026-64860935-11315055691
1.075,60

Stanje na dan: 01.07.2026. Broj izvoda 107 Promet
S t a n j e 1.321,97 0,00
8.481,45

02.07.2026.
02.07.2026. Google Workspace_inorbit. Dublin 7 -
424472XXXXXX935
5,
Google
Workspace_inorbit.
Dublin, 01.07.2026
14:39
HR99
HR99
11321040710
32,40

Stanje na dan: 02.07.2026. Broj izvoda 108 Promet
S t a n j e 32,40 0,00
8.449,05

03.07.2026.
03.07.2026. TENDERLY D.O.O. BEOGRAD-NOVI
BEOGRAD 11070
RS35265100000024355587
8 - BROJ RACUNA
7-1-1-2026 HR99
HR99
3550061910-518173447
14.872,60

Stanje na dan: 03.07.2026. Broj izvoda 109 Promet
S t a n j e 0,00 14.872,60
23.321,65

Stranica 1/5

Datum valute
Datum obrade Platitelj/Primatelj
Referenca plaćanja Isplata Uplata

08.07.2026.
08.07.2026. VESIĆ NEVIO
HR3924020063202676456 12 - POZAJMICA
PO UGOVORU HR99
HR00 080726
2026-67610968-11342097182
2.500,00

08.07.2026.
08.07.2026. EIGER OY 00170 HELSINKI
FI7557169020123191
13 - INV-9-1-1-2026
HR99
HR99
2026-77121897-5664908732
9.167,00

08.07.2026.
08.07.2026. Revolut**8088* Dublin 11 -
424472XXXXXX935
5,
Revolut**8088*
Dublin, 07.07.2026
21:57
HR99
HR99
11343721367
101,34

Stanje na dan: 08.07.2026. Broj izvoda 111 Promet
S t a n j e 2.601,34 9.167,00
29.887,31
";

    #[test]
    fn a_statement_is_read_entry_by_entry_with_the_direction_from_the_days_totals() {
        assert!(is_statement(JULY));
        let st = parse(JULY).unwrap();
        assert_eq!(st.iban, "HR9224020061100925189");
        assert_eq!(st.currency, "EUR");
        assert_eq!(st.from.to_string(), "2026-07-01");
        assert_eq!(st.to.to_string(), "2026-07-31");
        assert_eq!(st.unresolved_days, 0);
        assert_eq!(st.entries.len(), 7, "{:#?}", st.entries);

        let health = &st.entries[0];
        assert_eq!(
            health.counterparty,
            "HRVATSKI ZAVOD ZA ZDRAVSTVENO OSIGURANJE-OBVEZNO ZDRAV. OSIGURANJE"
        );
        assert_eq!(
            health.counterparty_iban.as_deref(),
            Some("HR6510010051550100001")
        );
        assert_eq!(health.description, "Placa za 6 / 2026");
        assert_eq!(health.reference_number, "HR68 8486-38846238650-26182");
        assert_eq!(health.payment_reference, "2026-64860923-11315093333");
        assert_eq!((health.amount_minor, health.credit), (24637, false));

        let salary = &st.entries[1];
        assert_eq!(salary.counterparty, "Vesic Nevio");
        assert_eq!(
            salary.counterparty_iban.as_deref(),
            Some("HR3924020063202676456")
        );
        assert_eq!(salary.description, "Placa za 6 / 2026");
        assert_eq!(salary.reference_number, "HR69 40002-38846238650-100");
        assert_eq!(salary.amount_minor, 107_560);

        let card = &st.entries[2];
        assert_eq!(card.counterparty, "Google Workspace_inorbit. Dublin");
        assert_eq!(card.counterparty_iban, None);
        assert!(
            card.description.starts_with("424472XXXXXX935 5, Google"),
            "{}",
            card.description
        );
        assert_eq!(card.reference_number, "HR99");
        assert_eq!(card.payment_reference, "11321040710");

        let tenderly = &st.entries[3];
        assert_eq!(
            tenderly.counterparty,
            "TENDERLY D.O.O. BEOGRAD-NOVI BEOGRAD 11070"
        );
        assert_eq!(
            tenderly.counterparty_iban.as_deref(),
            Some("RS35265100000024355587")
        );
        assert_eq!(tenderly.description, "BROJ RACUNA 7-1-1-2026");
        assert_eq!((tenderly.amount_minor, tenderly.credit), (1_487_260, true));
        assert_eq!(tenderly.booking_date.to_string(), "2026-07-03");

        // A mixed day: the loan and the card charge out, the invoice in.
        let by_ref = |r: &str| {
            st.entries
                .iter()
                .find(|e| e.payment_reference == r)
                .unwrap()
        };
        assert!(
            !by_ref("2026-67610968-11342097182").credit,
            "the loan goes out"
        );
        assert!(by_ref("2026-77121897-5664908732").credit, "Eiger pays in");
        assert!(!by_ref("11343721367").credit);
        assert_eq!(
            by_ref("2026-77121897-5664908732").description,
            "INV-9-1-1-2026"
        );
        assert_eq!(
            by_ref("2026-67610968-11342097182").reference_number,
            "HR00 080726"
        );
    }

    #[test]
    fn the_rows_are_shaped_like_the_providers_and_the_matcher_reads_the_number() {
        let st = parse(JULY).unwrap();
        let rows = provider_rows(&st);
        let tenderly = rows
            .iter()
            .find(|r| r["entry_reference"] == "izvod:3550061910-518173447")
            .unwrap();
        assert_eq!(tenderly["credit_debit_indicator"], "CRDT");
        assert_eq!(tenderly["transaction_amount"]["amount"], "14872.60");
        assert_eq!(tenderly["transaction_amount"]["currency"], "EUR");
        assert_eq!(
            tenderly["debtor"]["name"],
            "TENDERLY D.O.O. BEOGRAD-NOVI BEOGRAD 11070"
        );
        assert_eq!(tenderly["debtor_account"]["iban"], "RS35265100000024355587");
        assert_eq!(
            tenderly["creditor_account"]["iban"],
            "HR9224020061100925189"
        );
        assert_eq!(
            tenderly["remittance_information"][1],
            "BROJ RACUNA 7-1-1-2026"
        );
        assert_eq!(tenderly["booking_date"], "2026-07-03");
        let remittance = format!(
            "{} | {}",
            tenderly["remittance_information"][0].as_str().unwrap(),
            tenderly["remittance_information"][1].as_str().unwrap()
        );
        assert_eq!(
            crate::invoice::payments::number_in(&remittance),
            Some((7, "1".into(), "1".into(), 2026))
        );
        let eiger = rows
            .iter()
            .find(|r| r["entry_reference"] == "izvod:2026-77121897-5664908732")
            .unwrap();
        assert_eq!(
            crate::invoice::payments::number_in(
                eiger["remittance_information"][1].as_str().unwrap()
            ),
            Some((9, "1".into(), "1".into(), 2026))
        );
        let salary = rows
            .iter()
            .find(|r| r["entry_reference"] == "izvod:2026-64860935-11315055691")
            .unwrap();
        assert_eq!(salary["credit_debit_indicator"], "DBIT");
        assert_eq!(salary["creditor"]["name"], "Vesic Nevio");
        assert_eq!(salary["debtor_account"]["iban"], "HR9224020061100925189");
        assert_eq!(salary["transaction_amount"]["amount"], "1075.60");
    }

    #[test]
    fn a_day_whose_totals_fit_no_subset_is_left_out_and_counted() {
        let text = JULY.replace(
            "S t a n j e 2.601,34 9.167,00",
            "S t a n j e 2.601,34 9.000,00",
        );
        let st = parse(&text).unwrap();
        assert_eq!(st.unresolved_days, 1);
        assert_eq!(st.entries.len(), 4);
        // Without any day totals nothing says which way an entry went.
        let flat: String = JULY
            .lines()
            .filter(|l| !l.starts_with("Stanje na dan") && !l.starts_with("S t a n j e"))
            .collect::<Vec<_>>()
            .join("\n");
        let st = parse(&flat).unwrap();
        assert_eq!(st.entries.len(), 0);
        assert_eq!(st.undirected, 7);
        assert!(!is_statement("Račun br. 5\nIBAN: HR12"));
        assert_eq!(parse("nothing"), None);
    }
}
