//! Connectors over gRPC, with a mock kind: link, refuse a foreign state, test,
//! sync with dedupe, and credentials that are never plaintext at rest.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use base64::Engine as _;
use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sqlx::PgPool;
use tbd_db::{Capability, PartyId, UserId, create_org, ensure_user, grant};
use tbd_finance::connectors::{
    Auth, AuthContext, Connector, ConnectorError, Found, Kind, Linked, Pull,
};
use tbd_proto::finance::v1::{
    CompleteConnectorRequest, ListConnectorKindsRequest, ListConnectorRunsRequest,
    ListConnectorsRequest, ListDocumentsRequest, StartConnectorRequest, SyncConnectorRequest,
    TestConnectorRequest,
};
use tonic::{Code, Request, metadata::MetadataValue};
use uuid::Uuid;

use crate::support::start_with_kinds;

const OWNER: &str = "conn-owner";
const READER: &str = "conn-reader";

fn as_caller<T>(subject: &str, message: T) -> Request<T> {
    let claims = json!({ "sub": subject, "scp": ["tbd.finance"] });
    let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(claims.to_string());
    let mut request = Request::new(message);
    request.metadata_mut().insert(
        "x-jwt-payload",
        MetadataValue::try_from(encoded.as_str()).unwrap(),
    );
    request
}

/// A provider that links any code, and serves a fixed set of receipts.
#[derive(Debug, Clone)]
struct MockKind {
    pulls: Arc<Mutex<usize>>,
    refuse: bool,
}

#[async_trait]
impl Connector for MockKind {
    fn kind(&self) -> Kind {
        Kind {
            name: "mock",
            label: "Mock mailbox",
            description: "Two receipts, always.",
            auth: Auth::Oauth,
            consent_note: "nothing",
            configured: true,
        }
    }
    async fn start(&self, ctx: &AuthContext) -> Result<String, ConnectorError> {
        Ok(format!(
            "https://mock.test/auth?state={}&redirect={}",
            ctx.state, ctx.redirect_url
        ))
    }
    async fn complete(&self, _ctx: &AuthContext, code: &str) -> Result<Linked, ConnectorError> {
        if code == "bad" {
            return Err(ConnectorError::Provider("refused".into()));
        }
        Ok(Linked {
            credentials: json!({ "refresh_token": "SECRET-REFRESH-TOKEN" }),
            external_id: "inbox@example.test".into(),
            label: "inbox@example.test".into(),
        })
    }
    async fn test(&self, credentials: &Value) -> Result<String, ConnectorError> {
        if self.refuse {
            return Err(ConnectorError::Unlinked("revoked".into()));
        }
        assert_eq!(
            credentials["refresh_token"], "SECRET-REFRESH-TOKEN",
            "the store opened the credential"
        );
        Ok("inbox@example.test · 2 messages".into())
    }
    async fn pull(
        &self,
        _credentials: &Value,
        _config: &Value,
        _since: DateTime<Utc>,
        seen: &(dyn for<'a> Fn(&'a str) -> bool + Sync),
    ) -> Result<Pull, ConnectorError> {
        *self.pulls.lock().unwrap() += 1;
        let receipt = |id: &str, name: &str, body: &str| Found {
            external_ref: format!("{id}:{name}"),
            filename: name.into(),
            content_type: "application/pdf".into(),
            bytes: body.as_bytes().to_vec(),
            subject: format!("Your receipt {id}"),
            sender: "billing@vendor.test".into(),
            received_at: Some(Utc::now()),
        };
        // The provider is not asked for a message already pulled.
        let mut out = Vec::new();
        if !seen("m1") {
            out.push(receipt("m1", "Hetzner.pdf", "%PDF-hetzner"));
        }
        if !seen("m2") {
            out.push(receipt("m2", "Cloudflare.pdf", "%PDF-cloudflare"));
            // The same bytes from a second message: one document, two sources.
            out.push(receipt("m3", "Cloudflare-again.pdf", "%PDF-cloudflare"));
        }
        Ok(Pull {
            found: out,
            complete: true,
        })
    }
}

/// Start a sync and poll its run to the end. The call returns as soon as the
/// run is open, because the pull is detached from it.
async fn sync_and_wait(
    c: &mut tbd_proto::finance::v1::finance_service_client::FinanceServiceClient<
        tonic::transport::Channel,
    >,
    id: &str,
) -> tbd_proto::finance::v1::ConnectorRun {
    let started = c
        .sync_connector(as_caller(OWNER, SyncConnectorRequest { id: id.to_owned() }))
        .await
        .unwrap()
        .into_inner()
        .run
        .unwrap();
    assert_eq!(started.outcome, "", "a run is open, not decided");
    for _ in 0..200 {
        let runs = c
            .list_connector_runs(as_caller(
                OWNER,
                ListConnectorRunsRequest { id: id.to_owned() },
            ))
            .await
            .unwrap()
            .into_inner()
            .runs;
        if let Some(run) = runs
            .iter()
            .find(|r| r.id == started.id && !r.outcome.is_empty())
        {
            return run.clone();
        }
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    }
    panic!("run {} never finished", started.id);
}

async fn seed(pool: &PgPool) -> (Uuid, Uuid) {
    let owner: UserId = ensure_user(pool, OWNER, None, "Owner").await.unwrap();
    let reader = ensure_user(pool, READER, None, "Reader").await.unwrap();
    let company = create_org(pool, "Inorbit d.o.o.", None, Some("HR"), true)
        .await
        .unwrap();
    grant(
        pool,
        owner,
        PartyId(company.0),
        Capability::Own,
        Some(owner),
        None,
    )
    .await
    .unwrap();
    grant(
        pool,
        reader,
        PartyId(company.0),
        Capability::Read,
        Some(owner),
        None,
    )
    .await
    .unwrap();
    (owner.0, company.0)
}

fn kinds(refuse: bool) -> (tbd_finance::connectors::KindsFactory, Arc<Mutex<usize>>) {
    let pulls = Arc::new(Mutex::new(0));
    let p = Arc::clone(&pulls);
    (
        Arc::new(move || {
            vec![Box::new(MockKind {
                pulls: Arc::clone(&p),
                refuse,
            }) as Box<dyn Connector>]
        }),
        pulls,
    )
}

fn state_of(url: &str) -> String {
    url::Url::parse(url)
        .unwrap()
        .query_pairs()
        .find(|(k, _)| k == "state")
        .unwrap()
        .1
        .into_owned()
}

/// Start and complete a mock link for `party` as the owner.
async fn link(server: &crate::support::Server, party: Uuid) -> tbd_proto::finance::v1::Connector {
    let mut c = server.client().await;
    let started = c
        .start_connector(as_caller(
            OWNER,
            StartConnectorRequest {
                party_id: party.to_string(),
                kind: "mock".into(),
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert!(started.url.contains("finance.test/connectors/callback"));
    c.complete_connector(as_caller(
        OWNER,
        CompleteConnectorRequest {
            state: state_of(&started.url),
            code: "ok".into(),
        },
    ))
    .await
    .unwrap()
    .into_inner()
    .connector
    .unwrap()
}

#[tokio::test]
async fn a_link_seals_its_credential_and_a_foreign_state_is_not_found() {
    let (factory, _) = kinds(false);
    let (server, pool) = start_with_kinds(factory).await;
    let (personal, _company) = seed(&pool).await;
    let mut c = server.client().await;

    let listed = c
        .list_connector_kinds(as_caller(OWNER, ListConnectorKindsRequest {}))
        .await
        .unwrap()
        .into_inner()
        .kinds;
    assert_eq!(listed[0].name, "mock");
    assert!(listed[0].configured);

    // A state for the owner's personal party, presented by the reader (who is
    // granted the company only): not-found, and the code never reaches the kind.
    let started = c
        .start_connector(as_caller(
            OWNER,
            StartConnectorRequest {
                party_id: personal.to_string(),
                kind: "mock".into(),
            },
        ))
        .await
        .unwrap()
        .into_inner();
    let state = state_of(&started.url);
    let e = c
        .complete_connector(as_caller(
            READER,
            CompleteConnectorRequest {
                state: state.clone(),
                code: "stolen".into(),
            },
        ))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::NotFound, "{e}");
    let e = c
        .start_connector(as_caller(
            READER,
            StartConnectorRequest {
                party_id: personal.to_string(),
                kind: "mock".into(),
            },
        ))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::NotFound);

    let linked = c
        .complete_connector(as_caller(
            OWNER,
            CompleteConnectorRequest {
                state: state.clone(),
                code: "ok".into(),
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .connector
        .unwrap();
    assert_eq!(linked.status, "linked");
    assert_eq!(linked.label, "inbox@example.test");

    // At rest: sealed; the blob does not contain the token.
    let (blob,): (Vec<u8>,) =
        sqlx::query_as("select credentials from finance.connectors where id = $1")
            .bind(Uuid::parse_str(&linked.id).unwrap())
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(!String::from_utf8_lossy(&blob).contains("SECRET-REFRESH-TOKEN"));

    // The same state cannot be completed twice.
    let e = c
        .complete_connector(as_caller(
            OWNER,
            CompleteConnectorRequest {
                state,
                code: "ok".into(),
            },
        ))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::FailedPrecondition, "{e}");

    // The reader sees nothing of the personal party's connectors.
    let mine = c
        .list_connectors(as_caller(READER, ListConnectorsRequest::default()))
        .await
        .unwrap()
        .into_inner()
        .connectors;
    assert!(mine.iter().all(|x| x.party_id != personal.to_string()));
}

#[tokio::test]
async fn sync_stores_receipts_once_and_the_same_bytes_become_one_document() {
    let (factory, pulls) = kinds(false);
    let (server, pool) = start_with_kinds(factory).await;
    let (_, company) = seed(&pool).await;
    let linked = link(&server, company).await;
    let mut c = server.client().await;

    let status = c
        .test_connector(as_caller(
            OWNER,
            TestConnectorRequest {
                id: linked.id.clone(),
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .status;
    assert!(status.contains("2 messages"));

    let first = sync_and_wait(&mut c, &linked.id).await;
    assert_eq!(
        (first.found, first.stored, first.skipped),
        (3, 2, 1),
        "three attachments, two distinct documents"
    );
    assert_eq!(first.outcome, "ok");
    let again = sync_and_wait(&mut c, &linked.id).await;
    assert_eq!(again.stored, 0, "nothing pulled twice");
    assert_eq!(*pulls.lock().unwrap(), 2);

    let docs = c
        .list_documents(as_caller(
            OWNER,
            ListDocumentsRequest {
                kind: "receipt".into(),
                ..ListDocumentsRequest::default()
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .documents;
    assert_eq!(docs.len(), 2);
    let cloudflare = docs
        .iter()
        .find(|d| d.filename.starts_with("Cloudflare"))
        .unwrap();
    assert_eq!(cloudflare.sources.len(), 2, "one document, two sources");
    let mine = c
        .list_connectors(as_caller(READER, ListConnectorsRequest::default()))
        .await
        .unwrap()
        .into_inner()
        .connectors;
    assert_eq!(mine.len(), 1, "the reader sees the company's connector");
}

#[tokio::test]
async fn a_revoked_link_is_marked_expired_by_test() {
    let (factory, _) = kinds(true);
    let (server, pool) = start_with_kinds(factory).await;
    let (_, company) = seed(&pool).await;
    let mut c = server.client().await;
    let started = c
        .start_connector(as_caller(
            OWNER,
            StartConnectorRequest {
                party_id: company.to_string(),
                kind: "mock".into(),
            },
        ))
        .await
        .unwrap()
        .into_inner();
    let linked = c
        .complete_connector(as_caller(
            OWNER,
            CompleteConnectorRequest {
                state: state_of(&started.url),
                code: "ok".into(),
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .connector
        .unwrap();
    let e = c
        .test_connector(as_caller(
            OWNER,
            TestConnectorRequest {
                id: linked.id.clone(),
            },
        ))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::FailedPrecondition, "{e}");
    let (status,): (String,) =
        sqlx::query_as("select status from finance.connectors where id = $1")
            .bind(Uuid::parse_str(&linked.id).unwrap())
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(status, "expired");
}
