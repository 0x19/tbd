//! The rows the read RPCs hand out, and the two writes a person makes.
//!
//! Every function takes an [`Access`] and filters on its parties. There is no
//! variant that does not, so a handler cannot widen its view by picking the
//! wrong function. Writes check the row's party against the grant *before*
//! touching anything, and a row outside it is [`DbError::NotFound`] --
//! never "forbidden", which would confirm the row exists.

use chrono::{DateTime, NaiveDate, Utc};
use sqlx::PgPool;
use tbd_db::{Access, DbError, PartyId, VisibleParty, map_err, visible_parties};
use uuid::Uuid;

use crate::categorise;

// The row structs mirror columns whose meaning is documented where they are
// defined, in `migrations/`; repeating it per field here would drift.

/// A bank account with its sync state and latest balances.
#[allow(missing_docs)]
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AccountRow {
    pub id: Uuid,
    pub party_id: Uuid,
    pub connection_id: Option<Uuid>,
    pub provider: String,
    pub iban: Option<String>,
    pub currency: String,
    pub name: String,
    pub sync_enabled: bool,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub last_sync_status: Option<String>,
    pub last_sync_error: Option<String>,
    pub last_booked_through: Option<NaiveDate>,
    pub sync_backoff_until: Option<DateTime<Utc>>,
    pub sync_budget_day: Option<NaiveDate>,
    pub sync_budget_used: i32,
}

/// The latest balance of one type on one account.
#[allow(missing_docs)]
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct BalanceRow {
    pub account_id: Uuid,
    pub balance_type: String,
    pub amount_minor: i64,
    pub currency: String,
    pub observed_at: DateTime<Utc>,
}

#[allow(missing_docs)]
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct CategoryRow {
    pub id: Uuid,
    pub party_id: Uuid,
    pub slug: String,
    pub name: String,
    pub kind: String,
    pub deductible: bool,
    pub archived_at: Option<DateTime<Utc>>,
}

#[allow(missing_docs)]
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RuleRow {
    pub id: Uuid,
    pub party_id: Uuid,
    pub priority: i32,
    pub name: String,
    pub category_id: Uuid,
    pub match_counterparty_like: Option<String>,
    pub match_counterparty_iban: Option<String>,
    pub match_remittance_like: Option<String>,
    pub match_currency: Option<String>,
    pub match_credit_debit: Option<String>,
    pub enabled: bool,
    pub hits: i64,
}

#[allow(missing_docs)]
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ConnectionRow {
    pub id: Uuid,
    pub party_id: Uuid,
    pub provider: String,
    pub psu_type: String,
    pub aspsp_name: String,
    pub status: String,
    pub valid_until: Option<DateTime<Utc>>,
    pub authorized_at: Option<DateTime<Utc>>,
    pub accounts: i64,
}

/// What a rule looks like when a person writes it.
#[allow(missing_docs)]
#[derive(Debug, Clone, Default)]
pub struct RuleInput {
    pub id: Option<Uuid>,
    pub party_id: Uuid,
    pub priority: i32,
    pub name: String,
    pub category_id: Uuid,
    pub match_counterparty_like: Option<String>,
    pub match_counterparty_iban: Option<String>,
    pub match_remittance_like: Option<String>,
    pub match_currency: Option<String>,
    pub match_credit_debit: Option<String>,
    pub enabled: bool,
}

/// The parties the caller may read, with why.
///
/// # Errors
/// The database.
pub async fn parties(pool: &PgPool, access: &Access) -> Result<Vec<VisibleParty>, DbError> {
    visible_parties(pool, access.user()).await
}

/// Accounts in the view.
///
/// # Errors
/// The database.
pub async fn accounts(pool: &PgPool, view: &Access) -> Result<Vec<AccountRow>, DbError> {
    if view.is_empty() {
        return Ok(Vec::new());
    }
    sqlx::query_as::<_, AccountRow>(
        "select id, party_id, connection_id, provider, iban, currency, name, sync_enabled,
                last_synced_at, last_sync_status, last_sync_error, last_booked_through,
                sync_backoff_until, sync_budget_day, sync_budget_used
           from finance.accounts
          where party_id = any($1)
          order by party_id, currency, name",
    )
    .bind(view.party_ids())
    .fetch_all(pool)
    .await
    .map_err(map_err)
}

/// The latest balance of each type for accounts in the view.
///
/// # Errors
/// The database.
pub async fn latest_balances(pool: &PgPool, view: &Access) -> Result<Vec<BalanceRow>, DbError> {
    if view.is_empty() {
        return Ok(Vec::new());
    }
    sqlx::query_as::<_, BalanceRow>(
        "select distinct on (b.account_id, b.balance_type)
                b.account_id, b.balance_type, b.amount_minor, b.currency, b.observed_at
           from finance.balances b
           join finance.accounts a on a.id = b.account_id
          where a.party_id = any($1)
          order by b.account_id, b.balance_type, b.observed_at desc",
    )
    .bind(view.party_ids())
    .fetch_all(pool)
    .await
    .map_err(map_err)
}

/// Categories in the view, archived ones included and marked.
///
/// # Errors
/// The database.
pub async fn categories(pool: &PgPool, view: &Access) -> Result<Vec<CategoryRow>, DbError> {
    if view.is_empty() {
        return Ok(Vec::new());
    }
    sqlx::query_as::<_, CategoryRow>(
        "select id, party_id, slug, name, kind, deductible, archived_at
           from finance.categories where party_id = any($1)
          order by party_id, kind, name",
    )
    .bind(view.party_ids())
    .fetch_all(pool)
    .await
    .map_err(map_err)
}

/// Rules in the view, in the order they apply.
///
/// # Errors
/// The database.
pub async fn rules(pool: &PgPool, view: &Access) -> Result<Vec<RuleRow>, DbError> {
    if view.is_empty() {
        return Ok(Vec::new());
    }
    sqlx::query_as::<_, RuleRow>(
        "select id, party_id, priority, name, category_id, match_counterparty_like,
                match_counterparty_iban, match_remittance_like, match_currency,
                match_credit_debit, enabled, hits
           from finance.rules where party_id = any($1)
          order by party_id, priority, id",
    )
    .bind(view.party_ids())
    .fetch_all(pool)
    .await
    .map_err(map_err)
}

/// Connections in the view, with how many accounts each covers.
///
/// # Errors
/// The database.
pub async fn connections(pool: &PgPool, view: &Access) -> Result<Vec<ConnectionRow>, DbError> {
    if view.is_empty() {
        return Ok(Vec::new());
    }
    sqlx::query_as::<_, ConnectionRow>(
        "select c.id, c.party_id, c.provider, c.psu_type, c.aspsp_name, c.status,
                c.valid_until, c.authorized_at,
                (select count(*) from finance.accounts a where a.connection_id = c.id) as accounts
           from finance.connections c where c.party_id = any($1)
          order by c.created_at desc",
    )
    .bind(view.party_ids())
    .fetch_all(pool)
    .await
    .map_err(map_err)
}

/// Whose a transaction is, if the caller may know.
///
/// # Errors
/// The database, or the row is not in the view.
pub async fn transaction_party(
    pool: &PgPool,
    access: &Access,
    transaction: Uuid,
) -> Result<Uuid, DbError> {
    let row: Option<(Uuid,)> =
        sqlx::query_as("select party_id from finance.bank_transactions where id = $1")
            .bind(transaction)
            .fetch_optional(pool)
            .await
            .map_err(map_err)?;
    let party = row.map(|(p,)| p).ok_or(DbError::NotFound {
        what: "transaction",
    })?;
    access.require(PartyId(party), "transaction")?;
    Ok(party)
}

/// Whose an account is, if the caller may know.
///
/// # Errors
/// The database, or the row is not in the view.
pub async fn account_party(pool: &PgPool, access: &Access, account: Uuid) -> Result<Uuid, DbError> {
    let row: Option<(Uuid,)> =
        sqlx::query_as("select party_id from finance.accounts where id = $1")
            .bind(account)
            .fetch_optional(pool)
            .await
            .map_err(map_err)?;
    let party = row
        .map(|(p,)| p)
        .ok_or(DbError::NotFound { what: "account" })?;
    access.require(PartyId(party), "account")?;
    Ok(party)
}

/// Whose a pending connection is, if the caller may know. Looked up by the
/// state the bank's redirect carried.
///
/// # Errors
/// The database, or no such pending state in the view.
pub async fn connection_party_by_state(
    pool: &PgPool,
    access: &Access,
    state: &str,
) -> Result<Uuid, DbError> {
    let row: Option<(Uuid,)> =
        sqlx::query_as("select party_id from finance.connections where state = $1")
            .bind(state)
            .fetch_optional(pool)
            .await
            .map_err(map_err)?;
    let party = row
        .map(|(p,)| p)
        .ok_or(DbError::NotFound { what: "connection" })?;
    access.require(PartyId(party), "connection")?;
    Ok(party)
}

/// A category that belongs to a party, if the caller may know.
async fn category_of(
    pool: &PgPool,
    access: &Access,
    party: Uuid,
    category: Uuid,
) -> Result<(), DbError> {
    access.require(PartyId(party), "category")?;
    let row: Option<(Uuid,)> = sqlx::query_as(
        "select id from finance.categories where id = $1 and party_id = $2 and archived_at is null",
    )
    .bind(category)
    .bind(party)
    .fetch_optional(pool)
    .await
    .map_err(map_err)?;
    row.map(|_| ())
        .ok_or(DbError::NotFound { what: "category" })
}

/// Set a transaction's category by hand, for a row the caller may read, to a
/// category of the same party.
///
/// # Errors
/// The database, or the transaction or category is not in the view.
pub async fn declare(
    pool: &PgPool,
    access: &Access,
    transaction: Uuid,
    category: Uuid,
) -> Result<(), DbError> {
    let party = transaction_party(pool, access, transaction).await?;
    category_of(pool, access, party, category).await?;
    categorise::declare(pool, transaction, category).await
}

/// Create or change a rule, then reapply the party's rules.
///
/// # Errors
/// The database; the party, category or existing rule is not in the view; or
/// the input is malformed.
pub async fn upsert_rule(
    pool: &PgPool,
    access: &Access,
    input: RuleInput,
) -> Result<(RuleRow, categorise::Applied), DbError> {
    access.require(PartyId(input.party_id), "party")?;
    category_of(pool, access, input.party_id, input.category_id).await?;
    if input.name.trim().is_empty() {
        return Err(DbError::Invalid {
            field: "name",
            reason: "empty".into(),
        });
    }
    if let Some(cd) = &input.match_credit_debit
        && cd != "CRDT"
        && cd != "DBIT"
    {
        return Err(DbError::Invalid {
            field: "match_credit_debit",
            reason: "want CRDT or DBIT".into(),
        });
    }
    let normalise = |v: &Option<String>| {
        v.as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(crate::import::normalise)
    };
    let id = if let Some(id) = input.id {
        // Changing a rule the caller may not see is not-found, like reading it.
        let owned: Option<(Uuid,)> =
            sqlx::query_as("select id from finance.rules where id = $1 and party_id = $2")
                .bind(id)
                .bind(input.party_id)
                .fetch_optional(pool)
                .await
                .map_err(map_err)?;
        owned.ok_or(DbError::NotFound { what: "rule" })?;
        id
    } else {
        Uuid::new_v4()
    };
    sqlx::query(
        "insert into finance.rules
            (id, party_id, priority, name, category_id, match_counterparty_like,
             match_counterparty_iban, match_remittance_like, match_currency,
             match_credit_debit, enabled)
         values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
         on conflict (id) do update
            set priority = excluded.priority, name = excluded.name,
                category_id = excluded.category_id,
                match_counterparty_like = excluded.match_counterparty_like,
                match_counterparty_iban = excluded.match_counterparty_iban,
                match_remittance_like = excluded.match_remittance_like,
                match_currency = excluded.match_currency,
                match_credit_debit = excluded.match_credit_debit,
                enabled = excluded.enabled",
    )
    .bind(id)
    .bind(input.party_id)
    .bind(input.priority)
    .bind(input.name.trim())
    .bind(input.category_id)
    .bind(normalise(&input.match_counterparty_like))
    .bind(
        input
            .match_counterparty_iban
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty()),
    )
    .bind(normalise(&input.match_remittance_like))
    .bind(
        input
            .match_currency
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty()),
    )
    .bind(input.match_credit_debit.as_deref())
    .bind(input.enabled)
    .execute(pool)
    .await
    .map_err(map_err)?;
    let applied = categorise::apply_rules(pool, input.party_id).await?;
    let row = sqlx::query_as::<_, RuleRow>(
        "select id, party_id, priority, name, category_id, match_counterparty_like,
                match_counterparty_iban, match_remittance_like, match_currency,
                match_credit_debit, enabled, hits
           from finance.rules where id = $1",
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(map_err)?;
    Ok((row, applied))
}

/// Turn the scheduler on or off for one account the caller may see.
///
/// # Errors
/// The database, or the account is not in the view.
pub async fn set_sync_enabled(
    pool: &PgPool,
    access: &Access,
    account: Uuid,
    enabled: bool,
) -> Result<AccountRow, DbError> {
    account_party(pool, access, account).await?;
    sqlx::query("update finance.accounts set sync_enabled = $2 where id = $1")
        .bind(account)
        .bind(enabled)
        .execute(pool)
        .await
        .map_err(map_err)?;
    sqlx::query_as::<_, AccountRow>(
        "select id, party_id, connection_id, provider, iban, currency, name, sync_enabled,
                last_synced_at, last_sync_status, last_sync_error, last_booked_through,
                sync_backoff_until, sync_budget_day, sync_budget_used
           from finance.accounts where id = $1",
    )
    .bind(account)
    .fetch_one(pool)
    .await
    .map_err(map_err)
}
