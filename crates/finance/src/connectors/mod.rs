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
pub mod eracuni;
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

/// What a link is for. Decides the consent asked of the provider, so a
/// mailbox linked for sending only never holds a credential that could read
/// it: the guarantee is in the consent, not only in our code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Purpose {
    /// Pull receipts; send nothing.
    Read,
    /// Send mail as the account; read, list and pull nothing.
    Send,
    /// Both.
    Both,
}

impl Purpose {
    /// Every purpose, in the order a page offers them.
    pub const ALL: [Self; 3] = [Self::Read, Self::Send, Self::Both];

    /// The word on the wire.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Send => "send",
            Self::Both => "both",
        }
    }

    /// Whether the purpose wants the read consent, and the send consent.
    #[must_use]
    pub const fn wants(self) -> (bool, bool) {
        match self {
            Self::Read => (true, false),
            Self::Send => (false, true),
            Self::Both => (true, true),
        }
    }
}

impl std::str::FromStr for Purpose {
    type Err = String;

    /// Empty means both, so a caller from before purposes existed is unchanged.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "read" => Ok(Self::Read),
            "send" => Ok(Self::Send),
            "both" | "" => Ok(Self::Both),
            other => Err(format!("unknown purpose {other}; read, send or both")),
        }
    }
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
    /// What this kind can be linked for.
    pub purposes: &'static [Purpose],
}

/// What a link attempt needs from the environment.
#[derive(Debug, Clone)]
pub struct AuthContext {
    /// Ours, single-use, already stored on the pending row.
    pub state: String,
    /// Where the provider sends the browser back.
    pub redirect_url: String,
    /// What the link is for; the kind asks for the matching consent.
    pub purpose: Purpose,
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
    /// What the provider itself says the document is, when it knows: an
    /// intermediary that carried the invoice knows its supplier, number,
    /// date and total. Written as facts the reader does not second-guess.
    pub facts: Option<Facts>,
}

/// The fields a provider states about a document. Each is optional; what is
/// missing the reader fills from the text as usual.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Facts {
    /// The supplier's name.
    pub vendor: Option<String>,
    /// The document's own date.
    pub doc_date: Option<chrono::NaiveDate>,
    /// The total, in minor units, with its ISO currency.
    pub total: Option<(i64, String)>,
    /// The supplier's number for it.
    pub invoice_no: Option<String>,
}

/// What a linked credential may do, decided from the consent granted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Capabilities {
    /// The account may be read and pulled.
    pub read: bool,
    /// Mail may be sent as this account.
    pub send: bool,
}

impl Default for Capabilities {
    /// A kind that says nothing pulls and does not send.
    fn default() -> Self {
        Self {
            read: true,
            send: false,
        }
    }
}

/// A file that goes with a mail, or came with one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attachment {
    /// As shown to the recipient.
    pub filename: String,
    /// `application/pdf` and the like.
    pub content_type: String,
    /// The bytes.
    pub bytes: Vec<u8>,
}

/// A mail to send as the linked account.
#[derive(Debug, Clone, Default)]
pub struct Outgoing {
    /// Recipients.
    pub to: Vec<String>,
    /// Copied.
    pub cc: Vec<String>,
    /// Copied, unseen by the others.
    pub bcc: Vec<String>,
    /// The subject line.
    pub subject: String,
    /// The text body.
    pub text: String,
    /// An HTML body beside the text, when the composer made one.
    pub html: Option<String>,
    /// Files to attach.
    pub attachments: Vec<Attachment>,
    /// When answering: the provider's thread and the RFC 5322 id replied to.
    pub in_reply_to: Option<(String, String)>,
}

/// What the provider said about a sent mail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SentMail {
    /// The provider's id for the message.
    pub provider_id: String,
    /// The provider's thread, where replies arrive.
    pub thread_key: String,
    /// The RFC 5322 Message-ID it went out with.
    pub message_id: String,
}

/// A mail that arrived in a thread we started.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inbound {
    /// The provider's id for the message.
    pub provider_id: String,
    /// Its thread.
    pub thread_key: String,
    /// RFC 5322 ids.
    pub message_id: String,
    /// What it answers, when the sender's client said.
    pub in_reply_to: String,
    /// `Name <addr>`.
    pub from: String,
    /// Recipients as written.
    pub to: Vec<String>,
    /// Copied.
    pub cc: Vec<String>,
    /// The subject line.
    pub subject: String,
    /// The text body (HTML reduced when there was no text part).
    pub text: String,
    /// When it arrived.
    pub received_at: Option<DateTime<Utc>>,
    /// Files that came with it.
    pub attachments: Vec<Attachment>,
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

    /// Everything new since `since`, as documents, sent into `sink` one at a
    /// time as they are fetched so the caller can store and count them while
    /// the pull is still going. `seen` says whether a provider id was pulled
    /// before, so the provider is not asked for bytes it already gave. A kind
    /// with a per-round cap that hit it returns [`Reach::Truncated`]; the
    /// caller pulls again with the new ids seen.
    async fn pull(
        &self,
        credentials: &serde_json::Value,
        config: &serde_json::Value,
        since: DateTime<Utc>,
        seen: &(dyn for<'a> Fn(&'a str) -> bool + Sync),
        sink: tokio::sync::mpsc::Sender<Found>,
    ) -> Result<Reach, ConnectorError>;

    /// What this credential may do beyond pulling. Decided once, at link
    /// time, from the consent the person gave; a kind that only pulls says
    /// nothing.
    fn capabilities(&self, _credentials: &serde_json::Value) -> Capabilities {
        Capabilities::default()
    }

    /// Send a mail as the linked account. A kind that cannot refuses.
    async fn send(
        &self,
        _credentials: &serde_json::Value,
        _mail: &Outgoing,
    ) -> Result<SentMail, ConnectorError> {
        Err(ConnectorError::Invalid(
            "this kind of connector cannot send mail".into(),
        ))
    }

    /// Everything that arrived in `thread_key` that is not ours and not yet
    /// `seen` (by provider id). A kind that cannot read a thread says none.
    async fn replies(
        &self,
        _credentials: &serde_json::Value,
        _thread_key: &str,
        _seen: &(dyn for<'a> Fn(&'a str) -> bool + Sync),
    ) -> Result<Vec<Inbound>, ConnectorError> {
        Ok(Vec::new())
    }
}

/// How far one round of a pull got.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reach {
    /// Everything new was sent.
    Complete,
    /// The round's cap was hit; more remains.
    Truncated,
}

/// A source of kinds: the registry, or what a test injects.
pub type KindsFactory = std::sync::Arc<dyn Fn() -> Vec<Box<dyn Connector>> + Send + Sync>;

/// Every kind the service knows, in the order the UI offers them.
#[must_use]
pub fn registry(config: &crate::config::Connectors) -> Vec<Box<dyn Connector>> {
    vec![
        Box::new(gmail::Gmail::new(config)),
        Box::new(eracuni::Eracuni::new()),
    ]
}

/// The kind by name.
#[must_use]
pub fn by_name(config: &crate::config::Connectors, name: &str) -> Option<Box<dyn Connector>> {
    registry(config).into_iter().find(|k| k.kind().name == name)
}
