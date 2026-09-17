//! Pulling transactions from the bank, within the bank's allowance.
//!
//! One fact shapes everything here: most banks allow **four** unattended
//! fetches per account per day, and the fifth is a 429 that costs the rest of
//! the day. So this worker is mostly bookkeeping about when *not* to call.
//!
//! Every decision is made against rows in `finance.accounts`, inside the
//! transaction that claims the account, and committed **before** the network
//! call. A crash between claim and fetch therefore spends the call without
//! making it -- the safe direction, because the alternative is a restart that
//! spends the same call twice and blows the day's allowance on the second
//! replica's first tick.
//!
//! The shape is `tick()` testable on its own and `run()` looping it under a
//! cancellation token, like `crates/ledger/src/sweeper.rs`.

use std::{sync::Arc, time::Duration};

use chrono::{DateTime, Days, NaiveDate, Utc};
use sqlx::{PgPool, Postgres, Transaction};
use tbd_db::{DbError, map_err};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::{
    banking::{Provider, ProviderError},
    categorise,
    config::Sync as SyncConfig,
    import::{ProviderAccount, SeenKeys, ingest_balances, ingest_pages},
};

/// Why a sync was started. Decides which share of the budget it spends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trigger {
    /// The loop found the account due. Spends from `scheduled_budget`.
    Scheduled,
    /// A person asked. Spends from the reserve above `scheduled_budget`, up
    /// to `budget_per_day`.
    Manual,
}

impl Trigger {
    fn as_str(self) -> &'static str {
        match self {
            Self::Scheduled => "scheduled",
            Self::Manual => "manual",
        }
    }
}

/// How one account's sync ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// Fetched and ingested.
    Ok {
        /// Pages the bank returned.
        pages: usize,
        /// Rows new to us.
        inserted: usize,
        /// Rows already present.
        duplicates: usize,
    },
    /// The bank said to wait. `sync_backoff_until` is set.
    RateLimited,
    /// The consent is gone; the connection is marked and a person is needed.
    ConsentInvalid,
    /// The request did not complete. The call was still counted.
    Transport,
    /// Something else, recorded on the run.
    Error,
}

/// Why an account was not fetched this time. None of these are errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Skipped {
    /// The day's share is spent. The bank was not called.
    BudgetSpent,
    /// `sync_backoff_until` is in the future.
    BackingOff,
    /// The connection is not `authorized`.
    NoConsent,
    /// Another worker holds the account right now. It will be looked at
    /// again next tick; a person asking gets told rather than a not-found.
    Busy,
}

/// What one tick did.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Tick {
    /// Accounts fetched, with their outcomes.
    pub synced: Vec<(Uuid, Outcome)>,
    /// Accounts that were due but not fetched, and why.
    pub skipped: Vec<(Uuid, Skipped)>,
}

/// A sync failure that is ours, not the bank's.
#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    /// The database.
    #[error(transparent)]
    Db(#[from] DbError),
    /// The account does not exist or is not syncable.
    #[error("account not found")]
    NotFound,
}

/// One account as the claim returned it.
#[derive(Debug, sqlx::FromRow)]
struct Claimed {
    id: Uuid,
    party_id: Uuid,
    connection_id: Option<Uuid>,
    provider_uid: String,
    iban: Option<String>,
    currency: String,
    name: String,
    last_booked_through: Option<NaiveDate>,
    last_synced_at: Option<DateTime<Utc>>,
    sync_budget_day: Option<NaiveDate>,
    sync_budget_used: i32,
    sync_backoff_until: Option<DateTime<Utc>>,
    connection_status: Option<String>,
}

/// The worker.
#[derive(Debug)]
pub struct Syncer<P: Provider> {
    pool: PgPool,
    provider: Arc<P>,
    config: SyncConfig,
}

impl<P: Provider + 'static> Syncer<P> {
    /// Build one. Nothing runs until `tick` or `run`.
    #[must_use]
    pub fn new(pool: PgPool, provider: Arc<P>, config: SyncConfig) -> Self {
        Self {
            pool,
            provider,
            config,
        }
    }

    /// Loop `tick` until cancelled.
    pub async fn run(self, cancel: CancellationToken) {
        let interval = Duration::from_secs(self.config.interval_secs.max(1));
        loop {
            match self.tick(Utc::now()).await {
                Ok(tick) => {
                    if !tick.synced.is_empty() || !tick.skipped.is_empty() {
                        tracing::info!(
                            synced = tick.synced.len(),
                            skipped = tick.skipped.len(),
                            "sync tick"
                        );
                    }
                }
                Err(error) => tracing::warn!(%error, "sync tick failed"),
            }
            tokio::select! {
                () = cancel.cancelled() => return,
                () = tokio::time::sleep(interval) => {}
            }
        }
    }

    /// One pass over every due account.
    ///
    /// `now` is a parameter so a test can walk the clock across a day
    /// boundary and watch the budget reset.
    ///
    /// # Errors
    /// The database.
    pub async fn tick(&self, now: DateTime<Utc>) -> Result<Tick, SyncError> {
        let mut report = Tick::default();
        let min_interval = chrono::Duration::seconds(
            i64::try_from(self.config.min_interval_secs).unwrap_or(i64::MAX),
        );

        // Candidates: enabled, on this provider, and not fetched too recently.
        // Cheap to over-select here; the claim decides.
        let due: Vec<(Uuid,)> = sqlx::query_as(
            "select id from finance.accounts
              where sync_enabled and provider = $1
                and (last_synced_at is null or last_synced_at <= $2)
              order by last_synced_at nulls first",
        )
        .bind(self.provider.name())
        .bind(now - min_interval)
        .fetch_all(&self.pool)
        .await
        .map_err(map_err)?;

        for (id,) in due {
            match self.sync_one(id, Trigger::Scheduled, now).await {
                Ok(Ok(outcome)) => report.synced.push((id, outcome)),
                Ok(Err(skipped)) => report.skipped.push((id, skipped)),
                Err(SyncError::NotFound) => {}
                Err(e) => return Err(e),
            }
        }
        Ok(report)
    }

    /// Fetch one account now, from the reserve budget.
    ///
    /// # Errors
    /// The database, or the account does not exist.
    pub async fn refresh(
        &self,
        account_id: Uuid,
        now: DateTime<Utc>,
    ) -> Result<Result<Outcome, Skipped>, SyncError> {
        self.sync_one(account_id, Trigger::Manual, now).await
    }

    /// Claim, fetch, record.
    async fn sync_one(
        &self,
        account_id: Uuid,
        trigger: Trigger,
        now: DateTime<Utc>,
    ) -> Result<Result<Outcome, Skipped>, SyncError> {
        // 1. Claim: lock the row, decide against it, commit the decision.
        //    The transaction is short and holds no network call.
        let mut tx = self.pool.begin().await.map_err(map_err)?;
        let Some(claimed) = self.claim(&mut tx, account_id, trigger, now).await? else {
            return Err(SyncError::NotFound);
        };
        let claimed = match claimed {
            Ok(c) => c,
            Err(skipped) => {
                tx.rollback().await.map_err(map_err)?;
                return Ok(Err(skipped));
            }
        };
        let (from, to) = self.window(&claimed, now);
        let run_id = Uuid::new_v4();
        sqlx::query(
            "insert into finance.sync_runs (id, account_id, trigger, date_from, date_to)
             values ($1, $2, $3, $4, $5)",
        )
        .bind(run_id)
        .bind(claimed.id)
        .bind(if claimed.last_synced_at.is_none() {
            "initial"
        } else {
            trigger.as_str()
        })
        .bind(from)
        .bind(to)
        .execute(&mut *tx)
        .await
        .map_err(map_err)?;
        tx.commit().await.map_err(map_err)?;

        // 2. Fetch, with no lock held. Balances first: a snapshot is what a
        //    later reconciliation checks the transactions against.
        let outcome = self.fetch(&claimed, from, to).await;

        // 3. Record, whatever happened.
        let outcome = match outcome {
            Fetched::Ok { balances, pages } => {
                self.record_ok(&claimed, run_id, &balances, &pages, now)
                    .await?
            }
            Fetched::Failed(error) => self.record_failure(&claimed, run_id, &error, now).await?,
        };
        Ok(Ok(outcome))
    }

    /// Lock the account and decide whether this call may be spent.
    ///
    /// Returns `None` when there is no such account, `Some(Err)` when the
    /// account exists but must not be fetched now, and `Some(Ok)` with the
    /// budget already incremented -- inside the caller's transaction, so the
    /// commit is what spends it.
    async fn claim(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        account_id: Uuid,
        trigger: Trigger,
        now: DateTime<Utc>,
    ) -> Result<Option<Result<Claimed, Skipped>>, SyncError> {
        // Exists at all? Asked separately so a row another worker holds is
        // reported as busy rather than as missing.
        let exists: Option<(bool,)> =
            sqlx::query_as("select sync_enabled from finance.accounts where id = $1")
                .bind(account_id)
                .fetch_optional(&mut **tx)
                .await
                .map_err(map_err)?;
        match exists {
            None | Some((false,)) => return Ok(None),
            Some((true,)) => {}
        }
        let Some(row) = sqlx::query_as::<_, Claimed>(
            "select a.id, a.party_id, a.connection_id, a.provider_uid, a.iban, a.currency,
                    a.name, a.last_booked_through, a.last_synced_at, a.sync_budget_day,
                    a.sync_budget_used, a.sync_backoff_until, c.status as connection_status
               from finance.accounts a
               left join finance.connections c on c.id = a.connection_id
              where a.id = $1 and a.sync_enabled
                for update of a skip locked",
        )
        .bind(account_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_err)?
        else {
            return Ok(Some(Err(Skipped::Busy)));
        };

        // No consent, no call. An account imported from the prototype has no
        // connection yet; it is still syncable once one is linked.
        if row.connection_status.as_deref() != Some("authorized") {
            return Ok(Some(Err(Skipped::NoConsent)));
        }
        if row.sync_backoff_until.is_some_and(|until| until > now) {
            return Ok(Some(Err(Skipped::BackingOff)));
        }

        // The budget day is the UTC date. The bank's own window is unmeasured
        // (prototype/bank/FINDINGS.md); a UTC day is never *more* permissive
        // than a Zagreb day, and a rolling window shows up as a 429 that the
        // backoff then honours.
        let today = now.date_naive();
        let used = if row.sync_budget_day == Some(today) {
            row.sync_budget_used
        } else {
            0
        };
        let cap = match trigger {
            Trigger::Scheduled => self.config.scheduled_budget,
            Trigger::Manual => self.config.budget_per_day,
        };
        if u32::try_from(used).unwrap_or(u32::MAX) >= cap {
            return Ok(Some(Err(Skipped::BudgetSpent)));
        }

        sqlx::query(
            "update finance.accounts
                set sync_budget_day = $2, sync_budget_used = $3
              where id = $1",
        )
        .bind(row.id)
        .bind(today)
        .bind(used + 1)
        .execute(&mut **tx)
        .await
        .map_err(map_err)?;
        Ok(Some(Ok(row)))
    }

    /// The date window to ask for.
    ///
    /// A routine fetch starts `overlap_days` before the last booked date, so
    /// a transaction booked late is not missed; dedup makes the overlap free.
    /// A first fetch reaches back `initial_history_days`.
    fn window(&self, account: &Claimed, now: DateTime<Utc>) -> (NaiveDate, NaiveDate) {
        let today = now.date_naive();
        let initial = today
            .checked_sub_days(Days::new(u64::from(self.config.initial_history_days)))
            .unwrap_or(today);
        let from = account.last_booked_through.map_or(initial, |through| {
            through
                .checked_sub_days(Days::new(u64::from(self.config.overlap_days)))
                .unwrap_or(through)
                .max(initial)
        });
        (from, today)
    }

    async fn fetch(&self, account: &Claimed, from: NaiveDate, to: NaiveDate) -> Fetched {
        let balances = match self.provider.balances(&account.provider_uid).await {
            Ok(v) => v,
            Err(e) => return Fetched::Failed(e),
        };
        match self
            .provider
            .transactions(&account.provider_uid, from, to)
            .await
        {
            Ok(pages) => Fetched::Ok { balances, pages },
            Err(e) => Fetched::Failed(e),
        }
    }

    /// Ingest what the bank sent and write the success onto the run and the
    /// account.
    async fn record_ok(
        &self,
        account: &Claimed,
        run_id: Uuid,
        balances: &serde_json::Value,
        pages: &[serde_json::Value],
        now: DateTime<Utc>,
    ) -> Result<Outcome, SyncError> {
        let provider_account = ProviderAccount {
            uid: account.provider_uid.clone(),
            iban: account.iban.clone(),
            currency: account.currency.clone(),
            name: account.name.clone(),
        };
        let recorded = ingest_balances(&self.pool, account.id, balances).await?;
        let mut seen = SeenKeys::new();
        let got = ingest_pages(
            &self.pool,
            account.party_id,
            account.id,
            &provider_account,
            pages,
            "provider",
            &mut seen,
        )
        .await?;
        let through = latest_booking(pages).or(account.last_booked_through);
        sqlx::query(
            "update finance.accounts
                set last_synced_at = $2, last_booked_through = $3,
                    last_sync_status = 'ok', last_sync_error = null,
                    sync_backoff_until = null
              where id = $1",
        )
        .bind(account.id)
        .bind(now)
        .bind(through)
        .execute(&self.pool)
        .await
        .map_err(map_err)?;
        sqlx::query(
            "update finance.sync_runs
                set finished_at = $2, outcome = 'ok', pages = $3, inserted = $4,
                    duplicates = $5, skipped = $6, balances = $7
              where id = $1",
        )
        .bind(run_id)
        .bind(now)
        .bind(i32::try_from(pages.len()).unwrap_or(i32::MAX))
        .bind(i32::try_from(got.inserted).unwrap_or(i32::MAX))
        .bind(i32::try_from(got.duplicates).unwrap_or(i32::MAX))
        .bind(i32::try_from(got.skipped).unwrap_or(i32::MAX))
        .bind(i32::try_from(recorded).unwrap_or(i32::MAX))
        .execute(&self.pool)
        .await
        .map_err(map_err)?;

        // New rows get categorised now, not on the next manual pass. A pass
        // is a pure function of the rules, so running it after every sync
        // with new rows is free of surprises.
        if got.inserted > 0 {
            categorise::apply_rules(&self.pool, account.party_id).await?;
        }
        Ok(Outcome::Ok {
            pages: pages.len(),
            inserted: got.inserted,
            duplicates: got.duplicates,
        })
    }

    /// Write a failure onto the run and the account, and set the backoff.
    async fn record_failure(
        &self,
        account: &Claimed,
        run_id: Uuid,
        error: &ProviderError,
        now: DateTime<Utc>,
    ) -> Result<Outcome, SyncError> {
        let (status, outcome) = match error {
            ProviderError::RateLimited { .. } => ("rate_limited", Outcome::RateLimited),
            ProviderError::ConsentInvalid(_) => ("consent_invalid", Outcome::ConsentInvalid),
            ProviderError::Transport(_) => ("transport", Outcome::Transport),
            _ => ("error", Outcome::Error),
        };
        let backoff_until = match error {
            ProviderError::RateLimited { retry_after } => {
                let wait =
                    retry_after.unwrap_or(Duration::from_secs(self.config.default_backoff_secs));
                Some(
                    now + chrono::Duration::from_std(wait)
                        .unwrap_or_else(|_| chrono::Duration::hours(6)),
                )
            }
            _ => None,
        };
        // `last_synced_at` moves even on failure: a failing account must not
        // be retried every tick, only every `min_interval`.
        sqlx::query(
            "update finance.accounts
                set last_synced_at = $2, last_sync_status = $3,
                    last_sync_error = $4, sync_backoff_until = $5
              where id = $1",
        )
        .bind(account.id)
        .bind(now)
        .bind(status)
        .bind(error.to_string())
        .bind(backoff_until)
        .execute(&self.pool)
        .await
        .map_err(map_err)?;
        sqlx::query(
            "update finance.sync_runs set finished_at = $2, outcome = $3, error = $4
              where id = $1",
        )
        .bind(run_id)
        .bind(now)
        .bind(status)
        .bind(error.to_string())
        .execute(&self.pool)
        .await
        .map_err(map_err)?;

        // A consent that is gone is gone for every account on the connection.
        // Marking it once stops the others from each spending a call to learn
        // the same thing.
        if let (ProviderError::ConsentInvalid(_), Some(connection)) = (error, account.connection_id)
        {
            sqlx::query(
                "update finance.connections set status = 'expired', updated_at = $2
                  where id = $1 and status = 'authorized'",
            )
            .bind(connection)
            .bind(now)
            .execute(&self.pool)
            .await
            .map_err(map_err)?;
        }
        Ok(outcome)
    }
}

/// What the bank sent, or why it did not.
enum Fetched {
    Ok {
        balances: serde_json::Value,
        pages: Vec<serde_json::Value>,
    },
    Failed(ProviderError),
}

/// The latest `booking_date` across the pages, for the watermark.
fn latest_booking(pages: &[serde_json::Value]) -> Option<NaiveDate> {
    pages
        .iter()
        .filter_map(|p| p.get("transactions")?.as_array())
        .flatten()
        .filter_map(|t| t.get("booking_date")?.as_str())
        .filter_map(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .max()
}
