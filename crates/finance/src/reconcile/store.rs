//! The month's rows, the links, the policies: reconciliation against
//! Postgres. The rules are in the parent module; this is what feeds them
//! and what records their outcome.

use chrono::{Days, NaiveDate};
use sqlx::PgPool;
use tbd_db::{Access, DbError, PartyId, map_err};
use uuid::Uuid;

use super::{
    Decision, DocFacts, LINK, Need, Policy, SUGGEST, TxFacts, Why, classify, encode_all, normalise,
    score,
};
use crate::connectors::store::StoreError;

/// A transaction as the page needs it, from `transactions_enriched`.
#[allow(missing_docs)]
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TxRow {
    pub id: Uuid,
    pub account_id: Uuid,
    pub party_id: Uuid,
    pub status: String,
    pub amount_minor: i64,
    pub currency: String,
    pub scale: i16,
    pub credit_debit: String,
    pub booking_date: NaiveDate,
    pub value_date: Option<NaiveDate>,
    pub counterparty_name: Option<String>,
    pub counterparty_iban: Option<String>,
    pub remittance: Option<String>,
    pub reference_number: Option<String>,
    pub entry_reference: Option<String>,
    pub category_id: Option<Uuid>,
    pub category_source: Option<String>,
    pub category: Option<String>,
    pub internal: bool,
}

impl TxRow {
    fn facts(&self) -> TxFacts {
        TxFacts {
            credit: self.credit_debit == "CRDT" || self.amount_minor > 0,
            amount_minor: self.amount_minor,
            currency: self.currency.trim().to_owned(),
            counterparty_name: self.counterparty_name.clone().unwrap_or_default(),
            counterparty_iban: self.counterparty_iban.clone().unwrap_or_default(),
            remittance: self.remittance.clone().unwrap_or_default(),
            reference_number: self.reference_number.clone().unwrap_or_default(),
            booking_date: self.booking_date,
            internal: self.internal,
        }
    }
}

/// A receipt beside a transaction: linked, or offered.
#[allow(missing_docs)]
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DocRow {
    pub id: Uuid,
    pub vendor: Option<String>,
    pub doc_date: Option<NaiveDate>,
    pub total_minor: Option<i64>,
    pub currency: Option<String>,
    pub filename: Option<String>,
    pub invoice_no: Option<String>,
}

impl DocRow {
    fn facts(&self) -> DocFacts {
        DocFacts {
            id: self.id,
            vendor: self.vendor.clone().unwrap_or_default(),
            doc_date: self.doc_date,
            total_minor: self.total_minor,
            currency: self.currency.clone().unwrap_or_default().trim().to_owned(),
        }
    }
}

/// A link with its receipt.
#[allow(missing_docs)]
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct LinkRow {
    pub transaction_id: Uuid,
    pub source: String,
    pub confidence: i16,
    pub reason: String,
    #[sqlx(flatten)]
    pub document: DocRow,
}

/// A person's policy row.
#[allow(missing_docs)]
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PolicyRow {
    pub id: Uuid,
    pub party_id: Uuid,
    pub match_normalised: String,
    pub exact: bool,
    pub policy: String,
    pub note: String,
}

impl PolicyRow {
    fn rule(&self) -> Option<Policy> {
        Some(Policy {
            id: self.id,
            match_normalised: self.match_normalised.clone(),
            exact: self.exact,
            need: Need::from_policy(&self.policy)?,
        })
    }
}

/// One reconciled transaction.
#[derive(Debug, Clone)]
pub struct Row {
    /// The transaction.
    pub tx: TxRow,
    /// What is needed, and why.
    pub decision: Decision,
    /// Receipts linked to it.
    pub documents: Vec<LinkRow>,
    /// Likely receipts, best first, with score and why.
    pub suggestions: Vec<(DocRow, u8, Vec<Why>)>,
    /// The card's original charge, when the bank wrote it.
    pub original: Option<(i64, String)>,
}

impl Row {
    /// `covered`, `missing`, or empty when no receipt is needed.
    #[must_use]
    pub fn status(&self) -> &'static str {
        match self.decision.need {
            Need::Receipt if self.documents.is_empty() => "missing",
            Need::Receipt => "covered",
            _ => "",
        }
    }
}

const TX_COLUMNS: &str = "t.id, t.account_id, t.party_id, t.status, t.amount_minor, t.currency, t.scale,
    t.credit_debit, t.booking_date, t.value_date, t.counterparty_name, t.counterparty_iban, t.remittance,
    t.reference_number, t.entry_reference, t.category_id, t.category_source, c.name as category, t.internal";

const DOC_COLUMNS: &str =
    "d.id, d.vendor, d.doc_date, d.total_minor, d.currency, d.filename, d.invoice_no";

fn sql(s: &str) -> sqlx::AssertSqlSafe<String> {
    sqlx::AssertSqlSafe(s.to_owned())
}

/// The first and the day after the last of `YYYY-MM`.
///
/// # Errors
/// Not a month.
pub fn month_bounds(month: &str) -> Result<(NaiveDate, NaiveDate), StoreError> {
    let bad = || StoreError::State(format!("month: want YYYY-MM, got {month:?}"));
    let (y, m) = month.split_once('-').ok_or_else(bad)?;
    let y: i32 = y.parse().map_err(|_| bad())?;
    let m: u32 = m.parse().map_err(|_| bad())?;
    let from = NaiveDate::from_ymd_opt(y, m, 1).ok_or_else(bad)?;
    let to = if m == 12 {
        NaiveDate::from_ymd_opt(y + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(y, m + 1, 1)
    }
    .ok_or_else(bad)?;
    Ok((from, to))
}

/// The party's policies, a person's word first.
///
/// # Errors
/// The database.
pub async fn policies(pool: &PgPool, party: Uuid) -> Result<Vec<PolicyRow>, StoreError> {
    Ok(sqlx::query_as::<_, PolicyRow>(
        "select id, party_id, match_normalised, exact, policy, note from finance.counterparty_policies
          where party_id = $1 order by exact desc, length(match_normalised) desc",
    )
    .bind(party)
    .fetch_all(pool)
    .await
    .map_err(map_err)?)
}

/// Set a policy for a counterparty name or fragment. Same match, new word.
///
/// # Errors
/// The party is outside the grant (not found); the database.
pub async fn set_policy(
    pool: &PgPool,
    access: &Access,
    party: Uuid,
    matches: &str,
    exact: bool,
    policy: Need,
    note: &str,
) -> Result<PolicyRow, StoreError> {
    access.require(PartyId(party), "party_id")?;
    let normalised = normalise(matches);
    if normalised.len() < 2 {
        return Err(StoreError::State("match: too short".into()));
    }
    Ok(sqlx::query_as::<_, PolicyRow>(
        "insert into finance.counterparty_policies (id, party_id, match_normalised, exact, policy, note)
         values ($1, $2, $3, $4, $5, $6)
         on conflict (party_id, match_normalised) do update set exact = $4, policy = $5, note = $6
         returning id, party_id, match_normalised, exact, policy, note",
    )
    .bind(Uuid::new_v4())
    .bind(party)
    .bind(&normalised)
    .bind(exact)
    .bind(policy.as_str())
    .bind(note)
    .fetch_one(pool)
    .await
    .map_err(map_err)?)
}

/// Remove a policy.
///
/// # Errors
/// Not in the grant (not found); the database.
pub async fn delete_policy(pool: &PgPool, access: &Access, id: Uuid) -> Result<(), StoreError> {
    let row: Option<(Uuid,)> =
        sqlx::query_as("select party_id from finance.counterparty_policies where id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(map_err)?;
    let (party,) = row.ok_or(DbError::NotFound { what: "policy" })?;
    access.require(PartyId(party), "policy")?;
    sqlx::query("delete from finance.counterparty_policies where id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(map_err)?;
    Ok(())
}

async fn transactions(
    pool: &PgPool,
    party: Uuid,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Vec<TxRow>, StoreError> {
    Ok(sqlx::query_as::<_, TxRow>(sql(&format!(
        "select {TX_COLUMNS} from finance.transactions_enriched t
           left join finance.categories c on c.id = t.category_id
          where t.party_id = $1 and t.booking_date >= $2 and t.booking_date < $3
          order by t.booking_date, t.amount_minor"
    )))
    .bind(party)
    .bind(from)
    .bind(to)
    .fetch_all(pool)
    .await
    .map_err(map_err)?)
}

async fn one_transaction(pool: &PgPool, id: Uuid) -> Result<Option<TxRow>, StoreError> {
    Ok(sqlx::query_as::<_, TxRow>(sql(&format!(
        "select {TX_COLUMNS} from finance.transactions_enriched t
           left join finance.categories c on c.id = t.category_id
          where t.id = $1"
    )))
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(map_err)?)
}

async fn links(pool: &PgPool, tx_ids: &[Uuid]) -> Result<Vec<LinkRow>, StoreError> {
    Ok(sqlx::query_as::<_, LinkRow>(sql(&format!(
        "select l.transaction_id, l.source, l.confidence, l.reason, {DOC_COLUMNS}
           from finance.transaction_documents l join finance.documents d on d.id = l.document_id
          where l.transaction_id = any($1) order by l.created_at"
    )))
    .bind(tx_ids)
    .fetch_all(pool)
    .await
    .map_err(map_err)?)
}

/// Receipts of the party dated near the window, with an amount, not yet
/// linked to anything.
async fn candidates(
    pool: &PgPool,
    party: Uuid,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Vec<DocRow>, StoreError> {
    let from = from.checked_sub_days(Days::new(21)).unwrap_or(from);
    let to = to.checked_add_days(Days::new(7)).unwrap_or(to);
    Ok(sqlx::query_as::<_, DocRow>(sql(&format!(
        "select {DOC_COLUMNS} from finance.documents d
          where d.party_id = $1 and d.kind = 'receipt' and d.total_minor is not null and d.total_minor > 0
            and coalesce(d.doc_date, d.created_at::date) >= $2 and coalesce(d.doc_date, d.created_at::date) < $3
            and not exists (select 1 from finance.transaction_documents l
                             where l.document_id = d.id and l.source <> 'rejected')"
    )))
    .bind(party)
    .bind(from)
    .bind(to)
    .fetch_all(pool)
    .await
    .map_err(map_err)?)
}

/// The month reconciled: every transaction with its need, its links (new
/// inferred ones written first), and its suggestions.
///
/// # Errors
/// The party is outside the grant (not found); the database.
pub async fn month(
    pool: &PgPool,
    access: &Access,
    party: Uuid,
    month: &str,
) -> Result<(Vec<Row>, Vec<PolicyRow>), StoreError> {
    access.require(PartyId(party), "party_id")?;
    let (from, to) = month_bounds(month)?;
    let txs = transactions(pool, party, from, to).await?;
    let policy_rows = policies(pool, party).await?;
    let rules: Vec<Policy> = policy_rows.iter().filter_map(PolicyRow::rule).collect();
    let ids: Vec<Uuid> = txs.iter().map(|t| t.id).collect();
    let mut linked = links(pool, &ids).await?;
    // What a person undid stays undone: those pairs are not scored.
    let rejected: std::collections::HashSet<(Uuid, Uuid)> = linked
        .extract_if(.., |l| l.source == "rejected")
        .map(|l| (l.transaction_id, l.document.id))
        .collect();

    let mut rows: Vec<Row> = txs
        .into_iter()
        .map(|tx| {
            let facts = tx.facts();
            let decision = classify(&facts, &rules);
            let original = super::original_amount(&facts.remittance);
            let documents = linked
                .extract_if(.., |l| l.transaction_id == tx.id)
                .collect();
            Row {
                tx,
                decision,
                documents,
                suggestions: Vec::new(),
                original,
            }
        })
        .collect();

    // The matcher: every receipt against every uncovered transaction that
    // needs one, best pairs first, each side used once.
    let pool_docs = candidates(pool, party, from, to).await?;
    let mut pairs: Vec<(usize, usize, u8, Vec<Why>)> = Vec::new();
    for (i, row) in rows.iter().enumerate() {
        if row.decision.need != Need::Receipt || !row.documents.is_empty() {
            continue;
        }
        let facts = row.tx.facts();
        for (j, doc) in pool_docs.iter().enumerate() {
            if rejected.contains(&(row.tx.id, doc.id)) {
                continue;
            }
            if let Some((points, why)) = score(&facts, &doc.facts()) {
                pairs.push((i, j, points, why));
            }
        }
    }
    pairs.sort_by_key(|p| std::cmp::Reverse(p.2));
    let mut doc_taken = vec![false; pool_docs.len()];
    let mut tx_taken = vec![false; rows.len()];
    for (i, j, points, why) in &pairs {
        if *points < LINK || doc_taken[*j] || tx_taken[*i] {
            continue;
        }
        let doc = &pool_docs[*j];
        let tx_id = rows[*i].tx.id;
        sqlx::query(
            "insert into finance.transaction_documents (transaction_id, document_id, source, confidence, reason)
             values ($1, $2, 'inferred', $3, $4) on conflict do nothing",
        )
        .bind(tx_id)
        .bind(doc.id)
        .bind(i16::from(*points))
        .bind(encode_all(why))
        .execute(pool)
        .await
        .map_err(map_err)?;
        rows[*i].documents.push(LinkRow {
            transaction_id: tx_id,
            source: "inferred".into(),
            confidence: i16::from(*points),
            reason: encode_all(why),
            document: doc.clone(),
        });
        doc_taken[*j] = true;
        tx_taken[*i] = true;
    }
    for (i, j, points, why) in &pairs {
        if *points < SUGGEST || doc_taken[*j] || tx_taken[*i] || rows[*i].suggestions.len() >= 3 {
            continue;
        }
        rows[*i]
            .suggestions
            .push((pool_docs[*j].clone(), *points, why.clone()));
    }
    Ok((rows, policy_rows))
}

/// One transaction reconciled on its own, after a link or unlink.
///
/// # Errors
/// Not in the grant (not found); the database.
pub async fn one(pool: &PgPool, access: &Access, tx_id: Uuid) -> Result<Row, StoreError> {
    let tx = one_transaction(pool, tx_id)
        .await?
        .ok_or(DbError::NotFound {
            what: "transaction",
        })?;
    access.require(PartyId(tx.party_id), "transaction")?;
    let rules: Vec<Policy> = policies(pool, tx.party_id)
        .await?
        .iter()
        .filter_map(PolicyRow::rule)
        .collect();
    let facts = tx.facts();
    let decision = classify(&facts, &rules);
    let original = super::original_amount(&facts.remittance);
    let documents = links(pool, &[tx.id])
        .await?
        .into_iter()
        .filter(|l| l.source != "rejected")
        .collect();
    Ok(Row {
        tx,
        decision,
        documents,
        suggestions: Vec::new(),
        original,
    })
}

/// A person links a receipt to the transaction that paid it. Both must be
/// the same party's; a document already linked elsewhere moves.
///
/// # Errors
/// Not in the grant (not found); the database.
pub async fn link(
    pool: &PgPool,
    access: &Access,
    tx_id: Uuid,
    doc_id: Uuid,
) -> Result<Row, StoreError> {
    let tx = one_transaction(pool, tx_id)
        .await?
        .ok_or(DbError::NotFound {
            what: "transaction",
        })?;
    access.require(PartyId(tx.party_id), "transaction")?;
    let doc: Option<(Uuid,)> =
        sqlx::query_as("select party_id from finance.documents where id = $1 and kind = 'receipt'")
            .bind(doc_id)
            .fetch_optional(pool)
            .await
            .map_err(map_err)?;
    let (doc_party,) = doc.ok_or(DbError::NotFound { what: "document" })?;
    if doc_party != tx.party_id {
        return Err(DbError::NotFound { what: "document" }.into());
    }
    let mut txn = pool.begin().await.map_err(map_err)?;
    sqlx::query("delete from finance.transaction_documents where document_id = $1")
        .bind(doc_id)
        .execute(&mut *txn)
        .await
        .map_err(map_err)?;
    sqlx::query(
        "insert into finance.transaction_documents (transaction_id, document_id, source, confidence, reason)
         values ($1, $2, 'declared', 100, 'by_hand')",
    )
    .bind(tx_id)
    .bind(doc_id)
    .execute(&mut *txn)
    .await
    .map_err(map_err)?;
    txn.commit().await.map_err(map_err)?;
    one(pool, access, tx_id).await
}

/// Undo a link. A person's is removed; the matcher's is kept as rejected,
/// so it is not made again and the receipt is free for another transaction.
///
/// # Errors
/// Not in the grant (not found); the database.
pub async fn unlink(
    pool: &PgPool,
    access: &Access,
    tx_id: Uuid,
    doc_id: Uuid,
) -> Result<Row, StoreError> {
    one(pool, access, tx_id).await?;
    sqlx::query(
        "with gone as (
            delete from finance.transaction_documents
             where transaction_id = $1 and document_id = $2 and source = 'declared')
         update finance.transaction_documents set source = 'rejected', confidence = 0
          where transaction_id = $1 and document_id = $2 and source = 'inferred'",
    )
    .bind(tx_id)
    .bind(doc_id)
    .execute(pool)
    .await
    .map_err(map_err)?;
    one(pool, access, tx_id).await
}
