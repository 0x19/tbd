//! The connector and document RPCs.

use tbd_proto::finance::v1::{
    CompleteConnectorRequest, CompleteConnectorResponse, ConfigureConnectorRequest,
    ConfigureConnectorResponse, Connector, ConnectorKind, ConnectorRun, DeleteConnectorRequest,
    DeleteConnectorResponse, ListConnectorKindsRequest, ListConnectorKindsResponse,
    ListConnectorRunsRequest, ListConnectorRunsResponse, ListConnectorsRequest,
    ListConnectorsResponse, StartConnectorRequest, StartConnectorResponse, SyncConnectorRequest,
    SyncConnectorResponse, TestConnectorRequest, TestConnectorResponse, WatchConnectorsRequest,
    WatchConnectorsResponse,
};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::{
    connectors::{
        self, ConnectorError,
        store::{self, ConnectorRow, RunRow, StoreError},
    },
    service::{Finance, status_of},
};

fn connector_status(e: StoreError) -> Status {
    tracing::warn!(error = %e, "connector rpc refused");
    match e {
        StoreError::Db(d) => status_of(d),
        StoreError::Connector(ConnectorError::Unconfigured(m)) | StoreError::Refused(m) => {
            Status::failed_precondition(m)
        }
        StoreError::Connector(ConnectorError::Unlinked(m)) => {
            Status::failed_precondition(format!("not linked: {m}"))
        }
        StoreError::Connector(ConnectorError::Invalid(m)) => Status::invalid_argument(m),
        StoreError::Connector(ConnectorError::Provider(m)) => {
            Status::unavailable(format!("provider: {m}"))
        }
        StoreError::Crypto(c) => Status::internal(format!("credentials: {c}")),
        StoreError::UnknownKind(k) => Status::invalid_argument(format!("unknown kind {k}")),
        StoreError::State(s) => Status::failed_precondition(format!("connector is {s}")),
    }
}

fn uuid(s: &str, field: &str) -> Result<Uuid, Status> {
    Uuid::parse_str(s).map_err(|_| Status::invalid_argument(format!("{field}: not a uuid")))
}

fn t(v: Option<chrono::DateTime<chrono::Utc>>) -> String {
    v.map(|t| t.to_rfc3339()).unwrap_or_default()
}

fn connector_proto(c: ConnectorRow) -> Connector {
    Connector {
        id: c.id.to_string(),
        party_id: c.party_id.to_string(),
        kind: c.kind,
        label: c.label,
        status: c.status,
        config: c.config.to_string(),
        external_id: c.external_id.unwrap_or_default(),
        linked_at: t(c.linked_at),
        last_sync_at: t(c.last_sync_at),
        last_sync_status: c.last_sync_status.unwrap_or_default(),
        last_sync_error: c.last_sync_error.unwrap_or_default(),
        failure: c.failure.unwrap_or_default(),
        created_at: c.created_at.to_rfc3339(),
        can_send: c.can_send,
        can_read: c.can_read,
    }
}

fn run_proto(r: RunRow) -> ConnectorRun {
    ConnectorRun {
        id: r.id.to_string(),
        started_at: r.started_at.to_rfc3339(),
        finished_at: t(r.finished_at),
        trigger: r.trigger,
        outcome: r.outcome.unwrap_or_default(),
        found: u32::try_from(r.found).unwrap_or(0),
        stored: u32::try_from(r.stored).unwrap_or(0),
        skipped: u32::try_from(r.skipped).unwrap_or(0),
        error: r.error.unwrap_or_default(),
    }
}

impl Finance {
    pub(crate) fn kinds(&self) -> Vec<Box<dyn connectors::Connector>> {
        match &self.connector_kinds {
            Some(k) => k(),
            None => connectors::registry(&self.connectors),
        }
    }

    pub(crate) fn sealer(&self) -> Result<std::sync::Arc<connectors::crypto::Sealer>, Status> {
        self.sealer.clone().ok_or_else(|| {
            Status::failed_precondition("no FINANCE_CONNECTOR_KEY: connectors cannot be linked")
        })
    }

    pub(crate) fn done_c<T>(
        &self,
        timer: &mut tbd_common::metrics::RequestTimer,
        r: Result<T, StoreError>,
    ) -> Result<Response<T>, Status> {
        match r {
            Ok(v) => Ok(Response::new(v)),
            Err(e) => Err(self.reject(timer, connector_status(e))),
        }
    }

    pub(crate) async fn rpc_list_connector_kinds(
        &self,
        request: Request<ListConnectorKindsRequest>,
    ) -> Result<Response<ListConnectorKindsResponse>, Status> {
        let mut timer = self.admit("FinanceService/ListConnectorKinds").await?;
        let _ = self
            .access(&request)
            .await
            .map_err(|s| self.reject(&mut timer, s))?;
        let configured = self.sealer.is_some();
        Ok(Response::new(ListConnectorKindsResponse {
            kinds: self
                .kinds()
                .iter()
                .map(|k| {
                    let k = k.kind();
                    ConnectorKind {
                        name: k.name.into(),
                        label: k.label.into(),
                        description: k.description.into(),
                        auth: match k.auth {
                            connectors::Auth::Oauth => "oauth".into(),
                            connectors::Auth::Token => "token".into(),
                        },
                        consent_note: k.consent_note.into(),
                        configured: k.configured && configured,
                        purposes: k.purposes.iter().map(|p| p.as_str().to_owned()).collect(),
                    }
                })
                .collect(),
        }))
    }

    pub(crate) async fn rpc_list_connectors(
        &self,
        request: Request<ListConnectorsRequest>,
    ) -> Result<Response<ListConnectorsResponse>, Status> {
        let party_ids = request.get_ref().party_ids.clone();
        let (mut timer, pool, _, view) = self
            .invoice_context("FinanceService/ListConnectors", &request, &party_ids)
            .await?;
        let r = store::list(pool, &view)
            .await
            .map(|c| ListConnectorsResponse {
                connectors: c.into_iter().map(connector_proto).collect(),
            });
        self.done_c(&mut timer, r)
    }

    pub(crate) async fn rpc_start_connector(
        &self,
        request: Request<StartConnectorRequest>,
    ) -> Result<Response<StartConnectorResponse>, Status> {
        let req = request.get_ref().clone();
        let party = uuid(&req.party_id, "party_id")?;
        let (mut timer, pool, access, _) = self
            .invoice_context("FinanceService/StartConnector", &request, &[])
            .await?;
        if let Err(s) = self.sealer() {
            return Err(self.reject(&mut timer, s));
        }
        let kinds = self.kinds();
        let Some(kind) = kinds.iter().find(|k| k.kind().name == req.kind) else {
            return Err(self.reject(
                &mut timer,
                Status::invalid_argument(format!("unknown kind {}", req.kind)),
            ));
        };
        let purpose: connectors::Purpose = match req.purpose.parse() {
            Ok(p) => p,
            Err(e) => return Err(self.reject(&mut timer, Status::invalid_argument(e))),
        };
        let r = store::start(
            pool,
            &access,
            kind.as_ref(),
            party,
            purpose,
            &self.connectors.redirect_url,
        )
        .await
        .map(|(id, url, state)| StartConnectorResponse {
            connector_id: id.to_string(),
            url,
            state,
        });
        self.done_c(&mut timer, r)
    }

    pub(crate) async fn rpc_complete_connector(
        &self,
        request: Request<CompleteConnectorRequest>,
    ) -> Result<Response<CompleteConnectorResponse>, Status> {
        let req = request.get_ref().clone();
        if req.state.is_empty() || req.code.is_empty() {
            return Err(Status::invalid_argument("state and code are required"));
        }
        let (mut timer, pool, access, _) = self
            .invoice_context("FinanceService/CompleteConnector", &request, &[])
            .await?;
        let sealer = match self.sealer() {
            Ok(s) => s,
            Err(s) => return Err(self.reject(&mut timer, s)),
        };
        let kinds = self.kinds();
        let r = store::complete(
            pool,
            &access,
            &sealer,
            &kinds,
            &self.connectors.redirect_url,
            &req.state,
            &req.code,
        )
        .await
        .map(|c| CompleteConnectorResponse {
            connector: Some(connector_proto(c)),
        });
        self.done_c(&mut timer, r)
    }

    pub(crate) async fn rpc_test_connector(
        &self,
        request: Request<TestConnectorRequest>,
    ) -> Result<Response<TestConnectorResponse>, Status> {
        let id = uuid(&request.get_ref().id, "id")?;
        let (mut timer, pool, access, _) = self
            .invoice_context("FinanceService/TestConnector", &request, &[])
            .await?;
        let sealer = match self.sealer() {
            Ok(s) => s,
            Err(s) => return Err(self.reject(&mut timer, s)),
        };
        let kinds = self.kinds();
        let r = store::test(pool, &access, &sealer, &kinds, id)
            .await
            .map(|status| TestConnectorResponse { status });
        self.done_c(&mut timer, r)
    }

    pub(crate) async fn rpc_sync_connector(
        &self,
        request: Request<SyncConnectorRequest>,
    ) -> Result<Response<SyncConnectorResponse>, Status> {
        let id = uuid(&request.get_ref().id, "id")?;
        let (mut timer, pool, access, _) = self
            .invoice_context("FinanceService/SyncConnector", &request, &[])
            .await?;
        let sealer = match self.sealer() {
            Ok(s) => s,
            Err(s) => return Err(self.reject(&mut timer, s)),
        };
        let kinds = self.kinds();
        let r = match store::begin_sync(pool, &access, &kinds, id, "manual").await {
            Ok(started) => {
                // Detached: the pull outlives this call. The run row is the
                // handle; a dropped response cannot leave it unfinished, and
                // `run_sync` records every failure before returning it.
                let pool = pool.clone();
                let run = started.run.clone();
                tokio::spawn(async move {
                    let _ = store::run_sync(&pool, &sealer, &kinds, &started).await;
                });
                Ok(SyncConnectorResponse {
                    run: Some(run_proto(run)),
                })
            }
            Err(e) => Err(e),
        };
        self.done_c(&mut timer, r)
    }

    pub(crate) async fn rpc_configure_connector(
        &self,
        request: Request<ConfigureConnectorRequest>,
    ) -> Result<Response<ConfigureConnectorResponse>, Status> {
        let req = request.get_ref().clone();
        let id = uuid(&req.id, "id")?;
        let mut change = store::Change::default();
        if !req.config.trim().is_empty() {
            let config: serde_json::Value = serde_json::from_str(&req.config)
                .map_err(|e| Status::invalid_argument(format!("config: {e}")))?;
            if !config.is_object() {
                return Err(Status::invalid_argument("config: want a JSON object"));
            }
            change.config = Some(config);
        }
        if !req.party_id.is_empty() {
            change.party = Some(uuid(&req.party_id, "party_id")?);
        }
        if !req.label.trim().is_empty() {
            change.label = Some(req.label.trim().to_owned());
        }
        let (mut timer, pool, access, _) = self
            .invoice_context("FinanceService/ConfigureConnector", &request, &[])
            .await?;
        let r = store::configure(pool, &access, id, &change).await.map(|c| {
            ConfigureConnectorResponse {
                connector: Some(connector_proto(c)),
            }
        });
        self.done_c(&mut timer, r)
    }

    /// The connectors in view, then every change to one: a diff of the
    /// store taken once a second while the client listens. Polling the
    /// database here rather than from each browser keeps one query per
    /// watcher, and the run row is where progress is written, so this is
    /// what a pull looks like from outside.
    pub(crate) async fn rpc_watch_connectors(
        &self,
        request: Request<WatchConnectorsRequest>,
    ) -> Result<Response<ReceiverStream<Result<WatchConnectorsResponse, Status>>>, Status> {
        let party_ids = request.get_ref().party_ids.clone();
        let (timer, pool, _, view) = self
            .invoice_context("FinanceService/WatchConnectors", &request, &party_ids)
            .await?;
        let pool = pool.clone();
        let (tx, rx) = tokio::sync::mpsc::channel(16);
        tokio::spawn(async move {
            let _timer = timer;
            let guard = tbd_common::metrics::StreamGuard::open("watch_connectors");
            let mut last: std::collections::HashMap<Uuid, (ConnectorRow, Option<RunRow>)> =
                std::collections::HashMap::new();
            let mut tick = tokio::time::interval(std::time::Duration::from_secs(1));
            loop {
                tick.tick().await;
                let now = match store::snapshot(&pool, &view).await {
                    Ok(s) => s,
                    Err(e) => {
                        let _ = tx.send(Err(connector_status(e))).await;
                        return;
                    }
                };
                let mut seen = std::collections::HashSet::new();
                for (c, run) in now {
                    seen.insert(c.id);
                    if last
                        .get(&c.id)
                        .is_some_and(|(pc, pr)| *pc == c && *pr == run)
                    {
                        continue;
                    }
                    let event = WatchConnectorsResponse {
                        connector: Some(connector_proto(c.clone())),
                        run: run.clone().map(run_proto),
                        deleted: false,
                    };
                    last.insert(c.id, (c, run));
                    if tx.send(Ok(event)).await.is_err() {
                        return;
                    }
                    guard.item("out");
                }
                let gone: Vec<Uuid> = last
                    .keys()
                    .filter(|id| !seen.contains(id))
                    .copied()
                    .collect();
                for id in gone {
                    if let Some((c, _)) = last.remove(&id) {
                        let event = WatchConnectorsResponse {
                            connector: Some(connector_proto(c)),
                            run: None,
                            deleted: true,
                        };
                        if tx.send(Ok(event)).await.is_err() {
                            return;
                        }
                        guard.item("out");
                    }
                }
            }
        });
        Ok(Response::new(ReceiverStream::new(rx)))
    }

    pub(crate) async fn rpc_delete_connector(
        &self,
        request: Request<DeleteConnectorRequest>,
    ) -> Result<Response<DeleteConnectorResponse>, Status> {
        let id = uuid(&request.get_ref().id, "id")?;
        let (mut timer, pool, access, _) = self
            .invoice_context("FinanceService/DeleteConnector", &request, &[])
            .await?;
        let r = store::delete(pool, &access, id)
            .await
            .map(|()| DeleteConnectorResponse {});
        self.done_c(&mut timer, r)
    }

    pub(crate) async fn rpc_list_connector_runs(
        &self,
        request: Request<ListConnectorRunsRequest>,
    ) -> Result<Response<ListConnectorRunsResponse>, Status> {
        let id = uuid(&request.get_ref().id, "id")?;
        let (mut timer, pool, access, _) = self
            .invoice_context("FinanceService/ListConnectorRuns", &request, &[])
            .await?;
        let r = store::runs(pool, &access, id, 50)
            .await
            .map(|r| ListConnectorRunsResponse {
                runs: r.into_iter().map(run_proto).collect(),
            });
        self.done_c(&mut timer, r)
    }
}
