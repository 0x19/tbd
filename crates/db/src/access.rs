//! Request-scoped authorization, for any service.
//!
//! This is deliberately domain-free. It knows about users, parties and grants;
//! it knows nothing about accounts, invoices, facts or documents. Any service
//! that owns data belonging to somebody resolves an [`Access`] once per request
//! and filters with it. Finance is the first caller, not the only intended one.
//!
//! The rules it exists to enforce, in one place rather than in every handler:
//!
//! 1. **The allowed set comes from the token, never from the request.** There
//!    is no constructor taking a party id from a caller.
//! 2. **Absence is the denial.** A party the user has no grant for is
//!    [`DbError::NotFound`], never a forbidden -- a forbidden confirms the
//!    party exists, which is itself a leak when the id is guessable.
//! 3. **Narrowing is allowed, widening is not.** [`Access::narrow`] intersects
//!    with what the user already has, so a UI filter can never widen the set.
//!
//! ```no_run
//! # async fn example(pool: &sqlx::PgPool, subject: &str) -> Result<(), tbd_db::DbError> {
//! let access = tbd_db::Access::resolve(pool, subject, None, "").await?;
//!
//! // Every query the service runs carries this, and only this.
//! let rows = sqlx::query("select * from finance.bank_transactions where party_id = any($1)")
//!     .bind(access.party_ids())
//!     .fetch_all(pool)
//!     .await;
//! # Ok(())
//! # }
//! ```

use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    error::{DbError, map_err},
    identity::{PartyId, UserId, ensure_user, visible_party_ids},
};

/// What one caller may see, resolved once per request.
///
/// Cheap to clone and to pass down. Holds no connection.
#[derive(Debug, Clone)]
pub struct Access {
    user: UserId,
    parties: Vec<Uuid>,
}

impl Access {
    /// Resolve the caller from the subject Envoy verified.
    ///
    /// Provisions the user on first sight, then reads their grants. `subject`
    /// is the OIDC `sub` from `x-jwt-payload` — never a value from a request
    /// body or query string.
    ///
    /// # Errors
    /// The database is unreachable, or the subject is empty.
    pub async fn resolve(
        pool: &PgPool,
        subject: &str,
        email: Option<&str>,
        display_name: &str,
    ) -> Result<Self, DbError> {
        let user = ensure_user(pool, subject, email, display_name).await?;
        Self::for_user(pool, user).await
    }

    /// Read the grants of a user already resolved.
    ///
    /// # Errors
    /// The database is unreachable.
    pub async fn for_user(pool: &PgPool, user: UserId) -> Result<Self, DbError> {
        let parties = visible_party_ids(pool, user).await?;
        Ok(Self { user, parties })
    }

    /// Build one directly, without a database.
    ///
    /// **For in-process test stacks only** -- the chaos tool runs services
    /// without a Postgres behind them, and an access-control scenario still has
    /// to be able to say who is calling and what they may read.
    ///
    /// Production never reaches this: the gRPC surface resolves a caller from
    /// the subject Envoy verified, through [`Access::resolve`], which is the
    /// only path that consults real grants. This constructor cannot widen
    /// anything by itself -- what it produces is still subject to [`narrow`],
    /// [`require`] and, against a real database, the row-level security
    /// policies.
    ///
    /// [`narrow`]: Access::narrow
    /// [`require`]: Access::require
    #[must_use]
    pub fn for_parties(user: UserId, parties: Vec<Uuid>) -> Self {
        Self { user, parties }
    }

    /// The caller.
    #[must_use]
    pub const fn user(&self) -> UserId {
        self.user
    }

    /// Every party the caller may read, for `= any($1)` filters.
    #[must_use]
    pub fn party_ids(&self) -> &[Uuid] {
        &self.parties
    }

    /// Whether this party is visible to the caller.
    #[must_use]
    pub fn allows(&self, party: PartyId) -> bool {
        self.parties.contains(&party.0)
    }

    /// Refuse a party the caller may not see.
    ///
    /// Answers [`DbError::NotFound`] rather than a forbidden, on purpose: a
    /// forbidden tells the caller the party exists.
    ///
    /// # Errors
    /// The caller has no grant on this party.
    pub fn require(&self, party: PartyId, what: &'static str) -> Result<(), DbError> {
        if self.allows(party) {
            Ok(())
        } else {
            Err(DbError::NotFound { what })
        }
    }

    /// Intersect with a caller-supplied selection, such as a UI toggle.
    ///
    /// The result is always a subset of what the user already had, so a filter
    /// arriving from a client can narrow the view but never widen it. Ids the
    /// caller has no grant for are dropped silently rather than refused —
    /// refusing would confirm which of them exist.
    #[must_use]
    pub fn narrow(&self, wanted: &[Uuid]) -> Self {
        Self {
            user: self.user,
            parties: self
                .parties
                .iter()
                .filter(|id| wanted.contains(id))
                .copied()
                .collect(),
        }
    }

    /// True when the caller can see nothing. A caller with no grants must read
    /// as an empty result, not as an error and not as everything.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.parties.is_empty()
    }

    /// Record that this caller read another party's data.
    ///
    /// Only for parties that are not the caller's own — the point is the
    /// delegated-access case, where "who saw what" has to be answerable later.
    ///
    /// # Errors
    /// The database is unreachable.
    pub async fn log_read(
        &self,
        pool: &PgPool,
        party: PartyId,
        route: &str,
        rows_seen: Option<i32>,
        trace_id: &str,
    ) -> Result<(), DbError> {
        if party.0 == self.user.0 {
            return Ok(());
        }
        sqlx::query(
            "insert into access_log (user_id, party_id, route, rows_seen, trace_id)
             values ($1, $2, $3, $4, $5)",
        )
        .bind(self.user.0)
        .bind(party.0)
        .bind(route)
        .bind(rows_seen)
        .bind(trace_id)
        .execute(pool)
        .await
        .map_err(map_err)?;
        Ok(())
    }
}

/// Bind the caller to a transaction, for row-level security policies.
///
/// `set local`, so it is scoped to this transaction and released on commit or
/// rollback. **Not** `set`: sqlx hands out pooled connections, and a
/// session-level setting would outlive the request and be inherited by whoever
/// borrowed the connection next.
///
/// A service that uses RLS calls this at the start of every transaction. A
/// policy then reads `current_setting('app.user_id')`, so a query that forgets
/// its filter returns nothing rather than everything.
///
/// # Errors
/// The database is unreachable.
pub async fn bind_rls_user(
    tx: &mut Transaction<'_, Postgres>,
    user: UserId,
) -> Result<(), DbError> {
    sqlx::query("select set_config('app.user_id', $1::text, true)")
        .bind(user.0)
        .execute(&mut **tx)
        .await
        .map_err(map_err)?;
    Ok(())
}
