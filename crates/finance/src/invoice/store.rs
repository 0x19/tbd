//! Drafts, previews, approvals and the rows behind them.
//!
//! Every function takes an [`Access`] and refuses a row outside it as
//! not-found, like the rest of the service. The one transaction that matters
//! is [`approve`]: lock the draft, rebuild the document, compare the hash the
//! approver saw, take the number, render, store the PDF, write the approval
//! -- all or nothing.

use chrono::{DateTime, Datelike, NaiveDate, Utc};
use chrono_tz::Europe::Zagreb;
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Postgres, Transaction};
use tbd_db::{Access, DbError, PartyId, UserId, map_err};
use uuid::Uuid;

use super::{Client, InvoiceDoc, Issuer, Line, VatTreatment, numbering, render, totals};

/// Why an invoicing call was refused.
#[derive(Debug, thiserror::Error)]
pub enum InvoiceError {
    /// The database, or a row outside the grant (not-found).
    #[error(transparent)]
    Db(#[from] DbError),
    /// The invoice is not a draft, and only drafts change.
    #[error("invoice is {0}, not a draft")]
    NotDraft(String),
    /// The draft changed since the preview the approver saw.
    #[error("the draft changed since it was previewed; preview again")]
    StaleDraft,
    /// A draft is deleted, not cancelled: it took no number.
    #[error("a draft is deleted, not cancelled")]
    IsDraft,
    /// A header value that is not one.
    #[error("{0}")]
    Invalid(String),
    /// The issuing party has no issuer profile yet.
    #[error("no issuer profile for this party; set one first")]
    NoIssuer,
    /// A draft with no lines cannot be approved.
    #[error("an invoice needs at least one line")]
    NoLines,
    /// The renderer.
    #[error(transparent)]
    Render(#[from] render::RenderError),
    /// Serialisation of the canonical document.
    #[error("canonical document: {0}")]
    Canonical(#[from] serde_json::Error),
}

#[allow(missing_docs)]
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct IssuerRow {
    pub party_id: Uuid,
    pub legal_name: String,
    pub address_lines: Vec<String>,
    pub oib: String,
    pub vat_id: String,
    pub iban: String,
    pub swift: String,
    pub bank_name: String,
    pub court: String,
    pub registration_no: String,
    pub share_capital: String,
    pub board_member: String,
    pub issued_by: String,
    pub place_of_issue: String,
    pub operator_id: String,
    pub premises: String,
    pub device: String,
    pub due_days: i32,
}

impl IssuerRow {
    fn doc(&self) -> Issuer {
        Issuer {
            legal_name: self.legal_name.clone(),
            address_lines: self.address_lines.clone(),
            oib: self.oib.clone(),
            vat_id: self.vat_id.clone(),
            iban: self.iban.clone(),
            swift: self.swift.clone(),
            bank_name: self.bank_name.clone(),
            court: self.court.clone(),
            registration_no: self.registration_no.clone(),
            share_capital: self.share_capital.clone(),
            board_member: self.board_member.clone(),
            issued_by: self.issued_by.clone(),
            operator_id: self.operator_id.clone(),
        }
    }
}

#[allow(missing_docs)]
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ClientRow {
    pub id: Uuid,
    pub party_id: Uuid,
    pub name: String,
    pub address_lines: Vec<String>,
    pub country_code: String,
    pub tax_id: String,
    pub vat_treatment: String,
    pub recipients: Vec<String>,
    pub currency: String,
    pub archived_at: Option<DateTime<Utc>>,
    /// The company's default client: the invoices list opens on it and a
    /// new draft is for it. One per party.
    pub is_default: bool,
}

#[allow(missing_docs)]
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct InvoiceRow {
    pub id: Uuid,
    pub party_id: Uuid,
    pub client_id: Uuid,
    pub status: String,
    pub year: i32,
    pub ordinal: Option<i32>,
    pub premises: String,
    pub device: String,
    pub number: Option<String>,
    pub issued_at: Option<DateTime<Utc>>,
    pub delivery_date: NaiveDate,
    pub due_date: NaiveDate,
    pub place_of_issue: String,
    pub currency: String,
    pub subtotal_minor: i64,
    pub vat_minor: i64,
    pub total_minor: i64,
    pub vat_treatment: String,
    pub vat_note: String,
    pub note: String,
    pub content_hash: Option<String>,
    pub approved_at: Option<DateTime<Utc>>,
    pub document_id: Option<Uuid>,
    pub prefilled_from: Option<Uuid>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// What the payments cover so far, and when they covered it all.
    pub paid_minor: i64,
    pub paid_at: Option<DateTime<Utc>>,
    /// The first time it went out to the client.
    pub sent_at: Option<DateTime<Utc>>,
}

/// One time an invoice went out: the mail that carried it, to whom, when.
#[allow(missing_docs)]
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DeliveryRow {
    pub id: Uuid,
    pub invoice_id: Uuid,
    pub mail_id: Option<Uuid>,
    pub to_addrs: Vec<String>,
    pub sent_at: DateTime<Utc>,
    /// `invoice` (it went out) or `reminder` (it is still owed).
    pub kind: String,
}

#[allow(missing_docs)]
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct LineRow {
    pub id: Uuid,
    pub invoice_id: Uuid,
    pub position: i32,
    pub description: String,
    pub quantity_milli: i64,
    pub unit_price_minor: i64,
    pub amount_minor: i64,
    pub template_id: Option<Uuid>,
}

#[allow(missing_docs)]
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TemplateRow {
    pub id: Uuid,
    pub client_id: Uuid,
    pub position: i32,
    pub description: String,
    pub mode: String,
    pub quantity_milli: i64,
    pub unit_price_minor: i64,
    pub enabled: bool,
}

/// What a person writes on a line template.
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct TemplateInput {
    pub id: Option<Uuid>,
    pub client_id: Uuid,
    pub position: i32,
    pub description: String,
    pub mode: String,
    pub quantity_milli: i64,
    pub unit_price_minor: i64,
    pub enabled: bool,
}

/// What a person writes on an issuer profile.
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct IssuerInput {
    pub party_id: Uuid,
    pub legal_name: String,
    pub address_lines: Vec<String>,
    pub oib: String,
    pub vat_id: String,
    pub iban: String,
    pub swift: String,
    pub bank_name: String,
    pub court: String,
    pub registration_no: String,
    pub share_capital: String,
    pub board_member: String,
    pub issued_by: String,
    pub place_of_issue: String,
    pub operator_id: String,
    pub premises: String,
    pub device: String,
    pub due_days: i32,
}

/// What a person writes on a client.
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct ClientInput {
    pub id: Option<Uuid>,
    pub party_id: Uuid,
    pub name: String,
    pub address_lines: Vec<String>,
    pub country_code: String,
    pub tax_id: String,
    pub vat_treatment: VatTreatment,
    pub recipients: Vec<String>,
    pub currency: String,
}

/// What a person writes on a draft.
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct DraftInput {
    pub delivery_date: NaiveDate,
    pub due_date: NaiveDate,
    pub place_of_issue: String,
    pub note: String,
    pub lines: Vec<LineInput>,
    /// The header, each `None` meaning "as it is": the client (same party),
    /// the currency, the VAT treatment (its note follows), the series.
    pub client_id: Option<Uuid>,
    pub currency: Option<String>,
    pub vat_treatment: Option<VatTreatment>,
    pub premises: Option<String>,
    pub device: Option<String>,
}

/// One line as written.
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct LineInput {
    pub description: String,
    pub quantity_milli: i64,
    pub unit_price_minor: i64,
    pub template_id: Option<Uuid>,
}

/// A preview: the document, its hash, and the PDF.
#[derive(Debug, Clone)]
pub struct Preview {
    /// The canonical document the hash names.
    pub doc: InvoiceDoc,
    /// What `approve` must be given back.
    pub content_hash: String,
    /// Watermarked PDF bytes.
    pub pdf: Vec<u8>,
}

/// A statement assembled from constant column lists and nothing else.
pub(crate) fn sql(s: &str) -> sqlx::AssertSqlSafe<String> {
    sqlx::AssertSqlSafe(s.to_owned())
}

const ISSUER_COLUMNS: &str =
    "party_id, legal_name, address_lines, oib, vat_id, iban, swift, bank_name, court,
    registration_no, share_capital, board_member, issued_by, place_of_issue, operator_id, premises,
    device, due_days";
pub(crate) const CLIENT_COLUMNS: &str = "id, party_id, name, address_lines, country_code, tax_id, vat_treatment, recipients, currency, archived_at, is_default";
pub(crate) const INVOICE_COLUMNS: &str =
    "id, party_id, client_id, status, year, ordinal, premises, device, number, issued_at,
    delivery_date, due_date, place_of_issue, currency, subtotal_minor, vat_minor, total_minor,
    vat_treatment, vat_note, note, content_hash, approved_at, document_id, prefilled_from,
    cancelled_at, created_at, updated_at, paid_minor, paid_at, sent_at";

/// The issuer profile of a party the caller may read.
///
/// # Errors
/// The database, or the party is outside the grant.
pub async fn issuer(
    pool: &PgPool,
    access: &Access,
    party: Uuid,
) -> Result<Option<IssuerRow>, InvoiceError> {
    access.require(PartyId(party), "party")?;
    Ok(sqlx::query_as::<_, IssuerRow>(sql(&format!(
        "select {ISSUER_COLUMNS} from finance.issuers where party_id = $1"
    )))
    .bind(party)
    .fetch_optional(pool)
    .await
    .map_err(map_err)?)
}

/// The deliveries of these invoices, oldest first.
///
/// # Errors
/// The database.
pub async fn deliveries_of(
    pool: &PgPool,
    invoice_ids: &[Uuid],
) -> Result<Vec<DeliveryRow>, DbError> {
    if invoice_ids.is_empty() {
        return Ok(Vec::new());
    }
    sqlx::query_as::<_, DeliveryRow>(
        "select id, invoice_id, mail_id, to_addrs, sent_at, kind from finance.invoice_deliveries
          where invoice_id = any($1) order by sent_at",
    )
    .bind(invoice_ids)
    .fetch_all(pool)
    .await
    .map_err(map_err)
}

/// The issuer profiles of every party in the view.
///
/// # Errors
/// The database.
pub async fn issuers(pool: &PgPool, view: &Access) -> Result<Vec<IssuerRow>, InvoiceError> {
    if view.is_empty() {
        return Ok(Vec::new());
    }
    Ok(sqlx::query_as::<_, IssuerRow>(sql(&format!(
        "select {ISSUER_COLUMNS} from finance.issuers where party_id = any($1) order by legal_name"
    )))
    .bind(view.party_ids())
    .fetch_all(pool)
    .await
    .map_err(map_err)?)
}

/// Create or replace the issuer profile.
///
/// # Errors
/// The database, or the party is outside the grant.
pub async fn upsert_issuer(
    pool: &PgPool,
    access: &Access,
    input: IssuerInput,
) -> Result<IssuerRow, InvoiceError> {
    access.require(PartyId(input.party_id), "party")?;
    sqlx::query(
        "insert into finance.issuers (party_id, legal_name, address_lines, oib, vat_id, iban, swift,
            bank_name, court, registration_no, share_capital, board_member, issued_by, place_of_issue,
            operator_id, premises, device, due_days)
         values ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18)
         on conflict (party_id) do update set
            legal_name = excluded.legal_name, address_lines = excluded.address_lines, oib = excluded.oib,
            vat_id = excluded.vat_id, iban = excluded.iban, swift = excluded.swift,
            bank_name = excluded.bank_name, court = excluded.court,
            registration_no = excluded.registration_no, share_capital = excluded.share_capital,
            board_member = excluded.board_member, issued_by = excluded.issued_by,
            place_of_issue = excluded.place_of_issue, operator_id = excluded.operator_id,
            premises = excluded.premises, device = excluded.device, due_days = excluded.due_days,
            updated_at = clock_timestamp()",
    )
    .bind(input.party_id)
    .bind(&input.legal_name)
    .bind(&input.address_lines)
    .bind(&input.oib)
    .bind(&input.vat_id)
    .bind(&input.iban)
    .bind(&input.swift)
    .bind(&input.bank_name)
    .bind(&input.court)
    .bind(&input.registration_no)
    .bind(&input.share_capital)
    .bind(&input.board_member)
    .bind(&input.issued_by)
    .bind(&input.place_of_issue)
    .bind(&input.operator_id)
    .bind(&input.premises)
    .bind(&input.device)
    .bind(input.due_days)
    .execute(pool)
    .await
    .map_err(map_err)?;
    issuer(pool, access, input.party_id)
        .await?
        .ok_or(InvoiceError::NoIssuer)
}

/// Clients of the parties in the view.
///
/// # Errors
/// The database.
pub async fn clients(pool: &PgPool, view: &Access) -> Result<Vec<ClientRow>, InvoiceError> {
    if view.is_empty() {
        return Ok(Vec::new());
    }
    Ok(sqlx::query_as::<_, ClientRow>(sql(&format!(
        "select {CLIENT_COLUMNS} from finance.clients where party_id = any($1) order by name"
    )))
    .bind(view.party_ids())
    .fetch_all(pool)
    .await
    .map_err(map_err)?)
}

const TEMPLATE_COLUMNS: &str =
    "id, client_id, position, description, mode, quantity_milli, unit_price_minor, enabled";

/// Whose a client is, if the caller may know.
async fn client_party(pool: &PgPool, access: &Access, client: Uuid) -> Result<Uuid, DbError> {
    let row: Option<(Uuid,)> = sqlx::query_as("select party_id from finance.clients where id = $1")
        .bind(client)
        .fetch_optional(pool)
        .await
        .map_err(map_err)?;
    let party = row
        .map(|(p,)| p)
        .ok_or(DbError::NotFound { what: "client" })?;
    access.require(PartyId(party), "client")?;
    Ok(party)
}

/// A client's line templates, in order, disabled ones included and marked.
///
/// # Errors
/// The database, or the client is outside the grant.
pub async fn templates(
    pool: &PgPool,
    access: &Access,
    client: Uuid,
) -> Result<Vec<TemplateRow>, InvoiceError> {
    client_party(pool, access, client).await?;
    Ok(sqlx::query_as::<_, TemplateRow>(sql(&format!(
        "select {TEMPLATE_COLUMNS} from finance.line_templates where client_id = $1 order by position, created_at"
    )))
    .bind(client)
    .fetch_all(pool)
    .await
    .map_err(map_err)?)
}

/// Create or change a line template.
///
/// # Errors
/// The database; the client or template is outside the grant; bad input.
pub async fn upsert_template(
    pool: &PgPool,
    access: &Access,
    input: TemplateInput,
) -> Result<TemplateRow, InvoiceError> {
    client_party(pool, access, input.client_id).await?;
    if !matches!(input.mode.as_str(), "fixed" | "variable" | "optional") {
        return Err(DbError::Invalid {
            field: "mode",
            reason: "want fixed, variable or optional".into(),
        }
        .into());
    }
    if input.description.trim().is_empty() {
        return Err(DbError::Invalid {
            field: "description",
            reason: "empty".into(),
        }
        .into());
    }
    if input.quantity_milli <= 0 {
        return Err(DbError::Invalid {
            field: "quantity",
            reason: "must be positive".into(),
        }
        .into());
    }
    let id = match input.id {
        Some(id) => {
            let owned: Option<(Uuid,)> = sqlx::query_as(
                "select id from finance.line_templates where id = $1 and client_id = $2",
            )
            .bind(id)
            .bind(input.client_id)
            .fetch_optional(pool)
            .await
            .map_err(map_err)?;
            owned.ok_or(DbError::NotFound { what: "template" })?;
            id
        }
        None => Uuid::new_v4(),
    };
    sqlx::query(
        "insert into finance.line_templates (id, client_id, position, description, mode, quantity_milli,
            unit_price_minor, enabled)
         values ($1,$2,$3,$4,$5,$6,$7,$8)
         on conflict (id) do update set position = excluded.position, description = excluded.description,
            mode = excluded.mode, quantity_milli = excluded.quantity_milli,
            unit_price_minor = excluded.unit_price_minor, enabled = excluded.enabled,
            updated_at = clock_timestamp()",
    )
    .bind(id)
    .bind(input.client_id)
    .bind(input.position)
    .bind(input.description.trim())
    .bind(&input.mode)
    .bind(input.quantity_milli)
    .bind(input.unit_price_minor)
    .bind(input.enabled)
    .execute(pool)
    .await
    .map_err(map_err)?;
    Ok(sqlx::query_as::<_, TemplateRow>(sql(&format!(
        "select {TEMPLATE_COLUMNS} from finance.line_templates where id = $1"
    )))
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(map_err)?)
}

/// Remove a line template. Lines already drawn from it keep their text and
/// lose the link.
///
/// # Errors
/// The database, or the client or template is outside the grant.
pub async fn delete_template(
    pool: &PgPool,
    access: &Access,
    client: Uuid,
    id: Uuid,
) -> Result<(), InvoiceError> {
    client_party(pool, access, client).await?;
    let n = sqlx::query("delete from finance.line_templates where id = $1 and client_id = $2")
        .bind(id)
        .bind(client)
        .execute(pool)
        .await
        .map_err(map_err)?
        .rows_affected();
    if n == 0 {
        return Err(DbError::NotFound { what: "template" }.into());
    }
    Ok(())
}

/// Create or change a client.
///
/// # Errors
/// The database, or the party or existing client is outside the grant.
pub async fn upsert_client(
    pool: &PgPool,
    access: &Access,
    input: ClientInput,
) -> Result<ClientRow, InvoiceError> {
    access.require(PartyId(input.party_id), "party")?;
    if input.name.trim().is_empty() {
        return Err(DbError::Invalid {
            field: "name",
            reason: "empty".into(),
        }
        .into());
    }
    let id = match input.id {
        Some(id) => {
            let owned: Option<(Uuid,)> =
                sqlx::query_as("select id from finance.clients where id = $1 and party_id = $2")
                    .bind(id)
                    .bind(input.party_id)
                    .fetch_optional(pool)
                    .await
                    .map_err(map_err)?;
            owned.ok_or(DbError::NotFound { what: "client" })?;
            id
        }
        None => Uuid::new_v4(),
    };
    sqlx::query(
        "insert into finance.clients (id, party_id, name, address_lines, country_code, tax_id, vat_treatment,
            recipients, currency)
         values ($1,$2,$3,$4,$5,$6,$7,$8,$9)
         on conflict (id) do update set name = excluded.name, address_lines = excluded.address_lines,
            country_code = excluded.country_code, tax_id = excluded.tax_id,
            vat_treatment = excluded.vat_treatment, recipients = excluded.recipients,
            currency = excluded.currency",
    )
    .bind(id)
    .bind(input.party_id)
    .bind(input.name.trim())
    .bind(&input.address_lines)
    .bind(input.country_code.to_uppercase())
    .bind(input.tax_id.trim())
    .bind(input.vat_treatment.as_str())
    .bind(&input.recipients)
    .bind(input.currency.to_uppercase())
    .execute(pool)
    .await
    .map_err(map_err)?;
    let row = sqlx::query_as::<_, ClientRow>(sql(&format!(
        "select {CLIENT_COLUMNS} from finance.clients where id = $1"
    )))
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(map_err)?;
    Ok(row)
}

/// Invoices in the view, newest first.
///
/// # Errors
/// The database.
pub async fn invoices(pool: &PgPool, view: &Access) -> Result<Vec<InvoiceRow>, InvoiceError> {
    if view.is_empty() {
        return Ok(Vec::new());
    }
    Ok(sqlx::query_as::<_, InvoiceRow>(sql(&format!(
        "select {INVOICE_COLUMNS} from finance.invoices where party_id = any($1)
          order by coalesce(issued_at, created_at) desc"
    )))
    .bind(view.party_ids())
    .fetch_all(pool)
    .await
    .map_err(map_err)?)
}

/// One invoice with its lines, if the caller may see it.
///
/// # Errors
/// The database, or not in the view.
pub async fn invoice(
    pool: &PgPool,
    access: &Access,
    id: Uuid,
) -> Result<(InvoiceRow, Vec<LineRow>), InvoiceError> {
    let row = sqlx::query_as::<_, InvoiceRow>(sql(&format!(
        "select {INVOICE_COLUMNS} from finance.invoices where id = $1"
    )))
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(map_err)?
    .ok_or(DbError::NotFound { what: "invoice" })?;
    access.require(PartyId(row.party_id), "invoice")?;
    let lines = lines_of(pool, id).await?;
    Ok((row, lines))
}

async fn lines_of(pool: &PgPool, id: Uuid) -> Result<Vec<LineRow>, DbError> {
    sqlx::query_as::<_, LineRow>(
        "select id, invoice_id, position, description, quantity_milli, unit_price_minor, amount_minor,
                template_id
           from finance.invoice_lines where invoice_id = $1 order by position",
    )
    .bind(id)
    .fetch_all(pool)
    .await
    .map_err(map_err)
}

/// Today in Zagreb: the day an invoice is dated by, and aged from.
#[must_use]
pub fn today() -> NaiveDate {
    Utc::now().with_timezone(&Zagreb).date_naive()
}

/// A new draft for a client, pre-filled from the last approved invoice to
/// them when there is one -- "same as last month" is what a monthly invoice
/// almost always is. With `source`, a duplicate instead: that invoice's
/// header (client, currency, VAT treatment, series, place, note) and lines,
/// dated today; the source may be in any status.
///
/// # Errors
/// The database; the client or source is outside the grant; the party has
/// no issuer.
pub async fn create_draft(
    pool: &PgPool,
    access: &Access,
    client_id: Option<Uuid>,
    source: Option<Uuid>,
) -> Result<InvoiceRow, InvoiceError> {
    let source = match source {
        Some(id) => Some(invoice(pool, access, id).await?),
        None => None,
    };
    let client_id = client_id
        .or_else(|| source.as_ref().map(|(s, _)| s.client_id))
        .ok_or(DbError::NotFound { what: "client" })?;
    let client = sqlx::query_as::<_, ClientRow>(sql(&format!(
        "select {CLIENT_COLUMNS} from finance.clients where id = $1"
    )))
    .bind(client_id)
    .fetch_optional(pool)
    .await
    .map_err(map_err)?
    .ok_or(DbError::NotFound { what: "client" })?;
    access.require(PartyId(client.party_id), "client")?;
    if let Some((s, _)) = &source
        && s.party_id != client.party_id
    {
        return Err(DbError::NotFound { what: "invoice" }.into());
    }
    let issuer = issuer(pool, access, client.party_id)
        .await?
        .ok_or(InvoiceError::NoIssuer)?;

    let previous = match &source {
        Some((s, _)) => Some(s.clone()),
        None => sqlx::query_as::<_, InvoiceRow>(sql(&format!(
            "select {INVOICE_COLUMNS} from finance.invoices
              where client_id = $1 and status in ('approved', 'sent', 'paid')
              order by issued_at desc limit 1"
        )))
        .bind(client_id)
        .fetch_optional(pool)
        .await
        .map_err(map_err)?,
    };
    let inputs = starting_inputs(pool, client_id, source.as_ref(), previous.as_ref()).await?;
    let start = Start::of(source.as_ref().map(|(s, _)| s), &client, &issuer);
    let (treatment, currency, premises, device, place, note) = (
        start.treatment,
        start.currency,
        start.premises,
        start.device,
        start.place,
        start.note,
    );

    let id = Uuid::new_v4();
    let day = today();
    let due = day + chrono::Days::new(u64::try_from(issuer.due_days).unwrap_or(15));
    let mut tx = pool.begin().await.map_err(map_err)?;
    sqlx::query(
        "insert into finance.invoices (id, party_id, client_id, status, year, premises, device, delivery_date,
            due_date, place_of_issue, currency, vat_treatment, vat_note, prefilled_from, note)
         values ($1,$2,$3,'draft',$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)",
    )
    .bind(id)
    .bind(client.party_id)
    .bind(client_id)
    .bind(day.year())
    .bind(premises)
    .bind(device)
    .bind(day)
    .bind(due)
    .bind(place)
    .bind(currency)
    .bind(treatment.as_str())
    .bind(treatment.note())
    .bind(previous.as_ref().map(|p| p.id))
    .bind(note)
    .execute(&mut *tx)
    .await
    .map_err(map_err)?;
    write_lines(&mut tx, id, &inputs, treatment).await?;
    event(
        &mut tx,
        id,
        access.user(),
        "created",
        if source.is_some() {
            serde_json::json!({ "duplicated_from": previous.map(|p| p.id) })
        } else {
            serde_json::json!({ "prefilled_from": previous.map(|p| p.id) })
        },
    )
    .await?;
    tx.commit().await.map_err(map_err)?;
    invoice(pool, access, id).await.map(|(row, _)| row)
}

/// What a new draft's header starts as: a duplicate keeps its source's, a
/// fresh draft takes the client's and the issuer's.
struct Start<'a> {
    treatment: VatTreatment,
    currency: &'a str,
    premises: &'a str,
    device: &'a str,
    place: &'a str,
    note: &'a str,
}

impl<'a> Start<'a> {
    fn of(source: Option<&'a InvoiceRow>, client: &'a ClientRow, issuer: &'a IssuerRow) -> Self {
        let treatment = source
            .map(|s| s.vat_treatment.as_str())
            .and_then(VatTreatment::parse)
            .or_else(|| VatTreatment::parse(&client.vat_treatment))
            .unwrap_or(VatTreatment::OutsideScopeNonEu);
        Self {
            treatment,
            currency: source.map_or(client.currency.as_str(), |s| s.currency.as_str()),
            premises: source.map_or(issuer.premises.as_str(), |s| s.premises.as_str()),
            device: source.map_or(issuer.device.as_str(), |s| s.device.as_str()),
            place: source.map_or(issuer.place_of_issue.as_str(), |s| {
                s.place_of_issue.as_str()
            }),
            note: source.map_or("", |s| s.note.as_str()),
        }
    }
}

/// The lines a new draft starts with: a duplicate's own, else the templates
/// and the last invoice.
async fn starting_inputs(
    pool: &PgPool,
    client_id: Uuid,
    source: Option<&(InvoiceRow, Vec<LineRow>)>,
    previous: Option<&InvoiceRow>,
) -> Result<Vec<LineInput>, InvoiceError> {
    if let Some((_, lines)) = source {
        return Ok(lines
            .iter()
            .map(|l| LineInput {
                description: l.description.clone(),
                quantity_milli: l.quantity_milli,
                unit_price_minor: l.unit_price_minor,
                template_id: l.template_id,
            })
            .collect());
    }
    let previous_lines = match previous {
        Some(p) => lines_of(pool, p.id).await?,
        None => Vec::new(),
    };
    starting_lines(pool, client_id, &previous_lines).await
}

/// Delete a draft. Only a draft: an invoice that took a number is
/// cancelled and keeps it.
///
/// # Errors
/// The database; not in the view; not a draft.
pub async fn delete_draft(pool: &PgPool, access: &Access, id: Uuid) -> Result<(), InvoiceError> {
    let (row, _) = invoice(pool, access, id).await?;
    if row.status != "draft" {
        return Err(InvoiceError::NotDraft(row.status));
    }
    sqlx::query("delete from finance.invoices where id = $1 and status = 'draft'")
        .bind(id)
        .execute(pool)
        .await
        .map_err(map_err)?;
    Ok(())
}

/// The rows a new draft starts with.
///
/// The templates decide which rows exist; the last invoice decides what a
/// variable row cost last time. Without templates, the last invoice's lines
/// are the best guess there is.
async fn starting_lines(
    pool: &PgPool,
    client_id: Uuid,
    previous_lines: &[LineRow],
) -> Result<Vec<LineInput>, InvoiceError> {
    let templates: Vec<TemplateRow> = sqlx::query_as::<_, TemplateRow>(sql(&format!(
        "select {TEMPLATE_COLUMNS} from finance.line_templates
          where client_id = $1 and enabled and mode in ('fixed', 'variable')
          order by position, created_at"
    )))
    .bind(client_id)
    .fetch_all(pool)
    .await
    .map_err(map_err)?;
    if templates.is_empty() {
        return Ok(previous_lines
            .iter()
            .map(|l| LineInput {
                description: l.description.clone(),
                quantity_milli: l.quantity_milli,
                unit_price_minor: l.unit_price_minor,
                template_id: l.template_id,
            })
            .collect());
    }
    Ok(templates
        .iter()
        .map(|t| {
            let last = previous_lines
                .iter()
                .find(|l| l.template_id == Some(t.id) || l.description == t.description);
            let price = match (t.mode.as_str(), last) {
                ("variable", Some(l)) => l.unit_price_minor,
                _ => t.unit_price_minor,
            };
            LineInput {
                description: t.description.clone(),
                quantity_milli: t.quantity_milli,
                unit_price_minor: price,
                template_id: Some(t.id),
            }
        })
        .collect())
}

/// Make a client the party's default, the one before it no longer.
///
/// # Errors
/// Not in the grant (not found); the database.
pub async fn set_default_client(
    pool: &PgPool,
    access: &Access,
    client_id: Uuid,
) -> Result<ClientRow, InvoiceError> {
    let (party,): (Uuid,) = sqlx::query_as("select party_id from finance.clients where id = $1")
        .bind(client_id)
        .fetch_optional(pool)
        .await
        .map_err(map_err)?
        .ok_or(DbError::NotFound { what: "client" })?;
    access.require(PartyId(party), "client")?;
    let mut tx = pool.begin().await.map_err(map_err)?;
    sqlx::query("update finance.clients set is_default = false where party_id = $1 and is_default")
        .bind(party)
        .execute(&mut *tx)
        .await
        .map_err(map_err)?;
    sqlx::query("update finance.clients set is_default = true where id = $1")
        .bind(client_id)
        .execute(&mut *tx)
        .await
        .map_err(map_err)?;
    tx.commit().await.map_err(map_err)?;
    sqlx::query_as::<_, ClientRow>(sql(&format!(
        "select {CLIENT_COLUMNS} from finance.clients where id = $1"
    )))
    .bind(client_id)
    .fetch_one(pool)
    .await
    .map_err(|e| map_err(e).into())
}

/// Replace a draft's editable fields and lines, and recompute its totals.
///
/// # Errors
/// The database; not in the view; not a draft.
pub async fn update_draft(
    pool: &PgPool,
    access: &Access,
    id: Uuid,
    input: DraftInput,
) -> Result<InvoiceRow, InvoiceError> {
    let (row, _) = invoice(pool, access, id).await?;
    if row.status != "draft" {
        return Err(InvoiceError::NotDraft(row.status));
    }
    let treatment = input
        .vat_treatment
        .or_else(|| VatTreatment::parse(&row.vat_treatment))
        .unwrap_or(VatTreatment::OutsideScopeNonEu);
    // A new client must be the same party's: a draft never changes issuer.
    let client_id = match input.client_id {
        Some(c) if c != row.client_id => {
            let (party,): (Uuid,) =
                sqlx::query_as("select party_id from finance.clients where id = $1")
                    .bind(c)
                    .fetch_optional(pool)
                    .await
                    .map_err(map_err)?
                    .ok_or(DbError::NotFound { what: "client" })?;
            if party != row.party_id {
                return Err(DbError::NotFound { what: "client" }.into());
            }
            c
        }
        _ => row.client_id,
    };
    let currency = match input.currency.as_deref().map(str::trim) {
        Some(c) if !c.is_empty() => {
            if c.len() != 3 || !c.chars().all(|ch| ch.is_ascii_alphabetic()) {
                return Err(InvoiceError::Invalid(format!(
                    "currency: not a code: {c:?}"
                )));
            }
            c.to_ascii_uppercase()
        }
        _ => row.currency.clone(),
    };
    let series = |given: Option<&String>, current: &str| -> Result<String, InvoiceError> {
        match given.map(|s| s.trim()) {
            Some(s) if !s.is_empty() => {
                if s.len() > 20 || !s.chars().all(|ch| ch.is_ascii_alphanumeric()) {
                    return Err(InvoiceError::Invalid(format!(
                        "series: letters and digits only: {s:?}"
                    )));
                }
                Ok(s.to_owned())
            }
            _ => Ok(current.to_owned()),
        }
    };
    let premises = series(input.premises.as_ref(), &row.premises)?;
    let device = series(input.device.as_ref(), &row.device)?;
    let mut tx = pool.begin().await.map_err(map_err)?;
    sqlx::query(
        "update finance.invoices
            set delivery_date = $2, due_date = $3, place_of_issue = $4, note = $5, client_id = $6,
                currency = $7, vat_treatment = $8, vat_note = $9, premises = $10, device = $11,
                updated_at = clock_timestamp()
          where id = $1 and status = 'draft'",
    )
    .bind(id)
    .bind(input.delivery_date)
    .bind(input.due_date)
    .bind(input.place_of_issue.trim())
    .bind(input.note.trim())
    .bind(client_id)
    .bind(&currency)
    .bind(treatment.as_str())
    .bind(treatment.note())
    .bind(&premises)
    .bind(&device)
    .execute(&mut *tx)
    .await
    .map_err(map_err)?;
    write_lines(&mut tx, id, &input.lines, treatment).await?;
    event(
        &mut tx,
        id,
        access.user(),
        "edited",
        serde_json::json!({ "lines": input.lines.len() }),
    )
    .await?;
    tx.commit().await.map_err(map_err)?;
    invoice(pool, access, id).await.map(|(row, _)| row)
}

async fn write_lines(
    tx: &mut Transaction<'_, Postgres>,
    id: Uuid,
    inputs: &[LineInput],
    treatment: VatTreatment,
) -> Result<(), InvoiceError> {
    sqlx::query("delete from finance.invoice_lines where invoice_id = $1")
        .bind(id)
        .execute(&mut **tx)
        .await
        .map_err(map_err)?;
    let mut lines = Vec::with_capacity(inputs.len());
    for (i, l) in inputs.iter().enumerate() {
        if l.quantity_milli <= 0 {
            return Err(DbError::Invalid {
                field: "quantity",
                reason: "must be positive".into(),
            }
            .into());
        }
        lines.push(Line {
            position: i32::try_from(i + 1).unwrap_or(i32::MAX),
            description: l.description.trim().to_owned(),
            quantity_milli: l.quantity_milli,
            unit_price_minor: l.unit_price_minor,
            amount_minor: 0,
        });
    }
    totals::settle(&mut lines);
    for l in &lines {
        sqlx::query(
            "insert into finance.invoice_lines (id, invoice_id, position, description, quantity_milli,
                unit_price_minor, amount_minor, template_id) values ($1,$2,$3,$4,$5,$6,$7,$8)",
        )
        .bind(Uuid::new_v4())
        .bind(id)
        .bind(l.position)
        .bind(&l.description)
        .bind(l.quantity_milli)
        .bind(l.unit_price_minor)
        .bind(l.amount_minor)
        .bind(inputs.get(usize::try_from(l.position).unwrap_or(1).saturating_sub(1)).and_then(|i| i.template_id))
        .execute(&mut **tx)
        .await
        .map_err(map_err)?;
    }
    let (subtotal, vat, total) = totals::totals(&lines, treatment);
    sqlx::query(
        "update finance.invoices set subtotal_minor = $2, vat_minor = $3, total_minor = $4 where id = $1",
    )
    .bind(id)
    .bind(subtotal)
    .bind(vat)
    .bind(total)
    .execute(&mut **tx)
    .await
    .map_err(map_err)?;
    Ok(())
}

pub(crate) async fn event(
    tx: &mut Transaction<'_, Postgres>,
    invoice: Uuid,
    user: UserId,
    what: &str,
    detail: serde_json::Value,
) -> Result<(), DbError> {
    sqlx::query("insert into finance.invoice_events (id, invoice_id, user_id, event, detail) values ($1,$2,$3,$4,$5)")
        .bind(Uuid::new_v4())
        .bind(invoice)
        .bind(if user.0.is_nil() { None } else { Some(user.0) })
        .bind(what)
        .bind(detail)
        .execute(&mut **tx)
        .await
        .map_err(map_err)?;
    Ok(())
}

/// The canonical document for an invoice as it is right now: the stored one
/// for an approved invoice, a preview (next number, current time) for a
/// draft.
async fn document_of(
    pool: &PgPool,
    row: &InvoiceRow,
    lines: &[LineRow],
    issued_at: DateTime<Utc>,
    ordinal: Option<i32>,
) -> Result<InvoiceDoc, InvoiceError> {
    let issuer = sqlx::query_as::<_, IssuerRow>(sql(&format!(
        "select {ISSUER_COLUMNS} from finance.issuers where party_id = $1"
    )))
    .bind(row.party_id)
    .fetch_optional(pool)
    .await
    .map_err(map_err)?
    .ok_or(InvoiceError::NoIssuer)?;
    let client = sqlx::query_as::<_, ClientRow>(sql(&format!(
        "select {CLIENT_COLUMNS} from finance.clients where id = $1"
    )))
    .bind(row.client_id)
    .fetch_one(pool)
    .await
    .map_err(map_err)?;
    let treatment =
        VatTreatment::parse(&row.vat_treatment).unwrap_or(VatTreatment::OutsideScopeNonEu);
    let (number, preview) = match (row.number.as_deref(), ordinal) {
        (Some(n), _) => (n.to_owned(), false),
        (None, Some(next)) => (
            format!("{next}-{}-{}-{}", row.premises, row.device, row.year),
            true,
        ),
        (None, None) => (String::from("—"), true),
    };
    Ok(InvoiceDoc {
        number,
        number_preview: preview,
        issued_at,
        delivery_date: row.delivery_date,
        due_date: row.due_date,
        place_of_issue: row.place_of_issue.clone(),
        currency: row.currency.clone(),
        issuer: issuer.doc(),
        client: Client {
            name: client.name,
            address_lines: client.address_lines,
            country: country_name(&client.country_code),
            tax_id: client.tax_id,
        },
        lines: lines
            .iter()
            .map(|l| Line {
                position: l.position,
                description: l.description.clone(),
                quantity_milli: l.quantity_milli,
                unit_price_minor: l.unit_price_minor,
                amount_minor: l.amount_minor,
            })
            .collect(),
        subtotal_minor: row.subtotal_minor,
        vat_minor: row.vat_minor,
        total_minor: row.total_minor,
        vat_treatment: treatment,
        vat_note: if row.status == "draft" {
            treatment.note().to_owned()
        } else {
            row.vat_note.clone()
        },
        note: row.note.clone(),
    })
}

/// The few countries an invoice from here names. Anything else prints its code.
fn country_name(code: &str) -> String {
    match code {
        "RS" => "Serbia",
        "HR" => "Croatia",
        "DE" => "Germany",
        "AT" => "Austria",
        "SI" => "Slovenia",
        "US" => "United States",
        "GB" => "United Kingdom",
        "NL" => "Netherlands",
        "CH" => "Switzerland",
        other => other,
    }
    .to_owned()
}

/// Render a draft as it would be approved now: next number, current time,
/// watermarked. Writes nothing. The hash is what `approve` takes.
///
/// # Errors
/// The database; not in the view; the renderer.
pub async fn preview(pool: &PgPool, access: &Access, id: Uuid) -> Result<Preview, InvoiceError> {
    let (mut row, lines) = invoice(pool, access, id).await?;
    let next = if row.status == "draft" {
        // The number belongs to the year it is taken in, whatever year the
        // draft was written in.
        row.year = today().year();
        Some(numbering::peek(pool, &series_of(&row)).await?)
    } else {
        None
    };
    // The preview's time is truncated to the minute: the approval that follows
    // renders with its own "now", and a hash that changed because a second
    // ticked would refuse every approval.
    let now = minute(Utc::now());
    let doc = document_of(pool, &row, &lines, row.issued_at.unwrap_or(now), next).await?;
    let content_hash = doc.content_hash()?;
    let pdf = tokio::task::spawn_blocking({
        let doc = doc.clone();
        move || render::render(&doc)
    })
    .await
    .map_err(|e| DbError::Internal(format!("render task: {e}")))??
    .pdf;
    Ok(Preview {
        doc,
        content_hash,
        pdf,
    })
}

/// The numbering stream a draft approves into.
fn series_of(row: &InvoiceRow) -> numbering::Series<'_> {
    numbering::Series {
        party: row.party_id,
        year: row.year,
        premises: &row.premises,
        device: &row.device,
    }
}

fn minute(t: DateTime<Utc>) -> DateTime<Utc> {
    t.with_timezone(&Utc)
        .date_naive()
        .and_hms_opt(
            t.format("%H").to_string().parse().unwrap_or(0),
            t.format("%M").to_string().parse().unwrap_or(0),
            0,
        )
        .map_or(t, |n| n.and_utc())
}

/// What an approval produces.
#[derive(Debug, Clone)]
pub struct Approved {
    /// The invoice, now numbered.
    pub invoice: InvoiceRow,
    /// The stored PDF.
    pub document_id: Uuid,
}

/// Approve a draft: the one transaction that allocates a number.
///
/// The approver presents the hash from the preview they saw. The draft is
/// locked, the document rebuilt with the number it will take and the same
/// minute-truncated time, and the hash compared; a difference is
/// [`InvoiceError::StaleDraft`] and nothing changes. Then, still inside the
/// transaction: the number is taken under the counter's lock, the PDF is
/// rendered and stored content-addressed, the VAT note frozen onto the
/// invoice, the approval recorded with who and what hash.
///
/// # Errors
/// See [`InvoiceError`].
pub async fn approve(
    pool: &PgPool,
    access: &Access,
    id: Uuid,
    content_hash: &str,
) -> Result<Approved, InvoiceError> {
    let (mut row, lines) = invoice(pool, access, id).await?;
    if row.status != "draft" {
        return Err(InvoiceError::NotDraft(row.status));
    }
    if lines.is_empty() {
        return Err(InvoiceError::NoLines);
    }
    // The year of approval, not of the draft: a December draft approved in
    // January is January's invoice and takes January's counter.
    row.year = today().year();
    let mut tx = pool.begin().await.map_err(map_err)?;
    // Lock the draft: two approvals of one invoice serialise here, and the
    // second sees `approved` and is refused.
    let (status,): (String,) =
        sqlx::query_as("select status from finance.invoices where id = $1 for update")
            .bind(id)
            .fetch_one(&mut *tx)
            .await
            .map_err(map_err)?;
    if status != "draft" {
        return Err(InvoiceError::NotDraft(status));
    }
    let ordinal = numbering::allocate(&mut tx, &series_of(&row)).await?;
    let issued_at = minute(Utc::now());
    let mut doc = document_of(pool, &row, &lines, issued_at, Some(ordinal)).await?;
    if doc.content_hash()? != content_hash {
        // Rolls the allocation back with the transaction: no number was taken.
        return Err(InvoiceError::StaleDraft);
    }
    doc.number_preview = false;
    let final_hash = doc.content_hash()?;
    let pdf = tokio::task::spawn_blocking({
        let doc = doc.clone();
        move || render::render(&doc)
    })
    .await
    .map_err(|e| DbError::Internal(format!("render task: {e}")))??
    .pdf;

    let mut hasher = Sha256::new();
    hasher.update(&pdf);
    let sha = format!("{:x}", hasher.finalize());
    let document_id = Uuid::new_v4();
    sqlx::query(
        "insert into finance.documents (id, party_id, kind, sha256, content_type, size_bytes)
         values ($1, $2, 'invoice', $3, 'application/pdf', $4)",
    )
    .bind(document_id)
    .bind(row.party_id)
    .bind(&sha)
    .bind(i64::try_from(pdf.len()).unwrap_or(i64::MAX))
    .execute(&mut *tx)
    .await
    .map_err(map_err)?;
    sqlx::query("insert into finance.document_blobs (document_id, bytes) values ($1, $2)")
        .bind(document_id)
        .bind(&pdf)
        .execute(&mut *tx)
        .await
        .map_err(map_err)?;
    sqlx::query(
        "update finance.invoices
            set status = 'approved', ordinal = $2, issued_at = $3, approved_at = $3, approved_by = $4,
                content_hash = $5, document_id = $6, vat_note = $7, year = $8, updated_at = clock_timestamp()
          where id = $1",
    )
    .bind(id)
    .bind(ordinal)
    .bind(issued_at)
    .bind(if access.user().0.is_nil() { None } else { Some(access.user().0) })
    .bind(&final_hash)
    .bind(document_id)
    .bind(&doc.vat_note)
    .bind(row.year)
    .execute(&mut *tx)
    .await
    .map_err(map_err)?;
    event(
        &mut tx,
        id,
        access.user(),
        "approved",
        serde_json::json!({ "number": doc.number, "content_hash": final_hash, "presented": content_hash, "sha256": sha }),
    )
    .await?;
    tx.commit().await.map_err(map_err)?;
    let (invoice, _) = invoice_after(pool, id).await?;
    Ok(Approved {
        invoice,
        document_id,
    })
}

async fn invoice_after(
    pool: &PgPool,
    id: Uuid,
) -> Result<(InvoiceRow, Vec<LineRow>), InvoiceError> {
    let row = sqlx::query_as::<_, InvoiceRow>(sql(&format!(
        "select {INVOICE_COLUMNS} from finance.invoices where id = $1"
    )))
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(map_err)?;
    let lines = lines_of(pool, id).await?;
    Ok((row, lines))
}

/// Cancel an issued invoice. It keeps its number -- gapless means 1..n with
/// no holes, not that every number is live. A draft is deleted instead.
///
/// # Errors
/// The database; not in the view; a draft; already cancelled or paid.
pub async fn cancel(
    pool: &PgPool,
    access: &Access,
    id: Uuid,
    reason: &str,
) -> Result<InvoiceRow, InvoiceError> {
    let (row, _) = invoice(pool, access, id).await?;
    if row.status == "draft" {
        return Err(InvoiceError::IsDraft);
    }
    if row.status == "cancelled" || row.status == "paid" {
        return Err(InvoiceError::NotDraft(row.status));
    }
    let mut tx = pool.begin().await.map_err(map_err)?;
    sqlx::query(
        "update finance.invoices set status = 'cancelled', cancelled_at = clock_timestamp(),
            updated_at = clock_timestamp() where id = $1",
    )
    .bind(id)
    .execute(&mut *tx)
    .await
    .map_err(map_err)?;
    event(
        &mut tx,
        id,
        access.user(),
        "cancelled",
        serde_json::json!({ "reason": reason }),
    )
    .await?;
    tx.commit().await.map_err(map_err)?;
    invoice_after(pool, id).await.map(|(row, _)| row)
}

/// A stored document's bytes, if the caller may see it.
///
/// # Errors
/// The database; not in the view.
pub async fn document(
    pool: &PgPool,
    access: &Access,
    id: Uuid,
) -> Result<(String, Vec<u8>), InvoiceError> {
    let meta: Option<(Uuid, String)> =
        sqlx::query_as("select party_id, content_type from finance.documents where id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(map_err)?;
    let (party, content_type) = meta.ok_or(DbError::NotFound { what: "document" })?;
    access.require(PartyId(party), "document")?;
    let (bytes,): (Vec<u8>,) =
        sqlx::query_as("select bytes from finance.document_blobs where document_id = $1")
            .bind(id)
            .fetch_one(pool)
            .await
            .map_err(map_err)?;
    Ok((content_type, bytes))
}
