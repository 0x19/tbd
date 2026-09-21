//! Money in against the invoices: what settled them.
//!
//! The payer writes the invoice number in the remittance (`BROJ RACUNA
//! 9-1-1-2026`, model HR99, no structured reference; measured on every
//! Tenderly settlement) or, with a domestic bank, in the structured
//! *poziv na broj*. The matcher reads both, finds the invoice with that
//! number, and records the transaction against it when the currency agrees;
//! the amount is what arrived, so a part pays a part and the invoice stays
//! open with the rest. A person records a payment the bank has not shown
//! (declared) and undoes a match (rejected, kept so it is not remade). An
//! invoice is `paid` when its payments cover the total; undoing the last
//! payment is the one step back the machine has, `paid -> approved`.

use chrono::{DateTime, NaiveDate, Utc};
use regex::Regex;
use sqlx::{PgPool, Postgres, Transaction};
use tbd_db::{Access, DbError, PartyId, map_err};
use uuid::Uuid;

use super::store::{INVOICE_COLUMNS, InvoiceError, InvoiceRow, event, invoice, sql};

/// One payment against an invoice.
#[allow(missing_docs)]
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PaymentRow {
    pub id: Uuid,
    pub invoice_id: Uuid,
    pub party_id: Uuid,
    pub transaction_id: Option<Uuid>,
    pub amount_minor: i64,
    pub currency: String,
    pub paid_on: NaiveDate,
    pub source: String,
    pub reason: String,
    pub note: String,
    pub created_at: DateTime<Utc>,
    /// The bank's counterparty on the transaction, for the page.
    pub counterparty: Option<String>,
}

const PAYMENT_COLUMNS: &str =
    "p.id, p.invoice_id, p.party_id, p.transaction_id, p.amount_minor, p.currency,
    p.paid_on, p.source, p.reason, p.note, p.created_at, t.counterparty_name as counterparty";

/// The invoice number written in a remittance or a structured reference:
/// `ordinal-premises-device-year`, wherever it stands in the text.
#[must_use]
pub fn number_in(text: &str) -> Option<(i32, String, String, i32)> {
    let re = Regex::new(r"\b(\d{1,5})-(\d{1,3})-(\d{1,3})-(20\d{2})\b")
        .unwrap_or_else(|_| unreachable!());
    let c = re.captures(text)?;
    Some((
        c[1].parse().ok()?,
        c[2].to_owned(),
        c[3].to_owned(),
        c[4].parse().ok()?,
    ))
}

/// The payments of these invoices, oldest first.
///
/// # Errors
/// The database.
pub async fn payments_of(pool: &PgPool, invoice_ids: &[Uuid]) -> Result<Vec<PaymentRow>, DbError> {
    if invoice_ids.is_empty() {
        return Ok(Vec::new());
    }
    sqlx::query_as::<_, PaymentRow>(sql(&format!(
        "select {PAYMENT_COLUMNS} from finance.invoice_payments p
           left join finance.bank_transactions t on t.id = p.transaction_id
          where p.invoice_id = any($1) and p.source <> 'rejected'
          order by p.paid_on, p.created_at"
    )))
    .bind(invoice_ids)
    .fetch_all(pool)
    .await
    .map_err(map_err)
}

/// An incoming transaction the matcher looks at.
#[derive(Debug, sqlx::FromRow)]
struct Incoming {
    id: Uuid,
    amount_minor: i64,
    currency: String,
    booking_date: NaiveDate,
    remittance: Option<String>,
    reference_number: Option<String>,
}

/// Match the party's incoming transactions to its invoices by the number the
/// payer wrote. Idempotent and cheap: only transactions no payment row names
/// are looked at, so it runs before every read of the invoices.
///
/// # Errors
/// The database.
pub async fn settle(pool: &PgPool, party: Uuid) -> Result<usize, InvoiceError> {
    let candidates: Vec<Incoming> = sqlx::query_as(
        "select t.id, t.amount_minor, t.currency, t.booking_date, t.remittance, t.reference_number
           from finance.bank_transactions t
          where t.party_id = $1 and t.status = 'booked' and t.credit_debit = 'CRDT'
            and t.amount_minor > 0 and t.booking_date is not null
            and (t.remittance ~ '\\d+-\\d+-\\d+-20\\d\\d' or t.reference_number ~ '\\d+-\\d+-\\d+-20\\d\\d')
            and not exists (select 1 from finance.invoice_payments p where p.transaction_id = t.id)
          order by t.booking_date",
    )
    .bind(party)
    .fetch_all(pool)
    .await
    .map_err(map_err)?;
    let mut made = 0;
    for t in candidates {
        let text = format!(
            "{} {}",
            t.reference_number.as_deref().unwrap_or(""),
            t.remittance.as_deref().unwrap_or("")
        );
        let Some((ordinal, premises, device, year)) = number_in(&text) else {
            continue;
        };
        let hit: Option<(Uuid, String)> = sqlx::query_as(
            "select id, currency from finance.invoices
              where party_id = $1 and year = $2 and premises = $3 and device = $4 and ordinal = $5
                and status in ('approved', 'sent', 'paid')",
        )
        .bind(party)
        .bind(year)
        .bind(&premises)
        .bind(&device)
        .bind(ordinal)
        .fetch_optional(pool)
        .await
        .map_err(map_err)?;
        let Some((invoice_id, currency)) = hit else {
            continue;
        };
        if currency != t.currency {
            continue;
        }
        let mut tx = pool.begin().await.map_err(map_err)?;
        sqlx::query(
            "insert into finance.invoice_payments
                (id, invoice_id, party_id, transaction_id, amount_minor, currency, paid_on, source, reason)
             values ($1, $2, $3, $4, $5, $6, $7, 'inferred', $8)
             on conflict do nothing",
        )
        .bind(Uuid::new_v4())
        .bind(invoice_id)
        .bind(party)
        .bind(t.id)
        .bind(t.amount_minor)
        .bind(&t.currency)
        .bind(t.booking_date)
        .bind(format!("reference {ordinal}-{premises}-{device}-{year}"))
        .execute(&mut *tx)
        .await
        .map_err(map_err)?;
        recompute(&mut tx, invoice_id, None).await?;
        tx.commit().await.map_err(map_err)?;
        made += 1;
    }
    Ok(made)
}

/// A payment a person records: a bank transaction of the party (its amount
/// and day, unless given), or one the bank has not shown.
#[derive(Debug, Clone, Default)]
pub struct RecordInput {
    /// The transaction, when the payment is one in the bank.
    pub transaction_id: Option<Uuid>,
    /// Minor units; zero means the transaction's amount.
    pub amount_minor: i64,
    /// The day; `None` means the transaction's booking day, else today.
    pub paid_on: Option<NaiveDate>,
    /// Free text.
    pub note: String,
}

/// Record a payment against an invoice.
///
/// # Errors
/// Not in the grant (not found); the invoice is a draft or cancelled; the
/// transaction is another party's, not money in, or already settles an
/// invoice; a zero amount with no transaction; the database.
pub async fn record(
    pool: &PgPool,
    access: &Access,
    invoice_id: Uuid,
    input: &RecordInput,
) -> Result<InvoiceRow, InvoiceError> {
    let (row, _) = invoice(pool, access, invoice_id).await?;
    if !matches!(row.status.as_str(), "approved" | "sent" | "paid") {
        return Err(InvoiceError::NotDraft(format!(
            "{}; only an issued invoice takes a payment",
            row.status
        )));
    }
    let (amount, day, currency) = if let Some(tid) = input.transaction_id {
        from_transaction(pool, &row, invoice_id, tid, input).await?
    } else {
        if input.amount_minor == 0 {
            return Err(InvoiceError::Invalid(
                "a payment needs an amount or a transaction".into(),
            ));
        }
        (
            input.amount_minor,
            input.paid_on.unwrap_or_else(|| Utc::now().date_naive()),
            row.currency.clone(),
        )
    };
    let mut tx = pool.begin().await.map_err(map_err)?;
    if let Some(tid) = input.transaction_id {
        // A rejected match on this transaction gives way to the person's word.
        sqlx::query("delete from finance.invoice_payments where transaction_id = $1 and source = 'rejected'")
            .bind(tid)
            .execute(&mut *tx)
            .await
            .map_err(map_err)?;
    }
    sqlx::query(
        "insert into finance.invoice_payments
            (id, invoice_id, party_id, transaction_id, amount_minor, currency, paid_on, source, reason, note)
         values ($1, $2, $3, $4, $5, $6, $7, 'declared', 'by hand', $8)",
    )
    .bind(Uuid::new_v4())
    .bind(invoice_id)
    .bind(row.party_id)
    .bind(input.transaction_id)
    .bind(amount)
    .bind(&currency)
    .bind(day)
    .bind(input.note.trim())
    .execute(&mut *tx)
    .await
    .map_err(map_err)?;
    recompute(&mut tx, invoice_id, Some(access)).await?;
    tx.commit().await.map_err(map_err)?;
    invoice(pool, access, invoice_id).await.map(|(r, _)| r)
}

/// The amount, day and currency a recorded payment takes from its bank
/// transaction, once that transaction is checked: the party's, money in,
/// and not already settling another invoice.
async fn from_transaction(
    pool: &PgPool,
    row: &InvoiceRow,
    invoice_id: Uuid,
    tid: Uuid,
    input: &RecordInput,
) -> Result<(i64, NaiveDate, String), InvoiceError> {
    let t: Option<Incoming> = sqlx::query_as(
        "select t.id, t.amount_minor, t.currency, t.booking_date, t.remittance, t.reference_number
           from finance.bank_transactions t
          where t.id = $1 and t.party_id = $2 and t.credit_debit = 'CRDT' and t.booking_date is not null",
    )
    .bind(tid)
    .bind(row.party_id)
    .fetch_optional(pool)
    .await
    .map_err(map_err)?;
    let t = t.ok_or(DbError::NotFound {
        what: "transaction",
    })?;
    let taken: Option<(Uuid, String)> = sqlx::query_as(
        "select invoice_id, source from finance.invoice_payments where transaction_id = $1",
    )
    .bind(tid)
    .fetch_optional(pool)
    .await
    .map_err(map_err)?;
    match taken {
        Some((other, source)) if source != "rejected" && other != invoice_id => {
            return Err(InvoiceError::Invalid(
                "that transaction already settles another invoice".into(),
            ));
        }
        Some((_, source)) if source != "rejected" => {
            return Err(InvoiceError::Invalid(
                "that transaction is already recorded on this invoice".into(),
            ));
        }
        _ => {}
    }
    Ok((
        if input.amount_minor == 0 {
            t.amount_minor
        } else {
            input.amount_minor
        },
        input.paid_on.unwrap_or(t.booking_date),
        t.currency,
    ))
}

/// Undo a payment. A person's own record is deleted; a match the matcher
/// made is kept as `rejected`, so it is not made again.
///
/// # Errors
/// Not in the grant (not found); the database.
pub async fn unlink(
    pool: &PgPool,
    access: &Access,
    payment_id: Uuid,
) -> Result<InvoiceRow, InvoiceError> {
    let found: Option<(Uuid, Uuid, String)> = sqlx::query_as(
        "select invoice_id, party_id, source from finance.invoice_payments where id = $1",
    )
    .bind(payment_id)
    .fetch_optional(pool)
    .await
    .map_err(map_err)?;
    let (invoice_id, party, source) = found.ok_or(DbError::NotFound { what: "payment" })?;
    access.require(PartyId(party), "payment")?;
    let mut tx = pool.begin().await.map_err(map_err)?;
    if source == "inferred" {
        sqlx::query("update finance.invoice_payments set source = 'rejected' where id = $1")
            .bind(payment_id)
            .execute(&mut *tx)
            .await
            .map_err(map_err)?;
    } else {
        sqlx::query("delete from finance.invoice_payments where id = $1")
            .bind(payment_id)
            .execute(&mut *tx)
            .await
            .map_err(map_err)?;
    }
    recompute(&mut tx, invoice_id, Some(access)).await?;
    tx.commit().await.map_err(map_err)?;
    invoice(pool, access, invoice_id).await.map(|(r, _)| r)
}

/// The sum on the invoice and its status: `paid` when covered, back to
/// `approved` when no longer, with an event either way.
async fn recompute(
    tx: &mut Transaction<'_, Postgres>,
    invoice_id: Uuid,
    access: Option<&Access>,
) -> Result<(), InvoiceError> {
    let (paid, last): (i64, Option<NaiveDate>) = sqlx::query_as(
        "select coalesce(sum(amount_minor), 0)::bigint, max(paid_on)
           from finance.invoice_payments where invoice_id = $1 and source <> 'rejected'",
    )
    .bind(invoice_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(map_err)?;
    let row = sqlx::query_as::<_, InvoiceRow>(sql(&format!(
        "select {INVOICE_COLUMNS} from finance.invoices where id = $1 for update"
    )))
    .bind(invoice_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(map_err)?;
    let covered = paid >= row.total_minor && row.total_minor > 0;
    let status = match (row.status.as_str(), covered) {
        ("approved" | "sent", true) => "paid",
        ("paid", false) => "approved",
        (s, _) => s,
    };
    let paid_at = if covered {
        last.and_then(|d| d.and_hms_opt(0, 0, 0))
            .map(|d| d.and_utc())
    } else {
        None
    };
    sqlx::query(
        "update finance.invoices set paid_minor = $2, paid_at = $3, status = $4, updated_at = clock_timestamp()
          where id = $1",
    )
    .bind(invoice_id)
    .bind(paid)
    .bind(paid_at)
    .bind(status)
    .execute(&mut **tx)
    .await
    .map_err(map_err)?;
    if status != row.status {
        let user = access.map_or(tbd_db::UserId(Uuid::nil()), Access::user);
        event(
            tx,
            invoice_id,
            user,
            if status == "paid" { "paid" } else { "unpaid" },
            serde_json::json!({ "paid_minor": paid, "total_minor": row.total_minor }),
        )
        .await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_number_is_read_wherever_the_payer_wrote_it() {
        assert_eq!(
            number_in("HR99 | BROJ RACUNA 9-1-1-2026"),
            Some((9, "1".into(), "1".into(), 2026))
        );
        assert_eq!(
            number_in("HR00 12-1-1-2025 Racun 12-1-1-2025"),
            Some((12, "1".into(), "1".into(), 2025))
        );
        assert_eq!(number_in("HR99 | Naplata naknade platnog prometa"), None);
        assert_eq!(
            number_in("2026-03-05 uplata"),
            None,
            "a date is not a number"
        );
    }
}
