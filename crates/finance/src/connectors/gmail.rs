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

use super::{Auth, AuthContext, Connector, ConnectorError, Found, Kind, Linked, Reach};
use crate::{config::Connectors as Config, documents::mail};

const AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const USERINFO_URL: &str = "https://www.googleapis.com/oauth2/v3/userinfo";
const API: &str = "https://gmail.googleapis.com/gmail/v1/users/me";
const SCOPES: &str = "https://www.googleapis.com/auth/gmail.readonly email";
/// Never more than this per pull; a first pull of a busy mailbox is paged
/// over several runs rather than held open for minutes.
/// Messages fetched per round. A round runs detached from the RPC, so the
/// cap bounds one run's Gmail quota, not a request timeout; a mailbox with
/// more says `complete = false` and the caller comes round again.
const MAX_MESSAGES: usize = 500;

/// The kind.
#[derive(Debug, Clone)]
pub struct Gmail {
    client_id: String,
    client_secret: String,
    http: reqwest::Client,
}

impl Gmail {
    /// From configuration; unconfigured is allowed, and says so on link.
    #[must_use]
    pub fn new(config: &Config) -> Self {
        Self {
            client_id: config.google_client_id.clone(),
            client_secret: config.google_client_secret.clone(),
            http: reqwest::Client::new(),
        }
    }

    fn configured(&self) -> bool {
        !self.client_id.is_empty() && !self.client_secret.is_empty()
    }

    async fn access_token(&self, credentials: &Value) -> Result<String, ConnectorError> {
        let refresh = credentials
            .get("refresh_token")
            .and_then(Value::as_str)
            .ok_or_else(|| ConnectorError::Unlinked("no refresh token".into()))?;
        let resp = self
            .http
            .post(TOKEN_URL)
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
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
        token: &str,
        q: &str,
        throttle: &Pace,
    ) -> Result<Vec<String>, ConnectorError> {
        let mut ids = Vec::new();
        let mut page: Option<String> = None;
        loop {
            let mut url = reqwest::Url::parse(&format!("{API}/messages"))
                .map_err(|e| ConnectorError::Provider(e.to_string()))?;
            url.query_pairs_mut()
                .append_pair("q", q)
                .append_pair("maxResults", "100");
            if let Some(p) = &page {
                url.query_pairs_mut().append_pair("pageToken", p);
            }
            let list = self.get(token, url.as_str(), throttle).await?;
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
    /// quota refusal widens it for the rest of the pull.
    async fn get(&self, token: &str, url: &str, pace: &Pace) -> Result<Value, ConnectorError> {
        for (attempt, wait) in RETRY_WAITS.iter().enumerate() {
            pace.hold().await;
            let resp = self
                .http
                .get(url)
                .bearer_auth(token)
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
            consent_note: "Read-only access to the mailbox. Nothing is sent, moved or deleted; only \
                           messages matching the queries are read: PDF attachments are kept, and a \
                           receipt mail without one is printed to a page.",
            configured: self.configured(),
        }
    }

    async fn start(&self, ctx: &AuthContext) -> Result<String, ConnectorError> {
        if !self.configured() {
            return Err(ConnectorError::Unconfigured(
                "set FINANCE_GOOGLE_CLIENT_ID and FINANCE_GOOGLE_CLIENT_SECRET".into(),
            ));
        }
        let mut url =
            reqwest::Url::parse(AUTH_URL).map_err(|e| ConnectorError::Provider(e.to_string()))?;
        url.query_pairs_mut()
            .append_pair("client_id", &self.client_id)
            .append_pair("redirect_uri", &ctx.redirect_url)
            .append_pair("response_type", "code")
            .append_pair("scope", SCOPES)
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
        let resp = self
            .http
            .post(TOKEN_URL)
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
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
        let who = self.get(access, USERINFO_URL, &Pace::default()).await?;
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
        Ok(Linked {
            credentials: json!({ "refresh_token": refresh }),
            external_id: email.clone(),
            label: email,
        })
    }

    async fn test(&self, credentials: &Value) -> Result<String, ConnectorError> {
        let token = self.access_token(credentials).await?;
        let profile = self
            .get(&token, &format!("{API}/profile"), &Pace::default())
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
        let token = self.access_token(credentials).await?;
        let throttle = Pace::default();
        let query = config
            .get("query")
            .and_then(Value::as_str)
            .filter(|q| !q.trim().is_empty())
            .map_or_else(|| "has:attachment filename:pdf".to_owned(), str::to_owned);
        let q = format!("{query} after:{}", since.format("%Y/%m/%d"));

        let ids = self.list_ids(&token, &q, &throttle).await?;

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
                    &token,
                    &format!("{API}/messages/{id}?format=full"),
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
                        &token,
                        &format!("{API}/messages/{id}/attachments/{att}"),
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
                };
                // A closed sink is the caller done listening (its store
                // failed, or the run was abandoned): nothing more to fetch.
                if sink.send(found).await.is_err() {
                    return Ok(Reach::Truncated);
                }
            }
        }

        let bodies = self
            .pull_bodies(&token, config, since, seen, &sink, &throttle)
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
        token: &str,
        config: &Value,
        since: DateTime<Utc>,
        seen: &(dyn for<'a> Fn(&'a str) -> bool + Sync),
        sink: &tokio::sync::mpsc::Sender<Found>,
        throttle: &Pace,
    ) -> Result<Reach, ConnectorError> {
        let mailbox = self
            .get(token, &format!("{API}/profile"), throttle)
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
        let ids = self.list_ids(token, &q, throttle).await?;
        let unseen: Vec<&String> = ids.iter().filter(|id| !seen(id)).collect();
        let reach = if unseen.len() <= MAX_MESSAGES {
            Reach::Complete
        } else {
            Reach::Truncated
        };
        for id in unseen.into_iter().take(MAX_MESSAGES) {
            let message = self
                .get(token, &format!("{API}/messages/{id}?format=full"), throttle)
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
