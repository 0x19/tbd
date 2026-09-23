//! The shared database layer.
//!
//! Migrations are **per project, not per service**: foreign keys cross service
//! boundaries -- a finance account points at a party in `public` -- so there is
//! one ordered set in `/migrations` and one database. Services own schemas, not
//! migration directories.
//!
//! What lives here is what every store needs and none of them should each
//! invent: the migrator, the pool, the mapping from `sqlx::Error` to something
//! a caller can act on, the identity tables that ownership keys point at, and
//! [`Access`] — request-scoped authorization that is domain-free on purpose.
//! It knows about users, parties and grants and nothing about accounts,
//! invoices, facts or documents, so any service that owns data belonging to
//! somebody can use it unchanged. Finance is the first caller, not the only
//! intended one.
//!
//! It is deliberately **not** `tbd-common`. ARCHITECTURE invariant 3 keeps that
//! crate transport-free, and pulling `sqlx` into every binary through it would
//! break the spirit of that.
//!
//! ```no_run
//! # async fn example() -> Result<(), tbd_db::DbError> {
//! let pool = tbd_db::connect_lazy(&tbd_db::PgOptions {
//!     url: "postgres://localhost/tbd".into(),
//!     ..tbd_db::PgOptions::default()
//! })?;
//!
//! // Refuse to serve against a schema older than this binary.
//! tbd_db::require_current_schema(&pool).await?;
//!
//! // The caller's identity comes from the token Envoy verified, never from
//! // the request body.
//! let user = tbd_db::ensure_user(&pool, "kratos-identity-uuid", None, "Someone").await?;
//! let visible = tbd_db::visible_party_ids(&pool, user).await?;
//! # Ok(())
//! # }
//! ```

#![forbid(unsafe_code)]

mod access;
mod error;
mod identity;
mod pool;
#[cfg(feature = "testing")]
pub mod testing;

pub use access::{Access, bind_rls_user};
pub use error::{DbError, map_err};
pub use identity::{
    Capability, OrgId, Party, PartyId, PartyKind, UserId, VisibleParty, create_org, ensure_user,
    grant, revoke, visible_parties, visible_party_ids,
};
pub use pool::{
    MIGRATOR, PgOptions, applied_migrations, connect_lazy, expected_migrations, migrate,
    pool_counts, require_current_schema,
};
