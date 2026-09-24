//! Pulled documents, and what they say.
//!
//! A receipt arrives as bytes from a connector. The reader turns those into
//! text ([`pdf`]) and the text into fields ([`fields`]): vendor, date,
//! amount, number. It runs on every document as it is stored, on every
//! document it has never seen at start-up, and again on request. A person's
//! corrections are declared and outlive any re-read. A document of kind
//! `filing` (an ePorezna form) is handed to [`crate::filings`] instead. A
//! bank statement ([`statement`]) is, besides, read into bank transactions.

pub mod fields;
pub mod mail;
pub mod party;
pub mod pdf;
pub mod statement;
pub mod store;

use sqlx::PgPool;
use uuid::Uuid;

use crate::connectors::store::StoreError;

/// Read one document and write what was found. Never fails on the document
/// itself: an unreadable PDF records why and counts as read, so it is not
/// retried forever. Fails only on the database.
///
/// # Errors
/// The database.
pub async fn read(pool: &PgPool, id: Uuid) -> Result<(), StoreError> {
    let Some(to_read) = store::to_read(pool, id).await? else {
        return Ok(());
    };
    // An ePorezna form has its own reader, and no vendor, date or amount in
    // the receipt sense to look for.
    if to_read.kind == "filing" {
        return crate::filings::read(pool, id).await;
    }
    let bytes = store::bytes(pool, id).await?;
    let (text, error) = match pdf::text(bytes).await {
        Ok(t) => (Some(t), None),
        Err(e) => (None, Some(e.to_string())),
    };
    let found = text
        .as_deref()
        .map(|t| fields::read(t, &to_read.sender, to_read.received, &to_read.filename))
        .unwrap_or_default();
    // With no text there is still the mail: who sent it, and when.
    let found = if text.is_none() {
        fields::read("", &to_read.sender, to_read.received, &to_read.filename)
    } else {
        found
    };
    tracing::debug!(
        document = %id,
        vendor = ?found.vendor,
        date = ?found.date,
        amount = ?found.amount,
        invoice_no = ?found.invoice_no,
        error,
        "document read"
    );
    store::write_read(
        pool,
        id,
        text.as_deref(),
        &found,
        to_read.declared,
        pdf::ENGINE,
        error.as_deref(),
    )
    .await?;
    // A bank statement is also the bank's record: its entries become bank
    // transactions of the account it names, through the one ingest path.
    if let Some(t) = text.as_deref().filter(|t| statement::is_statement(t)) {
        if let Some(done) = statement::ingest(pool, t, &to_read.filename).await? {
            tracing::info!(
                document = %id,
                inserted = done.imported.inserted,
                duplicates = done.imported.duplicates,
                already_in_feed = done.already_in_feed,
                unresolved_days = done.unresolved_days,
                undirected = done.undirected,
                "statement read into bank transactions"
            );
        } else {
            tracing::debug!(document = %id, "statement of an account not kept here");
        }
    }
    // Whose it is, from the amount, the date and the text just written.
    if let Some(moved) = party::assign(pool, id).await? {
        tracing::debug!(document = %id, party = %moved.party_id, by = moved.by.as_str(), "document party");
    }
    Ok(())
}

/// Read every document the reader has never run on. For start-up, after a
/// deploy that brought the reader or improved it; one batch at a time so a
/// large backlog neither loads everything nor starves the pool.
///
/// # Errors
/// The database.
pub async fn backfill(pool: &PgPool) -> Result<usize, StoreError> {
    let mut done = 0;
    let mut failed = std::collections::HashSet::new();
    loop {
        // A document whose read failed on the database side (not the
        // reader's: that is recorded and counts as read) stays unread and
        // would come back every batch; it is skipped for this run.
        let batch: Vec<Uuid> = store::unread(pool, 50)
            .await?
            .into_iter()
            .filter(|id| !failed.contains(id))
            .collect();
        if batch.is_empty() {
            return Ok(done);
        }
        for id in batch {
            match read(pool, id).await {
                Ok(()) => done += 1,
                Err(e) => {
                    tracing::warn!(document = %id, error = %e, "backfill: document not read");
                    failed.insert(id);
                }
            }
        }
    }
}
