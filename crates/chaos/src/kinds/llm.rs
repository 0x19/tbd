//! The llm kind: an in-process `tbd-llm` with fault injection, both tiers on
//! the stub engine (a chaos stack must never need a model server).
//! Rendered by `tbd new service`; edit freely, the CLI never rewrites it.

use std::net::SocketAddr;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tbd_common::fault::Behavior;
use tbd_llm::{Config, Runtime};
use tbd_proto::llm::v1::{
    GenerateRequest, ListModelsRequest, Message, PingRequest, llm_service_client::LlmServiceClient,
};
use tonic::transport::Endpoint;
use tonic_health::pb::{HealthCheckRequest, health_client::HealthClient};

use super::{Kind, Target};
use crate::{
    service::{Instance, InstanceHandle, Peers, RequestCounts, Service, TaskHandle},
    validate::{Check, Endpoint as Ep},
};

/// The registry entry.
pub static KIND: Kind = Kind {
    name: "llm",
    label: "Llm",
    plural: "llms",
    surface: "grpc",
    target: Some(Target {
        help: "Llm gRPC URL (through Envoy: the engine LB, matched by service name)",
        default_url: "http://127.0.0.1:50057",
    }),
    fields: &[],
    fault: true,
    store_fault: false,
    counters: true,
    load_target: true,
    addable: true,
    parse: super::parse::<Llm>,
    checks: &[
        Check {
            name: "grpc_llm_ping",
            surface: "grpc",
            doc: "`Ping` echoes the message and is labelled a stub",
            run: |e| Box::pin(grpc_llm_ping(e)),
        },
        Check {
            name: "grpc_llm_models_lists_both_tiers",
            surface: "grpc",
            doc: "`ListModels` names the fast and the deep tier, each with its engine and model, and says whether the engine is up (a down engine is reported, not failed)",
            run: |e| Box::pin(grpc_llm_models_lists_both_tiers(e)),
        },
        Check {
            name: "grpc_llm_generate_unauthenticated",
            surface: "grpc",
            doc: "`Generate` without a verified caller is UNAUTHENTICATED: a caller is whoever Envoy verified, never a claim in the request",
            run: |e| Box::pin(grpc_llm_generate_unauthenticated(e)),
        },
    ],
};

/// `[stack.llms.<name>]` minus `listen`: a llm with an initial behaviour.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields, default)]
pub struct Llm {
    /// Initial fault behaviour.
    pub behavior: Behavior,
}

struct LlmHandle {
    addr: SocketAddr,
    runtime: Runtime,
    task: TaskHandle,
}

#[async_trait]
impl InstanceHandle for LlmHandle {
    async fn ready(&self) -> bool {
        let Ok(channel) = Endpoint::from_shared(format!("http://{}", self.addr)) else {
            return false;
        };
        let Ok(channel) = channel.connect().await else {
            return false;
        };
        let mut health = HealthClient::new(channel);
        let req = HealthCheckRequest {
            service: "tbd.llm.v1.LlmService".to_owned(),
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
impl Service for Llm {
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
        // Both tiers on the in-process stub: a chaos stack must never need a
        // model server, and the stub is labelled as such on every answer.
        let config = Config::stub(addr);
        let runtime = Runtime {
            fault: tbd_common::fault::FaultHandle::new(self.behavior.clone()),
            ..Default::default()
        };
        let (stop, stopped) = tokio::sync::oneshot::channel();
        let rt = runtime.clone();
        let instance_name = name.to_owned();
        let task = tokio::spawn(async move {
            if let Err(error) = tbd_llm::serve_with(listener, config, rt, async {
                let _ = stopped.await;
            })
            .await
            {
                tracing::error!(instance = %instance_name, %error, "llm exited with error");
            }
        });
        Ok(Instance::new(
            name,
            self.kind(),
            addr,
            LlmHandle {
                addr,
                runtime,
                task: TaskHandle::new(stop, task),
            },
        ))
    }
}

/// `LlmService/Ping` echoes and is labelled a stub. Not a named health
/// check: through Envoy's internal listener a health request lands on the
/// engine, and a check whose result depends on the path is worse than none.
async fn grpc_llm_ping(e: Ep) -> Result<String, String> {
    let mut c = LlmServiceClient::new(e.grpc()?);
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

/// `LlmService/ListModels` names both tiers. Presence is the check; `up` is
/// reported: against a deployed stack a model server that is down is a
/// finding for the reader, not a broken check.
async fn grpc_llm_models_lists_both_tiers(e: Ep) -> Result<String, String> {
    let mut c = LlmServiceClient::new(e.grpc()?);
    let r = c
        .list_models(ListModelsRequest {})
        .await
        .map_err(|e| e.to_string())?
        .into_inner();
    let mut tiers: Vec<i32> = r.models.iter().map(|m| m.tier).collect();
    tiers.sort_unstable();
    if tiers != [1, 2] {
        return Err(format!(
            "expected the fast and deep tiers, got {:?}",
            r.models
        ));
    }
    let described: Vec<String> = r
        .models
        .iter()
        .map(|m| {
            format!(
                "{}={}:{} up={} stub={} embeds={}",
                if m.tier == 1 { "fast" } else { "deep" },
                m.engine,
                m.model,
                m.up,
                m.stub,
                m.embeds
            )
        })
        .collect();
    Ok(described.join(" "))
}

/// `LlmService/Generate` with no `x-jwt-payload` is refused as unauthenticated
/// before any engine is asked.
async fn grpc_llm_generate_unauthenticated(e: Ep) -> Result<String, String> {
    let mut c = LlmServiceClient::new(e.grpc()?);
    let req = GenerateRequest {
        messages: vec![Message {
            role: "user".into(),
            content: "validate".into(),
        }],
        ..Default::default()
    };
    match c.generate(req).await {
        Err(s) if s.code() == tonic::Code::Unauthenticated => Ok("unauthenticated".into()),
        Err(s) => Err(format!("wrong code {:?}: {}", s.code(), s.message())),
        Ok(_) => Err("answered without a verified caller".into()),
    }
}
