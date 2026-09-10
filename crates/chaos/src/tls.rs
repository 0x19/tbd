//! Client-side TLS trust shared by `validate` and the load generator.
//!
//! `https://` and `wss://` targets verify against the web PKI roots compiled into
//! the binary, so a public edge (`devops/edge`, Let's Encrypt) works with no flags.
//! [`Trust::from_pem_file`] adds a private root for a staging edge or Caddy's
//! internal CA. `http://` and `ws://` targets are untouched: h2c and plain sockets.

use std::{path::Path, sync::Arc};

use anyhow::Context;
use rustls::{ClientConfig, RootCertStore};
use rustls_pki_types::{CertificateDer, pem::PemObject};
use tokio_tungstenite::{Connector, MaybeTlsStream, WebSocketStream};
use tonic::transport::{Certificate, ClientTlsConfig, Endpoint};

/// A WebSocket over a plain or TLS socket.
pub type Ws = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

/// Which roots to trust when a target is `https://` or `wss://`.
#[derive(Debug, Clone, Default)]
pub struct Trust {
    ca_pem: Option<Vec<u8>>,
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
        Ok(Self { ca_pem: Some(pem) })
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

    /// An HTTP client that trusts the same roots.
    #[must_use]
    pub fn http(&self, timeout: Option<std::time::Duration>) -> reqwest::Client {
        let mut b = reqwest::Client::builder();
        if let Some(t) = timeout {
            b = b.timeout(t);
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
        let (ws, _) =
            tokio_tungstenite::client_async_tls_with_config(url, stream, None, self.ws_connector())
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
