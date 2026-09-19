//! The Enable Banking client.
//!
//! Thin on purpose. It signs, sends, pages, and classifies the answer. It does
//! not retry, back off, cache, or count -- the syncer owns those against
//! database rows, because an allowance of four calls a day is not something
//! to track in a process that restarts.

use std::time::Duration;

use async_trait::async_trait;
use chrono::NaiveDate;
use reqwest::{StatusCode, header};
use serde_json::{Value, json};

use super::{
    Authorization, AuthorizationRequest, Provider, ProviderError, Psu, Session, SessionStatus,
    auth::Signer, timestamp,
};
use crate::import::accounts_in;

/// The production endpoint.
pub const DEFAULT_BASE_URL: &str = "https://api.enablebanking.com";

/// Pages beyond this are a bug or a hostile server, not a long history. One
/// real account here has 51.
const MAX_PAGES: usize = 200;

/// A signed client for one application.
#[derive(Debug)]
pub struct EnableBanking {
    base_url: String,
    signer: Signer,
    http: reqwest::Client,
}

impl EnableBanking {
    /// Build against a base URL. Tests point this at a fixture server.
    ///
    /// # Errors
    /// The HTTP client cannot be built, which reqwest reserves for a broken
    /// TLS setup.
    pub fn new(base_url: &str, signer: Signer, timeout: Duration) -> Result<Self, ProviderError> {
        let http = reqwest::Client::builder()
            .timeout(timeout)
            .user_agent(concat!("tbd-finance/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|e| ProviderError::Config {
                field: "http",
                reason: e.to_string(),
            })?;
        Ok(Self {
            base_url: base_url.trim_end_matches('/').to_owned(),
            signer,
            http,
        })
    }

    async fn send(&self, req: reqwest::RequestBuilder) -> Result<Value, ProviderError> {
        let token = self.signer.mint()?;
        let resp = req
            .bearer_auth(token)
            .header(header::ACCEPT, "application/json")
            .send()
            .await
            .map_err(|e| ProviderError::Transport(redact(&e.to_string())))?;
        let status = resp.status();
        let retry_after = resp
            .headers()
            .get(header::RETRY_AFTER)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.trim().parse::<u64>().ok())
            .map(Duration::from_secs);
        let text = resp
            .text()
            .await
            .map_err(|e| ProviderError::Transport(redact(&e.to_string())))?;
        classify(status, retry_after, &text)
    }

    async fn get(
        &self,
        path: &str,
        query: &[(&str, String)],
        psu: Option<&Psu>,
    ) -> Result<Value, ProviderError> {
        let mut req = self
            .http
            .get(format!("{}{path}", self.base_url))
            .query(query);
        // Attended: the person's address goes with the call, and the bank
        // does not count it against the unattended allowance. Erste requires
        // no PSU headers, so sending these two alone is within the rule
        // "all required headers or none".
        if let Some(psu) = psu {
            req = req.header("Psu-Ip-Address", &psu.ip);
            if let Some(ua) = &psu.user_agent {
                req = req.header("Psu-User-Agent", ua);
            }
        }
        self.send(req).await
    }

    async fn post(&self, path: &str, body: &Value) -> Result<Value, ProviderError> {
        let req = self
            .http
            .post(format!("{}{path}", self.base_url))
            .json(body);
        self.send(req).await
    }
}

/// Turn a response into a value or the right error.
///
/// Enable Banking wraps errors as `{code, error, message, detail: {error_name}}`
/// and the bank's own consent failures arrive as 4xx with an `error_name`
/// naming the consent, so that field decides between "re-authorize" and
/// "something else". A body is never logged whole: the token is not in it,
/// but account numbers are.
fn classify(
    status: StatusCode,
    retry_after: Option<Duration>,
    text: &str,
) -> Result<Value, ProviderError> {
    if status.is_success() {
        return serde_json::from_str(text)
            .map_err(|e| ProviderError::Malformed(format!("not json: {e}")));
    }
    let body: Value = serde_json::from_str(text).unwrap_or(Value::Null);
    let error_name = body
        .pointer("/detail/error_name")
        .or_else(|| body.get("error"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let message = body
        .get("message")
        .and_then(Value::as_str)
        .unwrap_or("")
        .chars()
        .take(300)
        .collect::<String>();
    let summary = format!("{error_name}: {message}");

    match status {
        StatusCode::TOO_MANY_REQUESTS => Err(ProviderError::RateLimited { retry_after }),
        StatusCode::UNAUTHORIZED => Err(ProviderError::Unauthorized(summary)),
        StatusCode::FORBIDDEN => {
            // 403 is also what an inactive application gets on every endpoint,
            // which is a configuration problem and not a consent one. The
            // error name tells them apart.
            if consent_related(error_name) {
                Err(ProviderError::ConsentInvalid(summary))
            } else {
                Err(ProviderError::Unauthorized(summary))
            }
        }
        s if consent_related(error_name) => {
            let _ = s;
            Err(ProviderError::ConsentInvalid(summary))
        }
        s => Err(ProviderError::Http {
            status: s.as_u16(),
            body: summary,
        }),
    }
}

fn consent_related(error_name: &str) -> bool {
    let n = error_name.to_ascii_uppercase();
    n.contains("CONSENT")
        || n.contains("SESSION") && (n.contains("EXPIRED") || n.contains("INVALID"))
}

/// Strip anything that looks like a bearer token from an error string.
fn redact(s: &str) -> String {
    s.split_whitespace()
        .map(|w| {
            if w.len() > 80 && w.matches('.').count() == 2 {
                "<token>"
            } else {
                w
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[async_trait]
impl Provider for EnableBanking {
    fn name(&self) -> &'static str {
        "enablebanking"
    }

    async fn start_authorization(
        &self,
        request: &AuthorizationRequest,
    ) -> Result<Authorization, ProviderError> {
        let body = json!({
            "access": { "valid_until": request.valid_until.to_rfc3339_opts(chrono::SecondsFormat::Secs, true) },
            "aspsp": { "name": request.aspsp_name, "country": request.aspsp_country },
            "state": request.state,
            "redirect_url": request.redirect_url,
            "psu_type": request.psu_type,
            "language": "hr",
        });
        let v = self.post("/auth", &body).await?;
        Ok(Authorization {
            url: str_field(&v, "url")?,
            authorization_id: str_field(&v, "authorization_id")?,
        })
    }

    async fn create_session(&self, code: &str) -> Result<Session, ProviderError> {
        let v = self.post("/sessions", &json!({ "code": code })).await?;
        Ok(Session {
            session_id: str_field(&v, "session_id")?,
            accounts: accounts_in(&v),
            valid_until: timestamp(v.pointer("/access/valid_until")),
            raw: v,
        })
    }

    async fn session(&self, session_id: &str) -> Result<SessionStatus, ProviderError> {
        let v = self
            .get(&format!("/sessions/{session_id}"), &[], None)
            .await?;
        Ok(SessionStatus {
            status: str_field(&v, "status")?,
            account_uids: v
                .get("accounts")
                .and_then(Value::as_array)
                .map(|a| {
                    a.iter()
                        .filter_map(|u| u.as_str().map(str::to_owned))
                        .collect()
                })
                .unwrap_or_default(),
            valid_until: timestamp(v.pointer("/access/valid_until")),
        })
    }

    async fn balances(&self, account_uid: &str, psu: Option<&Psu>) -> Result<Value, ProviderError> {
        self.get(&format!("/accounts/{account_uid}/balances"), &[], psu)
            .await
    }

    async fn transactions(
        &self,
        account_uid: &str,
        from: NaiveDate,
        to: NaiveDate,
        psu: Option<&Psu>,
    ) -> Result<Vec<Value>, ProviderError> {
        let path = format!("/accounts/{account_uid}/transactions");
        let base = [("date_from", from.to_string()), ("date_to", to.to_string())];
        let mut pages = Vec::new();
        let mut continuation: Option<String> = None;
        loop {
            // Erste refuses a continuation key sent alone (422: "dateFrom in
            // request is not the same as in continuationKey"), so every page
            // repeats the original window and adds the key.
            let mut query: Vec<(&str, String)> = base.to_vec();
            if let Some(key) = &continuation {
                query.push(("continuation_key", key.clone()));
            }
            let page = self.get(&path, &query, psu).await?;
            continuation = page
                .get("continuation_key")
                .and_then(Value::as_str)
                .filter(|k| !k.is_empty())
                .map(str::to_owned);
            pages.push(page);
            if continuation.is_none() {
                return Ok(pages);
            }
            if pages.len() >= MAX_PAGES {
                return Err(ProviderError::Malformed(format!(
                    "more than {MAX_PAGES} pages; the continuation never ends"
                )));
            }
        }
    }
}

fn str_field(v: &Value, key: &str) -> Result<String, ProviderError> {
    v.get(key)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| ProviderError::Malformed(format!("missing `{key}`")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_429_is_rate_limited_and_carries_the_header() {
        let e = classify(
            StatusCode::TOO_MANY_REQUESTS,
            Some(Duration::from_secs(21600)),
            "{}",
        )
        .unwrap_err();
        assert!(
            matches!(e, ProviderError::RateLimited { retry_after: Some(d) } if d.as_secs() == 21600)
        );
        assert!(!e.retryable(), "a 429 must never be retried in-process");
    }

    #[test]
    fn a_consent_error_asks_for_a_person() {
        let body = r#"{"code":403,"error":"FORBIDDEN","message":"x","detail":{"error_name":"ConsentExpiredException"}}"#;
        let e = classify(StatusCode::FORBIDDEN, None, body).unwrap_err();
        assert!(e.needs_consent(), "{e}");
    }

    #[test]
    fn an_inactive_application_is_unauthorized_not_a_consent_problem() {
        let body = r#"{"code":403,"error":"FORBIDDEN","message":"Application is not active"}"#;
        let e = classify(StatusCode::FORBIDDEN, None, body).unwrap_err();
        assert!(matches!(e, ProviderError::Unauthorized(_)), "{e}");
    }

    #[test]
    fn the_real_422_is_reported_with_its_name() {
        let body = r#"{"code":422,"error":"WRONG_REQUEST_PARAMETERS","message":"dateFrom in request is not the same as in continuationKey","detail":{"error_name":"ParameterValidationException"}}"#;
        let e = classify(StatusCode::UNPROCESSABLE_ENTITY, None, body).unwrap_err();
        assert!(
            e.to_string().contains("ParameterValidationException"),
            "{e}"
        );
    }

    #[test]
    fn a_token_shaped_word_is_redacted() {
        let tok = format!("aaaa.{}.cccc", "b".repeat(100));
        assert_eq!(
            redact(&format!("error sending {tok} here")),
            "error sending <token> here"
        );
    }
}
