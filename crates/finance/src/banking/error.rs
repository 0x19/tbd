//! What can go wrong talking to a bank, and which of it is worth retrying.

use std::time::Duration;

/// A failure from a provider call.
///
/// The variants exist to answer one question at the call site: may this be
/// tried again, and when? Collapsing them into a string would lose exactly the
/// distinction the 4-calls-per-day budget is built on.
#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    /// The request never reached the bank, or the response never came back.
    ///
    /// Safe to retry *only* for a GET: the bank either did not see it or did
    /// not finish answering, and a read costs nothing twice.
    #[error("transport: {0}")]
    Transport(String),

    /// The bank refused the credentials. Signing, the key, or the application.
    ///
    /// Never retried. A 401 that is retried is a 401 in a loop.
    #[error("unauthorized: {0}")]
    Unauthorized(String),

    /// The consent is gone: expired, revoked, or never covered this account.
    ///
    /// Needs a human at a bank's login page, so it is surfaced rather than
    /// retried. This is the one that will eventually happen on its own --
    /// consents last 180 days.
    #[error("consent invalid or expired: {0}")]
    ConsentInvalid(String),

    /// The daily allowance is spent.
    ///
    /// `retry_after` is the bank's own `Retry-After` when it sends one. Enable
    /// Banking documents 6h for an exhausted allowance, but a documented
    /// default is a guess about this bank, so the header wins when present and
    /// the caller decides what to do with the absence.
    #[error("rate limited{}", .retry_after.map(|d| format!(", retry after {}s", d.as_secs())).unwrap_or_default())]
    RateLimited {
        /// What the bank asked us to wait, if it said.
        retry_after: Option<Duration>,
    },

    /// The bank answered, with something we did not ask for.
    #[error("http {status}: {body}")]
    Http {
        /// The status code.
        status: u16,
        /// The body, truncated. Never contains the token.
        body: String,
    },

    /// The response was not the shape the contract says.
    #[error("malformed response: {0}")]
    Malformed(String),

    /// Configuration the call needed and did not have.
    #[error("{field}: {reason}")]
    Config {
        /// Which setting.
        field: &'static str,
        /// What is wrong with it.
        reason: String,
    },
}

impl ProviderError {
    /// Whether a *read* may be tried again immediately.
    ///
    /// False for `RateLimited` on purpose: retrying a 429 against an allowance
    /// of four calls a day is the worst thing this code could do, because it
    /// spends tomorrow's data to learn what the response already said.
    #[must_use]
    pub fn retryable(&self) -> bool {
        matches!(self, Self::Transport(_))
    }

    /// Whether this needs a person to go and re-authorize at the bank.
    #[must_use]
    pub fn needs_consent(&self) -> bool {
        matches!(self, Self::ConsentInvalid(_))
    }
}
