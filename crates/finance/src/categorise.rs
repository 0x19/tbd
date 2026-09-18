//! Applying rules to transactions.
//!
//! Two properties this exists to guarantee, both of which fail silently:
//!
//! **A human always wins.** A category set by a person is `declared`; one set
//! by a rule is `inferred`. Applying rules never touches a `declared` row. The
//! failure otherwise is quiet and infuriating: you correct a categorisation,
//! the next sync runs, and it reverts.
//!
//! **Precedence is total and stable.** Rules are ordered by `priority` then
//! `id`, so two rules matching one transaction resolve the same way on every
//! run and on every replica. Leaving it to insertion order means a
//! categorisation that changes between two runs over identical data, which is
//! the kind of bug that gets blamed on the bank.
//!
//! Internal transfers are not categorised here at all. They are derived from
//! whether the counterparty is an account we hold
//! (`finance.transactions_enriched`), because a category you can forget to
//! apply is one somebody will forget.

use sqlx::PgPool;
use tbd_db::{DbError, map_err};
use uuid::Uuid;

/// What one pass did.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Applied {
    /// Rows a rule claimed.
    pub categorised: usize,
    /// Rows left alone because a person had already decided.
    pub declared_kept: usize,
    /// Rows no rule matched.
    pub unmatched: usize,
}

/// Apply every enabled rule for a party, in priority order.
///
/// A pass is a **pure function of the rules**: every `inferred` categorisation
/// is cleared first, then each rule in priority order claims only rows that are
/// still unclaimed. `declared` rows are never touched.
///
/// The obvious alternative -- let each rule update whatever it matches -- is
/// wrong, and quietly: applying in priority order, the *last* rule to run is
/// the lowest priority, and it overwrites what the highest already decided.
/// Guarding on "not claimed by me" does not help, because a different rule is
/// always a different rule. This is not hypothetical; it is what the first
/// version of this function did.
///
/// Clearing first also means editing a rule takes effect. Without it, a row
/// owned by a low-priority rule would never be reconsidered by a high-priority
/// one added later.
///
/// # Errors
/// The database is unreachable.
pub async fn apply_rules(pool: &PgPool, party_id: Uuid) -> Result<Applied, DbError> {
    let mut report = Applied::default();

    // Start from nothing a rule decided. `declared` survives: a person's
    // decision outlives any number of passes.
    sqlx::query(
        "update finance.bank_transactions
            set category_id = null, category_source = null,
                category_rule_id = null, categorised_at = null
          where party_id = $1 and category_source = 'inferred'",
    )
    .bind(party_id)
    .execute(pool)
    .await
    .map_err(map_err)?;

    // `hits` is how many rows a rule owns after this pass, not how many times
    // it has ever run. A cumulative count answers no question anybody asks; the
    // useful signal is "this rule claims nothing", which means it is wrong or
    // obsolete.
    sqlx::query("update finance.rules set hits = 0 where party_id = $1")
        .bind(party_id)
        .execute(pool)
        .await
        .map_err(map_err)?;

    // Priority ascending, then id: a total order, so ties break the same way
    // everywhere. `for update skip locked` is not used -- this is a whole-party
    // pass, and two of them racing would fight over the same rows.
    let rules: Vec<(Uuid, Uuid)> = sqlx::query_as(
        "select id, category_id from finance.rules
          where party_id = $1 and enabled
          order by priority asc, id asc",
    )
    .bind(party_id)
    .fetch_all(pool)
    .await
    .map_err(map_err)?;

    for (rule_id, category_id) in rules {
        // One statement per rule, in priority order. `category_id is null` is
        // what makes the order mean something: a rule claims only what no
        // earlier, higher-priority rule took.
        let claimed = sqlx::query(
            "update finance.bank_transactions t
                set category_id = $2,
                    category_source = 'inferred',
                    category_rule_id = $3,
                    categorised_at = now()
              from finance.rules r
             where r.id = $3
               and t.party_id = $1
               and t.category_id is null
               -- Normalised on both sides: the real data spells the same payee
               -- `DRŽAVNI PRORAČUN` and `DRZAVNI PRORACUN`, and a rule that
               -- matched only one would silently miss half the rows.
               -- A pattern is a substring unless anchored: a leading `^` pins
               -- it to the start, a trailing `$` to the end. `INA ` alone
               -- claimed LESNINA, PERUTNINA, FINA and every TRGOVINA; `^INA `
               -- claims the fuel stations.
               and (r.match_counterparty_like is null
                    or finance.normalise(coalesce(t.counterparty_name, ''))
                       like case when left(r.match_counterparty_like, 1) = '^' then '' else '%' end
                            || rtrim(ltrim(r.match_counterparty_like, '^'), '$')
                            || case when right(r.match_counterparty_like, 1) = '$' then '' else '%' end)
               and (r.match_counterparty_iban is null
                    or t.counterparty_iban = r.match_counterparty_iban)
               and (r.match_remittance_like is null
                    or finance.normalise(coalesce(t.remittance, ''))
                       like case when left(r.match_remittance_like, 1) = '^' then '' else '%' end
                            || rtrim(ltrim(r.match_remittance_like, '^'), '$')
                            || case when right(r.match_remittance_like, 1) = '$' then '' else '%' end)
               and (r.match_currency is null or t.currency = r.match_currency)
               and (r.match_credit_debit is null or t.credit_debit = r.match_credit_debit)
               and (r.match_amount_min_minor is null or t.amount_minor >= r.match_amount_min_minor)
               and (r.match_amount_max_minor is null or t.amount_minor <= r.match_amount_max_minor)",
        )
        .bind(party_id)
        .bind(category_id)
        .bind(rule_id)
        .execute(pool)
        .await
        .map_err(map_err)?
        .rows_affected();

        if claimed > 0 {
            sqlx::query("update finance.rules set hits = $2 where id = $1")
                .bind(rule_id)
                .bind(i64::try_from(claimed).unwrap_or(i64::MAX))
                .execute(pool)
                .await
                .map_err(map_err)?;
        }
        report.categorised += usize::try_from(claimed).unwrap_or(0);
    }

    let (declared,): (i64,) = sqlx::query_as(
        "select count(*) from finance.bank_transactions
          where party_id = $1 and category_source = 'declared'",
    )
    .bind(party_id)
    .fetch_one(pool)
    .await
    .map_err(map_err)?;
    let (unmatched,): (i64,) = sqlx::query_as(
        "select count(*) from finance.bank_transactions
          where party_id = $1 and category_id is null",
    )
    .bind(party_id)
    .fetch_one(pool)
    .await
    .map_err(map_err)?;

    report.declared_kept = usize::try_from(declared).unwrap_or(0);
    report.unmatched = usize::try_from(unmatched).unwrap_or(0);
    Ok(report)
}

/// Set a category by hand. Beats every rule, now and on every later pass.
///
/// # Errors
/// The database is unreachable, or the transaction does not exist.
pub async fn declare(pool: &PgPool, transaction: Uuid, category: Uuid) -> Result<(), DbError> {
    let affected = sqlx::query(
        "update finance.bank_transactions
            set category_id = $2, category_source = 'declared',
                category_rule_id = null, categorised_at = now()
          where id = $1",
    )
    .bind(transaction)
    .bind(category)
    .execute(pool)
    .await
    .map_err(map_err)?
    .rows_affected();
    if affected == 0 {
        return Err(DbError::NotFound {
            what: "transaction",
        });
    }
    Ok(())
}

/// One line of the monthly picture.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SummaryRow {
    /// `YYYY-MM`.
    pub month: String,
    /// Whose money.
    pub party_id: Uuid,
    /// The category, or null when nothing has claimed the rows yet.
    pub category_id: Option<Uuid>,
    /// Category name, or null when nothing has claimed the rows yet.
    pub category: Option<String>,
    /// `expense`, `income`, `transfer`, `tax`, `capital`, or null.
    pub kind: Option<String>,
    /// ISO 4217. Totals never mix currencies.
    pub currency: String,
    /// Signed minor units.
    pub total_minor: i64,
    /// How many transactions.
    pub count: i64,
    /// Transfers between accounts the same person holds, when included.
    pub internal: bool,
}

/// The monthly breakdown for a set of parties.
///
/// Grouped by currency as well as category: a cross-currency total is never
/// computed, because it would need a rate and a rate needs a date and a source.
///
/// `include_internal` is false by default and should stay that way for any
/// question about spending. Transfers between accounts the same person holds
/// are real on both sides -- an owner draw is a genuine company expense and a
/// genuine personal receipt -- but counting them in a combined view is the same
/// euros twice.
///
/// # Errors
/// The database is unreachable.
pub async fn monthly_summary(
    pool: &PgPool,
    parties: &[Uuid],
    include_internal: bool,
    from_month: Option<&str>,
) -> Result<Vec<SummaryRow>, DbError> {
    if parties.is_empty() {
        return Ok(Vec::new());
    }
    sqlx::query_as::<_, SummaryRow>(
        "select to_char(t.booking_date, 'YYYY-MM') as month,
                t.party_id as party_id,
                c.id as category_id,
                c.name as category,
                c.kind as kind,
                t.currency as currency,
                sum(t.amount_minor)::bigint as total_minor,
                count(*)::bigint as count,
                t.internal as internal
           from finance.transactions_enriched t
           left join finance.categories c on c.id = t.category_id
          where t.party_id = any($1)
            and t.status = 'booked'
            and t.booking_date is not null
            and ($2 or not t.internal)
            and ($3::text is null or to_char(t.booking_date, 'YYYY-MM') >= $3)
          group by 1, 2, 3, 4, 5, 6, t.internal
          order by 1 desc, 7 asc",
    )
    .bind(parties)
    .bind(include_internal)
    .bind(from_month)
    .fetch_all(pool)
    .await
    .map_err(map_err)
}
