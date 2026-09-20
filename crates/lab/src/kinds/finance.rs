//! The finance kind: an in-process `tbd-finance` with fault injection.
//! Rendered by `tbd new service`; edit freely, the CLI never rewrites it.

use std::{future::Future, net::SocketAddr};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tbd_common::fault::Behavior;
use tbd_finance::{
    Config, Runtime,
    config::{Connectors, Mail, Metrics, Ping, Provider, Server, Store, Sync},
    store::{MemoryStore, Transaction},
};
use tbd_proto::finance::v1::{PingRequest, finance_service_client::FinanceServiceClient};
use tonic::transport::Endpoint;
use tonic_health::pb::{HealthCheckRequest, health_client::HealthClient};

use crate::kind::{Field, FieldKind, Kind, Target};
use crate::{
    check::{Check, Endpoint as Ep},
    service::{Instance, InstanceHandle, Peers, RequestCounts, Service, TaskHandle},
};

/// The registry entry.
pub static KIND: Kind = Kind {
    name: "finance",
    label: "Finance",
    plural: "finances",
    surface: "grpc",
    target: Some(Target {
        help: "Finance gRPC URL (through Envoy: the engine LB, matched by service name)",
        default_url: "http://127.0.0.1:50054",
    }),
    fields: &[
        Field {
            name: "database_url",
            label: "Postgres URL (empty: no store, data RPCs answer unavailable)",
            kind: FieldKind::Text,
            required: false,
            default: None,
        },
        Field {
            name: "seed",
            label: "Seed an in-memory store: `access` for the owner/accountant world",
            kind: FieldKind::Text,
            required: false,
            default: None,
        },
    ],
    fault: true,
    store_fault: false,
    counters: true,
    load_target: true,
    addable: true,
    parse: crate::kind::parse::<Finance>,
    checks: &[Check {
        name: "grpc_finance_ping",
        surface: "grpc",
        doc: "`Ping` echoes the message and is labelled a stub",
        run: |e| Box::pin(grpc_finance_ping(e)),
    }],
};

/// `[stack.finances.<name>]` minus `listen`: a finance with an initial behaviour.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields, default)]
pub struct Finance {
    /// Initial fault behaviour.
    pub behavior: Behavior,
    /// Postgres URL. Empty means no store: `Ping` answers and every data RPC
    /// is `UNAVAILABLE`, which is what a scenario that only exercises the
    /// surface wants.
    #[serde(default)]
    pub database_url: String,
    /// Seed an in-memory store instead of using Postgres.
    ///
    /// `access` builds the world every access-control scenario runs against:
    /// an owner with a personal party and a company, and an accountant granted
    /// the company only, with one transaction in each. It exists so access
    /// control is provable *continuously* against a running service -- a
    /// scenario cannot seed a Postgres, and a rule this important should not be
    /// proven only by a test that can.
    #[serde(default)]
    pub seed: String,
}

/// Subjects the seeded world knows, and the party each may read.
///
/// Fixed uuids so a scenario and its assertions can name them.
pub const OWNER_SUBJECT: &str = "chaos-owner";
/// The accountant: granted the company, never the personal party.
pub const READER_SUBJECT: &str = "chaos-reader";
const PERSONAL_PARTY: uuid::Uuid = uuid::uuid!("11111111-1111-4111-8111-111111111111");
const COMPANY_PARTY: uuid::Uuid = uuid::uuid!("22222222-2222-4222-8222-222222222222");

/// Start the service, seeded or not.
///
/// A seeded instance holds its data in memory, so an access-control scenario
/// runs anywhere `chaos` runs -- no Postgres, no Docker, no fixture left behind
/// by an earlier job. Hermetic, like every other scenario in the suite.
async fn serve(
    seed: String,
    listener: tokio::net::TcpListener,
    config: Config,
    runtime: Runtime,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> Result<(), tbd_finance::ServeError> {
    if seed == "access" {
        let (store, grants) = seeded();
        return tbd_finance::serve_seeded(listener, config, runtime, store, grants, shutdown).await;
    }
    tbd_finance::serve_with(listener, config, runtime, shutdown).await
}

/// The world named by `seed = "access"`.
fn seeded() -> (MemoryStore, Vec<(String, Vec<uuid::Uuid>)>) {
    let store = MemoryStore::new();
    store.insert([
        Transaction {
            id: uuid::uuid!("aaaaaaaa-0000-4000-8000-000000000001"),
            account_id: uuid::uuid!("aaaaaaaa-1111-4000-8000-000000000001"),
            party_id: PERSONAL_PARTY,
            status: "booked".into(),
            amount_minor: -4_250,
            currency: "EUR".into(),
            scale: 2,
            booking_date: chrono::NaiveDate::from_ymd_opt(2026, 9, 15),
            counterparty_name: Some("A PRIVATE MATTER".into()),
            remittance: Some("personal payment".into()),
            ..Transaction::default()
        },
        Transaction {
            id: uuid::uuid!("bbbbbbbb-0000-4000-8000-000000000001"),
            account_id: uuid::uuid!("bbbbbbbb-1111-4000-8000-000000000001"),
            party_id: COMPANY_PARTY,
            status: "booked".into(),
            amount_minor: 1_450_082,
            currency: "EUR".into(),
            scale: 2,
            booking_date: chrono::NaiveDate::from_ymd_opt(2026, 9, 4),
            counterparty_name: Some("TENDERLY D.O.O.".into()),
            remittance: Some("BROJ RACUNA 9-1-1-2026".into()),
            ..Transaction::default()
        },
    ]);
    let grants = vec![
        (
            OWNER_SUBJECT.to_owned(),
            vec![PERSONAL_PARTY, COMPANY_PARTY],
        ),
        (READER_SUBJECT.to_owned(), vec![COMPANY_PARTY]),
    ];
    (store, grants)
}

struct FinanceHandle {
    addr: SocketAddr,
    runtime: Runtime,
    task: TaskHandle,
}

#[async_trait]
impl InstanceHandle for FinanceHandle {
    async fn ready(&self) -> bool {
        let Ok(channel) = Endpoint::from_shared(format!("http://{}", self.addr)) else {
            return false;
        };
        let Ok(channel) = channel.connect().await else {
            return false;
        };
        let mut health = HealthClient::new(channel);
        let req = HealthCheckRequest {
            service: "tbd.finance.v1.FinanceService".to_owned(),
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
impl Service for Finance {
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
            store: Store {
                url: self.database_url.clone(),
                max_connections: 8,
            },
            // No bank behind a lab instance: the defaults mean no provider and
            // no sync, which is what a stack under test wants.
            provider: Provider::default(),
            sync: Sync::default(),
            // Likewise no mailbox connectors and no outgoing mail.
            connectors: Connectors::default(),
            mail: Mail::default(),
        };
        let runtime = Runtime {
            fault: tbd_common::fault::FaultHandle::new(self.behavior.clone()),
            ..Default::default()
        };
        let (stop, stopped) = tokio::sync::oneshot::channel();
        let rt = runtime.clone();
        let instance_name = name.to_owned();
        let seed = self.seed.clone();
        let task = tokio::spawn(async move {
            if let Err(error) = serve(seed, listener, config, rt, async {
                let _ = stopped.await;
            })
            .await
            {
                tracing::error!(instance = %instance_name, %error, "finance exited with error");
            }
        });
        Ok(Instance::new(
            name,
            self.kind(),
            addr,
            FinanceHandle {
                addr,
                runtime,
                task: TaskHandle::new(stop, task),
            },
        ))
    }
}

/// `FinanceService/Ping` echoes and is labelled a stub. Not a named health
/// check: through Envoy's internal listener a health request lands on the
/// engine, and a check whose result depends on the path is worse than none.
async fn grpc_finance_ping(e: Ep) -> Result<String, String> {
    let mut c = FinanceServiceClient::new(e.grpc()?);
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
