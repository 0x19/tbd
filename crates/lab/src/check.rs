//! What a check is, and what it is handed.
//!
//! The types only. Running them, reporting them and pointing them at a
//! configuration is the chaos tool's job; a kind needs no more than this to
//! declare the checks it ships with.

use crate::tls::{Grpc, Trust, Ws};

/// What one check receives: the target URL of its kind and the trust to use.
#[derive(Debug, Clone)]
pub struct Endpoint {
    /// `http://host:port` or `https://...`.
    pub url: String,
    /// Roots and bearer token.
    pub trust: Trust,
}

impl Endpoint {
    /// An HTTP client.
    #[must_use]
    pub fn http(&self) -> reqwest::Client {
        self.trust.http(None)
    }

    /// A gRPC channel to the URL.
    ///
    /// # Errors
    /// The URL is invalid.
    pub fn grpc(&self) -> Result<Grpc, String> {
        self.trust.grpc(&self.url, None).map_err(|e| e.to_string())
    }

    /// `ws://host:port<path>`.
    #[must_use]
    pub fn ws_url(&self, path: &str) -> String {
        self.url.replacen("http", "ws", 1) + path
    }

    /// A WebSocket at `path`.
    ///
    /// # Errors
    /// The handshake fails.
    pub async fn connect_ws(&self, path: &str) -> Result<Ws, String> {
        self.trust
            .connect_ws(&self.ws_url(path))
            .await
            .map_err(|e| e.to_string())
    }
}

/// The body of a check.
pub type CheckFn = fn(Endpoint) -> futures::future::BoxFuture<'static, Result<String, String>>;

/// One check of a kind.
pub struct Check {
    /// Name, unique across kinds.
    pub name: &'static str,
    /// Surface it exercises.
    pub surface: &'static str,
    /// When it passes, for the docs.
    pub doc: &'static str,
    /// The check.
    pub run: CheckFn,
}
