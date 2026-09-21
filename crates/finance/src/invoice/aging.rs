//! Receivables aging: what is still owed, and for how long. Pure functions
//! over the rows the store returns, so the report is testable without a
//! database and every surface (the list, the report) agrees on what "open",
//! "outstanding" and "overdue" mean.

use std::collections::BTreeMap;

use chrono::NaiveDate;
use uuid::Uuid;

use super::store::{ClientRow, InvoiceRow};

/// The buckets in order: not yet due, then by how long past due.
pub const BUCKETS: [&str; 5] = ["current", "d1_30", "d31_60", "d61_90", "d90_plus"];

/// Issued and not settled: what a receivable is.
#[must_use]
pub fn is_open(row: &InvoiceRow) -> bool {
    matches!(row.status.as_str(), "approved" | "sent")
}

/// What is still owed on it; zero unless open.
#[must_use]
pub fn outstanding(row: &InvoiceRow) -> i64 {
    if is_open(row) {
        (row.total_minor - row.paid_minor).max(0)
    } else {
        0
    }
}

/// Days past the due date; zero when not yet due or not open.
#[must_use]
pub fn days_overdue(row: &InvoiceRow, today: NaiveDate) -> i32 {
    if !is_open(row) {
        return 0;
    }
    i32::try_from((today - row.due_date).num_days().max(0)).unwrap_or(i32::MAX)
}

/// Which bucket a number of days past due falls in.
#[must_use]
pub fn bucket(days: i32) -> &'static str {
    match days {
        i32::MIN..=0 => BUCKETS[0],
        1..=30 => BUCKETS[1],
        31..=60 => BUCKETS[2],
        61..=90 => BUCKETS[3],
        _ => BUCKETS[4],
    }
}

/// One bucket of one currency.
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bucket {
    pub currency: String,
    pub bucket: &'static str,
    pub count: i32,
    pub amount_minor: i64,
}

/// One client's receivables in one currency.
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientAging {
    pub client_id: Uuid,
    pub client_name: String,
    pub currency: String,
    pub count: i32,
    pub outstanding_minor: i64,
    pub overdue_minor: i64,
    pub oldest_days: i32,
}

/// The report: every bucket of every currency with something open (all five,
/// so a chart is stable), and the clients who owe, worst first.
#[allow(missing_docs)]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Report {
    pub buckets: Vec<Bucket>,
    pub clients: Vec<ClientAging>,
}

/// The report over the rows as of `today`.
#[must_use]
pub fn report(rows: &[InvoiceRow], clients: &[ClientRow], today: NaiveDate) -> Report {
    let mut by_currency: BTreeMap<String, [(i32, i64); 5]> = BTreeMap::new();
    let mut by_client: BTreeMap<(Uuid, String), ClientAging> = BTreeMap::new();
    for row in rows.iter().filter(|r| is_open(r)) {
        let owed = outstanding(row);
        if owed == 0 {
            continue;
        }
        let days = days_overdue(row, today);
        let slot = BUCKETS
            .iter()
            .position(|b| *b == bucket(days))
            .unwrap_or_default();
        let cells = by_currency.entry(row.currency.clone()).or_default();
        cells[slot].0 += 1;
        cells[slot].1 += owed;
        let entry = by_client
            .entry((row.client_id, row.currency.clone()))
            .or_insert_with(|| ClientAging {
                client_id: row.client_id,
                client_name: clients
                    .iter()
                    .find(|c| c.id == row.client_id)
                    .map(|c| c.name.clone())
                    .unwrap_or_default(),
                currency: row.currency.clone(),
                count: 0,
                outstanding_minor: 0,
                overdue_minor: 0,
                oldest_days: 0,
            });
        entry.count += 1;
        entry.outstanding_minor += owed;
        if days > 0 {
            entry.overdue_minor += owed;
        }
        entry.oldest_days = entry.oldest_days.max(days);
    }
    let buckets = by_currency
        .into_iter()
        .flat_map(|(currency, cells)| {
            BUCKETS
                .iter()
                .zip(cells)
                .map(move |(bucket, (count, amount_minor))| Bucket {
                    currency: currency.clone(),
                    bucket,
                    count,
                    amount_minor,
                })
        })
        .collect();
    let mut clients: Vec<ClientAging> = by_client.into_values().collect();
    clients.sort_by(|a, b| {
        b.overdue_minor
            .cmp(&a.overdue_minor)
            .then(b.outstanding_minor.cmp(&a.outstanding_minor))
            .then(a.client_name.cmp(&b.client_name))
    });
    Report { buckets, clients }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;

    fn row(status: &str, due: &str, total: i64, paid: i64, client: Uuid) -> InvoiceRow {
        let now = Utc::now();
        InvoiceRow {
            id: Uuid::new_v4(),
            party_id: Uuid::nil(),
            client_id: client,
            status: status.into(),
            year: 2026,
            ordinal: Some(1),
            premises: "1".into(),
            device: "1".into(),
            number: Some("1-1-1-2026".into()),
            issued_at: Some(now),
            delivery_date: due.parse().unwrap(),
            due_date: due.parse().unwrap(),
            place_of_issue: String::new(),
            currency: "EUR".into(),
            subtotal_minor: total,
            vat_minor: 0,
            total_minor: total,
            vat_treatment: "outside_scope_non_eu".into(),
            vat_note: String::new(),
            note: String::new(),
            content_hash: None,
            approved_at: Some(now),
            document_id: None,
            prefilled_from: None,
            cancelled_at: None,
            created_at: now,
            updated_at: now,
            paid_minor: paid,
            paid_at: None,
            sent_at: None,
        }
    }

    fn named(id: Uuid, name: &str) -> ClientRow {
        ClientRow {
            id,
            party_id: Uuid::nil(),
            name: name.into(),
            address_lines: Vec::new(),
            country_code: "RS".into(),
            tax_id: String::new(),
            vat_treatment: "outside_scope_non_eu".into(),
            recipients: Vec::new(),
            currency: "EUR".into(),
            archived_at: None,
            is_default: false,
        }
    }

    #[test]
    fn days_and_buckets_follow_the_due_date_for_open_invoices_only() {
        let today: NaiveDate = "2026-09-21".parse().unwrap();
        let client = Uuid::new_v4();
        assert_eq!(
            days_overdue(&row("sent", "2026-09-21", 100, 0, client), today),
            0
        );
        assert_eq!(
            days_overdue(&row("sent", "2026-09-20", 100, 0, client), today),
            1
        );
        assert_eq!(
            days_overdue(&row("approved", "2026-06-01", 100, 0, client), today),
            112
        );
        assert_eq!(
            days_overdue(&row("paid", "2026-06-01", 100, 100, client), today),
            0
        );
        assert_eq!(
            days_overdue(&row("draft", "2026-06-01", 100, 0, client), today),
            0
        );
        assert_eq!(
            days_overdue(&row("sent", "2026-10-01", 100, 0, client), today),
            0
        );
        assert_eq!(bucket(0), "current");
        assert_eq!(bucket(1), "d1_30");
        assert_eq!(bucket(30), "d1_30");
        assert_eq!(bucket(31), "d31_60");
        assert_eq!(bucket(60), "d31_60");
        assert_eq!(bucket(61), "d61_90");
        assert_eq!(bucket(90), "d61_90");
        assert_eq!(bucket(91), "d90_plus");
        assert_eq!(bucket(4000), "d90_plus");
        assert_eq!(
            outstanding(&row("sent", "2026-09-01", 1000, 400, client)),
            600
        );
        assert_eq!(
            outstanding(&row("sent", "2026-09-01", 1000, 1400, client)),
            0
        );
        assert_eq!(
            outstanding(&row("paid", "2026-09-01", 1000, 1000, client)),
            0
        );
    }

    #[test]
    fn the_report_sums_by_bucket_and_ranks_clients_by_what_is_overdue() {
        let today: NaiveDate = "2026-09-21".parse().unwrap();
        let tenderly = Uuid::new_v4();
        let eiger = Uuid::new_v4();
        let rows = vec![
            row("sent", "2026-10-15", 1_450_082, 0, tenderly), // not due yet
            row("sent", "2026-09-01", 1_000_000, 400_000, tenderly), // 20 days, 6,000 owed
            row("approved", "2026-05-15", 500_000, 0, eiger),  // 129 days
            row("paid", "2026-01-15", 900_000, 900_000, eiger), // settled: absent
            row("cancelled", "2026-01-15", 900_000, 0, eiger), // absent
            row("draft", "2026-01-15", 900_000, 0, eiger),     // absent
        ];
        let clients = vec![named(tenderly, "Tenderly"), named(eiger, "Eiger")];
        let rep = report(&rows, &clients, today);
        let cell = |b: &str| rep.buckets.iter().find(|x| x.bucket == b).unwrap();
        assert_eq!(rep.buckets.len(), 5, "all five, in one currency");
        assert_eq!(
            (cell("current").count, cell("current").amount_minor),
            (1, 1_450_082)
        );
        assert_eq!(
            (cell("d1_30").count, cell("d1_30").amount_minor),
            (1, 600_000)
        );
        assert_eq!((cell("d31_60").count, cell("d31_60").amount_minor), (0, 0));
        assert_eq!((cell("d61_90").count, cell("d61_90").amount_minor), (0, 0));
        assert_eq!(
            (cell("d90_plus").count, cell("d90_plus").amount_minor),
            (1, 500_000)
        );
        assert_eq!(rep.clients.len(), 2);
        assert_eq!(
            rep.clients[0].client_name, "Tenderly",
            "6,000 overdue beats 5,000"
        );
        assert_eq!(rep.clients[0].outstanding_minor, 2_050_082);
        assert_eq!(rep.clients[0].overdue_minor, 600_000);
        assert_eq!(rep.clients[0].oldest_days, 20);
        assert_eq!(rep.clients[1].client_name, "Eiger");
        assert_eq!(rep.clients[1].oldest_days, 129);
        assert_eq!(rep.clients[1].count, 1);
        assert!(
            report(&[], &clients, today).buckets.is_empty(),
            "nothing open, no buckets"
        );
    }
}
