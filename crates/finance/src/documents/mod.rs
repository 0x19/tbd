//! Pulled documents, and what they say.
//!
//! A receipt arrives as bytes from a connector. The reader turns those into
//! text ([`pdf`]) and the text into fields ([`fields`]): vendor, date,
//! amount, number. It runs on every document as it is stored, on every
//! document it has never seen at start-up, and again on request. A person's
//! corrections are declared and outlive any re-read.

pub mod fields;
pub mod pdf;
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
    let bytes = store::bytes(pool, id).await?;
    let (text, error) = match pdf::text(bytes).await {
        Ok(t) => (Some(t), None),
        Err(e) => (None, Some(e.to_string())),
    };
    let found = text
        .as_deref()
        .map(|t| fields::read(t, &to_read.sender, to_read.received))
        .unwrap_or_default();
    // With no text there is still the mail: who sent it, and when.
    let found = if text.is_none() {
        fields::read("", &to_read.sender, to_read.received)
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
    .await
}

/// Read every document the reader has never run on. For start-up, after a
/// deploy that brought the reader or improved it; one batch at a time so a
/// large backlog neither loads everything nor starves the pool.
///
/// # Errors
/// The database.
pub async fn backfill(pool: &PgPool) -> Result<usize, StoreError> {
    let mut done = 0;
    loop {
        let batch = store::unread(pool, 50).await?;
        if batch.is_empty() {
            return Ok(done);
        }
        for id in batch {
            read(pool, id).await?;
            done += 1;
        }
    }
}
