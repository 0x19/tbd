//! Talking to banks.
//!
//! One trait, [`Provider`], with two implementations: the real Enable Banking
//! client and a [`Mock`] that exercises every failure the real one can have.
//! The sync worker is written against the trait and tested against the mock,
//! so "what happens on the fourth call of the day" is a unit test rather than
//! a discovery.
//!
//! The trait hands back the bank's JSON **unparsed**. That is deliberate and
//! is the single most important decision in this module: parsing lives in
//! [`crate::import`], where it was audited against 2,861 real rows. A
//! provider that parsed into its own structs would be a second parser, and
//! the two would drift -- silently, into wrong money.

pub mod adopt;
pub mod auth;
pub mod client;
pub mod connect;
pub mod error;
pub mod mock;

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use serde_json::Value;

pub use self::{client::EnableBanking, error::ProviderError, mock::Mock};
use crate::import::ProviderAccount;

/// What to ask the bank for when starting a consent.
#[derive(Debug, Clone)]
pub struct AuthorizationRequest {
    /// The bank, as the provider names it (`"Erste & Steiermärkische Bank"`).
    pub aspsp_name: String,
    /// ISO 3166-1 alpha-2.
    pub aspsp_country: String,
    /// `"business"` or `"personal"`. Erste needs it: the two are different
    /// login flows on the bank's side.
    pub psu_type: String,
    /// Where the bank sends the browser afterwards. Must be registered.
    pub redirect_url: String,
    /// Opaque, single-use, ours. Comes back on the redirect.
    pub state: String,
    /// How long the consent should last. Capped by the bank; Erste's maximum
    /// is 180 days.
    pub valid_until: DateTime<Utc>,
}

/// The bank's answer to an authorization request.
#[derive(Debug, Clone)]
pub struct Authorization {
    /// Where the person goes to log in.
    pub url: String,
    /// The provider's id for this attempt.
    pub authorization_id: String,
}

/// A consent that has been exchanged for a session.
#[derive(Debug, Clone)]
pub struct Session {
    /// The provider's session id, used on every later call.
    pub session_id: String,
    /// The accounts the consent covers, in the order the provider listed them.
    pub accounts: Vec<ProviderAccount>,
    /// When the consent lapses.
    pub valid_until: Option<DateTime<Utc>>,
    /// The whole response, for the record.
    pub raw: Value,
}

/// The state of an existing session.
#[derive(Debug, Clone)]
pub struct SessionStatus {
    /// `AUTHORIZED`, `EXPIRED`, `CLOSED`, ...
    pub status: String,
    /// Account uids, in the provider's current order. This is *not* always the
    /// order the session was created with, which is exactly the mismatch that
    /// filed 255 EUR rows on a USD account.
    pub account_uids: Vec<String>,
    /// When the consent lapses.
    pub valid_until: Option<DateTime<Utc>>,
}

/// A bank, behind whatever API it has.
///
/// Every method is a read except the two that establish consent. None of them
/// retries, backs off, or counts calls: those are the syncer's decisions,
/// made against database rows so they survive a restart. A provider that
/// silently retried a 429 would spend the day's allowance learning what the
/// first answer already said.
#[async_trait]
pub trait Provider: Send + Sync + std::fmt::Debug {
    /// The provider's name, as stored on `finance.accounts.provider`.
    fn name(&self) -> &'static str;

    /// Start a consent. The person must then visit the returned URL.
    async fn start_authorization(
        &self,
        request: &AuthorizationRequest,
    ) -> Result<Authorization, ProviderError>;

    /// Exchange the code from the redirect for a session.
    ///
    /// The code is single-use; a failed exchange consumes it, so this is never
    /// retried by anyone.
    async fn create_session(&self, code: &str) -> Result<Session, ProviderError>;

    /// The current state of a session.
    async fn session(&self, session_id: &str) -> Result<SessionStatus, ProviderError>;

    /// The balances of one account, as the bank reports them.
    async fn balances(&self, account_uid: &str) -> Result<Value, ProviderError>;

    /// Every page of transactions in the window, in order.
    ///
    /// Follows the provider's continuation until it ends. Each page is the
    /// raw response; the caller hands them to [`crate::import::ingest_pages`].
    async fn transactions(
        &self,
        account_uid: &str,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<Value>, ProviderError>;
}

/// Build the real client from configuration.
///
/// The key comes from the environment value when set, else the path. Read
/// here and nowhere else, so there is exactly one place that touches the
/// file and it never keeps the PEM around as text after parsing.
///
/// # Errors
/// Nothing is configured, the key file cannot be read, or the key is not
/// PKCS#8.
pub fn from_config(cfg: &crate::config::Provider) -> Result<EnableBanking, ProviderError> {
    if !cfg.configured() {
        return Err(ProviderError::Config {
            field: "provider",
            reason: "set FINANCE_EB_APPLICATION_ID and FINANCE_EB_PRIVATE_KEY(_PATH)".into(),
        });
    }
    let pem = if cfg.private_key.is_empty() {
        std::fs::read_to_string(&cfg.private_key_path).map_err(|e| ProviderError::Config {
            field: "private_key_path",
            reason: format!("{}: {e}", cfg.private_key_path),
        })?
    } else {
        cfg.private_key.clone()
    };
    let signer = auth::Signer::from_pem(&cfg.application_id, &pem)?;
    EnableBanking::new(
        &cfg.base_url,
        signer,
        std::time::Duration::from_secs(cfg.timeout_secs),
    )
}

/// Parse the provider's ISO 8601 timestamps, tolerating a missing field.
fn timestamp(value: Option<&Value>) -> Option<DateTime<Utc>> {
    value
        .and_then(Value::as_str)
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|t| t.with_timezone(&Utc))
}
