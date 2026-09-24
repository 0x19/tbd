//! The daemon on port 0 with the shipped config and a throwaway token.

#![allow(clippy::unwrap_used, clippy::expect_used, dead_code)]

use std::path::Path;

use tbd_sandboxd::{
    Config,
    server::{AppState, router},
};

pub const TOKEN: &str = "test-token-test-token-test-token-0123";

pub struct Daemon {
    pub url: String,
    _stop: tokio::sync::oneshot::Sender<()>,
}

impl Daemon {
    pub async fn start() -> Self {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../configs/sandboxd");
        let (config, _) = Config::load(&dir, "local").unwrap();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let (stop, stopped) = tokio::sync::oneshot::channel::<()>();
        tokio::spawn(async move {
            axum::serve(
                listener,
                router(AppState::new(config, TOKEN.as_bytes().to_vec())),
            )
            .with_graceful_shutdown(async {
                let _ = stopped.await;
            })
            .await
            .unwrap();
        });
        Self { url, _stop: stop }
    }
}

/// One run; the answer's JSON whatever the status.
pub async fn run(d: &Daemon, body: serde_json::Value) -> serde_json::Value {
    reqwest::Client::new()
        .post(format!("{}/run", d.url))
        .header("authorization", format!("Bearer {TOKEN}"))
        .json(&body)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}
