//! The cv kind: an in-process `tbd-cv` with fault injection.
//! Rendered by `tbd new service`; edit freely, the CLI never rewrites it.

use std::net::SocketAddr;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tbd_common::fault::Behavior;
use tbd_cv::{
    Config, Runtime,
    config::{Finance, Metrics, Notify, Ping, Private, Render, Server, Store},
};
use tbd_proto::cv::v1::{GetAccessRequest, PingRequest, cv_service_client::CvServiceClient};
use tonic::transport::Endpoint;
use tonic_health::pb::{HealthCheckRequest, health_client::HealthClient};

use super::{Kind, Target};
use crate::{
    service::{Instance, InstanceHandle, Peers, RequestCounts, Service, TaskHandle},
    validate::{Check, Endpoint as Ep},
};

/// The registry entry.
pub static KIND: Kind = Kind {
    name: "cv",
    label: "Cv",
    plural: "cvs",
    surface: "grpc",
    target: Some(Target {
        help: "Cv gRPC URL (through Envoy: the engine LB, matched by service name)",
        default_url: "http://127.0.0.1:50056",
    }),
    fields: &[],
    fault: true,
    store_fault: false,
    counters: true,
    load_target: false,
    addable: true,
    parse: super::parse::<Cv>,
    checks: &[
        Check {
            name: "grpc_cv_ping",
            surface: "grpc",
            doc: "`Ping` echoes the message and is labelled a stub",
            run: |e| Box::pin(grpc_cv_ping(e)),
        },
        Check {
            name: "grpc_cv_access_unauthenticated",
            surface: "grpc",
            doc: "`GetAccess` without a verified caller is UNAUTHENTICATED: identity comes from Envoy or not at all",
            run: |e| Box::pin(grpc_cv_access_unauthenticated(e)),
        },
    ],
};

/// `[stack.cvs.<name>]` minus `listen`: a cv with an initial behaviour.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields, default)]
pub struct Cv {
    /// Initial fault behaviour.
    pub behavior: Behavior,
}

struct CvHandle {
    addr: SocketAddr,
    runtime: Runtime,
    task: TaskHandle,
}

#[async_trait]
impl InstanceHandle for CvHandle {
    async fn ready(&self) -> bool {
        let Ok(channel) = Endpoint::from_shared(format!("http://{}", self.addr)) else {
            return false;
        };
        let Ok(channel) = channel.connect().await else {
            return false;
        };
        let mut health = HealthClient::new(channel);
        let req = HealthCheckRequest {
            service: "tbd.cv.v1.CvService".to_owned(),
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
impl Service for Cv {
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
            server: Server { listen: addr },
            metrics: Metrics { listen: None },
            ping: Ping::default(),
            store: Store::default(),
            finance: Finance::default(),
            notify: Notify::default(),
            private: Private::default(),
            render: Render::default(),
        };
        let runtime = Runtime {
            fault: tbd_common::fault::FaultHandle::new(self.behavior.clone()),
            ..Default::default()
        };
        let (stop, stopped) = tokio::sync::oneshot::channel();
        let rt = runtime.clone();
        let instance_name = name.to_owned();
        let task = tokio::spawn(async move {
            if let Err(error) = tbd_cv::serve_with(listener, config, rt, async {
                let _ = stopped.await;
            })
            .await
            {
                tracing::error!(instance = %instance_name, %error, "cv exited with error");
            }
        });
        Ok(Instance::new(
            name,
            self.kind(),
            addr,
            CvHandle {
                addr,
                runtime,
                task: TaskHandle::new(stop, task),
            },
        ))
    }
}

/// `CvService/Ping` echoes and is labelled a stub. Not a named health
/// check: through Envoy's internal listener a health request lands on the
/// engine, and a check whose result depends on the path is worse than none.
async fn grpc_cv_ping(e: Ep) -> Result<String, String> {
    let mut c = CvServiceClient::new(e.grpc()?);
    let r = c
        .ping(PingRequest {
            message: "validate".into(),
        })
        .await
        .map_err(|e| e.to_string())?
        .into_inner();
    if r.message == "validate" {
        Ok(format!("version={} stub={}", r.version, r.stub))
    } else {
        Err(format!("wrong echo {r:?}"))
    }
}

/// `CvService/GetAccess` with no `x-jwt-payload` is refused as unauthenticated:
/// a caller is whoever Envoy verified, never a claim in the request.
async fn grpc_cv_access_unauthenticated(e: Ep) -> Result<String, String> {
    let mut c = CvServiceClient::new(e.grpc()?);
    match c.get_access(GetAccessRequest {}).await {
        Err(s) if s.code() == tonic::Code::Unauthenticated => Ok("unauthenticated".into()),
        Err(s) => Err(format!("wrong code {:?}: {}", s.code(), s.message())),
        Ok(_) => Err("answered without a verified caller".into()),
    }
}
