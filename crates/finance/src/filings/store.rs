//! Filing rows: written by the reader, listed and fetched for the pages.

use chrono::{DateTime, Datelike, NaiveDate, Utc};
use serde_json::Value;
use sqlx::PgPool;
use tbd_db::{Access, DbError, PartyId, map_err};
use uuid::Uuid;

use super::{Form, PARSER_VERSION, Parsed};
use crate::connectors::store::StoreError;

/// A filing as stored, with the document's file name beside it.
#[allow(missing_docs)]
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct FilingRow {
    pub document_id: Uuid,
    pub party_id: Uuid,
    pub form: String,
    pub schema: String,
    pub period_from: Option<NaiveDate>,
    pub period_to: Option<NaiveDate>,
    pub oib: String,
    pub obveznik: String,
    pub prepared_at: Option<DateTime<Utc>>,
    pub author: String,
    pub identifier: String,
    pub report_mark: String,
    pub values: Value,
    pub rows: Value,
    pub parsed_at: DateTime<Utc>,
    pub parser_version: String,
    pub error: Option<String>,
    pub filename: Option<String>,
}

const COLUMNS: &str = "f.document_id, f.party_id, f.form, f.schema, f.period_from, f.period_to, f.oib, f.obveznik,
    f.prepared_at, f.author, f.identifier, f.report_mark, f.values, f.rows, f.parsed_at, f.parser_version, f.error,
    d.filename";

/// What a listing narrows by.
#[derive(Debug, Clone, Default)]
pub struct Filter {
    /// One form, or any.
    pub form: Option<Form>,
    /// The year the period starts in, or any.
    pub year: Option<i32>,
    /// Page size.
    pub limit: i64,
    /// Page start.
    pub offset: i64,
}

/// A page of filings.
#[derive(Debug, Clone, Default)]
pub struct Listing {
    /// The page, newest period first.
    pub filings: Vec<FilingRow>,
    /// Matches before limit and offset.
    pub total: i64,
    /// Every year with a filing in view, newest first.
    pub years: Vec<i32>,
}

fn sql(s: &str) -> sqlx::AssertSqlSafe<String> {
    sqlx::AssertSqlSafe(s.to_owned())
}

/// The party's OIB, when the party is an organisation.
///
/// # Errors
/// The database.
pub async fn org_oib(pool: &PgPool, party_id: Uuid) -> Result<Option<String>, StoreError> {
    let row: Option<(Option<String>,)> =
        sqlx::query_as("select oib from public.orgs where id = $1")
            .bind(party_id)
            .fetch_optional(pool)
            .await
            .map_err(map_err)?;
    Ok(row.and_then(|(oib,)| oib).filter(|o| !o.trim().is_empty()))
}

/// Whose a document is, if it exists.
///
/// # Errors
/// The database.
pub async fn party_of(pool: &PgPool, id: Uuid) -> Result<Option<Uuid>, StoreError> {
    let row: Option<(Uuid,)> =
        sqlx::query_as("select party_id from finance.documents where id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(map_err)?;
    Ok(row.map(|(p,)| p))
}

/// One line for the document's text, so the receipts search reaches it.
fn summary(p: &Parsed) -> String {
    let period = match (p.header.period_from, p.header.period_to) {
        (Some(a), Some(b)) if a == b => a.to_string(),
        (Some(a), Some(b)) => format!("{a} to {b}"),
        (Some(a), None) | (None, Some(a)) => a.to_string(),
        (None, None) => String::new(),
    };
    format!(
        "{} {} {} OIB {} {}",
        p.form.label(),
        p.header.schema,
        period,
        p.header.oib,
        p.header.obveznik
    )
    .split_whitespace()
    .collect::<Vec<_>>()
    .join(" ")
}

/// Write what the reader found: the filing row (replacing an earlier read)
/// and the document's read record.
///
/// # Errors
/// The database.
pub async fn write(pool: &PgPool, id: Uuid, party_id: Uuid, p: &Parsed) -> Result<(), StoreError> {
    let values = Value::Object(
        p.values
            .iter()
            .map(|(k, v)| (k.clone(), Value::String(v.clone())))
            .collect(),
    );
    let mut tx = pool.begin().await.map_err(map_err)?;
    sqlx::query(
        "insert into finance.filings
            (document_id, party_id, form, schema, period_from, period_to, oib, obveznik, prepared_at,
             author, identifier, report_mark, values, rows, parsed_at, parser_version, error)
         values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, now(), $15, $16)
         on conflict (document_id) do update
            set party_id = excluded.party_id, form = excluded.form, schema = excluded.schema,
                period_from = excluded.period_from, period_to = excluded.period_to, oib = excluded.oib,
                obveznik = excluded.obveznik, prepared_at = excluded.prepared_at, author = excluded.author,
                identifier = excluded.identifier, report_mark = excluded.report_mark, values = excluded.values,
                rows = excluded.rows, parsed_at = now(), parser_version = excluded.parser_version,
                error = excluded.error",
    )
    .bind(id)
    .bind(party_id)
    .bind(p.form.as_str())
    .bind(&p.header.schema)
    .bind(p.header.period_from)
    .bind(p.header.period_to)
    .bind(&p.header.oib)
    .bind(&p.header.obveznik)
    .bind(p.header.prepared_at)
    .bind(&p.header.author)
    .bind(&p.header.identifier)
    .bind(&p.header.report_mark)
    .bind(values)
    .bind(Value::Array(p.rows.clone()))
    .bind(PARSER_VERSION)
    .bind(&p.error)
    .execute(&mut *tx)
    .await
    .map_err(map_err)?;
    let mut record = serde_json::Map::new();
    record.insert("engine".into(), Value::String(PARSER_VERSION.into()));
    if let Some(e) = &p.error {
        record.insert("error".into(), Value::String(e.clone()));
    }
    sqlx::query(
        "update finance.documents
            set text = $2, extracted_at = now(),
                extracted = (coalesce(extracted, '{}'::jsonb) - 'engine' - 'error') || $3
          where id = $1",
    )
    .bind(id)
    .bind(summary(p))
    .bind(Value::Object(record))
    .execute(&mut *tx)
    .await
    .map_err(map_err)?;
    tx.commit().await.map_err(map_err)?;
    Ok(())
}

/// The document could not be read as a form: say so on the document and
/// count it as read, so it is not retried forever. An earlier filing row,
/// if any, is dropped: the bytes no longer support it.
///
/// # Errors
/// The database.
pub async fn write_unread(pool: &PgPool, id: Uuid, error: &str) -> Result<(), StoreError> {
    let mut tx = pool.begin().await.map_err(map_err)?;
    sqlx::query("delete from finance.filings where document_id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(map_err)?;
    sqlx::query(
        "update finance.documents
            set extracted_at = now(),
                extracted = (coalesce(extracted, '{}'::jsonb) - 'engine' - 'error')
                            || jsonb_build_object('engine', $2::text, 'error', $3::text)
          where id = $1",
    )
    .bind(id)
    .bind(PARSER_VERSION)
    .bind(error)
    .execute(&mut *tx)
    .await
    .map_err(map_err)?;
    tx.commit().await.map_err(map_err)?;
    Ok(())
}

/// The filings the caller may see, newest period first.
///
/// # Errors
/// The database.
pub async fn list(pool: &PgPool, view: &Access, filter: &Filter) -> Result<Listing, StoreError> {
    if view.is_empty() {
        return Ok(Listing::default());
    }
    let form = filter.form.map(Form::as_str).unwrap_or_default();
    let matching = "from finance.filings f join finance.documents d on d.id = f.document_id
         where f.party_id = any($1)
           and ($2 = '' or f.form = $2)
           and ($3::int is null or extract(year from f.period_from)::int = $3)";
    let filings = sqlx::query_as::<_, FilingRow>(sql(&format!(
        "select {COLUMNS} {matching}
          order by f.period_from desc nulls last, f.prepared_at desc nulls last, f.parsed_at desc
          limit $4 offset $5"
    )))
    .bind(view.party_ids())
    .bind(form)
    .bind(filter.year)
    .bind(filter.limit)
    .bind(filter.offset)
    .fetch_all(pool)
    .await
    .map_err(map_err)?;
    let (total,): (i64,) = sqlx::query_as(sql(&format!("select count(*) {matching}")))
        .bind(view.party_ids())
        .bind(form)
        .bind(filter.year)
        .fetch_one(pool)
        .await
        .map_err(map_err)?;
    let years: Vec<(i32,)> = sqlx::query_as(
        "select distinct extract(year from period_from)::int as y from finance.filings
          where party_id = any($1) and period_from is not null order by y desc",
    )
    .bind(view.party_ids())
    .fetch_all(pool)
    .await
    .map_err(map_err)?;
    Ok(Listing {
        filings,
        total,
        years: years.into_iter().map(|(y,)| y).collect(),
    })
}

/// One filing the caller may see.
///
/// # Errors
/// Not in the grant (not found); the database.
pub async fn get(pool: &PgPool, access: &Access, id: Uuid) -> Result<FilingRow, StoreError> {
    let row = sqlx::query_as::<_, FilingRow>(sql(&format!(
        "select {COLUMNS} from finance.filings f join finance.documents d on d.id = f.document_id
          where f.document_id = $1"
    )))
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(map_err)?
    .ok_or(DbError::NotFound { what: "filing" })?;
    access.require(PartyId(row.party_id), "filing")?;
    Ok(row)
}

/// The year a row's period starts in, for the page.
#[must_use]
pub fn year_of(row: &FilingRow) -> Option<i32> {
    row.period_from.map(|d| d.year())
}
