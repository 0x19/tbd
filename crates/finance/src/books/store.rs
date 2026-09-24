//! The one writer of the books tables, and their reads.
//!
//! Every write runs in one transaction on a company party: the chart and the
//! year's periods are seeded on the way, the codes are checked against the
//! party's chart, the set balances or nothing lands, and a month that is not
//! open refuses. Reads take the caller's [`Access`]; a party outside it reads
//! as not found, never as forbidden.

use chrono::{DateTime, Datelike, NaiveDate, Utc};
use sqlx::{PgConnection, PgPool};
use tbd_db::{Access, PartyId, UserId, map_err};
use uuid::Uuid;

use super::{BooksError, OpeningSource, PeriodStatus, SourceKind, chart};

/// An account of a party's chart.
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct AccountRow {
    pub code: String,
    pub name: String,
    pub parent_code: Option<String>,
    pub class: i16,
    pub kind: String,
    pub synthetic: bool,
    pub archived_at: Option<DateTime<Utc>>,
}

/// A month of a fiscal year.
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeriodRow {
    pub fiscal_year: i32,
    pub month: i16,
    pub status: PeriodStatus,
    /// None when the row was never written: the month is open by default.
    pub changed_at: Option<DateTime<Utc>>,
}

/// One account of an opening trial balance, signed.
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Opening {
    pub account_code: String,
    pub debit_minor: i64,
    pub credit_minor: i64,
}

/// What an import wrote.
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Imported {
    pub accounts: usize,
    pub total_debit_minor: i64,
    pub total_credit_minor: i64,
}

/// One line of an entry to post.
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NewLine {
    pub account_code: String,
    pub debit_minor: i64,
    pub credit_minor: i64,
    /// Empty means the books' currency, EUR.
    pub currency: String,
    pub original_minor: Option<i64>,
    pub original_currency: Option<String>,
    pub counterparty: Option<String>,
    pub vat_code: Option<String>,
}

/// An entry to post.
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewEntry {
    pub party_id: Uuid,
    pub entry_date: NaiveDate,
    pub source: SourceKind,
    pub source_id: String,
    pub rule_id: String,
    pub rule_version: String,
    pub memo: String,
    pub lines: Vec<NewLine>,
}

/// One account of a trial balance. Totals and the balance are arithmetic on
/// these four and live on the wire type.
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct TrialBalanceRow {
    pub account_code: String,
    pub name: String,
    pub class: i16,
    pub kind: String,
    pub opening_debit_minor: i64,
    pub opening_credit_minor: i64,
    pub period_debit_minor: i64,
    pub period_credit_minor: i64,
}

/// A trial balance: opening plus movement through a month.
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrialBalance {
    pub fiscal_year: i32,
    pub through_month: u32,
    pub opening_as_of: Option<NaiveDate>,
    pub rows: Vec<TrialBalanceRow>,
    pub periods: Vec<PeriodRow>,
    /// Every year with an opening or an entry, newest first.
    pub years: Vec<i32>,
}

/// Whether the party is a company. Books belong to companies.
async fn is_company(conn: &mut PgConnection, party: Uuid) -> Result<bool, BooksError> {
    let row: Option<(Uuid,)> = sqlx::query_as("select id from public.orgs where id = $1")
        .bind(party)
        .fetch_optional(conn)
        .await
        .map_err(map_err)?;
    Ok(row.is_some())
}

async fn require_company(conn: &mut PgConnection, party: Uuid) -> Result<(), BooksError> {
    if is_company(conn, party).await? {
        Ok(())
    } else {
        Err(BooksError::NotCompany)
    }
}

/// Whether the user holds `own` on the party today.
///
/// # Errors
/// The database.
pub async fn owns(pool: &PgPool, user: UserId, party: PartyId) -> Result<bool, BooksError> {
    let row: Option<(i32,)> = sqlx::query_as(
        "select 1 from public.party_access
          where user_id = $1 and party_id = $2 and capability = 'own'
            and (expires_at is null or expires_at > now())",
    )
    .bind(user.0)
    .bind(party.0)
    .fetch_optional(pool)
    .await
    .map_err(map_err)?;
    Ok(row.is_some())
}

/// The party's chart, code order.
///
/// # Errors
/// The party is outside the grant, or the database.
pub async fn accounts(
    pool: &PgPool,
    access: &Access,
    party: Uuid,
) -> Result<Vec<AccountRow>, BooksError> {
    access.require(PartyId(party), "party")?;
    let mut tx = pool.begin().await.map_err(map_err)?;
    if is_company(&mut tx, party).await? {
        chart::ensure(&mut tx, party).await?;
    }
    let rows = sqlx::query_as::<_, AccountRow>(
        "select code, name, parent_code, class, kind, synthetic, archived_at
           from finance.ledger_accounts where party_id = $1 order by code",
    )
    .bind(party)
    .fetch_all(&mut *tx)
    .await
    .map_err(map_err)?;
    tx.commit().await.map_err(map_err)?;
    Ok(rows)
}

/// Add an analytic account to the party's chart, or rename one. The parent is
/// the longest existing code that is a prefix; a code with no account above
/// it is refused, since every posting must roll up into a group a statement
/// can map.
///
/// # Errors
/// Not a company, a malformed code, no parent, or the database.
pub async fn upsert_account(
    pool: &PgPool,
    party: Uuid,
    code: &str,
    name: &str,
) -> Result<AccountRow, BooksError> {
    let code = code.trim();
    if code.len() < 4 || code.len() > 6 || !code.bytes().all(|b| b.is_ascii_digit()) {
        return Err(BooksError::Invalid(format!(
            "code {code:?}: an analytic account is 4 to 6 digits"
        )));
    }
    let name = name.trim();
    if name.is_empty() || name.len() > 200 {
        return Err(BooksError::Invalid("name: 1 to 200 characters".into()));
    }
    let mut tx = pool.begin().await.map_err(map_err)?;
    require_company(&mut tx, party).await?;
    chart::ensure(&mut tx, party).await?;
    let parent = chart::parent_in_db(&mut tx, party, code).await?;
    let Some(parent) = parent else {
        return Err(BooksError::Invalid(format!(
            "code {code}: no account above it in the chart"
        )));
    };
    let row = sqlx::query_as::<_, AccountRow>(
        "insert into finance.ledger_accounts (party_id, code, name, parent_code, synthetic)
         values ($1, $2, $3, $4, false)
         on conflict (party_id, code) do update set name = excluded.name, archived_at = null
         returning code, name, parent_code, class, kind, synthetic, archived_at",
    )
    .bind(party)
    .bind(code)
    .bind(name)
    .bind(parent)
    .fetch_one(&mut *tx)
    .await
    .map_err(map_err)?;
    tx.commit().await.map_err(map_err)?;
    Ok(row)
}

async fn ensure_periods(conn: &mut PgConnection, party: Uuid, year: i32) -> Result<(), BooksError> {
    sqlx::query(
        "insert into finance.periods (party_id, fiscal_year, month)
         select $1, $2, m from generate_series(1, 12) as m
         on conflict do nothing",
    )
    .bind(party)
    .bind(year)
    .execute(conn)
    .await
    .map_err(map_err)?;
    Ok(())
}

async fn periods_of(
    conn: &mut PgConnection,
    party: Uuid,
    year: i32,
) -> Result<Vec<PeriodRow>, BooksError> {
    let rows: Vec<(i16, String, DateTime<Utc>)> = sqlx::query_as(
        "select month, status, changed_at from finance.periods
          where party_id = $1 and fiscal_year = $2 order by month",
    )
    .bind(party)
    .bind(year)
    .fetch_all(conn)
    .await
    .map_err(map_err)?;
    Ok((1..=12i16)
        .map(|m| {
            rows.iter().find(|(month, _, _)| *month == m).map_or(
                PeriodRow {
                    fiscal_year: year,
                    month: m,
                    status: PeriodStatus::Open,
                    changed_at: None,
                },
                |(month, status, at)| PeriodRow {
                    fiscal_year: year,
                    month: *month,
                    status: PeriodStatus::parse(status).unwrap_or(PeriodStatus::Open),
                    changed_at: Some(*at),
                },
            )
        })
        .collect())
}

/// The twelve months of a year with their status; a month never written is
/// open.
///
/// # Errors
/// The party is outside the grant, or the database.
pub async fn periods(
    pool: &PgPool,
    access: &Access,
    party: Uuid,
    year: i32,
) -> Result<Vec<PeriodRow>, BooksError> {
    access.require(PartyId(party), "party")?;
    let mut conn = pool.acquire().await.map_err(map_err)?;
    periods_of(&mut conn, party, year).await
}

/// Move a month to a status. Open and closed go back and forth; locked is
/// final. The same status again is a no-op.
///
/// # Errors
/// Not a company, a locked month, or the database.
pub async fn set_period(
    pool: &PgPool,
    party: Uuid,
    year: i32,
    month: u32,
    status: PeriodStatus,
    by: Option<UserId>,
) -> Result<PeriodRow, BooksError> {
    if !(1..=12).contains(&month) {
        return Err(BooksError::Invalid("month: 1 to 12".into()));
    }
    let mut tx = pool.begin().await.map_err(map_err)?;
    require_company(&mut tx, party).await?;
    ensure_periods(&mut tx, party, year).await?;
    let (current,): (String,) = sqlx::query_as(
        "select status from finance.periods
          where party_id = $1 and fiscal_year = $2 and month = $3 for update",
    )
    .bind(party)
    .bind(year)
    .bind(i16::try_from(month).unwrap_or(1))
    .fetch_one(&mut *tx)
    .await
    .map_err(map_err)?;
    if PeriodStatus::parse(&current) == Some(PeriodStatus::Locked) && status != PeriodStatus::Locked
    {
        return Err(BooksError::Locked { year, month });
    }
    let (changed_at,): (DateTime<Utc>,) = sqlx::query_as(
        "update finance.periods
            set status = $4,
                changed_at = case when status = $4 then changed_at else clock_timestamp() end,
                changed_by = case when status = $4 then changed_by else $5 end
          where party_id = $1 and fiscal_year = $2 and month = $3
          returning changed_at",
    )
    .bind(party)
    .bind(year)
    .bind(i16::try_from(month).unwrap_or(1))
    .bind(status.as_str())
    .bind(by.map(|u| u.0))
    .fetch_one(&mut *tx)
    .await
    .map_err(map_err)?;
    tx.commit().await.map_err(map_err)?;
    Ok(PeriodRow {
        fiscal_year: year,
        month: i16::try_from(month).unwrap_or(1),
        status,
        changed_at: Some(changed_at),
    })
}

/// The party's chart as (code, synthetic), for checking a set of codes.
async fn known_codes(
    conn: &mut PgConnection,
    party: Uuid,
) -> Result<Vec<(String, bool)>, BooksError> {
    sqlx::query_as("select code, synthetic from finance.ledger_accounts where party_id = $1")
        .bind(party)
        .fetch_all(conn)
        .await
        .map_err(map_err)
        .map_err(BooksError::from)
}

fn check_code(known: &[(String, bool)], code: &str) -> Result<(), BooksError> {
    match known.iter().find(|(c, _)| c == code) {
        None => Err(BooksError::UnknownAccount(code.to_owned())),
        Some((_, true)) => Err(BooksError::Synthetic(code.to_owned())),
        Some((_, false)) => Ok(()),
    }
}

fn first_not_open(periods: &[PeriodRow]) -> Option<&PeriodRow> {
    periods.iter().find(|p| p.status != PeriodStatus::Open)
}

/// Import the opening trial balance of a year, replacing an earlier import of
/// the same year. Rows are kept as given, both sides and signed; the set must
/// balance to the cent, every code must be an analytic account of the
/// party's chart, and every month of the year must be open.
///
/// # Errors
/// Not a company, a month closed or locked, an unknown or synthetic code, a
/// duplicate code, an unbalanced set, an empty set, or the database.
pub async fn import_opening(
    pool: &PgPool,
    party: Uuid,
    year: i32,
    as_of: NaiveDate,
    source: OpeningSource,
    rows: &[Opening],
    by: Option<UserId>,
) -> Result<Imported, BooksError> {
    if rows.is_empty() {
        return Err(BooksError::Empty("no accounts to import"));
    }
    if as_of.year() != year {
        return Err(BooksError::Invalid(format!(
            "as_of {as_of} is not in fiscal year {year}"
        )));
    }
    let mut debit: i128 = 0;
    let mut credit: i128 = 0;
    for (i, r) in rows.iter().enumerate() {
        if rows[..i].iter().any(|o| o.account_code == r.account_code) {
            return Err(BooksError::Invalid(format!(
                "account {} is listed twice",
                r.account_code
            )));
        }
        debit += i128::from(r.debit_minor);
        credit += i128::from(r.credit_minor);
    }
    if debit != credit {
        return Err(BooksError::Unbalanced {
            debit: i64::try_from(debit).unwrap_or(i64::MAX),
            credit: i64::try_from(credit).unwrap_or(i64::MAX),
        });
    }
    let mut tx = pool.begin().await.map_err(map_err)?;
    require_company(&mut tx, party).await?;
    chart::ensure(&mut tx, party).await?;
    ensure_periods(&mut tx, party, year).await?;
    if let Some(p) = first_not_open(&periods_of(&mut tx, party, year).await?) {
        return Err(BooksError::PeriodNotOpen {
            year,
            month: u32::try_from(p.month).unwrap_or(0),
            status: p.status.as_str(),
        });
    }
    let known = known_codes(&mut tx, party).await?;
    for r in rows {
        check_code(&known, &r.account_code)?;
    }
    sqlx::query("delete from finance.opening_balances where party_id = $1 and fiscal_year = $2")
        .bind(party)
        .bind(year)
        .execute(&mut *tx)
        .await
        .map_err(map_err)?;
    for r in rows {
        sqlx::query(
            "insert into finance.opening_balances
                 (party_id, fiscal_year, as_of, account_code, debit_minor, credit_minor, source, imported_by)
             values ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(party)
        .bind(year)
        .bind(as_of)
        .bind(&r.account_code)
        .bind(r.debit_minor)
        .bind(r.credit_minor)
        .bind(source.as_str())
        .bind(by.map(|u| u.0))
        .execute(&mut *tx)
        .await
        .map_err(map_err)?;
    }
    tx.commit().await.map_err(map_err)?;
    Ok(Imported {
        accounts: rows.len(),
        total_debit_minor: i64::try_from(debit).unwrap_or(i64::MAX),
        total_credit_minor: i64::try_from(credit).unwrap_or(i64::MAX),
    })
}

/// Post one entry. At least two lines, one side each, balanced, every code
/// an analytic account of the party, the month open. The source key
/// (`source`, `source_id`, `rule_id`, `rule_version`) is unique per party:
/// posting the same thing twice is a conflict, not a second entry.
///
/// # Errors
/// As listed, `DbError::Conflict` for a repeated source key, or the database.
pub async fn post(pool: &PgPool, entry: &NewEntry) -> Result<Uuid, BooksError> {
    if entry.lines.len() < 2 {
        return Err(BooksError::Empty("an entry has at least two lines"));
    }
    let mut debit: i128 = 0;
    let mut credit: i128 = 0;
    for (i, l) in entry.lines.iter().enumerate() {
        let one_side = (l.debit_minor > 0) != (l.credit_minor > 0)
            && l.debit_minor >= 0
            && l.credit_minor >= 0;
        if !one_side {
            return Err(BooksError::OneSide { position: i + 1 });
        }
        debit += i128::from(l.debit_minor);
        credit += i128::from(l.credit_minor);
    }
    if debit != credit {
        return Err(BooksError::Unbalanced {
            debit: i64::try_from(debit).unwrap_or(i64::MAX),
            credit: i64::try_from(credit).unwrap_or(i64::MAX),
        });
    }
    let year = entry.entry_date.year();
    let month = entry.entry_date.month();
    let mut tx = pool.begin().await.map_err(map_err)?;
    require_company(&mut tx, entry.party_id).await?;
    chart::ensure(&mut tx, entry.party_id).await?;
    ensure_periods(&mut tx, entry.party_id, year).await?;
    let periods = periods_of(&mut tx, entry.party_id, year).await?;
    if let Some(p) = periods
        .iter()
        .find(|p| u32::try_from(p.month).unwrap_or(0) == month)
        .filter(|p| p.status != PeriodStatus::Open)
    {
        return Err(BooksError::PeriodNotOpen {
            year,
            month,
            status: p.status.as_str(),
        });
    }
    let known = known_codes(&mut tx, entry.party_id).await?;
    for l in &entry.lines {
        check_code(&known, &l.account_code)?;
    }
    let id = Uuid::now_v7();
    sqlx::query(
        "insert into finance.journal_entries
             (id, party_id, entry_date, source_kind, source_id, rule_id, rule_version, memo)
         values ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(id)
    .bind(entry.party_id)
    .bind(entry.entry_date)
    .bind(entry.source.as_str())
    .bind(&entry.source_id)
    .bind(&entry.rule_id)
    .bind(&entry.rule_version)
    .bind(&entry.memo)
    .execute(&mut *tx)
    .await
    .map_err(map_err)?;
    for (i, l) in entry.lines.iter().enumerate() {
        sqlx::query(
            "insert into finance.journal_lines
                 (entry_id, party_id, position, account_code, debit_minor, credit_minor,
                  currency, original_minor, original_currency, counterparty, vat_code)
             values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
        )
        .bind(id)
        .bind(entry.party_id)
        .bind(i16::try_from(i + 1).unwrap_or(i16::MAX))
        .bind(&l.account_code)
        .bind(l.debit_minor)
        .bind(l.credit_minor)
        .bind(if l.currency.is_empty() {
            "EUR"
        } else {
            l.currency.as_str()
        })
        .bind(l.original_minor)
        .bind(l.original_currency.as_deref())
        .bind(l.counterparty.as_deref())
        .bind(l.vat_code.as_deref())
        .execute(&mut *tx)
        .await
        .map_err(map_err)?;
    }
    tx.commit().await.map_err(map_err)?;
    Ok(id)
}

/// Every year the party has an opening or an entry in, newest first.
async fn years_of(conn: &mut PgConnection, party: Uuid) -> Result<Vec<i32>, BooksError> {
    let rows: Vec<(i32,)> = sqlx::query_as(
        "select fiscal_year from finance.opening_balances where party_id = $1
         union select fiscal_year from finance.journal_entries where party_id = $1
         order by 1 desc",
    )
    .bind(party)
    .fetch_all(conn)
    .await
    .map_err(map_err)?;
    Ok(rows.into_iter().map(|(y,)| y).collect())
}

/// The trial balance of a year through a month: every account with an
/// opening row or a posting, code order, opening and movement side by side.
///
/// # Errors
/// The party is outside the grant, or the database.
pub async fn trial_balance(
    pool: &PgPool,
    access: &Access,
    party: Uuid,
    year: i32,
    through_month: u32,
) -> Result<TrialBalance, BooksError> {
    access.require(PartyId(party), "party")?;
    let through = through_month.clamp(1, 12);
    let mut conn = pool.acquire().await.map_err(map_err)?;
    let rows = sqlx::query_as::<_, TrialBalanceRow>(
        "with opening as (
             select account_code, debit_minor, credit_minor
               from finance.opening_balances where party_id = $1 and fiscal_year = $2),
         movement as (
             select l.account_code, sum(l.debit_minor) as d, sum(l.credit_minor) as c
               from finance.journal_lines l
               join finance.journal_entries e on e.id = l.entry_id
              where l.party_id = $1 and e.fiscal_year = $2 and e.month <= $3
              group by l.account_code)
         select a.code as account_code, a.name, a.class, a.kind,
                coalesce(o.debit_minor, 0)  as opening_debit_minor,
                coalesce(o.credit_minor, 0) as opening_credit_minor,
                coalesce(m.d, 0)::bigint    as period_debit_minor,
                coalesce(m.c, 0)::bigint    as period_credit_minor
           from finance.ledger_accounts a
           left join opening o on o.account_code = a.code
           left join movement m on m.account_code = a.code
          where a.party_id = $1 and (o.account_code is not null or m.account_code is not null)
          order by a.code",
    )
    .bind(party)
    .bind(year)
    .bind(i16::try_from(through).unwrap_or(12))
    .fetch_all(&mut *conn)
    .await
    .map_err(map_err)?;
    // An aggregate over no rows is one row holding null.
    let (as_of,): (Option<NaiveDate>,) = sqlx::query_as(
        "select min(as_of) from finance.opening_balances where party_id = $1 and fiscal_year = $2",
    )
    .bind(party)
    .bind(year)
    .fetch_one(&mut *conn)
    .await
    .map_err(map_err)?;
    let periods = periods_of(&mut conn, party, year).await?;
    let years = years_of(&mut conn, party).await?;
    Ok(TrialBalance {
        fiscal_year: year,
        through_month: through,
        opening_as_of: as_of,
        rows,
        periods,
        years,
    })
}
