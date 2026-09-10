//! Shared state: the engine client. Cloning is cheap; tonic channels are
//! reference-counted and multiplex requests.

use tbd_proto::engine::v1::engine_service_client::EngineServiceClient;
use tonic::transport::{Channel, Endpoint};
use tonic_health::pb::{HealthCheckRequest, health_client::HealthClient};

use crate::{Config, ServeError};

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
    pub fn engine(&self) -> EngineServiceClient<Channel> {
        EngineServiceClient::new(self.channel.clone())
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
