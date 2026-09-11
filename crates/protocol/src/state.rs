//! Shared state: the registry of backends the gateway forwards to. One lazy,
//! reference-counted tonic channel per distinct URL (behind Envoy every
//! backend is the same internal listener, routed by gRPC service name), each
//! call carrying the current trace context in `traceparent` metadata and
//! measured client-side with the backend's name as a label.

use std::{
    collections::BTreeMap,
    sync::Arc,
    task::{Context, Poll},
    time::{Duration, Instant},
};

use serde::Serialize;
use tbd_common::{metrics::names, telemetry::propagation};
use tbd_proto::engine::v1::engine_service_client::EngineServiceClient;
use tonic::{
    codegen::InterceptedService,
    service::Interceptor,
    transport::{Channel, Endpoint},
};
use tonic_health::pb::{
    HealthCheckRequest, health_check_response::ServingStatus, health_client::HealthClient,
};

use crate::{
    ServeError,
    config::{Config, ENGINE},
};

/// The transport every backend client is built on: channel, client metrics,
/// trace propagation.
pub type Transport = InterceptedService<Measured, TraceInject>;

/// The engine client type.
pub type EngineClient = EngineServiceClient<Transport>;

/// One registered backend.
#[derive(Debug, Clone)]
pub struct Backend {
    name: Arc<str>,
    channel: Channel,
    /// The `grpc.health.v1` service name probed and re-reported for it.
    pub health_name: String,
    /// Whether `/readyz` fails while it is not `SERVING`.
    pub required: bool,
}

impl Backend {
    /// Registry name (`engine`, `ledger`, …).
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The traced, measured transport for a client constructor:
    /// `LedgerServiceClient::new(backend.transport())`.
    #[must_use]
    pub fn transport(&self) -> Transport {
        InterceptedService::new(
            Measured {
                inner: self.channel.clone(),
                backend: Arc::clone(&self.name),
            },
            TraceInject,
        )
    }

    /// The bare channel, for services that need their own wrapping.
    #[must_use]
    pub fn channel(&self) -> Channel {
        self.channel.clone()
    }

    /// One live `grpc.health.v1` check of [`Backend::health_name`], bounded
    /// by `timeout`. A transport error or a timeout is `Unknown`.
    pub async fn check(&self, timeout: Duration) -> ServiceState {
        let mut health = HealthClient::new(self.channel.clone());
        let request = HealthCheckRequest {
            service: self.health_name.clone(),
        };
        match tokio::time::timeout(timeout, health.check(request)).await {
            Ok(Ok(response)) => {
                if response.into_inner().status == ServingStatus::Serving as i32 {
                    ServiceState::Serving
                } else {
                    ServiceState::NotServing
                }
            }
            Ok(Err(status)) => {
                tracing::debug!(backend = %self.name, %status, "health check failed");
                ServiceState::Unknown
            }
            Err(_) => {
                tracing::debug!(backend = %self.name, ?timeout, "health check timed out");
                ServiceState::Unknown
            }
        }
    }
}

/// What a backend's health service last answered, as `/readyz` reports it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceState {
    /// `SERVING`.
    Serving,
    /// `NOT_SERVING`, or the health service does not know the name.
    NotServing,
    /// The backend could not be reached within the probe timeout.
    Unknown,
}

impl ServiceState {
    /// The status to re-report on the protocol's own health service.
    #[must_use]
    pub fn serving_status(self) -> tonic_health::ServingStatus {
        match self {
            Self::Serving => tonic_health::ServingStatus::Serving,
            Self::NotServing | Self::Unknown => tonic_health::ServingStatus::NotServing,
        }
    }
}

/// `GET /readyz`: every backend's live state; `ready` when every required one
/// is serving.
#[derive(Debug, Clone, Serialize)]
pub struct Readiness {
    /// Every required backend is `serving`.
    pub ready: bool,
    /// Per backend, by registry name.
    pub services: BTreeMap<String, ServiceState>,
}

#[derive(Debug)]
struct Services {
    engine: Backend,
    by_name: BTreeMap<String, Backend>,
}

/// Everything a handler needs.
#[derive(Debug, Clone)]
pub struct AppState {
    services: Arc<Services>,
    probe_timeout: Duration,
    probe_interval: Duration,
    service_subs: Arc<[String]>,
}

impl AppState {
    /// Open one lazily-connecting channel per distinct backend URL. The first
    /// RPC connects; failures surface per request, so the protocol starts
    /// even if a backend is not up yet.
    ///
    /// # Errors
    /// A URL does not parse, or the registry has no `engine`.
    pub fn connect_lazy(config: &Config) -> Result<Self, ServeError> {
        let mut channels: BTreeMap<&str, Channel> = BTreeMap::new();
        let mut by_name = BTreeMap::new();
        for (name, service) in &config.services {
            let channel = if let Some(channel) = channels.get(service.url.as_str()) {
                channel.clone()
            } else {
                let channel = Endpoint::from_shared(service.url.clone())
                    .map_err(|source| ServeError::BackendUrl {
                        name: name.clone(),
                        url: service.url.clone(),
                        source,
                    })?
                    .connect_lazy();
                channels.insert(service.url.as_str(), channel.clone());
                channel
            };
            by_name.insert(
                name.clone(),
                Backend {
                    name: Arc::from(name.as_str()),
                    channel,
                    health_name: service.service.clone(),
                    required: service.required,
                },
            );
        }
        let engine = by_name
            .get(ENGINE)
            .cloned()
            .ok_or(ServeError::MissingBackend(ENGINE))?;
        Ok(Self {
            services: Arc::new(Services { engine, by_name }),
            probe_timeout: config.health.probe_timeout,
            probe_interval: config.health.probe_interval,
            service_subs: config.principals.services.clone().into(),
        })
    }

    /// A fresh engine client over the shared channel.
    #[must_use]
    pub fn engine(&self) -> EngineClient {
        EngineServiceClient::new(self.services.engine.transport())
    }

    /// A registered backend by name.
    #[must_use]
    pub fn backend(&self, name: &str) -> Option<&Backend> {
        self.services.by_name.get(name)
    }

    /// Every registered backend, in name order.
    pub fn backends(&self) -> impl Iterator<Item = &Backend> {
        self.services.by_name.values()
    }

    /// A client for a backend: `state.client("ledger", LedgerServiceClient::new)`.
    /// Generated tonic clients share no constructor trait, so it is passed in.
    pub fn client<T>(&self, name: &str, new: impl FnOnce(Transport) -> T) -> Option<T> {
        self.backend(name).map(|b| new(b.transport()))
    }

    /// The probe timeout from `[health]`.
    #[must_use]
    pub fn probe_timeout(&self) -> Duration {
        self.probe_timeout
    }

    /// The probe interval from `[health]`.
    #[must_use]
    pub fn probe_interval(&self) -> Duration {
        self.probe_interval
    }

    /// Token subjects that are our own services (`[principals] services`).
    #[must_use]
    pub fn service_subs(&self) -> &[String] {
        &self.service_subs
    }

    /// Live state of every backend, probed concurrently within the probe
    /// timeout; `ready` when every required one is serving.
    pub async fn readiness(&self) -> Readiness {
        let checks = self.backends().map(|b| async move {
            (
                b.name().to_owned(),
                b.required,
                b.check(self.probe_timeout).await,
            )
        });
        let states = futures::future::join_all(checks).await;
        let ready = states
            .iter()
            .all(|(_, required, state)| !required || *state == ServiceState::Serving);
        Readiness {
            ready,
            services: states
                .into_iter()
                .map(|(name, _, state)| (name, state))
                .collect(),
        }
    }

    /// True when the engine reports `SERVING`.
    pub async fn engine_ready(&self) -> bool {
        self.services.engine.check(self.probe_timeout).await == ServiceState::Serving
    }
}

/// Injects the current span's `traceparent` into outgoing gRPC metadata.
#[derive(Debug, Clone, Copy)]
pub struct TraceInject;

struct MetadataInjector<'a>(&'a mut tonic::metadata::MetadataMap);

impl propagation::Injector for MetadataInjector<'_> {
    fn set(&mut self, key: &str, value: String) {
        if let (Ok(key), Ok(value)) = (
            tonic::metadata::MetadataKey::from_bytes(key.as_bytes()),
            value.parse::<tonic::metadata::MetadataValue<_>>(),
        ) {
            self.0.insert(key, value);
        }
    }
}

impl Interceptor for TraceInject {
    fn call(
        &mut self,
        mut request: tonic::Request<()>,
    ) -> Result<tonic::Request<()>, tonic::Status> {
        propagation::inject(&mut MetadataInjector(request.metadata_mut()));
        Ok(request)
    }
}

/// A tower service wrapping a backend's channel that records one client-side
/// sample per call: the backend's name, the route from the request path, and
/// the status from the `grpc-status` response header when the backend fails
/// immediately, `ok` otherwise.
#[derive(Debug, Clone)]
pub struct Measured {
    inner: Channel,
    backend: Arc<str>,
}

impl tower::Service<http::Request<tonic::body::Body>> for Measured {
    type Response = http::Response<tonic::body::Body>;
    type Error = tonic::transport::Error;
    type Future = futures::future::BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        tower::Service::poll_ready(&mut self.inner, cx)
    }

    fn call(&mut self, request: http::Request<tonic::body::Body>) -> Self::Future {
        let route = request.uri().path().trim_start_matches('/').to_owned();
        let backend = self.backend.to_string();
        let started = Instant::now();
        // Take the ready inner service and leave a fresh clone, per tower's
        // contract that `poll_ready` readiness belongs to the next `call`.
        let fresh = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, fresh);
        Box::pin(async move {
            let result = tower::Service::call(&mut inner, request).await;
            let status = match &result {
                Ok(response) => response
                    .headers()
                    .get("grpc-status")
                    .and_then(|v| v.to_str().ok())
                    .map_or_else(
                        || "ok".to_owned(),
                        |code| {
                            if code == "0" {
                                "ok".to_owned()
                            } else {
                                format!("grpc-{code}")
                            }
                        },
                    ),
                Err(_) => "transport".to_owned(),
            };
            metrics::counter!(
                names::ENGINE_CLIENT_REQUESTS_TOTAL,
                "backend" => backend.clone(),
                "route" => route.clone(),
                "status" => status
            )
            .increment(1);
            metrics::histogram!(names::ENGINE_CLIENT_DURATION, "backend" => backend, "route" => route)
                .record(started.elapsed().as_secs_f64());
            result
        })
    }
}
