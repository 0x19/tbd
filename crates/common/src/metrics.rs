//! Prometheus metrics: one exporter per process, a global `service` label,
//! process metrics, and the metric names every service uses.
//!
//! Services record through the `metrics` facade macros. With no exporter
//! installed the macros are no-ops, which is what the chaos tool relies on
//! when it runs several services in one process.

use std::{net::SocketAddr, time::Duration};

use metrics_exporter_prometheus::{Matcher, PrometheusBuilder};

/// Metric names. Shared so dashboards and alerts can rely on them.
pub mod names {
    /// Counter: requests handled. Labels `transport`, `route`, `status`.
    pub const REQUESTS_TOTAL: &str = "tbd_requests_total";
    /// Histogram (seconds): time to answer, or to first response on streams. Labels `transport`, `route`.
    pub const REQUEST_DURATION: &str = "tbd_request_duration_seconds";
    /// Gauge: requests currently being handled. Label `transport`.
    pub const REQUESTS_IN_FLIGHT: &str = "tbd_requests_in_flight";
    /// Gauge: open streams. Label `kind` (`subscribe`, `session`, `ws`, `sse`).
    pub const STREAMS_ACTIVE: &str = "tbd_streams_active";
    /// Counter: items on streams. Labels `kind`, `direction` (`in`, `out`).
    pub const STREAM_ITEMS_TOTAL: &str = "tbd_stream_items_total";
    /// Counter: calls from a service to the engine. Labels `route`, `status`.
    pub const ENGINE_CLIENT_REQUESTS_TOTAL: &str = "tbd_engine_client_requests_total";
    /// Histogram (seconds): engine call latency from the caller's side. Label `route`.
    pub const ENGINE_CLIENT_DURATION: &str = "tbd_engine_client_duration_seconds";
    /// Counter: faults injected by the fault handle. Label `kind`.
    pub const FAULTS_INJECTED_TOTAL: &str = "tbd_faults_injected_total";
    /// Gauge, always 1. Labels `version`.
    pub const BUILD_INFO: &str = "tbd_build_info";
    /// Gauge: 1 while the ledger's store answers its readiness probe.
    pub const LEDGER_STORE_UP: &str = "tbd_ledger_store_up";
    /// Counter: outbox batches handed to the sink. Label `status` (`ok`, `error`).
    pub const LEDGER_OUTBOX_BATCHES_TOTAL: &str = "tbd_ledger_outbox_batches_total";
    /// Counter: outbox events shipped. Label `kind`.
    pub const LEDGER_OUTBOX_EVENTS_TOTAL: &str = "tbd_ledger_outbox_events_total";
    /// Counter: erasures the sweeper executed.
    pub const LEDGER_ERASURES_EXECUTED_TOTAL: &str = "tbd_ledger_erasures_executed_total";
    /// Gauge: pooled database connections. Label `state` (`idle`, `in_use`).
    pub const DB_POOL_CONNECTIONS: &str = "tbd_db_pool_connections";
}

/// Errors from installing the exporter.
#[derive(Debug, thiserror::Error)]
pub enum MetricsError {
    /// The exporter could not bind or a recorder was already installed.
    #[error("metrics exporter: {0}")]
    Install(String),
}

/// Latency buckets in seconds, 0.5 ms to 30 s. Chosen for a request path
/// that is normally single-digit milliseconds but must show a hung backend.
pub const DURATION_BUCKETS: &[f64] = &[
    0.0005, 0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0, 30.0,
];

/// Install the Prometheus exporter on `addr` serving `/metrics`, tag every
/// metric with `service`, describe the shared metrics and start process
/// metrics collection. Call once per process, inside a tokio runtime.
pub fn install(addr: SocketAddr, service: &str) -> Result<(), MetricsError> {
    PrometheusBuilder::new()
        .with_http_listener(addr)
        .add_global_label("service", service)
        .set_buckets_for_metric(
            Matcher::Suffix("_duration_seconds".into()),
            DURATION_BUCKETS,
        )
        .map_err(|e| MetricsError::Install(e.to_string()))?
        .install()
        .map_err(|e| MetricsError::Install(e.to_string()))?;

    describe();
    metrics::gauge!(names::BUILD_INFO, "version" => crate::VERSION).set(1.0);

    let collector = metrics_process::Collector::default();
    collector.describe();
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_secs(5));
        loop {
            tick.tick().await;
            collector.collect();
        }
    });
    tracing::info!(%addr, "metrics exporter on /metrics");
    Ok(())
}

fn describe() {
    use metrics::{Unit, describe_counter, describe_gauge, describe_histogram};
    describe_counter!(
        names::REQUESTS_TOTAL,
        "Requests handled, by transport, route and status."
    );
    describe_histogram!(
        names::REQUEST_DURATION,
        Unit::Seconds,
        "Time to answer, or to first response on streams."
    );
    describe_gauge!(
        names::REQUESTS_IN_FLIGHT,
        "Requests currently being handled."
    );
    describe_gauge!(names::STREAMS_ACTIVE, "Open streams by kind.");
    describe_counter!(
        names::STREAM_ITEMS_TOTAL,
        "Items sent or received on streams."
    );
    describe_counter!(
        names::ENGINE_CLIENT_REQUESTS_TOTAL,
        "Calls to the engine from this service."
    );
    describe_histogram!(
        names::ENGINE_CLIENT_DURATION,
        Unit::Seconds,
        "Engine call latency from the caller's side."
    );
    describe_counter!(
        names::FAULTS_INJECTED_TOTAL,
        "Faults injected through the fault handle."
    );
    describe_gauge!(names::BUILD_INFO, "Always 1; carries the version label.");
    describe_gauge!(
        names::LEDGER_STORE_UP,
        "1 while the ledger's store answers its readiness probe."
    );
    describe_counter!(
        names::LEDGER_OUTBOX_BATCHES_TOTAL,
        "Outbox batches handed to the analytics sink, by status."
    );
    describe_counter!(
        names::LEDGER_OUTBOX_EVENTS_TOTAL,
        "Outbox events shipped to the analytics sink, by kind."
    );
    describe_counter!(
        names::LEDGER_ERASURES_EXECUTED_TOTAL,
        "Erasures the ledger's sweeper executed."
    );
    describe_gauge!(
        names::DB_POOL_CONNECTIONS,
        "Pooled database connections, by state."
    );
}

/// Records one request's outcome and duration on drop, and keeps the
/// in-flight gauge honest even on early returns.
pub struct RequestTimer {
    transport: &'static str,
    route: String,
    started: std::time::Instant,
    status: String,
}

impl RequestTimer {
    /// Start timing. Increments the in-flight gauge.
    pub fn start(transport: &'static str, route: impl Into<String>) -> Self {
        metrics::gauge!(names::REQUESTS_IN_FLIGHT, "transport" => transport).increment(1.0);
        Self {
            transport,
            route: route.into(),
            started: std::time::Instant::now(),
            status: "ok".into(),
        }
    }

    /// Set the status label recorded on drop (`ok`, an HTTP status, a gRPC code).
    pub fn set_status(&mut self, status: impl Into<String>) {
        self.status = status.into();
    }
}

impl Drop for RequestTimer {
    fn drop(&mut self) {
        metrics::gauge!(names::REQUESTS_IN_FLIGHT, "transport" => self.transport).decrement(1.0);
        metrics::counter!(
            names::REQUESTS_TOTAL,
            "transport" => self.transport,
            "route" => self.route.clone(),
            "status" => self.status.clone()
        )
        .increment(1);
        metrics::histogram!(names::REQUEST_DURATION, "transport" => self.transport, "route" => self.route.clone())
            .record(self.started.elapsed().as_secs_f64());
    }
}

/// Increments a stream gauge now and decrements it on drop.
pub struct StreamGuard {
    kind: &'static str,
}

impl StreamGuard {
    /// Count a stream of `kind` as open.
    pub fn open(kind: &'static str) -> Self {
        metrics::gauge!(names::STREAMS_ACTIVE, "kind" => kind).increment(1.0);
        Self { kind }
    }

    /// Count one item on this stream.
    pub fn item(&self, direction: &'static str) {
        metrics::counter!(names::STREAM_ITEMS_TOTAL, "kind" => self.kind, "direction" => direction)
            .increment(1);
    }
}

impl Drop for StreamGuard {
    fn drop(&mut self) {
        metrics::gauge!(names::STREAMS_ACTIVE, "kind" => self.kind).decrement(1.0);
    }
}
