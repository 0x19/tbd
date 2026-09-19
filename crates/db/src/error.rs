//! The one place a `sqlx` error becomes something a service can act on.

/// What went wrong talking to the database.
///
/// Deliberately small. A service that needs richer cases wraps this rather than
/// extending it: the distinction that matters to every caller is whether the
/// failure is retryable ([`DbError::Unavailable`]) or not.
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    /// The named thing does not exist.
    #[error("{what} not found")]
    NotFound {
        /// Which kind of thing: `user`, `party`, `account`.
        what: &'static str,
    },
    /// The request is malformed and will not succeed on retry.
    #[error("invalid {field}: {reason}")]
    Invalid {
        /// Which field.
        field: &'static str,
        /// Why.
        reason: String,
    },
    /// The request contradicts one already applied, such as a unique violation.
    #[error("conflict: {reason}")]
    Conflict {
        /// Why.
        reason: String,
    },
    /// The database could not be reached. Retryable.
    #[error("database unavailable: {source}")]
    Unavailable {
        /// Cause.
        #[source]
        source: Box<dyn std::error::Error + Send + Sync + 'static>,
    },
    /// A row could not be decoded, or an invariant broke. Not retryable.
    #[error("database internal: {0}")]
    Internal(String),
}

impl DbError {
    /// Wrap a cause as retryable.
    pub fn unavailable(source: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::Unavailable {
            source: Box::new(source),
        }
    }

    /// Whether a caller should try again later.
    #[must_use]
    pub const fn is_retryable(&self) -> bool {
        matches!(self, Self::Unavailable { .. })
    }
}

/// Map a `sqlx` error to [`DbError`].
///
/// The split that matters: everything that means "the database is not reachable
/// right now" becomes [`DbError::Unavailable`] so readiness and retry logic can
/// key on one variant, and a unique violation becomes [`DbError::Conflict`] so
/// an idempotent insert can recognise its own replay.
#[must_use]
pub fn map_err(e: sqlx::Error) -> DbError {
    match e {
        sqlx::Error::RowNotFound => DbError::NotFound { what: "row" },
        sqlx::Error::Database(db) => {
            if db.is_unique_violation() {
                DbError::Conflict {
                    reason: db.message().to_owned(),
                }
            } else if db.is_foreign_key_violation() {
                DbError::Invalid {
                    field: "reference",
                    reason: db.message().to_owned(),
                }
            } else if db.is_check_violation() {
                DbError::Invalid {
                    field: "value",
                    reason: db.message().to_owned(),
                }
            } else {
                DbError::Internal(format!("database: {}", db.message()))
            }
        }
        sqlx::Error::PoolTimedOut
        | sqlx::Error::PoolClosed
        | sqlx::Error::WorkerCrashed
        | sqlx::Error::Io(_)
        | sqlx::Error::Tls(_)
        | sqlx::Error::Protocol(_)
        | sqlx::Error::Configuration(_) => DbError::unavailable(e),
        other => DbError::Internal(other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::{DbError, map_err};

    #[test]
    fn row_not_found_is_not_retryable() {
        let e = map_err(sqlx::Error::RowNotFound);
        assert!(!e.is_retryable());
        assert!(matches!(e, DbError::NotFound { .. }));
    }

    #[test]
    fn a_closed_pool_is_retryable() {
        assert!(map_err(sqlx::Error::PoolClosed).is_retryable());
        assert!(map_err(sqlx::Error::PoolTimedOut).is_retryable());
    }

    #[test]
    fn the_message_says_which_field() {
        let e = DbError::Invalid {
            field: "currency",
            reason: "not ISO 4217".into(),
        };
        assert_eq!(e.to_string(), "invalid currency: not ISO 4217");
    }
}
