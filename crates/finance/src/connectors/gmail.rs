//! Gmail: receipts that arrive by mail.
//!
//! Google OAuth with the read-only Gmail scope and `email`, so the connector
//! can name the mailbox it is. A pull runs the configured query (default:
//! anything with a PDF attached, since the last pull), fetches each message,
//! and keeps the PDF attachments. A second query finds receipt mails with
//! no attachment: a Stripe-hosted invoice named in the body is fetched as
//! its PDF, and any other such mail is printed to one (`documents::mail`),
//! because the accountant files a page, not a message.

use async_trait::async_trait;
use base64::{Engine, engine::general_purpose::URL_SAFE};
use chrono::{DateTime, Utc};
use serde_json::{Value, json};

use super::{
    Attachment, Auth, AuthContext, Capabilities, Connector, ConnectorError, Found, Inbound, Kind,
    Linked, Outgoing, Purpose, Reach, SentMail,
};
use crate::{config::Connectors as Config, documents::mail};

const AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const USERINFO_URL: &str = "https://www.googleapis.com/oauth2/v3/userinfo";
const API: &str = "https://gmail.googleapis.com/gmail/v1/users/me";
/// Read to pull receipts, send to write mail, `email` to name the mailbox.
/// The purpose of the link decides which of the two are asked for
/// (`scopes_for`), so a mailbox linked for sending only never holds a
/// credential that could read it. The credential records the scopes
/// granted -- a person may untick one on Google's screen -- and
/// `capabilities` reads them.
const READ_SCOPE: &str = "https://www.googleapis.com/auth/gmail.readonly";
const SEND_SCOPE: &str = "https://www.googleapis.com/auth/gmail.send";
const EMAIL_SCOPE: &str = "email";

/// The scopes a purpose asks of Google, space-separated as the URL wants.
fn scopes_for(purpose: Purpose) -> String {
    let (read, send) = purpose.wants();
    let mut scopes = Vec::with_capacity(3);
    if read {
        scopes.push(READ_SCOPE);
    }
    if send {
        scopes.push(SEND_SCOPE);
    }
    scopes.push(EMAIL_SCOPE);
    scopes.join(" ")
}
/// Never more than this per pull; a first pull of a busy mailbox is paged
/// over several runs rather than held open for minutes.
/// Messages fetched per round. A round runs detached from the RPC, so the
/// cap bounds one run's Gmail quota, not a request timeout; a mailbox with
/// more says `complete = false` and the caller comes round again.
const MAX_MESSAGES: usize = 500;

/// One Google OAuth client: a project's consent screen and its status.
#[derive(Debug, Clone)]
struct OauthClient {
    id: String,
    secret: String,
}

impl OauthClient {
    fn configured(&self) -> bool {
        !self.id.is_empty() && !self.secret.is_empty()
    }
}

/// The name a credential records for the client that minted it, so a
/// refresh goes back to the same one: Google refuses a refresh token at any
/// other client.
const CLIENT_READ: &str = "read";
const CLIENT_SEND: &str = "send";

/// The kind.
#[derive(Debug, Clone)]
pub struct Gmail {
    /// The client links for reading (or both) go through: a Testing project
    /// with the restricted read scope, or an Internal one.
    read: OauthClient,
    /// The client send-only links go through when configured: a published
    /// project asking for `gmail.send` alone, whose refresh tokens last.
    send: Option<OauthClient>,
    http: reqwest::Client,
    token_url: String,
    api: String,
    /// Google's userinfo endpoint (the `email` scope): names the account on
    /// link, and is the whole test of a send-only credential.
    userinfo_url: String,
}

/// Google mints an access token for an hour. A throttled pull of a busy
/// mailbox runs longer than that -- one did, and died at 3,611 seconds with
/// "invalid authentication credentials", which read as a broken link -- so a
/// pull holds a session: the token is re-minted before its hour is up, and
/// once more if Google refuses it anyway. Only the token endpoint refusing
/// the *refresh* means the link is gone.
#[derive(Debug)]
struct Session {
    credentials: Value,
    minted: tokio::sync::Mutex<Minted>,
}

#[derive(Debug, Clone)]
struct Minted {
    token: String,
    at: std::time::Instant,
}

/// Re-mint at fifty minutes: ten to spare for a request already waiting
/// out a quota minute.
const RENEW_AFTER: std::time::Duration = std::time::Duration::from_mins(50);

impl Session {
    /// A session on a token already in hand (the code exchange's), with no
    /// refresh token behind it: a refusal is final.
    fn with_token(token: String) -> Self {
        Self {
            credentials: Value::Null,
            minted: tokio::sync::Mutex::new(Minted {
                token,
                at: std::time::Instant::now(),
            }),
        }
    }
}

impl Gmail {
    /// From configuration; unconfigured is allowed, and says so on link.
    #[must_use]
    pub fn new(config: &Config) -> Self {
        let send = OauthClient {
            id: config.google_send_client_id.clone(),
            secret: config.google_send_client_secret.clone(),
        };
        Self {
            read: OauthClient {
                id: config.google_client_id.clone(),
                secret: config.google_client_secret.clone(),
            },
            send: send.configured().then_some(send),
            http: reqwest::Client::new(),
            token_url: TOKEN_URL.to_owned(),
            api: API.to_owned(),
            userinfo_url: USERINFO_URL.to_owned(),
        }
    }

    /// The client a link of this purpose goes through, and its name for the
    /// credential. Sending only prefers the send client; without one, the
    /// read client serves, as it did before there were two.
    fn client_for(&self, purpose: Purpose) -> (&OauthClient, &'static str) {
        match (purpose, &self.send) {
            (Purpose::Send, Some(send)) => (send, CLIENT_SEND),
            _ => (&self.read, CLIENT_READ),
        }
    }

    /// The client a credential was minted at. One from before there were two
    /// names none and is the read client's.
    fn client_of(&self, credentials: &Value) -> &OauthClient {
        match (
            credentials.get("client").and_then(Value::as_str),
            &self.send,
        ) {
            (Some(CLIENT_SEND), Some(send)) => send,
            _ => &self.read,
        }
    }

    /// Google's endpoints replaced by a test's server.
    #[cfg(test)]
    fn with_endpoints(token_url: &str, api: &str) -> Self {
        Self {
            read: OauthClient {
                id: "client".into(),
                secret: "secret".into(),
            },
            send: None,
            http: reqwest::Client::new(),
            token_url: token_url.to_owned(),
            api: api.to_owned(),
            userinfo_url: format!("{api}/userinfo"),
        }
    }

    /// With a send client too.
    #[cfg(test)]
    fn with_send_client(mut self, id: &str, secret: &str) -> Self {
        self.send = Some(OauthClient {
            id: id.into(),
            secret: secret.into(),
        });
        self
    }

    /// Start a session: mint the first token.
    async fn open(&self, credentials: &Value) -> Result<Session, ConnectorError> {
        let token = self.access_token(credentials).await?;
        Ok(Session {
            credentials: credentials.clone(),
            minted: tokio::sync::Mutex::new(Minted {
                token,
                at: std::time::Instant::now(),
            }),
        })
    }

    /// The session's token, re-minted first when it is near its hour.
    async fn bearer(&self, session: &Session) -> Result<String, ConnectorError> {
        let mut minted = session.minted.lock().await;
        if minted.at.elapsed() >= RENEW_AFTER && session.credentials.get("refresh_token").is_some()
        {
            tracing::info!(
                age_secs = minted.at.elapsed().as_secs(),
                "gmail: access token near its hour; re-minting"
            );
            *minted = Minted {
                token: self.access_token(&session.credentials).await?,
                at: std::time::Instant::now(),
            };
        }
        Ok(minted.token.clone())
    }

    /// A fresh token after Google refused the one in hand. With no refresh
    /// token behind the session this fails as unlinked, which is the truth.
    async fn renew(&self, session: &Session) -> Result<(), ConnectorError> {
        let mut minted = session.minted.lock().await;
        *minted = Minted {
            token: self.access_token(&session.credentials).await?,
            at: std::time::Instant::now(),
        };
        Ok(())
    }

    fn configured(&self) -> bool {
        self.read.configured()
    }

    async fn access_token(&self, credentials: &Value) -> Result<String, ConnectorError> {
        let refresh = credentials
            .get("refresh_token")
            .and_then(Value::as_str)
            .ok_or_else(|| ConnectorError::Unlinked("no refresh token".into()))?;
        let client = self.client_of(credentials);
        let resp = self
            .http
            .post(&self.token_url)
            .form(&[
                ("client_id", client.id.as_str()),
                ("client_secret", client.secret.as_str()),
                ("refresh_token", refresh),
                ("grant_type", "refresh_token"),
            ])
            .send()
            .await
            .map_err(|e| ConnectorError::Provider(e.to_string()))?;
        let status = resp.status();
        let body: Value = resp.json().await.unwrap_or(Value::Null);
        if status == reqwest::StatusCode::BAD_REQUEST || status == reqwest::StatusCode::UNAUTHORIZED
        {
            return Err(ConnectorError::Unlinked(
                body.get("error")
                    .and_then(Value::as_str)
                    .unwrap_or("refresh refused")
                    .to_owned(),
            ));
        }
        body.get("access_token")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .ok_or_else(|| ConnectorError::Provider(format!("token endpoint answered {status}")))
    }

    /// Every message id the query matches, newest first, across all of
    /// Gmail's pages.
    async fn list_ids(
        &self,
        session: &Session,
        q: &str,
        throttle: &Pace,
    ) -> Result<Vec<String>, ConnectorError> {
        let mut ids = Vec::new();
        let mut page: Option<String> = None;
        loop {
            let mut url = reqwest::Url::parse(&format!("{}/messages", self.api))
                .map_err(|e| ConnectorError::Provider(e.to_string()))?;
            url.query_pairs_mut()
                .append_pair("q", q)
                .append_pair("maxResults", "100");
            if let Some(p) = &page {
                url.query_pairs_mut().append_pair("pageToken", p);
            }
            let list = self.get(session, url.as_str(), throttle).await?;
            for m in list
                .get("messages")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                if let Some(id) = m.get("id").and_then(Value::as_str) {
                    ids.push(id.to_owned());
                }
            }
            page = list
                .get("nextPageToken")
                .and_then(Value::as_str)
                .map(str::to_owned);
            if page.is_none() {
                break;
            }
        }
        Ok(ids)
    }

    /// One authenticated GET, with Google's transient refusals retried.
    /// Gmail says "slow down" as a 403 with a reason, not a 429, so the
    /// reason decides; a 403 for a missing scope is a link problem instead,
    /// and is not retried. `pace` is the gap kept before each request; a
    /// quota refusal widens it for the rest of the pull. A 401 is tried once
    /// more on a freshly minted token before it means the link is gone.
    async fn get(
        &self,
        session: &Session,
        url: &str,
        pace: &Pace,
    ) -> Result<Value, ConnectorError> {
        let mut renewed = false;
        for (attempt, wait) in RETRY_WAITS.iter().enumerate() {
            pace.hold().await;
            let token = self.bearer(session).await?;
            let resp = self
                .http
                .get(url)
                .bearer_auth(&token)
                .send()
                .await
                .map_err(|e| ConnectorError::Provider(e.to_string()))?;
            let status = resp.status();
            if status.is_success() {
                return resp
                    .json()
                    .await
                    .map_err(|e| ConnectorError::Provider(format!("gmail: {e}")));
            }
            let body = resp.text().await.unwrap_or_default();
            match classify(status, &body) {
                Refusal::Unlinked(why)
                    if status == reqwest::StatusCode::UNAUTHORIZED && !renewed =>
                {
                    // Google refused the token, which is not the same as
                    // refusing the link: an hour-old token in a long pull
                    // gets exactly this answer. One re-mint tells them apart.
                    tracing::info!(why, "gmail: token refused; re-minting once");
                    renewed = true;
                    self.renew(session).await?;
                }
                Refusal::Transient(why) if attempt + 1 < RETRY_WAITS.len() => {
                    // A per-minute quota clears when the minute does; the
                    // later waits are a full one. And the pull slows down,
                    // so the next minute is not spent the same way.
                    if why.contains("ateLimitExceeded") {
                        pace.slower();
                    }
                    tracing::info!(%status, why, wait_secs = wait, gap_ms = pace.gap_ms(), "gmail: transient, retrying");
                    tokio::time::sleep(std::time::Duration::from_secs(*wait)).await;
                }
                Refusal::Transient(why) => {
                    return Err(ConnectorError::Provider(format!(
                        "gmail {status} after {} attempts: {why}",
                        RETRY_WAITS.len()
                    )));
                }
                Refusal::Unlinked(why) => return Err(ConnectorError::Unlinked(why)),
                Refusal::Fatal(why) => return Err(ConnectorError::Provider(why)),
            }
        }
        Err(ConnectorError::Provider("gmail: retries exhausted".into()))
    }
}

/// Seconds waited before each retry. Three minutes and change in all, which
/// spans two rollovers of Google's per-minute quota.
const RETRY_WAITS: [u64; 7] = [0, 2, 5, 15, 30, 60, 60];

/// The gap kept before each request of one pull. Starts at none, since the
/// quota is unknown and most mailboxes never hit it; doubles on every quota
/// refusal, from one second, up to fifteen. Never narrows within a pull: a
/// project whose quota was hit once will be hit again at the old rate.
#[derive(Debug, Default)]
struct Pace {
    gap_ms: std::sync::atomic::AtomicU64,
}

impl Pace {
    const FIRST_MS: u64 = 1000;
    const MAX_MS: u64 = 15_000;

    fn gap_ms(&self) -> u64 {
        self.gap_ms.load(std::sync::atomic::Ordering::Relaxed)
    }

    async fn hold(&self) {
        let gap = self.gap_ms();
        if gap > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(gap)).await;
        }
    }

    fn slower(&self) {
        let gap = self.gap_ms();
        let next = (gap * 2).clamp(Self::FIRST_MS, Self::MAX_MS);
        self.gap_ms
            .store(next, std::sync::atomic::Ordering::Relaxed);
    }
}

/// What a non-2xx answer from Google means for us.
#[derive(Debug, PartialEq, Eq)]
enum Refusal {
    /// Rate limit, quota burst, or a Google-side error: wait and try again.
    Transient(String),
    /// The credential is no good: the token was refused, or the consent
    /// never covered the mailbox. Relinking is the fix.
    Unlinked(String),
    /// Anything else, with Google's reason.
    Fatal(String),
}

/// Read Google's error envelope: `{"error": {"code", "message", "status",
/// "errors": [{"reason", "message"}]}}`, in either of its two shapes.
fn classify(status: reqwest::StatusCode, body: &str) -> Refusal {
    let json: Value = serde_json::from_str(body).unwrap_or(Value::Null);
    let error = json.get("error");
    let reason = error
        .and_then(|e| e.pointer("/errors/0/reason"))
        .and_then(Value::as_str)
        .or_else(|| {
            error
                .and_then(|e| e.pointer("/details/0/reason"))
                .and_then(Value::as_str)
        })
        .unwrap_or("");
    let message = error
        .and_then(|e| e.get("message"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    let google_status = error
        .and_then(|e| e.get("status"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let why = if message.is_empty() {
        format!("gmail {status}")
    } else if reason.is_empty() {
        format!("gmail {status}: {message}")
    } else {
        format!("gmail {status}: {reason}: {message}")
    };
    if status == reqwest::StatusCode::UNAUTHORIZED {
        return Refusal::Unlinked(format!("token refused: {message}"));
    }
    let lower = message.to_ascii_lowercase();
    if matches!(
        reason,
        "insufficientPermissions" | "ACCESS_TOKEN_SCOPE_INSUFFICIENT" | "forbidden"
    ) || lower.contains("insufficient authentication scopes")
        || lower.contains("insufficient permission")
    {
        return Refusal::Unlinked(
            "the consent did not include reading the mailbox; link again and allow \"View your email messages\""
                .into(),
        );
    }
    if status == reqwest::StatusCode::TOO_MANY_REQUESTS
        || status.is_server_error()
        || matches!(
            reason,
            "rateLimitExceeded" | "userRateLimitExceeded" | "quotaExceeded" | "backendError"
        )
        || google_status == "RESOURCE_EXHAUSTED"
    {
        return Refusal::Transient(why);
    }
    Refusal::Fatal(why)
}

fn header<'a>(message: &'a Value, name: &str) -> &'a str {
    message
        .pointer("/payload/headers")
        .and_then(Value::as_array)
        .and_then(|hs| {
            hs.iter().find(|h| {
                h.get("name")
                    .and_then(Value::as_str)
                    .is_some_and(|n| n.eq_ignore_ascii_case(name))
            })
        })
        .and_then(|h| h.get("value"))
        .and_then(Value::as_str)
        .unwrap_or("")
}

/// Every part of a message, flattened.
fn parts(payload: &Value) -> Vec<&Value> {
    let mut out = vec![payload];
    if let Some(children) = payload.get("parts").and_then(Value::as_array) {
        for c in children {
            out.extend(parts(c));
        }
    }
    out
}

#[async_trait]
impl Connector for Gmail {
    fn kind(&self) -> Kind {
        Kind {
            name: "gmail",
            label: "Gmail / Google Workspace",
            description: "Receipts and invoices: PDF attachments, Stripe-hosted invoices, and receipt mails printed to PDF.",
            auth: Auth::Oauth,
            consent_note: "The consent follows the purpose. Reading is read-only: nothing is moved \
                           or deleted, and only messages matching the queries are read (PDF \
                           attachments are kept, a receipt mail without one is printed to a page). \
                           Sending is the send permission alone: a mailbox linked for sending only \
                           is never read, listed or pulled.",
            configured: self.configured(),
            purposes: &Purpose::ALL,
        }
    }

    fn capabilities(&self, credentials: &Value) -> Capabilities {
        let scope = credentials
            .get("scope")
            .and_then(Value::as_str)
            .unwrap_or("");
        Capabilities {
            // A credential from before scopes were recorded was linked with
            // the read scope, the only one there was.
            read: scope.is_empty() || scope.split_whitespace().any(|s| s == READ_SCOPE),
            send: scope.split_whitespace().any(|s| s == SEND_SCOPE),
        }
    }

    async fn send(&self, credentials: &Value, mail: &Outgoing) -> Result<SentMail, ConnectorError> {
        if !self.capabilities(credentials).send {
            return Err(ConnectorError::Unlinked(
                "the consent did not include sending; link the mailbox again and allow \"Send email on your behalf\"".into(),
            ));
        }
        let session = self.open(credentials).await?;
        let profile = self
            .get(&session, &format!("{}/profile", self.api), &Pace::default())
            .await?;
        let from = profile
            .get("emailAddress")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned();
        if from.is_empty() {
            return Err(ConnectorError::Provider(
                "gmail did not name the mailbox".into(),
            ));
        }
        let message_id = format!(
            "<{}@{}>",
            uuid::Uuid::new_v4().simple(),
            from.rsplit('@').next().unwrap_or("mail")
        );
        let raw = mime(&from, mail, &message_id);
        let mut body = json!({ "raw": URL_SAFE.encode(raw) });
        if let Some((thread, _)) = &mail.in_reply_to {
            body["threadId"] = Value::String(thread.clone());
        }
        let sent = self
            .post_json(&session, &format!("{}/messages/send", self.api), &body)
            .await?;
        Ok(SentMail {
            provider_id: sent
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned(),
            thread_key: sent
                .get("threadId")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned(),
            message_id,
        })
    }

    async fn replies(
        &self,
        credentials: &Value,
        thread_key: &str,
        seen: &(dyn for<'a> Fn(&'a str) -> bool + Sync),
    ) -> Result<Vec<Inbound>, ConnectorError> {
        let session = self.open(credentials).await?;
        let throttle = Pace::default();
        let profile = self
            .get(&session, &format!("{}/profile", self.api), &throttle)
            .await?;
        let own = profile
            .get("emailAddress")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_ascii_lowercase();
        let thread = self
            .get(
                &session,
                &format!("{}/threads/{thread_key}?format=full", self.api),
                &throttle,
            )
            .await?;
        let mut out = Vec::new();
        for message in thread
            .get("messages")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let id = message.get("id").and_then(Value::as_str).unwrap_or("");
            if id.is_empty() || seen(id) {
                continue;
            }
            let from = header(message, "From").to_owned();
            // Our own mail in the thread is not a reply.
            if !own.is_empty() && from.to_ascii_lowercase().contains(&own) {
                continue;
            }
            let Some(payload) = message.get("payload") else {
                continue;
            };
            let (text, html) = bodies(payload);
            let text = if text.trim().is_empty() {
                mail::html_to_text(&html)
            } else {
                mail::tidy(&text)
            };
            let attachments = self
                .attachments_of(&session, &throttle, id, payload)
                .await?;
            let list = |name: &str| -> Vec<String> {
                header(message, name)
                    .split(',')
                    .map(str::trim)
                    .filter(|a| !a.is_empty())
                    .map(str::to_owned)
                    .collect()
            };
            out.push(Inbound {
                provider_id: id.to_owned(),
                thread_key: thread_key.to_owned(),
                message_id: header(message, "Message-ID").to_owned(),
                in_reply_to: header(message, "In-Reply-To").to_owned(),
                from,
                to: list("To"),
                cc: list("Cc"),
                subject: header(message, "Subject").to_owned(),
                text,
                received_at: message
                    .get("internalDate")
                    .and_then(Value::as_str)
                    .and_then(|ms| ms.parse::<i64>().ok())
                    .and_then(DateTime::<Utc>::from_timestamp_millis),
                attachments,
            });
        }
        Ok(out)
    }

    async fn start(&self, ctx: &AuthContext) -> Result<String, ConnectorError> {
        if !self.configured() {
            return Err(ConnectorError::Unconfigured(
                "set FINANCE_GOOGLE_CLIENT_ID and FINANCE_GOOGLE_CLIENT_SECRET".into(),
            ));
        }
        let (client, _) = self.client_for(ctx.purpose);
        let mut url =
            reqwest::Url::parse(AUTH_URL).map_err(|e| ConnectorError::Provider(e.to_string()))?;
        url.query_pairs_mut()
            .append_pair("client_id", &client.id)
            .append_pair("redirect_uri", &ctx.redirect_url)
            .append_pair("response_type", "code")
            .append_pair("scope", &scopes_for(ctx.purpose))
            // `offline` + `consent` is what yields a refresh token every time,
            // not only on the first consent.
            .append_pair("access_type", "offline")
            .append_pair("prompt", "consent")
            .append_pair("include_granted_scopes", "true")
            .append_pair("state", &ctx.state);
        Ok(url.to_string())
    }

    async fn complete(&self, ctx: &AuthContext, code: &str) -> Result<Linked, ConnectorError> {
        if !self.configured() {
            return Err(ConnectorError::Unconfigured("no OAuth client".into()));
        }
        let (client, client_name) = self.client_for(ctx.purpose);
        let resp = self
            .http
            .post(&self.token_url)
            .form(&[
                ("client_id", client.id.as_str()),
                ("client_secret", client.secret.as_str()),
                ("code", code),
                ("grant_type", "authorization_code"),
                ("redirect_uri", ctx.redirect_url.as_str()),
            ])
            .send()
            .await
            .map_err(|e| ConnectorError::Provider(e.to_string()))?;
        let status = resp.status();
        let body: Value = resp.json().await.unwrap_or(Value::Null);
        if !status.is_success() {
            return Err(ConnectorError::Provider(format!(
                "google refused the code: {}",
                body.get("error_description")
                    .or_else(|| body.get("error"))
                    .and_then(Value::as_str)
                    .unwrap_or("")
            )));
        }
        let refresh = body
            .get("refresh_token")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                ConnectorError::Provider(
                    "google issued no refresh token; revoke the app's access and link again".into(),
                )
            })?;
        let access = body
            .get("access_token")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let who = self
            .get(
                &Session::with_token(access.to_owned()),
                &self.userinfo_url,
                &Pace::default(),
            )
            .await?;
        let email = who
            .get("email")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned();
        if email.is_empty() {
            return Err(ConnectorError::Provider(
                "google did not say which account this is".into(),
            ));
        }
        let scope = body
            .get("scope")
            .and_then(Value::as_str)
            .unwrap_or_default();
        Ok(Linked {
            credentials: json!({ "refresh_token": refresh, "scope": scope, "client": client_name }),
            external_id: email.clone(),
            label: email,
        })
    }

    async fn test(&self, credentials: &Value) -> Result<String, ConnectorError> {
        let session = self.open(credentials).await?;
        if !self.capabilities(credentials).read {
            // The send scope alone may not read the profile (Google's
            // `getProfile` wants a read scope); `email` names the account,
            // and that is all a send-only link is for.
            let who = self
                .get(&session, &self.userinfo_url, &Pace::default())
                .await?;
            return Ok(format!(
                "{} · send only",
                who.get("email").and_then(Value::as_str).unwrap_or("?")
            ));
        }
        let profile = self
            .get(&session, &format!("{}/profile", self.api), &Pace::default())
            .await?;
        Ok(format!(
            "{} · {} messages",
            profile
                .get("emailAddress")
                .and_then(Value::as_str)
                .unwrap_or("?"),
            profile
                .get("messagesTotal")
                .and_then(Value::as_u64)
                .unwrap_or(0)
        ))
    }

    async fn pull(
        &self,
        credentials: &Value,
        config: &Value,
        since: DateTime<Utc>,
        seen: &(dyn for<'a> Fn(&'a str) -> bool + Sync),
        sink: tokio::sync::mpsc::Sender<Found>,
    ) -> Result<Reach, ConnectorError> {
        let session = self.open(credentials).await?;
        let throttle = Pace::default();
        let query = config
            .get("query")
            .and_then(Value::as_str)
            .filter(|q| !q.trim().is_empty())
            .map_or_else(|| "has:attachment filename:pdf".to_owned(), str::to_owned);
        let q = format!("{query} after:{}", since.format("%Y/%m/%d"));

        let ids = self.list_ids(&session, &q, &throttle).await?;

        // Listing is cheap and complete; fetching is what is capped. Only
        // messages never pulled count against the cap, so a mailbox drains
        // over rounds rather than re-fetching its newest 500 forever.
        let unseen: Vec<&String> = ids.iter().filter(|id| !seen(id)).collect();
        let reach = if unseen.len() <= MAX_MESSAGES {
            Reach::Complete
        } else {
            Reach::Truncated
        };
        for id in unseen.into_iter().take(MAX_MESSAGES) {
            let message = self
                .get(
                    &session,
                    &format!("{}/messages/{id}?format=full", self.api),
                    &throttle,
                )
                .await?;
            let subject = header(&message, "Subject").to_owned();
            let sender = header(&message, "From").to_owned();
            let received_at = message
                .get("internalDate")
                .and_then(Value::as_str)
                .and_then(|ms| ms.parse::<i64>().ok())
                .and_then(DateTime::<Utc>::from_timestamp_millis);
            let Some(payload) = message.get("payload") else {
                continue;
            };
            for part in parts(payload) {
                let filename = part.get("filename").and_then(Value::as_str).unwrap_or("");
                let mime = part.get("mimeType").and_then(Value::as_str).unwrap_or("");
                let is_pdf = mime.eq_ignore_ascii_case("application/pdf")
                    || filename.to_ascii_lowercase().ends_with(".pdf");
                if !is_pdf || filename.is_empty() {
                    continue;
                }
                let Some(att) = part.pointer("/body/attachmentId").and_then(Value::as_str) else {
                    continue;
                };
                let blob = self
                    .get(
                        &session,
                        &format!("{}/messages/{id}/attachments/{att}", self.api),
                        &throttle,
                    )
                    .await?;
                let Some(data) = blob.get("data").and_then(Value::as_str) else {
                    continue;
                };
                let bytes = URL_SAFE
                    .decode(data)
                    .or_else(|_| base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(data))
                    .map_err(|e| ConnectorError::Provider(format!("attachment: {e}")))?;
                let found = Found {
                    external_ref: format!("{id}:{filename}"),
                    filename: filename.to_owned(),
                    content_type: "application/pdf".into(),
                    bytes,
                    subject: subject.clone(),
                    sender: sender.clone(),
                    received_at,
                    facts: None,
                };
                // A closed sink is the caller done listening (its store
                // failed, or the run was abandoned): nothing more to fetch.
                if sink.send(found).await.is_err() {
                    return Ok(Reach::Truncated);
                }
            }
        }

        let bodies = self
            .pull_bodies(&session, config, since, seen, &sink, &throttle)
            .await?;
        Ok(if reach == Reach::Complete && bodies == Reach::Complete {
            Reach::Complete
        } else {
            Reach::Truncated
        })
    }
}

impl Gmail {
    /// Receipt mails with nothing attached: the message is the receipt. A
    /// Stripe-hosted invoice named in it is fetched as its PDF; any other
    /// is printed to one.
    async fn pull_bodies(
        &self,
        session: &Session,
        config: &Value,
        since: DateTime<Utc>,
        seen: &(dyn for<'a> Fn(&'a str) -> bool + Sync),
        sink: &tokio::sync::mpsc::Sender<Found>,
        throttle: &Pace,
    ) -> Result<Reach, ConnectorError> {
        let mailbox = self
            .get(session, &format!("{}/profile", self.api), throttle)
            .await
            .ok()
            .and_then(|p| {
                p.get("emailAddress")
                    .and_then(Value::as_str)
                    .map(str::to_owned)
            })
            .unwrap_or_default();
        let body_query = config
            .get("body_query")
            .and_then(Value::as_str)
            .filter(|q| !q.trim().is_empty())
            .map_or_else(|| BODY_QUERY.to_owned(), str::to_owned);
        let q = format!("{body_query} after:{}", since.format("%Y/%m/%d"));
        let ids = self.list_ids(session, &q, throttle).await?;
        let unseen: Vec<&String> = ids.iter().filter(|id| !seen(id)).collect();
        let reach = if unseen.len() <= MAX_MESSAGES {
            Reach::Complete
        } else {
            Reach::Truncated
        };
        for id in unseen.into_iter().take(MAX_MESSAGES) {
            let message = self
                .get(
                    session,
                    &format!("{}/messages/{id}?format=full", self.api),
                    throttle,
                )
                .await?;
            let subject = header(&message, "Subject").to_owned();
            let sender = header(&message, "From").to_owned();
            let received_at = message
                .get("internalDate")
                .and_then(Value::as_str)
                .and_then(|ms| ms.parse::<i64>().ok())
                .and_then(DateTime::<Utc>::from_timestamp_millis);
            let Some(payload) = message.get("payload") else {
                continue;
            };
            if !looks_like_a_receipt(&subject, &sender) {
                continue;
            }
            let (text, html) = bodies(payload);
            if text.trim().is_empty() && html.trim().is_empty() {
                continue;
            }
            let found = self
                .body_document(id, &subject, &sender, received_at, &mailbox, text, html)
                .await?;
            if sink.send(found).await.is_err() {
                return Ok(Reach::Truncated);
            }
        }
        Ok(reach)
    }
}

/// Mails that are receipts and carry no file. Gmail's `{}` is OR: a subject
/// that says so, or a sender known to send receipts as mail.
const BODY_QUERY: &str = "-filename:pdf {subject:receipt subject:invoice subject:račun subject:racun \
                          subject:\"payment confirmation\" subject:\"order confirmation\" \
                          subject:purchase subject:\"your order\" subject:\"thanks for your payment\" \
                          subject:\"payment received\" subject:\"your subscription\" \
                          from:openai.com from:medium.com from:namecheap.com from:audible from:stripe.com \
                          from:playstation from:sony from:paypal.com from:apple.com}";

/// Words a receipt's subject carries. A sender match alone brought every
/// digest and newsletter Medium and Audible send; the subject decides.
const RECEIPT_WORDS: &[&str] = &[
    "receipt",
    "invoice",
    "račun",
    "racun",
    "payment",
    "purchase",
    "order",
    "renewal",
    "renewed",
    "subscription",
    "billing",
    "charged",
    "paid",
    "narudžb",
    "narudzb",
    "uplat",
    "plaćanj",
    "placanj",
];

/// A receipt: its subject says so, it is not a reply or a forward (a
/// conversation about one), and it is not a bot's.
fn looks_like_a_receipt(subject: &str, sender: &str) -> bool {
    let s = subject.trim_start().to_lowercase();
    let from = sender.to_ascii_lowercase();
    if s.starts_with("re:")
        || s.starts_with("fwd:")
        || s.starts_with("fw:")
        || s.starts_with("aw:")
        || from.contains("[bot]")
        || from.contains("notifications@github.com")
        || from.contains("newsletter")
        || from.contains("digest")
    {
        return false;
    }
    RECEIPT_WORDS.iter().any(|w| s.contains(w))
}

/// The text and the HTML bodies of a message, decoded; either may be empty.
fn bodies(payload: &Value) -> (String, String) {
    let mut text = String::new();
    let mut html = String::new();
    for part in parts(payload) {
        let mime = part
            .get("mimeType")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_ascii_lowercase();
        let Some(data) = part.pointer("/body/data").and_then(Value::as_str) else {
            continue;
        };
        let bytes = URL_SAFE
            .decode(data)
            .or_else(|_| base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(data))
            .unwrap_or_default();
        let s = String::from_utf8_lossy(&bytes);
        if mime == "text/plain" && text.is_empty() {
            text = s.into_owned();
        } else if mime == "text/html" && html.is_empty() {
            html = s.into_owned();
        }
    }
    (text, html)
}

/// A Stripe-hosted invoice named in a mail: `https://invoice.stripe.com/i/<acct>/<id>`.
/// The PDF sits at `/pdf` under it, for anyone holding the link.
fn stripe_invoice(text: &str) -> Option<String> {
    static LINK: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        regex::Regex::new(r"https://invoice\.stripe\.com/i/[A-Za-z0-9_]+/[A-Za-z0-9_]+")
            .unwrap_or_else(|e| panic!("{e}"))
    });
    LINK.find(text).map(|m| m.as_str().to_owned())
}

/// A file name from a subject: letters, digits and dashes, forty at most.
fn safe_name(subject: &str) -> String {
    let s: String = subject
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|w| !w.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    let s = s.to_lowercase();
    if s.is_empty() {
        "mail".into()
    } else {
        s.chars().take(40).collect()
    }
}

impl Gmail {
    /// One receipt mail as a document: the Stripe invoice it names, fetched
    /// as a PDF, or the mail itself printed to one.
    #[allow(clippy::too_many_arguments)]
    async fn body_document(
        &self,
        id: &str,
        subject: &str,
        sender: &str,
        received_at: Option<DateTime<Utc>>,
        mailbox: &str,
        text: String,
        html: String,
    ) -> Result<Found, ConnectorError> {
        // A Stripe-hosted invoice is the real document; the mail is a
        // pointer to it.
        let mut found = None;
        if let Some(link) = stripe_invoice(&format!("{text}\n{html}")) {
            match self.fetch_pdf(&format!("{link}/pdf")).await {
                Ok(bytes) => {
                    found = Some(Found {
                        external_ref: format!("{id}:stripe"),
                        filename: format!(
                            "stripe-{}.pdf",
                            link.rsplit('/').next().unwrap_or("invoice")
                        ),
                        content_type: "application/pdf".into(),
                        bytes,
                        subject: subject.to_owned(),
                        sender: sender.to_owned(),
                        received_at,
                        facts: None,
                    });
                }
                Err(e) => {
                    tracing::info!(message = %id, error = %e, "gmail: stripe invoice not fetched; printing the mail");
                }
            }
        }
        let found = if let Some(f) = found {
            f
        } else {
            let body = if text.trim().is_empty() {
                mail::html_to_text(&html)
            } else {
                mail::tidy(&text)
            };
            let printed = mail::Mail {
                message_id: id.to_owned(),
                mailbox: mailbox.to_owned(),
                from: sender.to_owned(),
                subject: subject.to_owned(),
                received: received_at,
                text: body,
            };
            let bytes = tokio::task::spawn_blocking(move || mail::print(&printed))
                .await
                .map_err(|e| ConnectorError::Provider(format!("print: {e}")))?
                .map_err(|e| ConnectorError::Provider(e.to_string()))?;
            Found {
                external_ref: format!("{id}:mail"),
                filename: format!(
                    "{}-{}.pdf",
                    received_at
                        .map(|t| t.format("%Y-%m-%d").to_string())
                        .unwrap_or_default(),
                    safe_name(subject)
                ),
                content_type: "application/pdf".into(),
                bytes,
                subject: subject.to_owned(),
                sender: sender.to_owned(),
                received_at,
                facts: None,
            }
        };
        Ok(found)
    }

    /// A public PDF, by its link, with no token: a Stripe invoice.
    async fn fetch_pdf(&self, url: &str) -> Result<Vec<u8>, ConnectorError> {
        let resp = self
            .http
            .get(url)
            .send()
            .await
            .map_err(|e| ConnectorError::Provider(e.to_string()))?;
        if !resp.status().is_success() {
            return Err(ConnectorError::Provider(format!(
                "{url}: {}",
                resp.status()
            )));
        }
        let bytes = resp
            .bytes()
            .await
            .map_err(|e| ConnectorError::Provider(e.to_string()))?;
        if !bytes.starts_with(b"%PDF") {
            return Err(ConnectorError::Provider(format!("{url}: not a pdf")));
        }
        Ok(bytes.to_vec())
    }
}

impl Gmail {
    /// Every named part of a message as bytes.
    async fn attachments_of(
        &self,
        session: &Session,
        throttle: &Pace,
        id: &str,
        payload: &Value,
    ) -> Result<Vec<Attachment>, ConnectorError> {
        let mut attachments = Vec::new();
        for part in parts(payload) {
            let filename = part.get("filename").and_then(Value::as_str).unwrap_or("");
            let Some(att) = part.pointer("/body/attachmentId").and_then(Value::as_str) else {
                continue;
            };
            if filename.is_empty() {
                continue;
            }
            let blob = self
                .get(
                    session,
                    &format!("{}/messages/{id}/attachments/{att}", self.api),
                    throttle,
                )
                .await?;
            let Some(data) = blob.get("data").and_then(Value::as_str) else {
                continue;
            };
            let bytes = URL_SAFE
                .decode(data)
                .or_else(|_| base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(data))
                .map_err(|e| ConnectorError::Provider(format!("attachment: {e}")))?;
            attachments.push(Attachment {
                filename: filename.to_owned(),
                content_type: part
                    .get("mimeType")
                    .and_then(Value::as_str)
                    .unwrap_or("application/octet-stream")
                    .to_owned(),
                bytes,
            });
        }
        Ok(attachments)
    }
}

impl Gmail {
    /// One authenticated POST of JSON. Google's refusals are classified like
    /// a GET's; a send is never retried, since a retry could send twice.
    async fn post_json(
        &self,
        session: &Session,
        url: &str,
        body: &Value,
    ) -> Result<Value, ConnectorError> {
        let token = self.bearer(session).await?;
        let resp = self
            .http
            .post(url)
            .bearer_auth(token)
            .json(body)
            .send()
            .await
            .map_err(|e| ConnectorError::Provider(e.to_string()))?;
        let status = resp.status();
        if status.is_success() {
            return resp
                .json()
                .await
                .map_err(|e| ConnectorError::Provider(format!("gmail: {e}")));
        }
        let text = resp.text().await.unwrap_or_default();
        Err(match classify(status, &text) {
            Refusal::Unlinked(why) => ConnectorError::Unlinked(why),
            Refusal::Transient(why) | Refusal::Fatal(why) => ConnectorError::Provider(why),
        })
    }
}

/// An RFC 5322 message: text (and HTML beside it when given) with the
/// attachments as base64 parts. Header values carry no line breaks; a
/// subject with a non-ASCII letter is encoded as UTF-8 base64.
#[allow(clippy::format_push_string)]
fn mime(from: &str, mail: &Outgoing, message_id: &str) -> Vec<u8> {
    let clean = |v: &str| v.replace(['\r', '\n'], " ");
    let header_word = |v: &str| {
        if v.is_ascii() {
            clean(v)
        } else {
            format!(
                "=?UTF-8?B?{}?=",
                base64::engine::general_purpose::STANDARD.encode(clean(v))
            )
        }
    };
    let list = |v: &[String]| v.iter().map(|a| clean(a)).collect::<Vec<_>>().join(", ");
    let boundary = format!("=_tbd_{}", uuid::Uuid::new_v4().simple());
    let alt = format!("=_alt_{}", uuid::Uuid::new_v4().simple());
    let mut out = String::new();
    out.push_str(&format!("From: {}\r\n", clean(from)));
    if !mail.to.is_empty() {
        out.push_str(&format!("To: {}\r\n", list(&mail.to)));
    }
    if !mail.cc.is_empty() {
        out.push_str(&format!("Cc: {}\r\n", list(&mail.cc)));
    }
    if !mail.bcc.is_empty() {
        out.push_str(&format!("Bcc: {}\r\n", list(&mail.bcc)));
    }
    out.push_str(&format!("Subject: {}\r\n", header_word(&mail.subject)));
    out.push_str(&format!("Message-ID: {message_id}\r\n"));
    if let Some((_, replied)) = &mail.in_reply_to
        && !replied.is_empty()
    {
        out.push_str(&format!(
            "In-Reply-To: {}\r\nReferences: {}\r\n",
            clean(replied),
            clean(replied)
        ));
    }
    out.push_str("MIME-Version: 1.0\r\n");
    let text_part = |out: &mut String| {
        out.push_str(
            "Content-Type: text/plain; charset=UTF-8\r\nContent-Transfer-Encoding: base64\r\n\r\n",
        );
        out.push_str(&wrap76(
            &base64::engine::general_purpose::STANDARD.encode(&mail.text),
        ));
        out.push_str("\r\n");
    };
    let body_parts = |out: &mut String| match &mail.html {
        Some(html) => {
            out.push_str(&format!(
                "Content-Type: multipart/alternative; boundary=\"{alt}\"\r\n\r\n"
            ));
            out.push_str(&format!("--{alt}\r\n"));
            text_part(out);
            out.push_str(&format!("--{alt}\r\nContent-Type: text/html; charset=UTF-8\r\nContent-Transfer-Encoding: base64\r\n\r\n"));
            out.push_str(&wrap76(
                &base64::engine::general_purpose::STANDARD.encode(html),
            ));
            out.push_str(&format!("\r\n--{alt}--\r\n"));
        }
        None => text_part(out),
    };
    if mail.attachments.is_empty() {
        body_parts(&mut out);
    } else {
        out.push_str(&format!(
            "Content-Type: multipart/mixed; boundary=\"{boundary}\"\r\n\r\n--{boundary}\r\n"
        ));
        body_parts(&mut out);
        for a in &mail.attachments {
            let name = clean(&a.filename).replace('"', "");
            out.push_str(&format!(
                "--{boundary}\r\nContent-Type: {}; name=\"{name}\"\r\nContent-Disposition: attachment; filename=\"{name}\"\r\nContent-Transfer-Encoding: base64\r\n\r\n",
                clean(&a.content_type)
            ));
            out.push_str(&wrap76(
                &base64::engine::general_purpose::STANDARD.encode(&a.bytes),
            ));
            out.push_str("\r\n");
        }
        out.push_str(&format!("--{boundary}--\r\n"));
    }
    out.into_bytes()
}

/// Base64 in lines of 76, as RFC 2045 asks.
fn wrap76(s: &str) -> String {
    s.as_bytes()
        .chunks(76)
        .map(|c| String::from_utf8_lossy(c).into_owned())
        .collect::<Vec<_>>()
        .join("\r\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn body(reason: &str, message: &str) -> String {
        serde_json::json!({
            "error": {
                "code": 403,
                "message": message,
                "errors": [{"domain": "global", "reason": reason, "message": message}],
                "status": "PERMISSION_DENIED"
            }
        })
        .to_string()
    }

    /// Google's answer to a token it no longer honours -- the exact wording
    /// an hour-old one gets.
    fn refused() -> wiremock::ResponseTemplate {
        wiremock::ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "error": {"code": 401, "message": "Request had invalid authentication credentials. Expected OAuth 2 access token, login cookie or other valid authentication credential. See https://developers.google.com/identity/sign-in/web/devconsole-project.", "status": "UNAUTHENTICATED"}
        }))
    }

    fn minted(token: &str) -> wiremock::ResponseTemplate {
        wiremock::ResponseTemplate::new(200)
            .set_body_json(serde_json::json!({ "access_token": token, "expires_in": 3599 }))
    }

    /// A Google that mints `first`, then `second`, and answers the profile
    /// only to `good`.
    async fn google(first: &str, second: &str, good: &str) -> wiremock::MockServer {
        use wiremock::matchers::{header, method, path};
        let server = wiremock::MockServer::start().await;
        wiremock::Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(minted(first))
            .up_to_n_times(1)
            .mount(&server)
            .await;
        wiremock::Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(minted(second))
            .mount(&server)
            .await;
        wiremock::Mock::given(method("GET"))
            .and(path("/profile"))
            .and(header("authorization", format!("Bearer {good}").as_str()))
            .respond_with(wiremock::ResponseTemplate::new(200).set_body_json(
                serde_json::json!({ "emailAddress": "nevio@inorbit.hr", "messagesTotal": 7 }),
            ))
            .mount(&server)
            .await;
        wiremock::Mock::given(method("GET"))
            .and(path("/profile"))
            .respond_with(refused())
            .mount(&server)
            .await;
        server
    }

    async fn mints(server: &wiremock::MockServer) -> usize {
        server
            .received_requests()
            .await
            .unwrap_or_default()
            .iter()
            .filter(|r| r.url.path() == "/token")
            .count()
    }

    #[tokio::test]
    async fn a_token_refused_mid_pull_is_re_minted_once_and_the_pull_goes_on() {
        // The first token is refused the way an hour-old one is; the second
        // works. The link is fine, and the call must say so.
        let server = google("t1", "t2", "t2").await;
        let gmail = Gmail::with_endpoints(&format!("{}/token", server.uri()), &server.uri());
        let status = gmail
            .test(&json!({ "refresh_token": "r" }))
            .await
            .unwrap_or_else(|e| panic!("a refused token is re-minted, not fatal: {e}"));
        assert!(status.contains("nevio@inorbit.hr"), "{status}");
        assert_eq!(mints(&server).await, 2, "the first mint, then one re-mint");
    }

    #[tokio::test]
    async fn a_token_near_its_hour_is_re_minted_before_the_request() {
        let server = google("t1", "t2", "t2").await;
        let gmail = Gmail::with_endpoints(&format!("{}/token", server.uri()), &server.uri());
        let session = gmail
            .open(&json!({ "refresh_token": "r" }))
            .await
            .unwrap_or_else(|e| panic!("{e}"));
        // Fifty-one minutes old: the next request must not go out on it.
        session.minted.lock().await.at = std::time::Instant::now()
            .checked_sub(RENEW_AFTER + std::time::Duration::from_secs(60))
            .unwrap_or_else(|| panic!("the clock started less than an hour ago"));
        let profile = gmail
            .get(
                &session,
                &format!("{}/profile", server.uri()),
                &Pace::default(),
            )
            .await
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(profile["messagesTotal"], 7);
        assert_eq!(mints(&server).await, 2);
        let refusals = server
            .received_requests()
            .await
            .unwrap_or_default()
            .iter()
            .filter(|r| r.url.path() == "/profile")
            .count();
        assert_eq!(
            refusals, 1,
            "re-minted first, so the old token was never sent"
        );
    }

    #[tokio::test]
    async fn a_token_refused_after_a_fresh_mint_means_the_link_is_gone() {
        // Google refuses everything: one re-mint, then the truth, no loop.
        let server = google("t1", "t2", "never").await;
        let gmail = Gmail::with_endpoints(&format!("{}/token", server.uri()), &server.uri());
        let err = gmail
            .test(&json!({ "refresh_token": "r" }))
            .await
            .expect_err("refused twice is unlinked");
        assert!(matches!(err, ConnectorError::Unlinked(_)), "{err}");
        assert_eq!(mints(&server).await, 2, "exactly one re-mint, not a loop");
    }

    #[tokio::test]
    async fn a_send_only_link_goes_through_the_send_client_and_refreshes_there() {
        use wiremock::matchers::{body_string_contains, method, path};
        let server = wiremock::MockServer::start().await;
        // The token endpoint answers only the client whose secret it sees.
        for (secret, token) in [("secret", "read-token"), ("send-secret", "send-token")] {
            wiremock::Mock::given(method("POST"))
                .and(path("/token"))
                .and(body_string_contains(format!("client_secret={secret}")))
                .respond_with(wiremock::ResponseTemplate::new(200).set_body_json(
                    serde_json::json!({ "access_token": token, "refresh_token": "r", "scope": scopes_for(Purpose::Send) }),
                ))
                .mount(&server)
                .await;
        }
        wiremock::Mock::given(method("GET"))
            .and(path("/userinfo"))
            .respond_with(
                wiremock::ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({ "email": "nevio@tenderly.co" })),
            )
            .mount(&server)
            .await;
        let gmail = Gmail::with_endpoints(&format!("{}/token", server.uri()), &server.uri())
            .with_send_client("send-client", "send-secret");
        let ctx = |purpose| AuthContext {
            state: "s".into(),
            redirect_url: "https://f/cb".into(),
            purpose,
        };

        // The consent URL names the client the purpose chooses.
        assert!(
            gmail
                .start(&ctx(Purpose::Send))
                .await
                .unwrap()
                .contains("client_id=send-client")
        );
        assert!(
            gmail
                .start(&ctx(Purpose::Read))
                .await
                .unwrap()
                .contains("client_id=client")
        );
        assert!(
            gmail
                .start(&ctx(Purpose::Both))
                .await
                .unwrap()
                .contains("client_id=client")
        );

        // The exchange goes to the same client and the credential remembers it.
        let linked = gmail.complete(&ctx(Purpose::Send), "code").await.unwrap();
        assert_eq!(linked.credentials["client"], "send");
        let read = gmail.complete(&ctx(Purpose::Both), "code").await.unwrap();
        assert_eq!(read.credentials["client"], "read");

        // A refresh goes back to the client that minted the credential; one
        // from before there were two is the read client's.
        assert_eq!(
            gmail.access_token(&linked.credentials).await.unwrap(),
            "send-token"
        );
        assert_eq!(
            gmail.access_token(&read.credentials).await.unwrap(),
            "read-token"
        );
        assert_eq!(
            gmail
                .access_token(&json!({ "refresh_token": "r" }))
                .await
                .unwrap(),
            "read-token"
        );

        // Without a send client, sending only falls back to the read client.
        let one = Gmail::with_endpoints(&format!("{}/token", server.uri()), &server.uri());
        assert!(
            one.start(&ctx(Purpose::Send))
                .await
                .unwrap()
                .contains("client_id=client")
        );
    }

    #[test]
    fn the_purpose_decides_the_scopes_asked_and_the_grant_decides_the_capabilities() {
        assert_eq!(scopes_for(Purpose::Read), format!("{READ_SCOPE} email"));
        assert_eq!(scopes_for(Purpose::Send), format!("{SEND_SCOPE} email"));
        assert_eq!(
            scopes_for(Purpose::Both),
            format!("{READ_SCOPE} {SEND_SCOPE} email")
        );
        let g = Gmail::new(&Config::default());
        let send_only = g.capabilities(&json!({ "scope": scopes_for(Purpose::Send) }));
        assert!(!send_only.read && send_only.send);
        let read_only = g.capabilities(&json!({ "scope": scopes_for(Purpose::Read) }));
        assert!(read_only.read && !read_only.send);
        let both = g.capabilities(&json!({ "scope": scopes_for(Purpose::Both) }));
        assert!(both.read && both.send);
    }

    #[tokio::test]
    async fn a_send_only_credential_is_tested_through_userinfo_never_the_mailbox() {
        use wiremock::matchers::{method, path};
        let server = google("t1", "t2", "t1").await;
        wiremock::Mock::given(method("GET"))
            .and(path("/userinfo"))
            .respond_with(
                wiremock::ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({ "email": "nevio@tenderly.co" })),
            )
            .mount(&server)
            .await;
        let gmail = Gmail::with_endpoints(&format!("{}/token", server.uri()), &server.uri());
        let status = gmail
            .test(&json!({ "refresh_token": "r", "scope": scopes_for(Purpose::Send) }))
            .await
            .expect("userinfo answers");
        assert_eq!(status, "nevio@tenderly.co · send only");
        let paths: Vec<String> = server
            .received_requests()
            .await
            .unwrap_or_default()
            .iter()
            .map(|r| r.url.path().to_owned())
            .collect();
        assert!(paths.contains(&"/userinfo".to_owned()), "{paths:?}");
        assert!(
            !paths.contains(&"/profile".to_owned()),
            "the mailbox is never touched: {paths:?}"
        );
    }

    #[test]
    fn the_send_scope_decides_the_capability_and_a_mail_is_a_proper_mime() {
        let g = Gmail::new(&Config::default());
        assert!(!g.capabilities(&json!({ "refresh_token": "r" })).send);
        assert!(
            !g.capabilities(
                &json!({ "scope": "https://www.googleapis.com/auth/gmail.readonly email" })
            )
            .send
        );
        assert!(
            g.capabilities(&json!({ "scope": scopes_for(Purpose::Both) }))
                .send
        );
        let mail = Outgoing {
            to: vec!["a@b.hr".into()],
            cc: vec!["c@b.hr".into()],
            bcc: vec![],
            subject: "Računi 8/2026".into(),
            text: "Bok,\nu privitku.".into(),
            html: None,
            attachments: vec![Attachment {
                filename: "racun.pdf".into(),
                content_type: "application/pdf".into(),
                bytes: b"%PDF-x".to_vec(),
            }],
            in_reply_to: None,
        };
        let raw = String::from_utf8(mime("me@inorbit.hr", &mail, "<id@inorbit.hr>")).unwrap();
        assert!(raw.starts_with("From: me@inorbit.hr\r\nTo: a@b.hr\r\nCc: c@b.hr\r\n"));
        assert!(
            raw.contains("Subject: =?UTF-8?B?"),
            "a non-ascii subject is encoded"
        );
        assert!(raw.contains("Message-ID: <id@inorbit.hr>"));
        assert!(raw.contains("Content-Type: multipart/mixed; boundary="));
        assert!(raw.contains("Content-Disposition: attachment; filename=\"racun.pdf\""));
        assert!(!raw.contains("Bcc:"), "no empty header");
    }

    #[test]
    fn a_stripe_invoice_link_is_found_and_a_subject_names_a_file() {
        let mail =
            "View your invoice: https://invoice.stripe.com/i/acct_1ABC/live_YWNjdF8x?s=em <br>";
        assert_eq!(
            stripe_invoice(mail).as_deref(),
            Some("https://invoice.stripe.com/i/acct_1ABC/live_YWNjdF8x")
        );
        assert_eq!(stripe_invoice("no link here"), None);
        assert_eq!(
            safe_name("Your receipt from OpenAI, LLC #1234"),
            "your-receipt-from-openai-llc-1234"
        );
        assert_eq!(safe_name("!!!"), "mail");
        assert!(looks_like_a_receipt(
            "Your receipt from OpenAI",
            "OpenAI <noreply@openai.com>"
        ));
        assert!(!looks_like_a_receipt("Re: invoice 12", "Someone <a@b.hr>"));
        assert!(!looks_like_a_receipt("Fwd: račun", "Someone <a@b.hr>"));
        assert!(!looks_like_a_receipt(
            "purchase",
            "linear[bot] <x@linear.app>"
        ));
        assert!(!looks_like_a_receipt(
            "Today's highlights",
            "Medium Daily Digest <noreply@medium.com>"
        ));
        assert!(!looks_like_a_receipt(
            "New this week",
            "Audible <newsletters@audible.de>"
        ));
        assert!(looks_like_a_receipt(
            "Thank You For Your Purchase",
            "PlayStation <email@email.playstation.com>"
        ));
        assert!(looks_like_a_receipt(
            "Namecheap Renewal Receipt",
            "Namecheap Renewals <renewals@namecheap.com>"
        ));
    }

    #[test]
    fn bodies_are_decoded_by_part() {
        let payload = serde_json::json!({
            "mimeType": "multipart/alternative",
            "parts": [
                {"mimeType": "text/plain", "body": {"data": URL_SAFE.encode("Total $5.00")}},
                {"mimeType": "text/html", "body": {"data": URL_SAFE.encode("<p>Total $5.00</p>")}}
            ]
        });
        let (text, html) = bodies(&payload);
        assert_eq!(text, "Total $5.00");
        assert_eq!(html, "<p>Total $5.00</p>");
    }

    #[test]
    fn a_rate_limit_is_a_403_worth_retrying() {
        let r = classify(
            reqwest::StatusCode::FORBIDDEN,
            &body(
                "userRateLimitExceeded",
                "User-rate limit exceeded. Retry after ...",
            ),
        );
        assert!(matches!(r, Refusal::Transient(w) if w.contains("userRateLimitExceeded")));
        assert!(matches!(
            classify(reqwest::StatusCode::TOO_MANY_REQUESTS, ""),
            Refusal::Transient(_)
        ));
        assert!(matches!(
            classify(reqwest::StatusCode::BAD_GATEWAY, "<html>"),
            Refusal::Transient(_)
        ));
    }

    #[test]
    fn a_missing_scope_means_relink_not_retry() {
        let r = classify(
            reqwest::StatusCode::FORBIDDEN,
            &body(
                "insufficientPermissions",
                "Request had insufficient authentication scopes.",
            ),
        );
        assert!(matches!(r, Refusal::Unlinked(w) if w.contains("link again")));
        // The newer envelope shape, with no `errors` array.
        let newer = serde_json::json!({"error": {"code": 403, "message": "Request had insufficient authentication scopes.", "status": "PERMISSION_DENIED", "details": [{"reason": "ACCESS_TOKEN_SCOPE_INSUFFICIENT"}]}}).to_string();
        assert!(matches!(
            classify(reqwest::StatusCode::FORBIDDEN, &newer),
            Refusal::Unlinked(_)
        ));
    }

    #[test]
    fn a_pace_only_widens_and_stays_bounded() {
        let throttle = Pace::default();
        assert_eq!(throttle.gap_ms(), 0, "nothing until Google complains");
        throttle.slower();
        assert_eq!(throttle.gap_ms(), 1000);
        throttle.slower();
        assert_eq!(throttle.gap_ms(), 2000);
        for _ in 0..10 {
            throttle.slower();
        }
        assert_eq!(throttle.gap_ms(), 15_000);
        assert!(
            RETRY_WAITS.iter().sum::<u64>() > 120,
            "the waits span two quota minutes"
        );
    }

    #[test]
    fn anything_else_carries_googles_reason() {
        let r = classify(
            reqwest::StatusCode::FORBIDDEN,
            &body(
                "accessNotConfigured",
                "Gmail API has not been used in project 1 before",
            ),
        );
        assert_eq!(
            r,
            Refusal::Fatal("gmail 403 Forbidden: accessNotConfigured: Gmail API has not been used in project 1 before".into())
        );
        assert_eq!(
            classify(reqwest::StatusCode::UNAUTHORIZED, ""),
            Refusal::Unlinked("token refused: ".into())
        );
    }
}
