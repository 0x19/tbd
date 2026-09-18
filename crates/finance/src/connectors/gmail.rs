//! Gmail: receipts that arrive by mail.
//!
//! Google OAuth with the read-only Gmail scope and `email`, so the connector
//! can name the mailbox it is. A pull runs the configured query (default:
//! anything with a PDF attached, since the last pull), fetches each message,
//! and keeps the PDF attachments. Hosted-link receipts (Stripe's "view your
//! receipt") are a later step; they need a fetch of the link.

use async_trait::async_trait;
use base64::{Engine, engine::general_purpose::URL_SAFE};
use chrono::{DateTime, Utc};
use serde_json::{Value, json};

use super::{Auth, AuthContext, Connector, ConnectorError, Found, Kind, Linked};
use crate::config::Connectors as Config;

const AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const USERINFO_URL: &str = "https://www.googleapis.com/oauth2/v3/userinfo";
const API: &str = "https://gmail.googleapis.com/gmail/v1/users/me";
const SCOPES: &str = "https://www.googleapis.com/auth/gmail.readonly email";
/// Never more than this per pull; a first pull of a busy mailbox is paged
/// over several runs rather than held open for minutes.
const MAX_MESSAGES: usize = 200;

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

    async fn get(&self, token: &str, url: &str) -> Result<Value, ConnectorError> {
        let resp = self
            .http
            .get(url)
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| ConnectorError::Provider(e.to_string()))?;
        let status = resp.status();
        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(ConnectorError::Unlinked("token refused".into()));
        }
        if !status.is_success() {
            return Err(ConnectorError::Provider(format!("gmail {status}")));
        }
        resp.json()
            .await
            .map_err(|e| ConnectorError::Provider(format!("gmail: {e}")))
    }
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
            description: "Receipts and invoices that arrive as PDF attachments.",
            auth: Auth::Oauth,
            consent_note: "Read-only access to the mailbox. Nothing is sent, moved or deleted; only \
                           messages matching the query are read, and only their PDF attachments are kept.",
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
        let who = self.get(access, USERINFO_URL).await?;
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
        let profile = self.get(&token, &format!("{API}/profile")).await?;
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
    ) -> Result<(Vec<Found>, Option<Value>), ConnectorError> {
        let token = self.access_token(credentials).await?;
        let query = config
            .get("query")
            .and_then(Value::as_str)
            .filter(|q| !q.trim().is_empty())
            .map_or_else(|| "has:attachment filename:pdf".to_owned(), str::to_owned);
        let q = format!("{query} after:{}", since.format("%Y/%m/%d"));

        let mut ids = Vec::new();
        let mut page: Option<String> = None;
        loop {
            let mut url = reqwest::Url::parse(&format!("{API}/messages"))
                .map_err(|e| ConnectorError::Provider(e.to_string()))?;
            url.query_pairs_mut()
                .append_pair("q", &q)
                .append_pair("maxResults", "100");
            if let Some(p) = &page {
                url.query_pairs_mut().append_pair("pageToken", p);
            }
            let list = self.get(&token, url.as_str()).await?;
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
            if page.is_none() || ids.len() >= MAX_MESSAGES {
                break;
            }
        }

        let mut found = Vec::new();
        for id in ids.iter().take(MAX_MESSAGES) {
            if seen(id) {
                continue;
            }
            let message = self
                .get(&token, &format!("{API}/messages/{id}?format=full"))
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
                    .get(&token, &format!("{API}/messages/{id}/attachments/{att}"))
                    .await?;
                let Some(data) = blob.get("data").and_then(Value::as_str) else {
                    continue;
                };
                let bytes = URL_SAFE
                    .decode(data)
                    .or_else(|_| base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(data))
                    .map_err(|e| ConnectorError::Provider(format!("attachment: {e}")))?;
                found.push(Found {
                    external_ref: format!("{id}:{filename}"),
                    filename: filename.to_owned(),
                    content_type: "application/pdf".into(),
                    bytes,
                    subject: subject.clone(),
                    sender: sender.clone(),
                    received_at,
                });
            }
        }
        Ok((found, None))
    }
}
