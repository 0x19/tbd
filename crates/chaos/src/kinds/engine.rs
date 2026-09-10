//! The engine kind: an in-process `tbd-engine` with fault injection.

use std::{net::SocketAddr, time::Duration};

use async_trait::async_trait;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tbd_common::{fault::Behavior, telemetry::TelemetryArgs};
use tbd_engine::{Config, Runtime};
use tbd_proto::engine::v1::{
    EvaluateRequest, SubscribeRequest, engine_service_client::EngineServiceClient,
};
use tonic::transport::Endpoint;
use tonic_health::pb::{HealthCheckRequest, health_client::HealthClient};

use super::{Field, FieldKind, Kind, Target};
use crate::{
    service::{Instance, InstanceHandle, Peers, RequestCounts, Service, TaskHandle},
    validate::{Check, Endpoint as Ep},
};

/// The registry entry.
pub static KIND: Kind = Kind {
    name: "engine",
    label: "Engine",
    plural: "engines",
    surface: "grpc",
    target: Some(Target {
        help: "Engine gRPC URL",
        default_url: "http://127.0.0.1:50051",
    }),
    fields: &[Field {
        name: "heartbeat",
        label: "Heartbeat",
        kind: FieldKind::Duration,
        required: false,
        default: Some("1s"),
    }],
    fault: true,
    counters: true,
    load_target: false,
    addable: true,
    parse: super::parse::<Engine>,
    checks: &[
        Check {
            name: "grpc_engine_health",
            surface: "grpc",
            doc: "the named health check reports `SERVING`",
            run: |e| Box::pin(grpc_engine_health(e)),
        },
        Check {
            name: "grpc_engine_evaluate",
            surface: "grpc",
            doc: "`Evaluate` answers with a stub-labelled response",
            run: |e| Box::pin(grpc_engine_evaluate(e)),
        },
        Check {
            name: "grpc_engine_subscribe",
            surface: "grpc",
            doc: "`Subscribe` streams two events",
            run: |e| Box::pin(grpc_engine_subscribe(e)),
        },
    ],
};

/// `[stack.engines.<name>]` minus `listen`: an engine with an initial
/// behaviour and heartbeat interval.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields, default)]
pub struct Engine {
    /// Initial fault behaviour.
    pub behavior: Behavior,
    /// Heartbeat interval on streams.
    #[serde(with = "humantime_serde")]
    pub heartbeat: Duration,
}

impl Default for Engine {
    fn default() -> Self {
        Self {
            behavior: Behavior::Healthy,
            heartbeat: Duration::from_secs(1),
        }
    }
}

struct EngineHandle {
    addr: SocketAddr,
    runtime: Runtime,
    task: TaskHandle,
}

#[async_trait]
impl InstanceHandle for EngineHandle {
    async fn ready(&self) -> bool {
        let Ok(channel) = Endpoint::from_shared(format!("http://{}", self.addr)) else {
            return false;
        };
        let Ok(channel) = channel.connect().await else {
            return false;
        };
        let mut health = HealthClient::new(channel);
        let req = HealthCheckRequest {
            service: "tbd.engine.v1.EngineService".to_owned(),
        };
        health.check(req).await.is_ok()
    }

    async fn stop(self: Box<Self>) {
        self.task.stop().await;
    }

    fn fault(&self) -> Option<tbd_common::fault::FaultHandle> {
        Some(self.runtime.fault.clone())
    }

    fn requests(&self) -> Option<RequestCounts> {
        let s = self.runtime.stats.snapshot();
        Some(RequestCounts {
            total: s.requests_total,
            failed: s.requests_failed,
        })
    }
}

#[async_trait]
impl Service for Engine {
    fn kind(&self) -> &'static str {
        KIND.name
    }

    async fn start(
        &self,
        name: &str,
        listen: SocketAddr,
        _peers: &Peers<'_>,
    ) -> anyhow::Result<Instance> {
        let listener = tokio::net::TcpListener::bind(listen).await?;
        let addr = listener.local_addr()?;
        let config = Config {
            listen_addr: addr,
            heartbeat_ms: u64::try_from(self.heartbeat.as_millis()).unwrap_or(u64::MAX),
            metrics_addr: None,
            telemetry: TelemetryArgs::default(),
        };
        let runtime = Runtime {
            fault: tbd_common::fault::FaultHandle::new(self.behavior.clone()),
            ..Default::default()
        };
        let (stop, stopped) = tokio::sync::oneshot::channel();
        let rt = runtime.clone();
        let instance_name = name.to_owned();
        let task = tokio::spawn(async move {
            if let Err(error) = tbd_engine::serve_with(listener, config, rt, async {
                let _ = stopped.await;
            })
            .await
            {
                tracing::error!(instance = %instance_name, %error, "engine exited with error");
            }
        });
        Ok(Instance::new(
            name,
            self.kind(),
            addr,
            EngineHandle {
                addr,
                runtime,
                task: TaskHandle::new(stop, task),
            },
        ))
    }
}

async fn grpc_engine_health(e: Ep) -> Result<String, String> {
    let mut h = HealthClient::new(e.grpc()?);
    let resp = h
        .check(HealthCheckRequest {
            service: "tbd.engine.v1.EngineService".into(),
        })
        .await
        .map_err(|e| e.to_string())?
        .into_inner();
    if resp.status == tonic_health::pb::health_check_response::ServingStatus::Serving as i32 {
        Ok("SERVING".into())
    } else {
        Err(format!("status {}", resp.status))
    }
}

async fn grpc_engine_evaluate(e: Ep) -> Result<String, String> {
    let mut c = EngineServiceClient::new(e.grpc()?);
    let r = c
        .evaluate(EvaluateRequest {
            subject_id: "validate".into(),
            payload: b"hi".to_vec(),
        })
        .await
        .map_err(|e| e.to_string())?
        .into_inner();
    Ok(format!("stub={} model={}", r.stub, r.model_version))
}

async fn grpc_engine_subscribe(e: Ep) -> Result<String, String> {
    let mut c = EngineServiceClient::new(e.grpc()?);
    let mut s = c
        .subscribe(SubscribeRequest {
            subject_id: "validate".into(),
        })
        .await
        .map_err(|e| e.to_string())?
        .into_inner();
    let mut n = 0;
    while n < 2 {
        s.next()
            .await
            .ok_or("stream ended")?
            .map_err(|e| e.to_string())?;
        n += 1;
    }
    Ok(format!("{n} events"))
}
