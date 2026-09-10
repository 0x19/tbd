//! Client-side TLS trust shared by `validate` and the load generator.
//!
//! `https://` and `wss://` targets verify against the web PKI roots compiled into
//! the binary, so a public edge (`devops/edge`, Let's Encrypt) works with no flags.
//! [`Trust::from_pem_file`] adds a private root for a staging edge or Caddy's
//! internal CA. `http://` and `ws://` targets are untouched: h2c and plain sockets.
//!
//! The same object carries the bearer token ([`crate::auth::Auth`]) that Envoy
//! demands on every API route: [`Trust::snapshot`] fetches it once, and every
//! client built afterwards (HTTP, WebSocket, gRPC) sends it.

use std::{path::Path, sync::Arc};

use anyhow::Context;
use rustls::{ClientConfig, RootCertStore};
use rustls_pki_types::{CertificateDer, pem::PemObject};
use tokio_tungstenite::{
    Connector, MaybeTlsStream, WebSocketStream, tungstenite, tungstenite::client::IntoClientRequest,
};
use tonic::{
    metadata::{Ascii, MetadataValue},
    service::{Interceptor, interceptor::InterceptedService},
    transport::{Certificate, Channel, ClientTlsConfig, Endpoint},
};

use crate::auth::Auth;

/// A WebSocket over a plain or TLS socket.
pub type Ws = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

/// A gRPC channel that sends the bearer token, when there is one.
pub type Grpc = InterceptedService<Channel, Bearer>;

/// tonic interceptor adding `authorization: Bearer ...`.
#[derive(Debug, Clone)]
pub struct Bearer(Option<MetadataValue<Ascii>>);

impl Interceptor for Bearer {
    fn call(&mut self, mut req: tonic::Request<()>) -> Result<tonic::Request<()>, tonic::Status> {
        if let Some(v) = &self.0 {
            req.metadata_mut().insert("authorization", v.clone());
        }
        Ok(req)
    }
}

/// Which roots to trust when a target is `https://` or `wss://`, and which
/// bearer token to send.
#[derive(Debug, Clone, Default)]
pub struct Trust {
    ca_pem: Option<Vec<u8>>,
    auth: Option<Auth>,
    bearer: Option<String>,
}

impl Trust {
    /// Trust the web PKI roots plus every certificate in the PEM file at `path`.
    ///
    /// # Errors
    /// The file cannot be read or holds no certificate.
    pub fn from_pem_file(path: &Path) -> anyhow::Result<Self> {
        let pem = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
        let count = CertificateDer::pem_slice_iter(&pem).flatten().count();
        anyhow::ensure!(count > 0, "no certificate in {}", path.display());
        Ok(Self {
            ca_pem: Some(pem),
            ..Self::default()
        })
    }

    /// Attach a token source; `None` sends no token.
    #[must_use]
    pub fn with_auth(mut self, auth: Option<Auth>) -> Self {
        self.auth = auth;
        self
    }

    /// Whether a token source is configured.
    #[must_use]
    pub fn has_auth(&self) -> bool {
        self.auth.is_some()
    }

    /// A copy holding the current bearer token, fetched (or refreshed) now.
    /// Clients built from it send the token; call it at the start of a run.
    ///
    /// # Errors
    /// The token could not be obtained.
    pub async fn snapshot(&self) -> anyhow::Result<Self> {
        let bearer = match &self.auth {
            Some(auth) => Some(auth.bearer().await?),
            None => None,
        };
        Ok(Self {
            ca_pem: self.ca_pem.clone(),
            auth: self.auth.clone(),
            bearer,
        })
    }

    /// The `Authorization` header value, if a token was snapshotted.
    #[must_use]
    pub fn authorization(&self) -> Option<String> {
        self.bearer.as_ref().map(|b| format!("Bearer {b}"))
    }

    /// A gRPC channel to `url` (lazy connect) that sends the token.
    ///
    /// # Errors
    /// The URL is invalid or the extra root cannot be parsed.
    pub fn grpc(
        &self,
        url: &str,
        timeout: Option<std::time::Duration>,
    ) -> Result<Grpc, tonic::transport::Error> {
        let mut ep = self.endpoint(url)?;
        if let Some(t) = timeout {
            ep = ep.timeout(t);
        }
        let bearer = Bearer(
            self.authorization()
                .and_then(|v| MetadataValue::try_from(v).ok()),
        );
        Ok(InterceptedService::new(ep.connect_lazy(), bearer))
    }

    /// A gRPC endpoint for `url`; TLS only when the scheme is `https`.
    ///
    /// # Errors
    /// The URL is invalid or the extra root cannot be parsed.
    pub fn endpoint(&self, url: &str) -> Result<Endpoint, tonic::transport::Error> {
        let ep = Endpoint::from_shared(url.to_owned())?;
        if !url.starts_with("https://") {
            return Ok(ep);
        }
        let mut tls = ClientTlsConfig::new().with_webpki_roots();
        if let Some(pem) = &self.ca_pem {
            tls = tls.ca_certificate(Certificate::from_pem(pem));
        }
        ep.tls_config(tls)
    }

    /// An HTTP client that trusts the same roots and sends the token.
    #[must_use]
    pub fn http(&self, timeout: Option<std::time::Duration>) -> reqwest::Client {
        let mut b = reqwest::Client::builder();
        if let Some(t) = timeout {
            b = b.timeout(t);
        }
        if let Some(value) = self
            .authorization()
            .and_then(|v| reqwest::header::HeaderValue::from_str(&v).ok())
        {
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert(reqwest::header::AUTHORIZATION, value);
            b = b.default_headers(headers);
        }
        if let Some(certs) = self
            .ca_pem
            .as_deref()
            .and_then(|pem| reqwest::Certificate::from_pem_bundle(pem).ok())
        {
            for c in certs {
                b = b.add_root_certificate(c);
            }
        }
        b.build().unwrap_or_default()
    }

    /// Open a WebSocket with `TCP_NODELAY` set. `connect_async` leaves Nagle on,
    /// which stalls small frames by ~40 ms against a server with delayed ACKs.
    ///
    /// # Errors
    /// DNS, TCP, TLS or the WebSocket handshake failed.
    pub async fn connect_ws(&self, url: &str) -> anyhow::Result<Ws> {
        let parsed = url::Url::parse(url)?;
        let host = parsed
            .host_str()
            .ok_or_else(|| anyhow::anyhow!("no host in {url}"))?;
        let port = parsed
            .port_or_known_default()
            .ok_or_else(|| anyhow::anyhow!("no port in {url}"))?;
        let stream = tokio::net::TcpStream::connect((host, port)).await?;
        stream.set_nodelay(true)?;
        let mut request = url.into_client_request()?;
        if let Some(value) = self
            .authorization()
            .and_then(|v| tungstenite::http::HeaderValue::from_str(&v).ok())
        {
            request
                .headers_mut()
                .insert(tungstenite::http::header::AUTHORIZATION, value);
        }
        let (ws, _) = tokio_tungstenite::client_async_tls_with_config(
            request,
            stream,
            None,
            self.ws_connector(),
        )
        .await?;
        Ok(ws)
    }

    /// `None` keeps tungstenite's default (web PKI roots); a private root needs an
    /// explicit rustls config.
    fn ws_connector(&self) -> Option<Connector> {
        let pem = self.ca_pem.as_deref()?;
        let mut roots = RootCertStore {
            roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
        };
        roots.add_parsable_certificates(CertificateDer::pem_slice_iter(pem).filter_map(Result::ok));
        let config = ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth();
        Some(Connector::Rustls(Arc::new(config)))
    }
}
