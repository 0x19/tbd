//! `tbd.finance.v1.FinanceService` implementation.
//!
//! Everything here is a **stub** and says so on the wire (`stub = true`).
//! Every RPC first consults the fault handle in [`Runtime`], so an embedder can
//! make this service slow, failing or hung at runtime.

use std::pin::Pin;
use std::sync::Arc;

use sqlx::PgPool;
use tbd_common::{
    fault::{ErrorKind, Fault},
    metrics::{RequestTimer, names},
    principal::Principal,
};
use tbd_db::{Access, DbError};
use tbd_proto::finance::v1::{
    Account, AgingReportRequest, AgingReportResponse, ApproveInvoiceRequest,
    ApproveInvoiceResponse, Balance, CancelInvoiceRequest, CancelInvoiceResponse, Category,
    CompleteConnectionRequest, CompleteConnectionResponse, CompleteConnectorRequest,
    CompleteConnectorResponse, ConfigureConnectorRequest, ConfigureConnectorResponse, Connection,
    CreateInvoiceRequest, CreateInvoiceResponse, DeclareCategoryRequest, DeclareCategoryResponse,
    DeleteConnectionRequest, DeleteConnectionResponse, DeleteConnectorRequest,
    DeleteConnectorResponse, DeleteLineTemplateRequest, DeleteLineTemplateResponse,
    GetDocumentRequest, GetDocumentResponse, GetInvoiceDocumentRequest, GetInvoiceDocumentResponse,
    GetInvoiceRequest, GetInvoiceResponse, GetIssuerRequest, GetIssuerResponse,
    GetTransactionRequest, GetTransactionResponse, ListAccountsRequest, ListAccountsResponse,
    ListCategoriesRequest, ListCategoriesResponse, ListClientsRequest, ListClientsResponse,
    ListConnectionsRequest, ListConnectionsResponse, ListConnectorKindsRequest,
    ListConnectorKindsResponse, ListConnectorRunsRequest, ListConnectorRunsResponse,
    ListConnectorsRequest, ListConnectorsResponse, ListDocumentsRequest, ListDocumentsResponse,
    ListInvoicesRequest, ListInvoicesResponse, ListLineTemplatesRequest, ListLineTemplatesResponse,
    ListPartiesRequest, ListPartiesResponse, ListRulesRequest, ListRulesResponse,
    ListTransactionsRequest, ListTransactionsResponse, MonthlySummaryRequest,
    MonthlySummaryResponse, Party, PingRequest, PingResponse, PreviewInvoiceRequest,
    PreviewInvoiceResponse, RefreshAccountRequest, RefreshAccountResponse, Rule,
    SetAccountSyncRequest, SetAccountSyncResponse, StartConnectionRequest, StartConnectionResponse,
    StartConnectorRequest, StartConnectorResponse, SummaryRow, SyncConnectorRequest,
    SyncConnectorResponse, TestConnectorRequest, TestConnectorResponse, Transaction,
    UpdateInvoiceRequest, UpdateInvoiceResponse, UpsertCategoryRequest, UpsertCategoryResponse,
    UpsertClientRequest, UpsertClientResponse, UpsertIssuerRequest, UpsertIssuerResponse,
    UpsertLineTemplateRequest, UpsertLineTemplateResponse, UpsertRuleRequest, UpsertRuleResponse,
    WatchConnectorsRequest, WatchConnectorsResponse, finance_service_server::FinanceService,
};
use tbd_proto::finance::v1::{
    CreateIssuerRequest, CreateIssuerResponse, DeleteCounterpartyPolicyRequest,
    DeleteCounterpartyPolicyResponse, DeleteInvoiceRequest, DeleteInvoiceResponse,
    LinkDocumentRequest, LinkDocumentResponse, ListIssuersRequest, ListIssuersResponse,
    MonthlyReconciliationRequest, MonthlyReconciliationResponse, RecordPaymentRequest,
    RecordPaymentResponse, SetCounterpartyPolicyRequest, SetCounterpartyPolicyResponse,
    SetDefaultClientRequest, SetDefaultClientResponse, SetTransactionNoteRequest,
    SetTransactionNoteResponse, UnlinkDocumentRequest, UnlinkDocumentResponse,
    UnlinkPaymentRequest, UnlinkPaymentResponse,
};
use tbd_proto::finance::v1::{
    DeleteMailTemplateRequest, DeleteMailTemplateResponse, ExtractDocumentRequest,
    ExtractDocumentResponse, GetMailRequest, GetMailResponse, ListMailRequest, ListMailResponse,
    ListMailTemplatesRequest, ListMailTemplatesResponse, SendMailRequest, SendMailResponse,
    UpdateDocumentRequest, UpdateDocumentResponse, UploadDocumentRequest, UploadDocumentResponse,
    UpsertMailTemplateRequest, UpsertMailTemplateResponse,
};
use tbd_proto::finance::v1::{
    GetFilingRequest, GetFilingResponse, ListFilingsRequest, ListFilingsResponse,
};
use tonic::{Code, Request, Response, Status};
use uuid::Uuid;

use crate::{
    Runtime,
    banking::{Provider, connect},
    categorise,
    config::{Ping, Sync as SyncConfig},
    money,
    store::{self, MemoryStore, PgStore, Store, TransactionFilter},
    sync::{Outcome, Skipped, Syncer},
};

/// The service. Cheap to clone; holds its configuration section and shared handles.
#[derive(Clone)]
pub struct Finance {
    ping: Ping,
    runtime: Runtime,
    /// The store, when there is one. Without it every data RPC answers
    /// `UNAVAILABLE`, which is what a scaffolded deployment looks like before
    /// its database exists.
    store: Option<Arc<dyn Store>>,
    /// The pool, kept separately because resolving a caller's grants needs the
    /// identity tables and the memory store has none. A memory-backed stack
    /// therefore takes its caller from `Access::for_parties` instead.
    pool: Option<PgPool>,
    /// Subject to readable parties, for memory-backed stacks only. Empty with a
    /// database, where grants are rows in `public.party_access`.
    grants: Vec<(String, Vec<Uuid>)>,
    /// The bank, when one is configured. Without it the connection and
    /// refresh RPCs answer `FAILED_PRECONDITION` and say why.
    provider: Option<Arc<dyn Provider>>,
    sync: SyncConfig,
    redirect_url: String,
    /// `[connectors]`, and the sealer built from its key.
    pub(crate) connectors: crate::config::Connectors,
    /// `[mail]`: the recipient allowlist.
    pub(crate) mail: crate::config::Mail,
    pub(crate) sealer: Option<Arc<crate::connectors::crypto::Sealer>>,
    /// Kinds to use instead of the registry -- tests inject a mock kind.
    pub(crate) connector_kinds: Option<crate::connectors::KindsFactory>,
}

impl std::fmt::Debug for Finance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Finance")
            .field("store", &self.store.as_ref().map(|s| s.kind()))
            .field("bank", &self.provider.as_ref().map(|p| p.name()))
            .field("sealer", &self.sealer.is_some())
            .finish_non_exhaustive()
    }
}

impl Finance {
    /// Build a service from its `[ping]` configuration, without a store.
    ///
    /// Every data RPC then answers `UNAVAILABLE`, which is what a scaffolded
    /// deployment does before its database exists -- and what the chaos stack
    /// does when a scenario gives it no `database_url`.
    #[must_use]
    pub fn new(ping: Ping, runtime: Runtime) -> Self {
        Self {
            ping,
            runtime,
            store: None,
            pool: None,
            grants: Vec::new(),
            provider: None,
            sync: SyncConfig::default(),
            redirect_url: String::new(),
            connectors: crate::config::Connectors::default(),
            mail: crate::config::Mail::default(),
            sealer: None,
            connector_kinds: None,
        }
    }

    /// Build a service backed by a database.
    #[must_use]
    pub fn with_pool(ping: Ping, runtime: Runtime, pool: PgPool) -> Self {
        Self {
            ping,
            runtime,
            store: Some(Arc::new(PgStore::new(pool.clone()))),
            pool: Some(pool),
            grants: Vec::new(),
            provider: None,
            sync: SyncConfig::default(),
            redirect_url: String::new(),
            connectors: crate::config::Connectors::default(),
            mail: crate::config::Mail::default(),
            sealer: None,
            connector_kinds: None,
        }
    }

    /// Build a service over an in-memory store, for stacks with no database.
    ///
    /// Callers are resolved with [`tbd_db::Access::for_parties`] from
    /// `grants`, because there are no identity tables to read. Chaos uses this
    /// so an access-control scenario can run against a live service.
    #[must_use]
    pub fn with_memory(
        ping: Ping,
        runtime: Runtime,
        store: MemoryStore,
        grants: Vec<(String, Vec<Uuid>)>,
    ) -> Self {
        Self {
            ping,
            runtime,
            store: Some(Arc::new(store)),
            pool: None,
            grants: Vec::new(),
            provider: None,
            sync: SyncConfig::default(),
            redirect_url: String::new(),
            connectors: crate::config::Connectors::default(),
            mail: crate::config::Mail::default(),
            sealer: None,
            connector_kinds: None,
        }
        .with_grants(grants)
    }

    fn with_grants(mut self, grants: Vec<(String, Vec<Uuid>)>) -> Self {
        self.grants = grants;
        self
    }

    /// The `[mail]` section.
    #[must_use]
    pub fn with_mail(mut self, mail: crate::config::Mail) -> Self {
        self.mail = mail;
        self
    }

    /// Attach the connector configuration; the sealer is built from its key.
    ///
    /// # Errors
    /// The key is not 32 bytes of base64.
    pub fn with_connectors(
        mut self,
        connectors: crate::config::Connectors,
    ) -> Result<Self, crate::connectors::crypto::CryptoError> {
        self.sealer = if connectors.key.is_empty() {
            None
        } else {
            Some(Arc::new(crate::connectors::crypto::Sealer::from_base64(
                &connectors.key,
            )?))
        };
        self.connectors = connectors;
        Ok(self)
    }

    /// Use these kinds instead of the registry. Tests.
    #[must_use]
    pub fn with_connector_kinds(mut self, kinds: crate::connectors::KindsFactory) -> Self {
        self.connector_kinds = Some(kinds);
        self
    }

    /// Attach a bank, so connections can be made and accounts refreshed.
    #[must_use]
    pub fn with_bank(
        mut self,
        provider: Arc<dyn Provider>,
        sync: SyncConfig,
        redirect_url: String,
    ) -> Self {
        self.provider = Some(provider);
        self.sync = sync;
        self.redirect_url = redirect_url;
        self
    }

    fn pool(&self) -> Result<&PgPool, Status> {
        self.pool
            .as_ref()
            .ok_or_else(|| Status::unavailable("this needs a database"))
    }

    fn bank(&self) -> Result<&Arc<dyn Provider>, Status> {
        self.provider
            .as_ref()
            .ok_or_else(|| Status::failed_precondition("no bank configured"))
    }

    /// The pool, the caller's access and its narrowed view, or the status
    /// that stops the RPC. Every read starts here.
    pub(crate) async fn read_context(
        &self,
        request: &Request<impl Sized>,
        party_ids: &[String],
    ) -> Result<(&PgPool, Access, Access), Status> {
        let pool = self.pool()?;
        let access = self.access(request).await?;
        let narrow = parse_uuids(party_ids, "party_ids")?;
        let view = store::view(&access, &narrow);
        Ok((pool, access, view))
    }

    fn store(&self) -> Result<&Arc<dyn Store>, Status> {
        self.store
            .as_ref()
            .ok_or_else(|| Status::unavailable("no store configured"))
    }

    /// Who is calling, from the claims Envoy verified and forwarded.
    ///
    /// Envoy strips `x-jwt-payload` from anything a client sends and is the
    /// only way in, so its presence means the token was checked. Nothing here
    /// verifies anything; an absent or unparsable header is simply not
    /// authenticated.
    fn principal<T>(request: &Request<T>) -> Result<Principal, Status> {
        let headers = request.metadata().clone().into_headers();
        Principal::from_headers(&headers, &[])
            .ok_or_else(|| Status::unauthenticated("no verified caller"))
    }

    /// The caller's readable parties, provisioning the user on first sight.
    ///
    /// With a database this reads real grants. Without one -- a chaos stack --
    /// it uses the grants the stack was built with, keyed by subject. Either
    /// way the set comes from the verified subject and never from the request.
    pub(crate) async fn access(&self, request: &Request<impl Sized>) -> Result<Access, Status> {
        let principal = Self::principal(request)?;
        if let Some(pool) = &self.pool {
            return Access::resolve(pool, &principal.sub, None, "")
                .await
                .map_err(status_of);
        }
        let parties = self
            .grants
            .iter()
            .find(|(subject, _)| *subject == principal.sub)
            .map(|(_, parties)| parties.clone())
            .unwrap_or_default();
        Ok(Access::for_parties(tbd_db::UserId(Uuid::nil()), parties))
    }

    /// Count the request, start its timer and apply any injected fault
    /// before real work.
    pub(crate) async fn admit(&self, route: &'static str) -> Result<RequestTimer, Status> {
        self.runtime.stats.request();
        let mut timer = RequestTimer::start("grpc", route);
        if let Err(fault) = self.runtime.fault.apply().await {
            self.runtime.stats.failure();
            metrics::counter!(names::FAULTS_INJECTED_TOTAL, "kind" => format!("{:?}", fault.kind).to_lowercase())
                .increment(1);
            let status = status_from(fault);
            timer.set_status(format!("{:?}", status.code()));
            return Err(status);
        }
        Ok(timer)
    }

    pub(crate) fn reject(&self, timer: &mut RequestTimer, status: Status) -> Status {
        self.runtime.stats.failure();
        timer.set_status(format!("{:?}", status.code()));
        status
    }
}

/// The listing filter from the request, or why it is malformed.
fn transaction_filter(req: &ListTransactionsRequest) -> Result<TransactionFilter, Status> {
    let mut filter = TransactionFilter {
        offset: req.offset,
        ..TransactionFilter::default()
    };
    if !req.month.is_empty() {
        if req.month.len() != 7
            || chrono::NaiveDate::parse_from_str(&format!("{}-01", req.month), "%Y-%m-%d").is_err()
        {
            return Err(Status::invalid_argument("month: want YYYY-MM"));
        }
        filter.month = Some(req.month.clone());
    }
    if !req.category_id.is_empty() {
        filter.category = Some(if req.category_id == "none" {
            None
        } else {
            Some(
                Uuid::parse_str(&req.category_id)
                    .map_err(|_| Status::invalid_argument("category_id: not a uuid"))?,
            )
        });
    }
    if !req.account_id.is_empty() {
        filter.account_id = Some(
            Uuid::parse_str(&req.account_id)
                .map_err(|_| Status::invalid_argument("account_id: not a uuid"))?,
        );
    }
    if !req.search.trim().is_empty() {
        filter.search = Some(req.search.trim().to_owned());
    }
    Ok(filter)
}

/// The one place a store error becomes a status.
pub(crate) fn status_of(e: DbError) -> Status {
    match e {
        DbError::NotFound { what } => Status::not_found(what),
        DbError::Invalid { field, reason } => {
            Status::invalid_argument(format!("invalid {field}: {reason}"))
        }
        DbError::Conflict { reason } => Status::aborted(reason),
        // Retryable, and the cause is ours: do not hand the caller the detail.
        DbError::Unavailable { .. } => Status::unavailable("store unavailable"),
        DbError::Internal(_) => Status::internal("store error"),
    }
}

/// Map a transport-neutral fault onto a gRPC status.
fn status_from(fault: Fault) -> Status {
    let code = match fault.kind {
        ErrorKind::Unavailable => Code::Unavailable,
        ErrorKind::Internal => Code::Internal,
        ErrorKind::Overloaded => Code::ResourceExhausted,
        ErrorKind::Timeout => Code::DeadlineExceeded,
    };
    Status::new(code, fault.message)
}

#[tonic::async_trait]
impl FinanceService for Finance {
    async fn ping(&self, request: Request<PingRequest>) -> Result<Response<PingResponse>, Status> {
        let mut timer = self.admit("FinanceService/Ping").await?;
        let message = request.into_inner().message;
        if message.len() > self.ping.max_message_len {
            return Err(self.reject(
                &mut timer,
                Status::invalid_argument(format!(
                    "message longer than {} bytes",
                    self.ping.max_message_len
                )),
            ));
        }
        tracing::debug!(len = message.len(), "ping (stub)");
        Ok(Response::new(PingResponse {
            message,
            version: tbd_common::VERSION.to_owned(),
            stub: true,
        }))
    }

    async fn list_transactions(
        &self,
        request: Request<ListTransactionsRequest>,
    ) -> Result<Response<ListTransactionsResponse>, Status> {
        let mut timer = self.admit("FinanceService/ListTransactions").await?;
        let access = match self.access(&request).await {
            Ok(access) => access,
            Err(status) => return Err(self.reject(&mut timer, status)),
        };
        let req = request.into_inner();

        // A malformed id is a bad request. It is *not* treated as "no such
        // party", so a caller cannot probe which ids are well formed and which
        // exist by watching the error change.
        let mut narrow = Vec::with_capacity(req.party_ids.len());
        for id in &req.party_ids {
            match Uuid::parse_str(id) {
                Ok(id) => narrow.push(id),
                Err(_) => {
                    return Err(self.reject(
                        &mut timer,
                        Status::invalid_argument("party_ids: not a uuid"),
                    ));
                }
            }
        }

        let limit = if req.limit == 0 {
            store::DEFAULT_LIMIT
        } else {
            req.limit
        };
        let store = match self.store() {
            Ok(store) => store,
            Err(status) => return Err(self.reject(&mut timer, status)),
        };
        let filter = match transaction_filter(&req) {
            Ok(f) => f,
            Err(status) => return Err(self.reject(&mut timer, status)),
        };
        let rows = match store.transactions(&access, &narrow, &filter, limit).await {
            Ok(rows) => rows,
            Err(e) => return Err(self.reject(&mut timer, status_of(e))),
        };

        let view = if narrow.is_empty() {
            access.clone()
        } else {
            access.narrow(&narrow)
        };
        tracing::debug!(
            rows = rows.len(),
            parties = view.party_ids().len(),
            "list_transactions"
        );
        Ok(Response::new(ListTransactionsResponse {
            transactions: rows.into_iter().map(transaction_proto).collect(),
            party_ids: view.party_ids().iter().map(ToString::to_string).collect(),
        }))
    }

    async fn monthly_summary(
        &self,
        request: Request<MonthlySummaryRequest>,
    ) -> Result<Response<MonthlySummaryResponse>, Status> {
        let mut timer = self.admit("FinanceService/MonthlySummary").await?;
        let req = request.get_ref().clone();
        let (pool, _, view) = match self.read_context(&request, &req.party_ids).await {
            Ok(c) => c,
            Err(status) => return Err(self.reject(&mut timer, status)),
        };
        let from = if req.from_month.is_empty() {
            None
        } else {
            Some(req.from_month.as_str())
        };
        let rows =
            match categorise::monthly_summary(pool, view.party_ids(), req.include_internal, from)
                .await
            {
                Ok(rows) => rows,
                Err(e) => return Err(self.reject(&mut timer, status_of(e))),
            };
        Ok(Response::new(MonthlySummaryResponse {
            rows: rows
                .into_iter()
                .map(|r| SummaryRow {
                    month: r.month,
                    party_id: r.party_id.to_string(),
                    category_id: r.category_id.map(|c| c.to_string()).unwrap_or_default(),
                    category: r.category.unwrap_or_default(),
                    kind: r.kind.unwrap_or_default(),
                    currency: r.currency,
                    total_minor: r.total_minor,
                    count: u32::try_from(r.count).unwrap_or(u32::MAX),
                    internal: r.internal,
                })
                .collect(),
            party_ids: view.party_ids().iter().map(ToString::to_string).collect(),
        }))
    }

    async fn list_parties(
        &self,
        request: Request<ListPartiesRequest>,
    ) -> Result<Response<ListPartiesResponse>, Status> {
        let mut timer = self.admit("FinanceService/ListParties").await?;
        let (pool, access, _) = match self.read_context(&request, &[]).await {
            Ok(c) => c,
            Err(status) => return Err(self.reject(&mut timer, status)),
        };
        let rows = match money::parties(pool, &access).await {
            Ok(rows) => rows,
            Err(e) => return Err(self.reject(&mut timer, status_of(e))),
        };
        Ok(Response::new(ListPartiesResponse {
            parties: rows
                .into_iter()
                .map(|p| Party {
                    id: p.party.id.0.to_string(),
                    kind: format!("{:?}", p.party.kind).to_lowercase(),
                    display_name: p.party.display_name,
                    capability: format!("{:?}", p.capability).to_lowercase(),
                })
                .collect(),
        }))
    }

    async fn list_accounts(
        &self,
        request: Request<ListAccountsRequest>,
    ) -> Result<Response<ListAccountsResponse>, Status> {
        let mut timer = self.admit("FinanceService/ListAccounts").await?;
        let party_ids = request.get_ref().party_ids.clone();
        let (pool, _, view) = match self.read_context(&request, &party_ids).await {
            Ok(c) => c,
            Err(status) => return Err(self.reject(&mut timer, status)),
        };
        let (accounts, balances) = match (
            money::accounts(pool, &view).await,
            money::latest_balances(pool, &view).await,
        ) {
            (Ok(a), Ok(b)) => (a, b),
            (Err(e), _) | (_, Err(e)) => return Err(self.reject(&mut timer, status_of(e))),
        };
        Ok(Response::new(ListAccountsResponse {
            accounts: accounts
                .into_iter()
                .map(|a| account_proto(a, &balances))
                .collect(),
        }))
    }

    async fn refresh_account(
        &self,
        request: Request<RefreshAccountRequest>,
    ) -> Result<Response<RefreshAccountResponse>, Status> {
        let mut timer = self.admit("FinanceService/RefreshAccount").await?;
        let Ok(account) = Uuid::parse_str(&request.get_ref().account_id) else {
            return Err(self.reject(
                &mut timer,
                Status::invalid_argument("account_id: not a uuid"),
            ));
        };
        let (pool, access, _) = match self.read_context(&request, &[]).await {
            Ok(c) => c,
            Err(status) => return Err(self.reject(&mut timer, status)),
        };
        let bank = match self.bank() {
            Ok(b) => b,
            Err(status) => return Err(self.reject(&mut timer, status)),
        };
        if let Err(e) = money::account_party(pool, &access, account).await {
            return Err(self.reject(&mut timer, status_of(e)));
        }
        // The person's address, as Envoy saw it and the gateway forwarded it.
        // With it the fetch is attended and outside the bank's allowance;
        // without it, it spends the reserve like before.
        let psu = request
            .metadata()
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.split(',').next())
            .map(str::trim)
            .filter(|ip| !ip.is_empty())
            .map(|ip| crate::banking::Psu {
                ip: ip.to_owned(),
                user_agent: request
                    .metadata()
                    .get("user-agent")
                    .and_then(|v| v.to_str().ok())
                    .map(str::to_owned),
            });
        let attended = psu.is_some();
        let syncer = Syncer::new(pool.clone(), Arc::clone(bank), self.sync.clone());
        let result = match syncer.refresh(account, chrono::Utc::now(), psu).await {
            Ok(r) => r,
            Err(crate::sync::SyncError::NotFound) => {
                return Err(self.reject(&mut timer, Status::not_found("account")));
            }
            Err(crate::sync::SyncError::Db(e)) => return Err(self.reject(&mut timer, status_of(e))),
        };
        Ok(Response::new(match result {
            Ok(Outcome::Ok {
                inserted,
                duplicates,
                booked,
                ..
            }) => RefreshAccountResponse {
                outcome: "ok".into(),
                skipped: String::new(),
                inserted: u32::try_from(inserted).unwrap_or(u32::MAX),
                booked: u32::try_from(booked).unwrap_or(u32::MAX),
                duplicates: u32::try_from(duplicates).unwrap_or(u32::MAX),
                attended,
            },
            Ok(outcome) => RefreshAccountResponse {
                outcome: match outcome {
                    Outcome::RateLimited => "rate_limited",
                    Outcome::ConsentInvalid => "consent_invalid",
                    Outcome::Transport => "transport",
                    _ => "error",
                }
                .into(),
                ..RefreshAccountResponse::default()
            },
            Err(skipped) => RefreshAccountResponse {
                outcome: "skipped".into(),
                skipped: match skipped {
                    Skipped::BudgetSpent => "budget_spent",
                    Skipped::BackingOff => "backing_off",
                    Skipped::NoConsent => "no_consent",
                    Skipped::Busy => "busy",
                }
                .into(),
                ..RefreshAccountResponse::default()
            },
        }))
    }

    async fn set_account_sync(
        &self,
        request: Request<SetAccountSyncRequest>,
    ) -> Result<Response<SetAccountSyncResponse>, Status> {
        let mut timer = self.admit("FinanceService/SetAccountSync").await?;
        let req = request.get_ref().clone();
        let Ok(account) = Uuid::parse_str(&req.account_id) else {
            return Err(self.reject(
                &mut timer,
                Status::invalid_argument("account_id: not a uuid"),
            ));
        };
        let (pool, access, view) = match self.read_context(&request, &[]).await {
            Ok(c) => c,
            Err(status) => return Err(self.reject(&mut timer, status)),
        };
        let row = match money::set_sync_enabled(pool, &access, account, req.enabled).await {
            Ok(row) => row,
            Err(e) => return Err(self.reject(&mut timer, status_of(e))),
        };
        let balances = match money::latest_balances(pool, &view).await {
            Ok(b) => b,
            Err(e) => return Err(self.reject(&mut timer, status_of(e))),
        };
        Ok(Response::new(SetAccountSyncResponse {
            account: Some(account_proto(row, &balances)),
        }))
    }

    async fn list_categories(
        &self,
        request: Request<ListCategoriesRequest>,
    ) -> Result<Response<ListCategoriesResponse>, Status> {
        let mut timer = self.admit("FinanceService/ListCategories").await?;
        let party_ids = request.get_ref().party_ids.clone();
        let (pool, _, view) = match self.read_context(&request, &party_ids).await {
            Ok(c) => c,
            Err(status) => return Err(self.reject(&mut timer, status)),
        };
        let rows = match money::categories(pool, &view).await {
            Ok(rows) => rows,
            Err(e) => return Err(self.reject(&mut timer, status_of(e))),
        };
        Ok(Response::new(ListCategoriesResponse {
            categories: rows.into_iter().map(category_proto).collect(),
        }))
    }

    async fn get_transaction(
        &self,
        request: Request<GetTransactionRequest>,
    ) -> Result<Response<GetTransactionResponse>, Status> {
        let mut timer = self.admit("FinanceService/GetTransaction").await?;
        let Ok(id) = Uuid::parse_str(&request.get_ref().id) else {
            return Err(self.reject(&mut timer, Status::invalid_argument("id: not a uuid")));
        };
        let (_, access, _) = match self.read_context(&request, &[]).await {
            Ok(c) => c,
            Err(status) => return Err(self.reject(&mut timer, status)),
        };
        let store = match self.store() {
            Ok(store) => store,
            Err(status) => return Err(self.reject(&mut timer, status)),
        };
        let filter = TransactionFilter {
            id: Some(id),
            ..TransactionFilter::default()
        };
        // The store already narrows to the grant, so a foreign row is simply
        // absent: not-found, with nothing to tell it apart from a wrong id.
        let row = match store.transactions(&access, &[], &filter, 1).await {
            Ok(mut rows) => rows.pop(),
            Err(e) => return Err(self.reject(&mut timer, status_of(e))),
        };
        let Some(row) = row else {
            return Err(self.reject(&mut timer, Status::not_found("transaction")));
        };
        Ok(Response::new(GetTransactionResponse {
            transaction: Some(transaction_proto(row)),
        }))
    }

    async fn upsert_category(
        &self,
        request: Request<UpsertCategoryRequest>,
    ) -> Result<Response<UpsertCategoryResponse>, Status> {
        let mut timer = self.admit("FinanceService/UpsertCategory").await?;
        let req = request.get_ref().clone();
        let Ok(party_id) = Uuid::parse_str(&req.party_id) else {
            return Err(self.reject(&mut timer, Status::invalid_argument("party_id: not a uuid")));
        };
        let id = if req.id.trim().is_empty() {
            None
        } else {
            match Uuid::parse_str(&req.id) {
                Ok(id) => Some(id),
                Err(_) => {
                    return Err(self.reject(&mut timer, Status::invalid_argument("id: not a uuid")));
                }
            }
        };
        let (pool, access, _) = match self.read_context(&request, &[]).await {
            Ok(c) => c,
            Err(status) => return Err(self.reject(&mut timer, status)),
        };
        let input = money::CategoryInput {
            id,
            party_id,
            name: req.name,
            kind: req.kind,
            deductible: req.deductible,
            archived: req.archived,
        };
        let (row, applied) = match money::upsert_category(pool, &access, input).await {
            Ok(r) => r,
            Err(e) => return Err(self.reject(&mut timer, status_of(e))),
        };
        Ok(Response::new(UpsertCategoryResponse {
            category: Some(category_proto(row)),
            categorised: u32::try_from(applied.categorised).unwrap_or(u32::MAX),
            unmatched: u32::try_from(applied.unmatched).unwrap_or(u32::MAX),
        }))
    }

    async fn declare_category(
        &self,
        request: Request<DeclareCategoryRequest>,
    ) -> Result<Response<DeclareCategoryResponse>, Status> {
        let mut timer = self.admit("FinanceService/DeclareCategory").await?;
        let req = request.get_ref().clone();
        let (Ok(transaction), Ok(category)) = (
            Uuid::parse_str(&req.transaction_id),
            Uuid::parse_str(&req.category_id),
        ) else {
            return Err(self.reject(
                &mut timer,
                Status::invalid_argument("transaction_id, category_id: want uuids"),
            ));
        };
        let (pool, access, _) = match self.read_context(&request, &[]).await {
            Ok(c) => c,
            Err(status) => return Err(self.reject(&mut timer, status)),
        };
        if let Err(e) = money::declare(pool, &access, transaction, category).await {
            return Err(self.reject(&mut timer, status_of(e)));
        }
        let store = match self.store() {
            Ok(store) => store,
            Err(status) => return Err(self.reject(&mut timer, status)),
        };
        let filter = TransactionFilter {
            id: Some(transaction),
            ..TransactionFilter::default()
        };
        let row = match store.transactions(&access, &[], &filter, 1).await {
            Ok(mut rows) => rows.pop(),
            Err(e) => return Err(self.reject(&mut timer, status_of(e))),
        };
        Ok(Response::new(DeclareCategoryResponse {
            transaction: row.map(transaction_proto),
        }))
    }

    async fn list_rules(
        &self,
        request: Request<ListRulesRequest>,
    ) -> Result<Response<ListRulesResponse>, Status> {
        let mut timer = self.admit("FinanceService/ListRules").await?;
        let party_ids = request.get_ref().party_ids.clone();
        let (pool, _, view) = match self.read_context(&request, &party_ids).await {
            Ok(c) => c,
            Err(status) => return Err(self.reject(&mut timer, status)),
        };
        let rows = match money::rules(pool, &view).await {
            Ok(rows) => rows,
            Err(e) => return Err(self.reject(&mut timer, status_of(e))),
        };
        Ok(Response::new(ListRulesResponse {
            rules: rows.into_iter().map(rule_proto).collect(),
        }))
    }

    async fn upsert_rule(
        &self,
        request: Request<UpsertRuleRequest>,
    ) -> Result<Response<UpsertRuleResponse>, Status> {
        let mut timer = self.admit("FinanceService/UpsertRule").await?;
        let req = request.get_ref().clone();
        let input = match rule_input(&req) {
            Ok(i) => i,
            Err(status) => return Err(self.reject(&mut timer, status)),
        };
        let (pool, access, _) = match self.read_context(&request, &[]).await {
            Ok(c) => c,
            Err(status) => return Err(self.reject(&mut timer, status)),
        };
        let (row, applied) = match money::upsert_rule(pool, &access, input).await {
            Ok(r) => r,
            Err(e) => return Err(self.reject(&mut timer, status_of(e))),
        };
        Ok(Response::new(UpsertRuleResponse {
            rule: Some(rule_proto(row)),
            categorised: u32::try_from(applied.categorised).unwrap_or(u32::MAX),
            unmatched: u32::try_from(applied.unmatched).unwrap_or(u32::MAX),
        }))
    }

    async fn start_connection(
        &self,
        request: Request<StartConnectionRequest>,
    ) -> Result<Response<StartConnectionResponse>, Status> {
        let mut timer = self.admit("FinanceService/StartConnection").await?;
        let req = request.get_ref().clone();
        let Ok(party) = Uuid::parse_str(&req.party_id) else {
            return Err(self.reject(&mut timer, Status::invalid_argument("party_id: not a uuid")));
        };
        if req.psu_type != "business" && req.psu_type != "personal" {
            return Err(self.reject(
                &mut timer,
                Status::invalid_argument("psu_type: want business or personal"),
            ));
        }
        let (pool, access, _) = match self.read_context(&request, &[]).await {
            Ok(c) => c,
            Err(status) => return Err(self.reject(&mut timer, status)),
        };
        let bank = match self.bank() {
            Ok(b) => b,
            Err(status) => return Err(self.reject(&mut timer, status)),
        };
        if self.redirect_url.is_empty() {
            return Err(self.reject(
                &mut timer,
                Status::failed_precondition("no redirect_url configured"),
            ));
        }
        // Not yours is not-found, like everything else.
        if let Err(e) = access.require(tbd_db::PartyId(party), "party") {
            return Err(self.reject(&mut timer, status_of(e)));
        }
        let aspsp_name = if req.aspsp_name.is_empty() {
            "Erste & Steiermärkische Bank"
        } else {
            &req.aspsp_name
        };
        let aspsp_country = if req.aspsp_country.is_empty() {
            "HR"
        } else {
            &req.aspsp_country
        };
        match connect::start(
            pool,
            bank.as_ref(),
            party,
            &req.psu_type,
            aspsp_name,
            aspsp_country,
            &self.redirect_url,
        )
        .await
        {
            Ok(started) => Ok(Response::new(StartConnectionResponse {
                connection_id: started.connection_id.to_string(),
                url: started.url,
            })),
            Err(e) => Err(self.reject(&mut timer, connect_status(e))),
        }
    }

    async fn complete_connection(
        &self,
        request: Request<CompleteConnectionRequest>,
    ) -> Result<Response<CompleteConnectionResponse>, Status> {
        let mut timer = self.admit("FinanceService/CompleteConnection").await?;
        let req = request.get_ref().clone();
        if req.state.is_empty() || req.code.is_empty() {
            return Err(self.reject(
                &mut timer,
                Status::invalid_argument("state and code are required"),
            ));
        }
        let (pool, access, _) = match self.read_context(&request, &[]).await {
            Ok(c) => c,
            Err(status) => return Err(self.reject(&mut timer, status)),
        };
        let bank = match self.bank() {
            Ok(b) => b,
            Err(status) => return Err(self.reject(&mut timer, status)),
        };
        // The state names the party; the caller must be allowed that party.
        let party = match money::connection_party_by_state(pool, &access, &req.state).await {
            Ok(p) => p,
            Err(e) => return Err(self.reject(&mut timer, status_of(e))),
        };
        match connect::complete(pool, bank.as_ref(), party, &req.state, &req.code).await {
            Ok(done) => Ok(Response::new(CompleteConnectionResponse {
                connection_id: done.connection_id.to_string(),
                account_ids: done.accounts.iter().map(ToString::to_string).collect(),
            })),
            Err(e) => Err(self.reject(&mut timer, connect_status(e))),
        }
    }

    async fn list_connections(
        &self,
        request: Request<ListConnectionsRequest>,
    ) -> Result<Response<ListConnectionsResponse>, Status> {
        let mut timer = self.admit("FinanceService/ListConnections").await?;
        let party_ids = request.get_ref().party_ids.clone();
        let (pool, _, view) = match self.read_context(&request, &party_ids).await {
            Ok(c) => c,
            Err(status) => return Err(self.reject(&mut timer, status)),
        };
        let rows = match money::connections(pool, &view).await {
            Ok(rows) => rows,
            Err(e) => return Err(self.reject(&mut timer, status_of(e))),
        };
        Ok(Response::new(ListConnectionsResponse {
            connections: rows
                .into_iter()
                .map(|c| Connection {
                    id: c.id.to_string(),
                    party_id: c.party_id.to_string(),
                    provider: c.provider,
                    psu_type: c.psu_type,
                    aspsp_name: c.aspsp_name,
                    status: c.status,
                    valid_until: c.valid_until.map(|t| t.to_rfc3339()).unwrap_or_default(),
                    authorized_at: c.authorized_at.map(|t| t.to_rfc3339()).unwrap_or_default(),
                    accounts: u32::try_from(c.accounts).unwrap_or(u32::MAX),
                    failure: c.failure.unwrap_or_default(),
                    created_at: c.created_at.to_rfc3339(),
                    replaced_by: c.replaced_by.map(|u| u.to_string()).unwrap_or_default(),
                })
                .collect(),
        }))
    }

    async fn delete_connection(
        &self,
        request: Request<DeleteConnectionRequest>,
    ) -> Result<Response<DeleteConnectionResponse>, Status> {
        let mut timer = self.admit("FinanceService/DeleteConnection").await?;
        let Ok(id) = Uuid::parse_str(&request.get_ref().id) else {
            return Err(self.reject(&mut timer, Status::invalid_argument("id: not a uuid")));
        };
        let (pool, access, _) = match self.read_context(&request, &[]).await {
            Ok(c) => c,
            Err(status) => return Err(self.reject(&mut timer, status)),
        };
        let party = match money::connection_party(pool, &access, id).await {
            Ok(p) => p,
            Err(e) => return Err(self.reject(&mut timer, status_of(e))),
        };
        match connect::remove(pool, party, id).await {
            Ok(()) => Ok(Response::new(DeleteConnectionResponse {})),
            Err(e) => Err(self.reject(&mut timer, connect_status(e))),
        }
    }

    async fn get_issuer(
        &self,
        r: Request<GetIssuerRequest>,
    ) -> Result<Response<GetIssuerResponse>, Status> {
        self.rpc_get_issuer(r).await
    }
    async fn upsert_issuer(
        &self,
        r: Request<UpsertIssuerRequest>,
    ) -> Result<Response<UpsertIssuerResponse>, Status> {
        self.rpc_upsert_issuer(r).await
    }
    async fn list_clients(
        &self,
        r: Request<ListClientsRequest>,
    ) -> Result<Response<ListClientsResponse>, Status> {
        self.rpc_list_clients(r).await
    }
    async fn upsert_client(
        &self,
        r: Request<UpsertClientRequest>,
    ) -> Result<Response<UpsertClientResponse>, Status> {
        self.rpc_upsert_client(r).await
    }
    async fn list_invoices(
        &self,
        r: Request<ListInvoicesRequest>,
    ) -> Result<Response<ListInvoicesResponse>, Status> {
        self.rpc_list_invoices(r).await
    }
    async fn get_invoice(
        &self,
        r: Request<GetInvoiceRequest>,
    ) -> Result<Response<GetInvoiceResponse>, Status> {
        self.rpc_get_invoice(r).await
    }
    async fn aging_report(
        &self,
        r: Request<AgingReportRequest>,
    ) -> Result<Response<AgingReportResponse>, Status> {
        self.rpc_aging_report(r).await
    }
    async fn create_invoice(
        &self,
        r: Request<CreateInvoiceRequest>,
    ) -> Result<Response<CreateInvoiceResponse>, Status> {
        self.rpc_create_invoice(r).await
    }
    async fn update_invoice(
        &self,
        r: Request<UpdateInvoiceRequest>,
    ) -> Result<Response<UpdateInvoiceResponse>, Status> {
        self.rpc_update_invoice(r).await
    }
    async fn preview_invoice(
        &self,
        r: Request<PreviewInvoiceRequest>,
    ) -> Result<Response<PreviewInvoiceResponse>, Status> {
        self.rpc_preview_invoice(r).await
    }
    async fn approve_invoice(
        &self,
        r: Request<ApproveInvoiceRequest>,
    ) -> Result<Response<ApproveInvoiceResponse>, Status> {
        self.rpc_approve_invoice(r).await
    }
    async fn cancel_invoice(
        &self,
        r: Request<CancelInvoiceRequest>,
    ) -> Result<Response<CancelInvoiceResponse>, Status> {
        self.rpc_cancel_invoice(r).await
    }
    async fn list_issuers(
        &self,
        r: Request<ListIssuersRequest>,
    ) -> Result<Response<ListIssuersResponse>, Status> {
        self.rpc_list_issuers(r).await
    }
    async fn create_issuer(
        &self,
        r: Request<CreateIssuerRequest>,
    ) -> Result<Response<CreateIssuerResponse>, Status> {
        self.rpc_create_issuer(r).await
    }
    async fn set_default_client(
        &self,
        r: Request<SetDefaultClientRequest>,
    ) -> Result<Response<SetDefaultClientResponse>, Status> {
        self.rpc_set_default_client(r).await
    }
    async fn delete_invoice(
        &self,
        r: Request<DeleteInvoiceRequest>,
    ) -> Result<Response<DeleteInvoiceResponse>, Status> {
        self.rpc_delete_invoice(r).await
    }
    async fn record_payment(
        &self,
        r: Request<RecordPaymentRequest>,
    ) -> Result<Response<RecordPaymentResponse>, Status> {
        self.rpc_record_payment(r).await
    }
    async fn unlink_payment(
        &self,
        r: Request<UnlinkPaymentRequest>,
    ) -> Result<Response<UnlinkPaymentResponse>, Status> {
        self.rpc_unlink_payment(r).await
    }
    async fn get_invoice_document(
        &self,
        r: Request<GetInvoiceDocumentRequest>,
    ) -> Result<Response<GetInvoiceDocumentResponse>, Status> {
        self.rpc_get_invoice_document(r).await
    }
    async fn list_line_templates(
        &self,
        r: Request<ListLineTemplatesRequest>,
    ) -> Result<Response<ListLineTemplatesResponse>, Status> {
        self.rpc_list_line_templates(r).await
    }
    async fn upsert_line_template(
        &self,
        r: Request<UpsertLineTemplateRequest>,
    ) -> Result<Response<UpsertLineTemplateResponse>, Status> {
        self.rpc_upsert_line_template(r).await
    }
    async fn delete_line_template(
        &self,
        r: Request<DeleteLineTemplateRequest>,
    ) -> Result<Response<DeleteLineTemplateResponse>, Status> {
        self.rpc_delete_line_template(r).await
    }

    async fn list_connector_kinds(
        &self,
        r: Request<ListConnectorKindsRequest>,
    ) -> Result<Response<ListConnectorKindsResponse>, Status> {
        self.rpc_list_connector_kinds(r).await
    }
    async fn list_connectors(
        &self,
        r: Request<ListConnectorsRequest>,
    ) -> Result<Response<ListConnectorsResponse>, Status> {
        self.rpc_list_connectors(r).await
    }
    async fn monthly_reconciliation(
        &self,
        r: Request<MonthlyReconciliationRequest>,
    ) -> Result<Response<MonthlyReconciliationResponse>, Status> {
        self.rpc_monthly_reconciliation(r).await
    }
    async fn link_document(
        &self,
        r: Request<LinkDocumentRequest>,
    ) -> Result<Response<LinkDocumentResponse>, Status> {
        self.rpc_link_document(r).await
    }
    async fn unlink_document(
        &self,
        r: Request<UnlinkDocumentRequest>,
    ) -> Result<Response<UnlinkDocumentResponse>, Status> {
        self.rpc_unlink_document(r).await
    }

    async fn set_transaction_note(
        &self,
        r: Request<SetTransactionNoteRequest>,
    ) -> Result<Response<SetTransactionNoteResponse>, Status> {
        self.rpc_set_transaction_note(r).await
    }
    async fn set_counterparty_policy(
        &self,
        r: Request<SetCounterpartyPolicyRequest>,
    ) -> Result<Response<SetCounterpartyPolicyResponse>, Status> {
        self.rpc_set_counterparty_policy(r).await
    }
    async fn delete_counterparty_policy(
        &self,
        r: Request<DeleteCounterpartyPolicyRequest>,
    ) -> Result<Response<DeleteCounterpartyPolicyResponse>, Status> {
        self.rpc_delete_counterparty_policy(r).await
    }
    async fn update_document(
        &self,
        r: Request<UpdateDocumentRequest>,
    ) -> Result<Response<UpdateDocumentResponse>, Status> {
        self.rpc_update_document(r).await
    }
    async fn extract_document(
        &self,
        r: Request<ExtractDocumentRequest>,
    ) -> Result<Response<ExtractDocumentResponse>, Status> {
        self.rpc_extract_document(r).await
    }
    async fn upload_document(
        &self,
        r: Request<UploadDocumentRequest>,
    ) -> Result<Response<UploadDocumentResponse>, Status> {
        self.rpc_upload_document(r).await
    }
    async fn list_filings(
        &self,
        r: Request<ListFilingsRequest>,
    ) -> Result<Response<ListFilingsResponse>, Status> {
        self.rpc_list_filings(r).await
    }
    async fn get_filing(
        &self,
        r: Request<GetFilingRequest>,
    ) -> Result<Response<GetFilingResponse>, Status> {
        self.rpc_get_filing(r).await
    }
    async fn list_mail_templates(
        &self,
        r: Request<ListMailTemplatesRequest>,
    ) -> Result<Response<ListMailTemplatesResponse>, Status> {
        self.rpc_list_mail_templates(r).await
    }
    async fn upsert_mail_template(
        &self,
        r: Request<UpsertMailTemplateRequest>,
    ) -> Result<Response<UpsertMailTemplateResponse>, Status> {
        self.rpc_upsert_mail_template(r).await
    }
    async fn delete_mail_template(
        &self,
        r: Request<DeleteMailTemplateRequest>,
    ) -> Result<Response<DeleteMailTemplateResponse>, Status> {
        self.rpc_delete_mail_template(r).await
    }
    async fn send_mail(
        &self,
        r: Request<SendMailRequest>,
    ) -> Result<Response<SendMailResponse>, Status> {
        self.rpc_send_mail(r).await
    }
    async fn list_mail(
        &self,
        r: Request<ListMailRequest>,
    ) -> Result<Response<ListMailResponse>, Status> {
        self.rpc_list_mail(r).await
    }
    async fn get_mail(
        &self,
        r: Request<GetMailRequest>,
    ) -> Result<Response<GetMailResponse>, Status> {
        self.rpc_get_mail(r).await
    }
    type WatchConnectorsStream = Pin<
        Box<
            dyn tokio_stream::Stream<Item = Result<WatchConnectorsResponse, Status>>
                + Send
                + 'static,
        >,
    >;
    async fn watch_connectors(
        &self,
        r: Request<WatchConnectorsRequest>,
    ) -> Result<Response<Self::WatchConnectorsStream>, Status> {
        let stream = self.rpc_watch_connectors(r).await?.into_inner();
        Ok(Response::new(Box::pin(stream)))
    }
    async fn start_connector(
        &self,
        r: Request<StartConnectorRequest>,
    ) -> Result<Response<StartConnectorResponse>, Status> {
        self.rpc_start_connector(r).await
    }
    async fn complete_connector(
        &self,
        r: Request<CompleteConnectorRequest>,
    ) -> Result<Response<CompleteConnectorResponse>, Status> {
        self.rpc_complete_connector(r).await
    }
    async fn test_connector(
        &self,
        r: Request<TestConnectorRequest>,
    ) -> Result<Response<TestConnectorResponse>, Status> {
        self.rpc_test_connector(r).await
    }
    async fn sync_connector(
        &self,
        r: Request<SyncConnectorRequest>,
    ) -> Result<Response<SyncConnectorResponse>, Status> {
        self.rpc_sync_connector(r).await
    }
    async fn configure_connector(
        &self,
        r: Request<ConfigureConnectorRequest>,
    ) -> Result<Response<ConfigureConnectorResponse>, Status> {
        self.rpc_configure_connector(r).await
    }
    async fn delete_connector(
        &self,
        r: Request<DeleteConnectorRequest>,
    ) -> Result<Response<DeleteConnectorResponse>, Status> {
        self.rpc_delete_connector(r).await
    }
    async fn list_connector_runs(
        &self,
        r: Request<ListConnectorRunsRequest>,
    ) -> Result<Response<ListConnectorRunsResponse>, Status> {
        self.rpc_list_connector_runs(r).await
    }
    async fn list_documents(
        &self,
        r: Request<ListDocumentsRequest>,
    ) -> Result<Response<ListDocumentsResponse>, Status> {
        self.rpc_list_documents(r).await
    }
    async fn get_document(
        &self,
        r: Request<GetDocumentRequest>,
    ) -> Result<Response<GetDocumentResponse>, Status> {
        self.rpc_get_document(r).await
    }
}

/// One account row and its latest balances, on the wire.
fn account_proto(a: money::AccountRow, balances: &[money::BalanceRow]) -> Account {
    let today = chrono::Utc::now().date_naive();
    let used = if a.sync_budget_day == Some(today) {
        a.sync_budget_used
    } else {
        0
    };
    Account {
        id: a.id.to_string(),
        party_id: a.party_id.to_string(),
        connection_id: a.connection_id.map(|c| c.to_string()).unwrap_or_default(),
        provider: a.provider,
        iban: a.iban.unwrap_or_default(),
        currency: a.currency,
        name: a.name,
        sync_enabled: a.sync_enabled,
        last_synced_at: a.last_synced_at.map(|t| t.to_rfc3339()).unwrap_or_default(),
        last_sync_status: a.last_sync_status.unwrap_or_default(),
        last_sync_error: a.last_sync_error.unwrap_or_default(),
        last_booked_through: a
            .last_booked_through
            .map(|d| d.to_string())
            .unwrap_or_default(),
        sync_backoff_until: a
            .sync_backoff_until
            .map(|t| t.to_rfc3339())
            .unwrap_or_default(),
        sync_budget_used: u32::try_from(used).unwrap_or(0),
        balances: balances
            .iter()
            .filter(|b| b.account_id == a.id)
            .map(|b| Balance {
                balance_type: b.balance_type.clone(),
                amount_minor: b.amount_minor,
                currency: b.currency.clone(),
                observed_at: b.observed_at.to_rfc3339(),
            })
            .collect(),
    }
}

/// Uuids from strings, or a bad request naming the field. A malformed id is
/// never "not found", so a caller cannot learn which ids are well formed by
/// watching the error change.
fn parse_uuids(ids: &[String], field: &str) -> Result<Vec<Uuid>, Status> {
    ids.iter()
        .map(|id| {
            Uuid::parse_str(id)
                .map_err(|_| Status::invalid_argument(format!("{field}: not a uuid")))
        })
        .collect()
}

fn transaction_proto(t: store::Transaction) -> Transaction {
    Transaction {
        id: t.id.to_string(),
        account_id: t.account_id.to_string(),
        party_id: t.party_id.to_string(),
        status: t.status.to_uppercase(),
        amount_minor: t.amount_minor,
        currency: t.currency,
        scale: u32::try_from(t.scale).unwrap_or(2),
        booking_date: t.booking_date.map(|d| d.to_string()).unwrap_or_default(),
        counterparty_name: t.counterparty_name.unwrap_or_default(),
        remittance: t.remittance.unwrap_or_default(),
        value_date: t.value_date.map(|d| d.to_string()).unwrap_or_default(),
        counterparty_iban: t.counterparty_iban.unwrap_or_default(),
        category_id: t.category_id.map(|c| c.to_string()).unwrap_or_default(),
        category: t.category.unwrap_or_default(),
        category_source: t.category_source.unwrap_or_default(),
        internal: t.internal,
        reference_number: t.reference_number.unwrap_or_default(),
        entry_reference: t.entry_reference.unwrap_or_default(),
        category_rule_id: t
            .category_rule_id
            .map(|r| r.to_string())
            .unwrap_or_default(),
        categorised_at: t.categorised_at.map(|t| t.to_rfc3339()).unwrap_or_default(),
        account_name: t.account_name.unwrap_or_default(),
        raw: t.raw.unwrap_or_default(),
    }
}

fn category_proto(c: money::CategoryRow) -> Category {
    Category {
        id: c.id.to_string(),
        party_id: c.party_id.to_string(),
        slug: c.slug,
        name: c.name,
        kind: c.kind,
        deductible: c.deductible,
        archived: c.archived_at.is_some(),
    }
}

fn rule_proto(r: money::RuleRow) -> Rule {
    Rule {
        id: r.id.to_string(),
        party_id: r.party_id.to_string(),
        priority: r.priority,
        name: r.name,
        category_id: r.category_id.to_string(),
        match_counterparty_like: r.match_counterparty_like.unwrap_or_default(),
        match_counterparty_iban: r.match_counterparty_iban.unwrap_or_default(),
        match_remittance_like: r.match_remittance_like.unwrap_or_default(),
        match_currency: r.match_currency.unwrap_or_default(),
        match_credit_debit: r.match_credit_debit.unwrap_or_default(),
        enabled: r.enabled,
        hits: r.hits,
    }
}

fn rule_input(req: &UpsertRuleRequest) -> Result<money::RuleInput, Status> {
    let opt = |s: &str| {
        if s.trim().is_empty() {
            None
        } else {
            Some(s.to_owned())
        }
    };
    Ok(money::RuleInput {
        id: if req.id.is_empty() {
            None
        } else {
            Some(Uuid::parse_str(&req.id).map_err(|_| Status::invalid_argument("id: not a uuid"))?)
        },
        party_id: Uuid::parse_str(&req.party_id)
            .map_err(|_| Status::invalid_argument("party_id: not a uuid"))?,
        priority: req.priority,
        name: req.name.clone(),
        category_id: Uuid::parse_str(&req.category_id)
            .map_err(|_| Status::invalid_argument("category_id: not a uuid"))?,
        match_counterparty_like: opt(&req.match_counterparty_like),
        match_counterparty_iban: opt(&req.match_counterparty_iban),
        match_remittance_like: opt(&req.match_remittance_like),
        match_currency: opt(&req.match_currency),
        match_credit_debit: opt(&req.match_credit_debit),
        enabled: req.enabled,
    })
}

fn connect_status(e: connect::ConnectError) -> Status {
    match e {
        connect::ConnectError::NotFound => Status::not_found("connection"),
        connect::ConnectError::AlreadyCompleted => {
            Status::already_exists("that authorization was already completed")
        }
        connect::ConnectError::Provider(p) => Status::unavailable(format!("bank: {p}")),
        connect::ConnectError::Db(d) => status_of(d),
    }
}
