//! Bearer tokens for the deployed stack. Envoy admits API calls only with a
//! JWT from the identity provider (docs/auth/README.md), so `validate` and load
//! runs against a real environment carry one: either a fixed token (`--token`)
//! or one fetched with the client-credentials grant and refreshed before it
//! expires. In-process stacks (`chaos up`, scenarios) have no Envoy and no auth.

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::Context;
use serde::Deserialize;
use tokio::sync::Mutex;

/// Where tokens come from.
#[derive(Debug, Clone)]
pub enum Auth {
    /// A token supplied by the caller; never refreshed.
    Static(String),
    /// Client credentials: fetched on first use, refreshed 60 s before expiry.
    ClientCredentials(Arc<ClientCredentials>),
}

/// The client-credentials grant against `token_url`.
#[derive(Debug)]
pub struct ClientCredentials {
    token_url: String,
    client_id: String,
    client_secret: String,
    scope: String,
    audience: String,
    http: reqwest::Client,
    cached: Mutex<Option<Cached>>,
}

#[derive(Debug, Clone)]
struct Cached {
    token: String,
    expires: Instant,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    #[serde(default)]
    expires_in: Option<u64>,
}

impl Auth {
    /// A fixed bearer token.
    #[must_use]
    pub fn token(token: impl Into<String>) -> Self {
        Self::Static(token.into())
    }

    /// Client credentials. `scope` and `audience` may be empty.
    #[must_use]
    pub fn client_credentials(
        token_url: impl Into<String>,
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
        scope: impl Into<String>,
        audience: impl Into<String>,
    ) -> Self {
        Self::ClientCredentials(Arc::new(ClientCredentials {
            token_url: token_url.into(),
            client_id: client_id.into(),
            client_secret: client_secret.into(),
            scope: scope.into(),
            audience: audience.into(),
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
            cached: Mutex::new(None),
        }))
    }

    /// The current bearer token, fetching or refreshing if needed.
    ///
    /// # Errors
    /// The token endpoint is unreachable, rejects the client, or answers
    /// without an access token.
    pub async fn bearer(&self) -> anyhow::Result<String> {
        match self {
            Self::Static(t) => Ok(t.clone()),
            Self::ClientCredentials(cc) => cc.bearer().await,
        }
    }

    /// The `Authorization` header value.
    ///
    /// # Errors
    /// Same as [`Auth::bearer`].
    pub async fn header(&self) -> anyhow::Result<String> {
        Ok(format!("Bearer {}", self.bearer().await?))
    }
}

impl ClientCredentials {
    async fn bearer(&self) -> anyhow::Result<String> {
        let mut cached = self.cached.lock().await;
        if let Some(c) = cached.as_ref()
            && c.expires > Instant::now()
        {
            return Ok(c.token.clone());
        }
        let mut form = vec![("grant_type", "client_credentials")];
        if !self.scope.is_empty() {
            form.push(("scope", &self.scope));
        }
        if !self.audience.is_empty() {
            form.push(("audience", &self.audience));
        }
        let response = self
            .http
            .post(&self.token_url)
            .basic_auth(&self.client_id, Some(&self.client_secret))
            .form(&form)
            .send()
            .await
            .with_context(|| format!("token endpoint {}", self.token_url))?;
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        anyhow::ensure!(
            status.is_success(),
            "token endpoint {} answered {status}: {body}",
            self.token_url
        );
        let parsed: TokenResponse =
            serde_json::from_str(&body).context("token endpoint answered without a token")?;
        let ttl = parsed.expires_in.unwrap_or(300);
        *cached = Some(Cached {
            token: parsed.access_token.clone(),
            expires: Instant::now() + Duration::from_secs(ttl.saturating_sub(60).max(5)),
        });
        Ok(parsed.access_token)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use axum::{Router, routing::post};

    use super::*;

    #[tokio::test]
    async fn client_credentials_fetch_once_and_reuse_until_expiry() {
        static CALLS: AtomicUsize = AtomicUsize::new(0);
        let app = Router::new().route(
            "/oauth2/token",
            post(|headers: axum::http::HeaderMap, body: String| async move {
                CALLS.fetch_add(1, Ordering::SeqCst);
                assert!(
                    headers["authorization"]
                        .to_str()
                        .unwrap()
                        .starts_with("Basic ")
                );
                assert!(body.contains("grant_type=client_credentials"));
                assert!(body.contains("audience=tbd-api"));
                axum::Json(serde_json::json!({ "access_token": "tok", "expires_in": 3600 }))
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(axum::serve(listener, app).into_future());

        let auth = Auth::client_credentials(
            format!("http://{addr}/oauth2/token"),
            "tbd-chaos",
            "secret",
            "tbd.api",
            "tbd-api",
        );
        assert_eq!(auth.header().await.unwrap(), "Bearer tok");
        assert_eq!(auth.bearer().await.unwrap(), "tok");
        assert_eq!(
            CALLS.load(Ordering::SeqCst),
            1,
            "second call served from cache"
        );
        assert_eq!(Auth::token("fixed").header().await.unwrap(), "Bearer fixed");
    }
}
