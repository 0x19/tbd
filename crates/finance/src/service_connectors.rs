//! The connector and document RPCs.

use tbd_db::DbError;
use tbd_proto::finance::v1::{
    CompleteConnectorRequest, CompleteConnectorResponse, ConfigureConnectorRequest,
    ConfigureConnectorResponse, Connector, ConnectorKind, ConnectorRun, DeleteConnectorRequest,
    DeleteConnectorResponse, Document, DocumentSource, GetDocumentRequest, GetDocumentResponse,
    ListConnectorKindsRequest, ListConnectorKindsResponse, ListConnectorRunsRequest,
    ListConnectorRunsResponse, ListConnectorsRequest, ListConnectorsResponse, ListDocumentsRequest,
    ListDocumentsResponse, StartConnectorRequest, StartConnectorResponse, SyncConnectorRequest,
    SyncConnectorResponse, TestConnectorRequest, TestConnectorResponse,
};
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
        StoreError::Connector(ConnectorError::Unconfigured(m)) => Status::failed_precondition(m),
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

/// A document row with its sources, for the listing.
#[allow(missing_docs)]
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DocumentRow {
    pub id: Uuid,
    pub party_id: Uuid,
    pub kind: String,
    pub filename: Option<String>,
    pub content_type: String,
    pub size_bytes: i64,
    pub sha256: String,
    pub vendor: Option<String>,
    pub doc_date: Option<chrono::NaiveDate>,
    pub total_minor: Option<i64>,
    pub currency: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[allow(missing_docs)]
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SourceRow {
    pub document_id: Uuid,
    pub connector_id: Option<Uuid>,
    pub external_ref: String,
    pub subject: String,
    pub sender: String,
    pub received_at: Option<chrono::DateTime<chrono::Utc>>,
}

fn document_proto(d: DocumentRow, sources: Vec<SourceRow>) -> Document {
    Document {
        id: d.id.to_string(),
        party_id: d.party_id.to_string(),
        kind: d.kind,
        filename: d.filename.unwrap_or_default(),
        content_type: d.content_type,
        size_bytes: d.size_bytes,
        sha256: d.sha256,
        vendor: d.vendor.unwrap_or_default(),
        doc_date: d.doc_date.map(|d| d.to_string()).unwrap_or_default(),
        total_minor: d.total_minor.map(|m| m.to_string()).unwrap_or_default(),
        currency: d.currency.unwrap_or_default(),
        created_at: d.created_at.to_rfc3339(),
        sources: sources
            .into_iter()
            .map(|s| DocumentSource {
                connector_id: s.connector_id.map(|c| c.to_string()).unwrap_or_default(),
                external_ref: s.external_ref,
                subject: s.subject,
                sender: s.sender,
                received_at: t(s.received_at),
            })
            .collect(),
    }
}

const DOC_COLUMNS: &str = "id, party_id, kind, filename, content_type, size_bytes, sha256, vendor, doc_date, total_minor, currency, created_at";

impl Finance {
    fn kinds(&self) -> Vec<Box<dyn connectors::Connector>> {
        match &self.connector_kinds {
            Some(k) => k(),
            None => connectors::registry(&self.connectors),
        }
    }

    fn sealer(&self) -> Result<&connectors::crypto::Sealer, Status> {
        self.sealer.as_deref().ok_or_else(|| {
            Status::failed_precondition("no FINANCE_CONNECTOR_KEY: connectors cannot be linked")
        })
    }

    fn done_c<T>(
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
        let r = store::start(
            pool,
            &access,
            kind.as_ref(),
            party,
            &self.connectors.redirect_url,
        )
        .await
        .map(|(id, url)| StartConnectorResponse {
            connector_id: id.to_string(),
            url,
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
            sealer,
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
        let r = store::test(pool, &access, sealer, &kinds, id)
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
        let r = store::sync(pool, &access, sealer, &kinds, id, "manual")
            .await
            .map(|p| SyncConnectorResponse {
                found: u32::try_from(p.found).unwrap_or(u32::MAX),
                stored: u32::try_from(p.stored).unwrap_or(u32::MAX),
                skipped: u32::try_from(p.skipped).unwrap_or(u32::MAX),
            });
        self.done_c(&mut timer, r)
    }

    pub(crate) async fn rpc_configure_connector(
        &self,
        request: Request<ConfigureConnectorRequest>,
    ) -> Result<Response<ConfigureConnectorResponse>, Status> {
        let req = request.get_ref().clone();
        let id = uuid(&req.id, "id")?;
        let config: serde_json::Value = if req.config.trim().is_empty() {
            serde_json::json!({})
        } else {
            serde_json::from_str(&req.config)
                .map_err(|e| Status::invalid_argument(format!("config: {e}")))?
        };
        if !config.is_object() {
            return Err(Status::invalid_argument("config: want a JSON object"));
        }
        let (mut timer, pool, access, _) = self
            .invoice_context("FinanceService/ConfigureConnector", &request, &[])
            .await?;
        let r =
            store::configure(pool, &access, id, config)
                .await
                .map(|c| ConfigureConnectorResponse {
                    connector: Some(connector_proto(c)),
                });
        self.done_c(&mut timer, r)
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

    pub(crate) async fn rpc_list_documents(
        &self,
        request: Request<ListDocumentsRequest>,
    ) -> Result<Response<ListDocumentsResponse>, Status> {
        let req = request.get_ref().clone();
        let (mut timer, pool, _, view) = self
            .invoice_context("FinanceService/ListDocuments", &request, &req.party_ids)
            .await?;
        let r = async {
            if view.is_empty() {
                return Ok(ListDocumentsResponse { documents: Vec::new() });
            }
            let limit = i64::from(req.limit.clamp(1, 500).max(if req.limit == 0 { 100 } else { 1 }));
            let docs = sqlx::query_as::<_, DocumentRow>(sqlx::AssertSqlSafe(format!(
                "select {DOC_COLUMNS} from finance.documents
                  where party_id = any($1) and ($2 = '' or kind = $2)
                  order by coalesce(doc_date, created_at::date) desc, created_at desc limit $3 offset $4"
            )))
            .bind(view.party_ids())
            .bind(&req.kind)
            .bind(limit)
            .bind(i64::from(req.offset))
            .fetch_all(pool)
            .await
            .map_err(tbd_db::map_err)?;
            let ids: Vec<Uuid> = docs.iter().map(|d| d.id).collect();
            let sources = sqlx::query_as::<_, SourceRow>(
                "select document_id, connector_id, external_ref, subject, sender, received_at
                   from finance.document_sources where document_id = any($1)",
            )
            .bind(&ids)
            .fetch_all(pool)
            .await
            .map_err(tbd_db::map_err)?;
            Ok::<_, StoreError>(ListDocumentsResponse {
                documents: docs
                    .into_iter()
                    .map(|d| {
                        let mine: Vec<SourceRow> = sources.iter().filter(|s| s.document_id == d.id).cloned().collect();
                        document_proto(d, mine)
                    })
                    .collect(),
            })
        }
        .await;
        self.done_c(&mut timer, r)
    }

    pub(crate) async fn rpc_get_document(
        &self,
        request: Request<GetDocumentRequest>,
    ) -> Result<Response<GetDocumentResponse>, Status> {
        let id = uuid(&request.get_ref().id, "id")?;
        let (mut timer, pool, access, _) = self
            .invoice_context("FinanceService/GetDocument", &request, &[])
            .await?;
        let r = async {
            let doc = sqlx::query_as::<_, DocumentRow>(sqlx::AssertSqlSafe(format!(
                "select {DOC_COLUMNS} from finance.documents where id = $1"
            )))
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(tbd_db::map_err)?
            .ok_or(DbError::NotFound { what: "document" })?;
            access.require(tbd_db::PartyId(doc.party_id), "document")?;
            let (bytes,): (Vec<u8>,) =
                sqlx::query_as("select bytes from finance.document_blobs where document_id = $1")
                    .bind(id)
                    .fetch_one(pool)
                    .await
                    .map_err(tbd_db::map_err)?;
            let sources = sqlx::query_as::<_, SourceRow>(
                "select document_id, connector_id, external_ref, subject, sender, received_at
                   from finance.document_sources where document_id = $1",
            )
            .bind(id)
            .fetch_all(pool)
            .await
            .map_err(tbd_db::map_err)?;
            Ok::<_, StoreError>(GetDocumentResponse {
                document: Some(document_proto(doc, sources)),
                bytes,
            })
        }
        .await;
        self.done_c(&mut timer, r)
    }
}
