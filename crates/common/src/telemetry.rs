//! Logs and traces, configured from CLI flags or environment.
//!
//! - Logs go to stdout as text or JSON through `tracing-subscriber`.
//! - Traces are exported to an OTLP/gRPC endpoint when
//!   `OTEL_EXPORTER_OTLP_ENDPOINT` is set. Without it the tracer still runs, so
//!   every request gets a trace id and context propagates between services; it
//!   just goes nowhere.
//! - Every request span carries a `trace_id` field, so JSON log lines can be
//!   joined to traces in the backend.
//!
//! Call [`init`] once, first thing in `main`, and keep the returned
//! [`Telemetry`] alive; drop or call [`Telemetry::shutdown`] on exit to flush.

use std::time::Duration;

use clap::{Args, ValueEnum};
use opentelemetry::{KeyValue, trace::TracerProvider as _};
use opentelemetry_otlp::WithExportConfig as _;
use opentelemetry_sdk::{Resource, trace::SdkTracerProvider};
use tracing_subscriber::{EnvFilter, Layer, fmt, layer::SubscriberExt, util::SubscriberInitExt};

/// Output format for logs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum LogFormat {
    /// Human-readable, for terminals.
    Text,
    /// One JSON object per line, for containers and log shippers.
    Json,
}

/// Telemetry flags, `#[command(flatten)]`-ed into each service's config.
/// Env names are the OpenTelemetry standard ones where one exists.
#[derive(Debug, Clone, Args)]
pub struct TelemetryArgs {
    /// Log output format.
    #[arg(
        long = "log-format",
        env = "LOG_FORMAT",
        default_value = "text",
        value_enum,
        global = true
    )]
    pub format: LogFormat,

    /// `tracing` filter directive, e.g. `info,tbd=debug`.
    #[arg(
        long = "log-filter",
        env = "RUST_LOG",
        default_value = "info",
        global = true
    )]
    pub filter: String,

    /// OTLP/gRPC endpoint for traces, e.g. `http://alloy:4317`. Unset disables export.
    #[arg(
        long = "otlp-endpoint",
        env = "OTEL_EXPORTER_OTLP_ENDPOINT",
        global = true
    )]
    pub otlp_endpoint: Option<String>,

    /// `service.name` resource attribute. Defaults to the binary's name.
    #[arg(long = "service-name", env = "OTEL_SERVICE_NAME", global = true)]
    pub service_name: Option<String>,

    /// Fraction of traces to sample at the root, `0.0..=1.0`.
    #[arg(
        long = "trace-sample-ratio",
        env = "OTEL_TRACES_SAMPLER_ARG",
        default_value_t = 1.0,
        global = true
    )]
    pub sample_ratio: f64,

    /// Pyroscope server for in-process CPU profiles, e.g. `http://pyroscope:4040`. Unset disables.
    #[arg(
        long = "pyroscope-server",
        env = "PYROSCOPE_SERVER_ADDRESS",
        global = true
    )]
    pub pyroscope_server: Option<String>,
}

impl Default for TelemetryArgs {
    fn default() -> Self {
        Self {
            format: LogFormat::Text,
            filter: "info".into(),
            otlp_endpoint: None,
            service_name: None,
            sample_ratio: 1.0,
            pyroscope_server: None,
        }
    }
}

/// Errors from telemetry setup.
#[derive(Debug, thiserror::Error)]
pub enum TelemetryError {
    /// The filter directive did not parse.
    #[error("invalid log filter {0:?}: {1}")]
    Filter(String, tracing_subscriber::filter::ParseError),
    /// A global subscriber was already installed.
    #[error("tracing subscriber already set: {0}")]
    AlreadySet(#[from] tracing_subscriber::util::TryInitError),
    /// The OTLP exporter could not be built.
    #[error("otlp exporter: {0}")]
    Exporter(String),
}

/// Handle to flush on exit.
#[derive(Debug)]
pub struct Telemetry {
    provider: Option<SdkTracerProvider>,
    /// Effective service name.
    pub service_name: String,
}

impl Telemetry {
    /// Flush pending spans. Safe to call once; later calls are no-ops.
    pub fn shutdown(&mut self) {
        if let Some(provider) = self.provider.take()
            && let Err(error) = provider.shutdown()
        {
            tracing::warn!(%error, "tracer provider shutdown");
        }
    }
}

impl Drop for Telemetry {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// Install the global `tracing` subscriber and the tracer. Spans get trace
/// ids and propagate whether or not an OTLP endpoint is configured; export
/// happens only when it is.
///
/// `default_service_name` is used when neither the flag nor `OTEL_SERVICE_NAME`
/// is set; pass the binary name.
pub fn init(args: &TelemetryArgs, default_service_name: &str) -> Result<Telemetry, TelemetryError> {
    let service_name = args
        .service_name
        .clone()
        .unwrap_or_else(|| default_service_name.to_owned());
    let filter = EnvFilter::try_new(&args.filter)
        .map_err(|e| TelemetryError::Filter(args.filter.clone(), e))?;

    let provider = tracer_provider(
        args.otlp_endpoint.as_deref(),
        &service_name,
        args.sample_ratio,
    )?;
    opentelemetry::global::set_text_map_propagator(
        opentelemetry_sdk::propagation::TraceContextPropagator::new(),
    );
    let otel_layer =
        tracing_opentelemetry::layer().with_tracer(provider.tracer(service_name.clone()));

    let registry = tracing_subscriber::registry().with(filter).with(otel_layer);
    match args.format {
        LogFormat::Text => registry.with(fmt::layer().with_target(true)).try_init()?,
        LogFormat::Json => registry
            .with(
                fmt::layer()
                    .json()
                    .flatten_event(true)
                    .with_current_span(true)
                    .with_span_list(false)
                    .boxed(),
            )
            .try_init()?,
    }

    if let Some(endpoint) = &args.otlp_endpoint {
        tracing::info!(service = %service_name, %endpoint, ratio = args.sample_ratio, "otlp trace export on");
    }
    Ok(Telemetry {
        provider: Some(provider),
        service_name,
    })
}

fn tracer_provider(
    endpoint: Option<&str>,
    service_name: &str,
    ratio: f64,
) -> Result<SdkTracerProvider, TelemetryError> {
    use opentelemetry_sdk::trace::Sampler;
    let resource = Resource::builder()
        .with_service_name(service_name.to_owned())
        .with_attributes([KeyValue::new(
            opentelemetry_semantic_conventions::resource::SERVICE_VERSION,
            crate::VERSION,
        )])
        .build();
    let sampler = if ratio >= 1.0 {
        Sampler::AlwaysOn
    } else {
        Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(ratio.max(0.0))))
    };
    let builder = SdkTracerProvider::builder()
        .with_resource(resource)
        .with_sampler(sampler);
    let builder = match endpoint {
        Some(endpoint) => {
            let exporter = opentelemetry_otlp::SpanExporter::builder()
                .with_tonic()
                .with_endpoint(endpoint)
                .with_timeout(Duration::from_secs(5))
                .build()
                .map_err(|e| TelemetryError::Exporter(e.to_string()))?;
            builder.with_batch_exporter(exporter)
        }
        None => builder,
    };
    Ok(builder.build())
}

/// The current span's trace id as a hex string, or `None` when tracing is
/// not exporting or the span is not sampled. Record it on request spans so it
/// lands in JSON logs.
pub fn current_trace_id() -> Option<String> {
    use opentelemetry::trace::TraceContextExt as _;
    use tracing_opentelemetry::OpenTelemetrySpanExt as _;
    let context = tracing::Span::current().context();
    let span = context.span();
    let sc = span.span_context();
    sc.is_valid().then(|| sc.trace_id().to_string())
}

/// W3C `traceparent` propagation over any string map: HTTP headers, gRPC
/// metadata. Services implement the two traits for their header types.
pub mod propagation {
    pub use opentelemetry::propagation::{Extractor, Injector};

    /// Inject the current span's context into `carrier`.
    pub fn inject(carrier: &mut dyn Injector) {
        use tracing_opentelemetry::OpenTelemetrySpanExt as _;
        let context = tracing::Span::current().context();
        opentelemetry::global::get_text_map_propagator(|p| p.inject_context(&context, carrier));
    }

    /// Make `span` a child of the context found in `carrier`, if any, and
    /// return the trace id to record.
    pub fn adopt_parent(span: &tracing::Span, carrier: &dyn Extractor) -> Option<String> {
        use opentelemetry::trace::TraceContextExt as _;
        use tracing_opentelemetry::OpenTelemetrySpanExt as _;
        let parent = opentelemetry::global::get_text_map_propagator(|p| p.extract(carrier));
        if parent.span().span_context().is_valid()
            && let Err(error) = span.set_parent(parent)
        {
            tracing::debug!(%error, "could not attach remote trace parent");
        }
        let context = span.context();
        let sc = context.span().span_context().clone();
        sc.is_valid().then(|| sc.trace_id().to_string())
    }
}
