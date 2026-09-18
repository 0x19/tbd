//! Connectors: linked external accounts the system pulls documents from.
//!
//! The bank link was the first; this is the general shape. A [`Kind`] is a
//! registry entry -- what a connector of that kind needs to link, and what
//! it can do -- and a [`Connector`] is the code behind it. Rows live in
//! `finance.connectors` with their credentials sealed ([`crypto`]); the store
//! is the only thing that opens them, and only to hand them to the kind for
//! the duration of a call.
//!
//! Adding a kind is one file implementing [`Connector`] and one line in
//! [`registry`]. The UI reads the registry, so a new kind appears there
//! without a UI change.

pub mod crypto;
pub mod gmail;
pub mod store;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// How a kind gets linked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Auth {
    /// Send the person to the provider; the callback carries a code.
    Oauth,
    /// The person pastes a token the provider issued.
    Token,
}

/// A registry entry: what the UI shows when offering a kind to add.
#[derive(Debug, Clone, Serialize)]
pub struct Kind {
    /// The name rows carry: `gmail`.
    pub name: &'static str,
    /// What a person reads.
    pub label: &'static str,
    /// One line on what it pulls.
    pub description: &'static str,
    /// How it links.
    pub auth: Auth,
    /// What the person is told before linking (scopes, what is read).
    pub consent_note: &'static str,
    /// Whether the service has what it needs to link this kind (an OAuth
    /// client, say). False means "configure the server first", not "hidden".
    pub configured: bool,
}

/// What a link attempt needs from the environment.
#[derive(Debug, Clone)]
pub struct AuthContext {
    /// Ours, single-use, already stored on the pending row.
    pub state: String,
    /// Where the provider sends the browser back.
    pub redirect_url: String,
}

/// What a completed link yields: the credential to seal and the identity.
#[derive(Debug, Clone)]
pub struct Linked {
    /// The kind's credential, as JSON. Sealed by the store; never logged.
    pub credentials: serde_json::Value,
    /// The account this is: an email address, an account id.
    pub external_id: String,
    /// What a person sees.
    pub label: String,
}

/// One thing a pull found.
#[derive(Debug, Clone)]
pub struct Found {
    /// The provider's id for it; a document seen before is skipped by it.
    pub external_ref: String,
    /// The file name to keep, `Hetzner_2026-08-11.pdf`.
    pub filename: String,
    /// `application/pdf`, mostly.
    pub content_type: String,
    /// The bytes.
    pub bytes: Vec<u8>,
    /// The message subject, or the portal's title for it.
    pub subject: String,
    /// Who sent it.
    pub sender: String,
    /// When it arrived.
    pub received_at: Option<DateTime<Utc>>,
}

/// What a pull did.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Pulled {
    /// Candidates the provider listed.
    pub found: usize,
    /// New documents stored.
    pub stored: usize,
    /// Seen before, or not a document.
    pub skipped: usize,
}

/// A connector failure.
#[derive(Debug, thiserror::Error)]
pub enum ConnectorError {
    /// The provider could not be reached or answered badly.
    #[error("provider: {0}")]
    Provider(String),
    /// The link is gone: revoked, expired, or the token refused.
    #[error("not linked: {0}")]
    Unlinked(String),
    /// The server lacks what this kind needs (an OAuth client).
    #[error("not configured: {0}")]
    Unconfigured(String),
    /// Bad input.
    #[error("{0}")]
    Invalid(String),
}

/// The code behind a kind.
#[async_trait]
pub trait Connector: Send + Sync + std::fmt::Debug {
    /// The registry entry.
    fn kind(&self) -> Kind;

    /// Begin a link: the URL to send the person to. Token kinds return an
    /// error; they complete directly.
    async fn start(&self, ctx: &AuthContext) -> Result<String, ConnectorError>;

    /// Finish a link with what the callback carried (the code), or with a
    /// pasted token for token kinds.
    async fn complete(&self, ctx: &AuthContext, code: &str) -> Result<Linked, ConnectorError>;

    /// Prove the credential still works, cheaply. Returns a one-line status.
    async fn test(&self, credentials: &serde_json::Value) -> Result<String, ConnectorError>;

    /// Everything new since `since`, as documents. `seen` says whether a
    /// provider id was pulled before, so the provider is not asked for bytes
    /// it already gave. A kind that stops short of everything new says so
    /// with `complete = false`; the caller pulls again with the new ids seen.
    async fn pull(
        &self,
        credentials: &serde_json::Value,
        config: &serde_json::Value,
        since: DateTime<Utc>,
        seen: &(dyn for<'a> Fn(&'a str) -> bool + Sync),
    ) -> Result<Pull, ConnectorError>;
}

/// One round of a pull.
#[derive(Debug, Default)]
pub struct Pull {
    /// What was fetched this round.
    pub found: Vec<Found>,
    /// Whether that was everything new. A kind with a per-round cap
    /// returns `false` when it hit it.
    pub complete: bool,
}

/// A source of kinds: the registry, or what a test injects.
pub type KindsFactory = std::sync::Arc<dyn Fn() -> Vec<Box<dyn Connector>> + Send + Sync>;

/// Every kind the service knows, in the order the UI offers them.
#[must_use]
pub fn registry(config: &crate::config::Connectors) -> Vec<Box<dyn Connector>> {
    vec![Box::new(gmail::Gmail::new(config))]
}

/// The kind by name.
#[must_use]
pub fn by_name(config: &crate::config::Connectors, name: &str) -> Option<Box<dyn Connector>> {
    registry(config).into_iter().find(|k| k.kind().name == name)
}
