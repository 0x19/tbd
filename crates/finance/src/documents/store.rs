//! Document rows: listing with search, one with its bytes, a person's
//! corrections, and what the reader wrote.

use chrono::{DateTime, NaiveDate, Utc};
use serde_json::Value;
use sqlx::PgPool;
use tbd_db::{Access, DbError, PartyId, map_err};
use uuid::Uuid;

use super::fields::Fields;
use crate::connectors::store::StoreError;

#[allow(missing_docs)]
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DocumentRow {
    pub id: Uuid,
    pub party_id: Uuid,
    pub kind: String,
    pub filename: Option<String>,
    pub content_type: String,
    pub size_bytes: i64,
    pub sha256: String,
    pub vendor: Option<String>,
    pub doc_date: Option<NaiveDate>,
    pub total_minor: Option<i64>,
    pub currency: Option<String>,
    pub invoice_no: Option<String>,
    pub extracted: Value,
    pub extracted_at: Option<DateTime<Utc>>,
    pub declared_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[allow(missing_docs)]
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SourceRow {
    pub document_id: Uuid,
    pub connector_id: Option<Uuid>,
    pub external_ref: String,
    pub subject: String,
    pub sender: String,
    pub received_at: Option<DateTime<Utc>>,
}

/// A vendor and how many documents carry it.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct VendorCount {
    /// As stored.
    pub vendor: String,
    /// Documents in the view with it.
    pub count: i64,
}

const COLUMNS: &str = "d.id, d.party_id, d.kind, d.filename, d.content_type, d.size_bytes, d.sha256, d.vendor,
    d.doc_date, d.total_minor, d.currency, d.invoice_no, d.extracted, d.extracted_at, d.declared_at, d.created_at";

fn sql(s: &str) -> sqlx::AssertSqlSafe<String> {
    sqlx::AssertSqlSafe(s.to_owned())
}

/// What a listing narrows to. Every field optional; the view is not.
#[derive(Debug, Clone, Default)]
pub struct Filter {
    /// `receipt`, `invoice`, or empty.
    pub kind: String,
    /// Free text, matched case-insensitively across the fields and the text.
    pub q: String,
    /// Inclusive date bounds on the effective date.
    pub from: Option<NaiveDate>,
    /// Inclusive.
    pub to: Option<NaiveDate>,
    /// Exact vendor.
    pub vendor: String,
    /// Page.
    pub limit: i64,
    /// Page.
    pub offset: i64,
}

/// A page of a listing, with what the whole match looks like.
#[derive(Debug, Default)]
pub struct Listing {
    /// The page, each with its sources.
    pub documents: Vec<(DocumentRow, Vec<SourceRow>)>,
    /// Matches before paging.
    pub total: i64,
    /// Every vendor in the view with its count, the filter's choices.
    pub vendors: Vec<VendorCount>,
}

/// The documents in view matching `filter`, newest first by their own date
/// (or receipt), with their sources, the match count and the vendor list.
///
/// # Errors
/// The database.
pub async fn list(pool: &PgPool, view: &Access, filter: &Filter) -> Result<Listing, StoreError> {
    if view.is_empty() {
        return Ok(Listing::default());
    }
    // One place the match is written, so the page, the count and the
    // vendors agree. `effective` is the document's date, or the mail's.
    let matching = "from finance.documents d
          left join lateral (
            select min(received_at) as received_at,
                   string_agg(subject, ' ') as subjects,
                   string_agg(sender, ' ') as senders
              from finance.document_sources s where s.document_id = d.id
          ) src on true
         where d.party_id = any($1)
           and ($2 = '' or d.kind = $2)
           and ($3 = '' or d.vendor = $3)
           and ($4::date is null or coalesce(d.doc_date, src.received_at::date, d.created_at::date) >= $4)
           and ($5::date is null or coalesce(d.doc_date, src.received_at::date, d.created_at::date) <= $5)
           and ($6 = '' or concat_ws(' ', d.vendor, d.filename, d.invoice_no, d.currency, src.subjects, src.senders, d.text)
                            ilike '%' || $6 || '%')";
    let like = filter.q.replace('%', "\\%").replace('_', "\\_");
    let documents = sqlx::query_as::<_, DocumentRow>(sql(&format!(
        "select {COLUMNS} {matching}
          order by coalesce(d.doc_date, src.received_at::date, d.created_at::date) desc, d.created_at desc
          limit $7 offset $8"
    )))
    .bind(view.party_ids())
    .bind(&filter.kind)
    .bind(&filter.vendor)
    .bind(filter.from)
    .bind(filter.to)
    .bind(&like)
    .bind(filter.limit)
    .bind(filter.offset)
    .fetch_all(pool)
    .await
    .map_err(map_err)?;
    let (total,): (i64,) = sqlx::query_as(sql(&format!("select count(*) {matching}")))
        .bind(view.party_ids())
        .bind(&filter.kind)
        .bind(&filter.vendor)
        .bind(filter.from)
        .bind(filter.to)
        .bind(&like)
        .fetch_one(pool)
        .await
        .map_err(map_err)?;
    let vendors = sqlx::query_as::<_, VendorCount>(
        "select vendor, count(*) as count from finance.documents
          where party_id = any($1) and ($2 = '' or kind = $2) and vendor is not null and vendor <> ''
          group by vendor order by count desc, vendor",
    )
    .bind(view.party_ids())
    .bind(&filter.kind)
    .fetch_all(pool)
    .await
    .map_err(map_err)?;
    let ids: Vec<Uuid> = documents.iter().map(|d| d.id).collect();
    let sources = sources_of(pool, &ids).await?;
    Ok(Listing {
        documents: documents
            .into_iter()
            .map(|d| {
                let mine = sources
                    .iter()
                    .filter(|s| s.document_id == d.id)
                    .cloned()
                    .collect();
                (d, mine)
            })
            .collect(),
        total,
        vendors,
    })
}

async fn sources_of(pool: &PgPool, ids: &[Uuid]) -> Result<Vec<SourceRow>, StoreError> {
    Ok(sqlx::query_as::<_, SourceRow>(
        "select document_id, connector_id, external_ref, subject, sender, received_at
           from finance.document_sources where document_id = any($1) order by received_at",
    )
    .bind(ids)
    .fetch_all(pool)
    .await
    .map_err(map_err)?)
}

/// One document the caller may see, with its sources.
///
/// # Errors
/// Not in the grant (not found); the database.
pub async fn get(
    pool: &PgPool,
    access: &Access,
    id: Uuid,
) -> Result<(DocumentRow, Vec<SourceRow>), StoreError> {
    let doc = sqlx::query_as::<_, DocumentRow>(sql(&format!(
        "select {COLUMNS} from finance.documents d where d.id = $1"
    )))
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(map_err)?
    .ok_or(DbError::NotFound { what: "document" })?;
    access.require(PartyId(doc.party_id), "document")?;
    let sources = sources_of(pool, &[id]).await?;
    Ok((doc, sources))
}

/// The document's bytes. The caller has checked the grant through [`get`].
///
/// # Errors
/// The database.
pub async fn bytes(pool: &PgPool, id: Uuid) -> Result<Vec<u8>, StoreError> {
    let (bytes,): (Vec<u8>,) =
        sqlx::query_as("select bytes from finance.document_blobs where document_id = $1")
            .bind(id)
            .fetch_one(pool)
            .await
            .map_err(map_err)?;
    Ok(bytes)
}

/// What a person sets. `None` clears a field; the whole set is declared.
#[derive(Debug, Clone, Default)]
pub struct Declared {
    /// The supplier.
    pub vendor: Option<String>,
    /// The document's date.
    pub doc_date: Option<NaiveDate>,
    /// Minor units.
    pub total_minor: Option<i64>,
    /// ISO code.
    pub currency: Option<String>,
    /// The supplier's number.
    pub invoice_no: Option<String>,
}

/// Set the fields by hand. Every field in `declared` is written as given,
/// `declared_at` is stamped, and the record of how each was found says so.
///
/// # Errors
/// Not in the grant (not found); the database.
pub async fn update(
    pool: &PgPool,
    access: &Access,
    id: Uuid,
    declared: &Declared,
) -> Result<(DocumentRow, Vec<SourceRow>), StoreError> {
    let (doc, _) = get(pool, access, id).await?;
    let mut found_by = doc.extracted.as_object().cloned().unwrap_or_default();
    for key in ["vendor", "date", "amount", "invoice_no"] {
        found_by.insert(key.to_owned(), Value::String("declared".into()));
    }
    sqlx::query(
        "update finance.documents
            set vendor = $2, doc_date = $3, total_minor = $4, currency = $5, invoice_no = $6,
                declared_at = now(), extracted = $7
          where id = $1",
    )
    .bind(id)
    .bind(&declared.vendor)
    .bind(declared.doc_date)
    .bind(declared.total_minor)
    .bind(&declared.currency)
    .bind(&declared.invoice_no)
    .bind(Value::Object(found_by))
    .execute(pool)
    .await
    .map_err(map_err)?;
    get(pool, access, id).await
}

/// What the reader needs from a row: the bytes, the mail it came in, and
/// whether a person has already spoken.
#[derive(Debug)]
pub struct ToRead {
    /// The document.
    pub id: Uuid,
    /// The earliest source: sender and receipt time.
    pub sender: String,
    /// When the mail arrived, the date of last resort.
    pub received: Option<NaiveDate>,
    /// Fields were set by hand; the reader leaves them.
    pub declared: bool,
}

/// What the reader needs for `id`, if the document exists.
///
/// # Errors
/// The database.
pub async fn to_read(pool: &PgPool, id: Uuid) -> Result<Option<ToRead>, StoreError> {
    let row: Option<(Uuid, Option<DateTime<Utc>>)> =
        sqlx::query_as("select id, declared_at from finance.documents where id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(map_err)?;
    let Some((id, declared_at)) = row else {
        return Ok(None);
    };
    let source = sources_of(pool, &[id]).await?.into_iter().next();
    Ok(Some(ToRead {
        id,
        sender: source
            .as_ref()
            .map(|s| s.sender.clone())
            .unwrap_or_default(),
        received: source.and_then(|s| s.received_at).map(|t| t.date_naive()),
        declared: declared_at.is_some(),
    }))
}

/// Write what the reader found. Declared fields are left; the text and the
/// record of the read are always written, so search works and the page can
/// say when the document was last read and with what.
///
/// # Errors
/// The database.
pub async fn write_read(
    pool: &PgPool,
    id: Uuid,
    text: Option<&str>,
    fields: &Fields,
    declared: bool,
    engine: &str,
    error: Option<&str>,
) -> Result<(), StoreError> {
    let mut record = serde_json::Map::new();
    record.insert("engine".into(), Value::String(engine.to_owned()));
    if let Some(e) = error {
        record.insert("error".into(), Value::String(e.to_owned()));
    }
    let by = |key: &str, by: Option<super::fields::By>| {
        (
            key.to_owned(),
            Value::String(if declared {
                "declared".into()
            } else {
                by.map(|b| b.as_str().to_owned()).unwrap_or_default()
            }),
        )
    };
    record.extend([
        by("vendor", fields.vendor.as_ref().map(|v| v.1)),
        by("date", fields.date.as_ref().map(|v| v.1)),
        by("amount", fields.amount.as_ref().map(|v| v.2)),
        by("invoice_no", fields.invoice_no.as_ref().map(|v| v.1)),
    ]);
    if declared {
        sqlx::query(
            "update finance.documents set text = coalesce($2, text), extracted = $3, extracted_at = now()
              where id = $1",
        )
        .bind(id)
        .bind(text)
        .bind(Value::Object(record))
        .execute(pool)
        .await
        .map_err(map_err)?;
        return Ok(());
    }
    sqlx::query(
        "update finance.documents
            set text = coalesce($2, text), extracted = $3, extracted_at = now(),
                vendor = $4, doc_date = $5, total_minor = $6, currency = $7, invoice_no = $8
          where id = $1",
    )
    .bind(id)
    .bind(text)
    .bind(Value::Object(record))
    .bind(fields.vendor.as_ref().map(|v| &v.0))
    .bind(fields.date.as_ref().map(|v| v.0))
    .bind(fields.amount.as_ref().map(|v| v.0))
    .bind(fields.amount.as_ref().map(|v| &v.1))
    .bind(fields.invoice_no.as_ref().map(|v| &v.0))
    .execute(pool)
    .await
    .map_err(map_err)?;
    Ok(())
}

/// Documents the reader has never run on, oldest first.
///
/// # Errors
/// The database.
pub async fn unread(pool: &PgPool, limit: i64) -> Result<Vec<Uuid>, StoreError> {
    let rows: Vec<(Uuid,)> = sqlx::query_as(
        "select id from finance.documents where extracted_at is null order by created_at limit $1",
    )
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(map_err)?;
    Ok(rows.into_iter().map(|(id,)| id).collect())
}
