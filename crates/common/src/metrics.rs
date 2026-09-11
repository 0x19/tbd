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
    // Labels: `backend` (the protocol's registry name), `route`, `status`.
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
    /// Gauge: the pool's configured maximum, so saturation is a ratio.
    pub const DB_POOL_MAX_CONNECTIONS: &str = "tbd_db_pool_max_connections";
    /// Histogram (seconds) of every store operation, by `op`.
    pub const LEDGER_STORE_OP_DURATION: &str = "tbd_ledger_store_op_duration_seconds";
    /// Counter of store operations by `op` and `result` (`ok` or the error kind).
    pub const LEDGER_STORE_OPS_TOTAL: &str = "tbd_ledger_store_ops_total";
    /// Counter of facts written, by `source`; replays are not facts.
    pub const LEDGER_FACTS_APPENDED_TOTAL: &str = "tbd_ledger_facts_appended_total";
    /// Counter of appends answered from the idempotency table.
    pub const LEDGER_APPENDS_REPLAYED_TOTAL: &str = "tbd_ledger_appends_replayed_total";
    /// Counter of retractions (a tombstone written, the values gone).
    pub const LEDGER_FACTS_RETRACTED_TOTAL: &str = "tbd_ledger_facts_retracted_total";
    /// Histogram (bytes) of accepted envelopes, by `field` (`value`, `origin`).
    pub const LEDGER_ENVELOPE_BYTES: &str = "tbd_ledger_envelope_bytes";
    /// Histogram of facts per page returned, by `op` (`current`, `history`).
    pub const LEDGER_PAGE_FACTS: &str = "tbd_ledger_page_facts";
    /// Counter of erasure requests.
    pub const LEDGER_ERASURES_REQUESTED_TOTAL: &str = "tbd_ledger_erasures_requested_total";
    /// Counter of erasures cancelled by a restore inside the window.
    pub const LEDGER_ERASURES_RESTORED_TOTAL: &str = "tbd_ledger_erasures_restored_total";
    /// Counter of tombstones written on surviving subjects by erasure cascades.
    pub const LEDGER_ERASURE_TOMBSTONES_TOTAL: &str = "tbd_ledger_erasure_tombstones_total";
    /// Gauge of erasures waiting, by `state` (`pending`, `due`).
    pub const LEDGER_ERASURES_PENDING: &str = "tbd_ledger_erasures_pending";
    /// Counter of idempotency rows purged after their TTL.
    pub const LEDGER_IDEMPOTENCY_PURGED_TOTAL: &str = "tbd_ledger_idempotency_purged_total";
    /// Gauge of outbox events not yet published.
    pub const LEDGER_OUTBOX_PENDING: &str = "tbd_ledger_outbox_pending";
    /// Gauge (seconds): age of the oldest unpublished outbox event.
    pub const LEDGER_OUTBOX_OLDEST_SECONDS: &str = "tbd_ledger_outbox_oldest_seconds";
    /// Histogram (seconds) of one batch handed to the analytics sink.
    pub const LEDGER_OUTBOX_PUBLISH_DURATION: &str = "tbd_ledger_outbox_publish_duration_seconds";
    /// Histogram (seconds): recorded-to-acked age of the oldest event in each batch.
    pub const LEDGER_OUTBOX_LAG_SECONDS: &str = "tbd_ledger_outbox_lag_seconds";
    /// Counter of subjects deleted from the analytics store after an erasure.
    pub const LEDGER_ANALYTICS_DELETES_TOTAL: &str = "tbd_ledger_analytics_deletes_total";
    /// Gauge: estimated rows per ledger table, by `table`.
    pub const LEDGER_TABLE_ROWS: &str = "tbd_ledger_table_rows";
    /// Gauge (bytes): on-disk size per ledger table with its indexes, by `table`.
    pub const LEDGER_TABLE_BYTES: &str = "tbd_ledger_table_bytes";
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

/// Lag buckets in seconds, 10 ms to an hour: an outbox that is healthy drains
/// in under a second and one that is stuck must still be visible.
pub const LAG_BUCKETS: &[f64] = &[
    0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0, 30.0, 60.0, 300.0, 900.0, 3600.0,
];

/// Size buckets in bytes, 64 B to 64 KiB (the ledger's envelope cap).
pub const BYTES_BUCKETS: &[f64] = &[64.0, 256.0, 1024.0, 4096.0, 16384.0, 65536.0];

/// Count buckets, 1 to 1000 (the ledger's page cap).
pub const COUNT_BUCKETS: &[f64] = &[1.0, 2.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0];

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
        .set_buckets_for_metric(
            Matcher::Full(names::LEDGER_OUTBOX_LAG_SECONDS.into()),
            LAG_BUCKETS,
        )
        .map_err(|e| MetricsError::Install(e.to_string()))?
        .set_buckets_for_metric(
            Matcher::Full(names::LEDGER_ENVELOPE_BYTES.into()),
            BYTES_BUCKETS,
        )
        .map_err(|e| MetricsError::Install(e.to_string()))?
        .set_buckets_for_metric(
            Matcher::Full(names::LEDGER_PAGE_FACTS.into()),
            COUNT_BUCKETS,
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
    describe_ledger();
}

/// The ledger's metrics, described apart so each list stays readable.
fn describe_ledger() {
    use metrics::{Unit, describe_counter, describe_gauge, describe_histogram};
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
    describe_gauge!(
        names::DB_POOL_MAX_CONNECTIONS,
        "The pool's configured maximum."
    );
    describe_histogram!(
        names::LEDGER_STORE_OP_DURATION,
        Unit::Seconds,
        "Time inside the ledger's store, by operation."
    );
    describe_counter!(
        names::LEDGER_STORE_OPS_TOTAL,
        "Store operations, by operation and result."
    );
    describe_counter!(
        names::LEDGER_FACTS_APPENDED_TOTAL,
        "Facts written, by source; replays are not counted."
    );
    describe_counter!(
        names::LEDGER_APPENDS_REPLAYED_TOTAL,
        "Appends answered from the idempotency table."
    );
    describe_counter!(
        names::LEDGER_FACTS_RETRACTED_TOTAL,
        "Retractions: a tombstone written, the values deleted."
    );
    describe_histogram!(
        names::LEDGER_ENVELOPE_BYTES,
        Unit::Bytes,
        "Accepted envelope sizes, by field."
    );
    describe_histogram!(
        names::LEDGER_PAGE_FACTS,
        "Facts per page returned, by operation."
    );
    describe_counter!(names::LEDGER_ERASURES_REQUESTED_TOTAL, "Erasure requests.");
    describe_counter!(
        names::LEDGER_ERASURES_RESTORED_TOTAL,
        "Erasures cancelled by a restore inside the grace window."
    );
    describe_counter!(
        names::LEDGER_ERASURE_TOMBSTONES_TOTAL,
        "Tombstones written on surviving subjects by erasure cascades."
    );
    describe_gauge!(
        names::LEDGER_ERASURES_PENDING,
        "Erasures inside their grace window (pending) and past it (due)."
    );
    describe_counter!(
        names::LEDGER_IDEMPOTENCY_PURGED_TOTAL,
        "Idempotency rows purged after their TTL."
    );
    describe_ledger_outbox();
}

/// The ledger's outbox, analytics and table metrics.
fn describe_ledger_outbox() {
    use metrics::{Unit, describe_counter, describe_gauge, describe_histogram};
    describe_gauge!(
        names::LEDGER_OUTBOX_PENDING,
        "Outbox events not yet published."
    );
    describe_gauge!(
        names::LEDGER_OUTBOX_OLDEST_SECONDS,
        Unit::Seconds,
        "Age of the oldest unpublished outbox event."
    );
    describe_histogram!(
        names::LEDGER_OUTBOX_PUBLISH_DURATION,
        Unit::Seconds,
        "Time to hand one outbox batch to the analytics sink."
    );
    describe_histogram!(
        names::LEDGER_OUTBOX_LAG_SECONDS,
        Unit::Seconds,
        "Recorded-to-acked age of the oldest event in each batch."
    );
    describe_counter!(
        names::LEDGER_ANALYTICS_DELETES_TOTAL,
        "Subjects deleted from the analytics store after an erasure."
    );
    describe_gauge!(names::LEDGER_TABLE_ROWS, "Estimated rows per ledger table.");
    describe_gauge!(
        names::LEDGER_TABLE_BYTES,
        Unit::Bytes,
        "On-disk size per ledger table, indexes included."
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
