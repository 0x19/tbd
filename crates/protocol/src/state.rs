//! Shared state: the engine client. Cloning is cheap; tonic channels are
//! reference-counted and multiplex requests.
//!
//! Every call to the engine carries the current trace context in
//! `traceparent` metadata and is measured client-side.

use std::{
    task::{Context, Poll},
    time::Instant,
};

use tbd_common::{metrics::names, telemetry::propagation};
use tbd_proto::engine::v1::engine_service_client::EngineServiceClient;
use tonic::{
    codegen::InterceptedService,
    service::Interceptor,
    transport::{Channel, Endpoint},
};
use tonic_health::pb::{HealthCheckRequest, health_client::HealthClient};

use crate::{Config, ServeError};

/// The engine client type: channel, client metrics, trace propagation.
pub type EngineClient = EngineServiceClient<InterceptedService<Measured, TraceInject>>;

/// Everything a handler needs.
#[derive(Debug, Clone)]
pub struct AppState {
    channel: Channel,
}

impl AppState {
    /// Create a lazily-connecting channel to the engine. The first RPC
    /// connects; failures surface per request, so the protocol starts even if
    /// the engine is not up yet.
    pub fn connect_lazy(config: &Config) -> Result<Self, ServeError> {
        let channel = Endpoint::from_shared(config.engine_url.clone())
            .map_err(|source| ServeError::EngineUrl {
                url: config.engine_url.clone(),
                source,
            })?
            .connect_lazy();
        Ok(Self { channel })
    }

    /// A fresh engine client over the shared channel.
    pub fn engine(&self) -> EngineClient {
        EngineServiceClient::with_interceptor(
            Measured {
                inner: self.channel.clone(),
            },
            TraceInject,
        )
    }

    /// True when the engine reports `SERVING` for its Engine service.
    pub async fn engine_ready(&self) -> bool {
        let mut health = HealthClient::new(self.channel.clone());
        let req = HealthCheckRequest {
            service: "tbd.engine.v1.EngineService".to_owned(),
        };
        match health.check(req).await {
            Ok(resp) => {
                resp.into_inner().status
                    == tonic_health::pb::health_check_response::ServingStatus::Serving as i32
            }
            Err(status) => {
                tracing::debug!(%status, "engine health check failed");
                false
            }
        }
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

/// A tower service wrapping the channel that records one client-side sample
/// per call: route from the request path, status from the `grpc-status`
/// response header when the engine fails immediately, `ok` otherwise.
#[derive(Debug, Clone)]
pub struct Measured {
    inner: Channel,
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
            metrics::counter!(names::ENGINE_CLIENT_REQUESTS_TOTAL, "route" => route.clone(), "status" => status).increment(1);
            metrics::histogram!(names::ENGINE_CLIENT_DURATION, "route" => route)
                .record(started.elapsed().as_secs_f64());
            result
        })
    }
}
