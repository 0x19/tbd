//! The runner kind: an in-process `tbd-runner` with fault injection.
//! Rendered by `tbd new service`; edit freely, the CLI never rewrites it.

use std::net::SocketAddr;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tbd_common::fault::Behavior;
use tbd_proto::runner::v1::{PingRequest, runner_service_client::RunnerServiceClient};
use tbd_runner::{Config, Runtime};
use tonic::transport::Endpoint;
use tonic_health::pb::{HealthCheckRequest, health_client::HealthClient};

use super::{Kind, Target};
use crate::{
    service::{Instance, InstanceHandle, Peers, RequestCounts, Service, TaskHandle},
    validate::{Check, Endpoint as Ep},
};

/// The registry entry.
pub static KIND: Kind = Kind {
    name: "runner",
    label: "Runner",
    plural: "runners",
    surface: "grpc",
    target: Some(Target {
        help: "Runner gRPC URL (through Envoy: the engine LB, matched by service name)",
        default_url: "http://127.0.0.1:50060",
    }),
    fields: &[],
    fault: true,
    store_fault: false,
    counters: true,
    load_target: false,
    addable: true,
    parse: super::parse::<Runner>,
    checks: &[
        Check {
            name: "grpc_runner_ping",
            surface: "grpc",
            doc: "`Ping` echoes the message and is labelled a stub",
            run: |e| Box::pin(grpc_runner_ping(e)),
        },
        Check {
            name: "grpc_runner_unauthenticated",
            surface: "grpc",
            doc: "`Run` without a verified caller is UNAUTHENTICATED before anything runs: code runs only for someone the gateway verified",
            run: |e| Box::pin(grpc_runner_unauthenticated(e)),
        },
    ],
};

/// `[stack.runners.<name>]` minus `listen`: a runner with an initial behaviour.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields, default)]
pub struct Runner {
    /// Initial fault behaviour.
    pub behavior: Behavior,
}

struct RunnerHandle {
    addr: SocketAddr,
    runtime: Runtime,
    task: TaskHandle,
}

#[async_trait]
impl InstanceHandle for RunnerHandle {
    async fn ready(&self) -> bool {
        let Ok(channel) = Endpoint::from_shared(format!("http://{}", self.addr)) else {
            return false;
        };
        let Ok(channel) = channel.connect().await else {
            return false;
        };
        let mut health = HealthClient::new(channel);
        let req = HealthCheckRequest {
            service: "tbd.runner.v1.RunnerService".to_owned(),
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
impl Service for Runner {
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
        // The in-process stub engine: a chaos stack never runs real code.
        let config = Config::stub(addr);
        let runtime = Runtime {
            fault: tbd_common::fault::FaultHandle::new(self.behavior.clone()),
            ..Default::default()
        };
        let (stop, stopped) = tokio::sync::oneshot::channel();
        let rt = runtime.clone();
        let instance_name = name.to_owned();
        let task = tokio::spawn(async move {
            if let Err(error) = tbd_runner::serve_with(listener, config, rt, async {
                let _ = stopped.await;
            })
            .await
            {
                tracing::error!(instance = %instance_name, %error, "runner exited with error");
            }
        });
        Ok(Instance::new(
            name,
            self.kind(),
            addr,
            RunnerHandle {
                addr,
                runtime,
                task: TaskHandle::new(stop, task),
            },
        ))
    }
}

/// `RunnerService/Ping` echoes and is labelled a stub. Not a named health
/// check: through Envoy's internal listener a health request lands on the
/// engine, and a check whose result depends on the path is worse than none.
async fn grpc_runner_ping(e: Ep) -> Result<String, String> {
    let mut c = RunnerServiceClient::new(e.grpc()?);
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

/// `RunnerService/Run` with no `x-jwt-payload` is refused as unauthenticated,
/// before any engine is asked.
async fn grpc_runner_unauthenticated(e: Ep) -> Result<String, String> {
    use tbd_proto::runner::v1::{Language, RunRequest};
    let mut c = RunnerServiceClient::new(e.grpc()?);
    let req = RunRequest {
        language: Language::Go as i32,
        source: "package main\nfunc main(){}\n".into(),
        stdin: String::new(),
    };
    match c.run(req).await {
        Err(s) if s.code() == tonic::Code::Unauthenticated => Ok("unauthenticated".into()),
        Err(s) => Err(format!("wrong code {:?}: {}", s.code(), s.message())),
        Ok(_) => Err("ran without a verified caller".into()),
    }
}
