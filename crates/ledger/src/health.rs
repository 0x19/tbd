//! Readiness on the store: the gRPC health service says SERVING only while a
//! probe of the store succeeds within its timeout.

use std::{sync::Arc, time::Duration};

use tbd_common::metrics::names;
use tbd_proto::ledger::v1::ledger_service_server::LedgerServiceServer;
use tokio_util::sync::CancellationToken;
use tonic_health::server::HealthReporter;

use crate::{Ledger, store::Store};

/// One probe: `Ok` when the store answered in time.
pub async fn probe_once(store: &Arc<dyn Store>, timeout: Duration) -> Result<(), String> {
    match tokio::time::timeout(timeout, store.ping()).await {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(e.to_string()),
        Err(_) => Err(format!("no answer within {timeout:?}")),
    }
}

/// Report the probe's outcome on the health service and the gauge.
pub async fn report(reporter: &HealthReporter, up: bool) {
    metrics::gauge!(names::LEDGER_STORE_UP).set(if up { 1.0 } else { 0.0 });
    if up {
        reporter.set_serving::<LedgerServiceServer<Ledger>>().await;
    } else {
        reporter
            .set_not_serving::<LedgerServiceServer<Ledger>>()
            .await;
    }
}

/// Probe every `interval` until cancelled, logging transitions.
pub async fn run(
    store: Arc<dyn Store>,
    reporter: HealthReporter,
    interval: Duration,
    timeout: Duration,
    initially_up: bool,
    cancel: CancellationToken,
) {
    let mut up = initially_up;
    loop {
        tokio::select! {
            () = cancel.cancelled() => break,
            () = tokio::time::sleep(interval) => {}
        }
        let now = probe_once(&store, timeout).await;
        match (&now, up) {
            (Ok(()), false) => tracing::info!(store = %store.kind(), "store reachable; serving"),
            (Err(error), true) => {
                tracing::warn!(store = %store.kind(), %error, "store unreachable; not serving");
            }
            _ => {}
        }
        up = now.is_ok();
        report(&reporter, up).await;
    }
}
