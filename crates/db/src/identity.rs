//! Parties, users, and who may read whose money.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{DbError, map_err};

macro_rules! id {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(pub Uuid);

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}

id!(PartyId, "Anyone money can belong to or flow to.");
id!(
    UserId,
    "A party that can sign in. Shares its id with the party."
);
id!(
    OrgId,
    "A party that is an organisation. Shares its id with the party."
);

/// Person or organisation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartyKind {
    /// A human.
    Person,
    /// A company.
    Org,
}

impl PartyKind {
    /// The value stored in `parties.kind`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Person => "person",
            Self::Org => "org",
        }
    }
}

/// What a user may do with a party's data.
///
/// There is no `none`: absence of a row is the denial. A user who may not see a
/// party has no row for it, rather than a row with a lesser capability -- so a
/// query that forgets to check the capability still cannot widen the set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Capability {
    /// The party is the user's own, or their organisation's.
    Own,
    /// Read-only, granted by someone with `Own`.
    Read,
}

impl Capability {
    /// The value stored in `party_access.capability`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Own => "own",
            Self::Read => "read",
        }
    }
}

/// A row of `parties`.
#[derive(Debug, Clone)]
pub struct Party {
    /// The id.
    pub id: PartyId,
    /// Person or organisation.
    pub kind: PartyKind,
    /// What to call it.
    pub display_name: String,
    /// ISO 3166-1 alpha-2, when known.
    pub country_code: Option<String>,
}

/// A party a user may read, with the capability that allows it.
#[derive(Debug, Clone)]
pub struct VisibleParty {
    /// The party.
    pub party: Party,
    /// Why it is visible.
    pub capability: Capability,
}

/// Find or create the user behind a verified token, and return their party.
///
/// `subject` is the OIDC `sub` Envoy verified. Just-in-time: the first request
/// from a new identity creates the party and the user, and grants them `own` on
/// themselves. No webhook from the identity plane, no sync job.
///
/// Idempotent. Concurrent first requests from the same subject settle on one
/// row, because `subject` is unique and the insert takes the conflict path.
///
/// # Errors
/// The database is unreachable, or the row cannot be written.
pub async fn ensure_user(
    pool: &PgPool,
    subject: &str,
    email: Option<&str>,
    display_name: &str,
) -> Result<UserId, DbError> {
    if subject.is_empty() {
        return Err(DbError::Invalid {
            field: "subject",
            reason: "empty".into(),
        });
    }

    // The common case: the user exists. One round trip, no transaction.
    let existing: Option<(Uuid,)> =
        sqlx::query_as("update users set last_seen_at = now() where subject = $1 returning id")
            .bind(subject)
            .fetch_optional(pool)
            .await
            .map_err(map_err)?;
    if let Some((id,)) = existing {
        return Ok(UserId(id));
    }

    // First sight: party, user and self-grant in one transaction, so a failure
    // cannot leave a user without access to their own data.
    //
    // Serialised per subject by a transaction-scoped advisory lock. Without it
    // a stampede of first requests races: the losing transaction's `users`
    // insert takes the conflict path, so no user row exists, and its
    // `party_access` insert then violates the foreign key -- leaving an orphan
    // party behind. The lock is released on commit or rollback, is held only
    // for the first sight of one subject, and never blocks the common path
    // above.
    let mut tx = pool.begin().await.map_err(map_err)?;
    sqlx::query("select pg_advisory_xact_lock(hashtext($1))")
        .bind(subject)
        .execute(&mut *tx)
        .await
        .map_err(map_err)?;

    // Re-check under the lock: another transaction may have created it while
    // we waited.
    if let Some((id,)) = sqlx::query_as::<_, (Uuid,)>(
        "update users set last_seen_at = now() where subject = $1 returning id",
    )
    .bind(subject)
    .fetch_optional(&mut *tx)
    .await
    .map_err(map_err)?
    {
        tx.commit().await.map_err(map_err)?;
        return Ok(UserId(id));
    }

    let id = Uuid::new_v4();
    let name = if display_name.is_empty() {
        subject
    } else {
        display_name
    };
    sqlx::query("insert into parties (id, kind, display_name) values ($1, 'person', $2)")
        .bind(id)
        .bind(name)
        .execute(&mut *tx)
        .await
        .map_err(map_err)?;
    sqlx::query("insert into users (id, subject, email, last_seen_at) values ($1, $2, $3, now())")
        .bind(id)
        .bind(subject)
        .bind(email)
        .execute(&mut *tx)
        .await
        .map_err(map_err)?;
    sqlx::query("insert into party_access (user_id, party_id, capability) values ($1, $1, 'own')")
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(map_err)?;
    tx.commit().await.map_err(map_err)?;
    Ok(UserId(id))
}

/// Every party this user may read, with the capability.
///
/// **This is the only way a service learns which data a caller may see.** No
/// RPC accepts a party filter from the client; a request naming a party outside
/// this set is answered as not-found, never as forbidden, because a forbidden
/// would confirm the party exists.
///
/// # Errors
/// The database is unreachable.
pub async fn visible_parties(pool: &PgPool, user: UserId) -> Result<Vec<VisibleParty>, DbError> {
    let rows: Vec<(Uuid, String, String, Option<String>, String)> = sqlx::query_as(
        "select p.id, p.kind, p.display_name, p.country_code, a.capability
           from party_access a
           join parties p on p.id = a.party_id
          where a.user_id = $1
            and (a.expires_at is null or a.expires_at > now())
            and p.archived_at is null
          order by p.kind, p.display_name",
    )
    .bind(user.0)
    .fetch_all(pool)
    .await
    .map_err(map_err)?;

    rows.into_iter()
        .map(|(id, kind, display_name, country_code, capability)| {
            Ok(VisibleParty {
                party: Party {
                    id: PartyId(id),
                    kind: match kind.as_str() {
                        "person" => PartyKind::Person,
                        "org" => PartyKind::Org,
                        other => {
                            return Err(DbError::Internal(format!("unknown party kind {other}")));
                        }
                    },
                    display_name,
                    country_code,
                },
                capability: match capability.as_str() {
                    "own" => Capability::Own,
                    "read" => Capability::Read,
                    other => {
                        return Err(DbError::Internal(format!("unknown capability {other}")));
                    }
                },
            })
        })
        .collect()
}

/// Just the ids, for the `party_id = any($1)` filter every finance query carries.
///
/// # Errors
/// The database is unreachable.
pub async fn visible_party_ids(pool: &PgPool, user: UserId) -> Result<Vec<Uuid>, DbError> {
    let rows: Vec<(Uuid,)> = sqlx::query_as(
        "select party_id from party_access
          where user_id = $1 and (expires_at is null or expires_at > now())",
    )
    .bind(user.0)
    .fetch_all(pool)
    .await
    .map_err(map_err)?;
    Ok(rows.into_iter().map(|(id,)| id).collect())
}

/// Grant a user access to a party's data.
///
/// # Errors
/// The database is unreachable, or either party does not exist.
pub async fn grant(
    pool: &PgPool,
    user: UserId,
    party: PartyId,
    capability: Capability,
    granted_by: Option<UserId>,
    expires_at: Option<DateTime<Utc>>,
) -> Result<(), DbError> {
    sqlx::query(
        "insert into party_access (user_id, party_id, capability, granted_by, expires_at)
         values ($1, $2, $3, $4, $5)
         on conflict (user_id, party_id)
         do update set capability = excluded.capability, expires_at = excluded.expires_at",
    )
    .bind(user.0)
    .bind(party.0)
    .bind(capability.as_str())
    .bind(granted_by.map(|u| u.0))
    .bind(expires_at)
    .execute(pool)
    .await
    .map_err(map_err)?;
    Ok(())
}

/// Withdraw access. Removing the row is the denial.
///
/// # Errors
/// The database is unreachable.
pub async fn revoke(pool: &PgPool, user: UserId, party: PartyId) -> Result<(), DbError> {
    sqlx::query("delete from party_access where user_id = $1 and party_id = $2")
        .bind(user.0)
        .bind(party.0)
        .execute(pool)
        .await
        .map_err(map_err)?;
    Ok(())
}

/// Create an organisation party.
///
/// # Errors
/// The database is unreachable, or the OIB collides with another org.
pub async fn create_org(
    pool: &PgPool,
    legal_name: &str,
    oib: Option<&str>,
    country_code: Option<&str>,
    internal: bool,
) -> Result<OrgId, DbError> {
    let id = Uuid::new_v4();
    let mut tx = pool.begin().await.map_err(map_err)?;
    sqlx::query(
        "insert into parties (id, kind, display_name, country_code) values ($1, 'org', $2, $3)",
    )
    .bind(id)
    .bind(legal_name)
    .bind(country_code)
    .execute(&mut *tx)
    .await
    .map_err(map_err)?;
    sqlx::query("insert into orgs (id, legal_name, oib, internal) values ($1, $2, $3, $4)")
        .bind(id)
        .bind(legal_name)
        .bind(oib)
        .bind(internal)
        .execute(&mut *tx)
        .await
        .map_err(map_err)?;
    tx.commit().await.map_err(map_err)?;
    Ok(OrgId(id))
}
