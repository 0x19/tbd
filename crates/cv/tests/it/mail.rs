//! The owner is told by mail through a real finance service and its linked
//! mailbox; the stranger's approval mail is refused by that environment's
//! allow list, and that is fine.

use std::{
    path::Path,
    sync::{Arc, Mutex},
    time::Duration,
};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use tbd_db::{Capability, PartyId, UserId};
use tbd_finance::connectors::{
    Auth, AuthContext, Capabilities, Connector, ConnectorError, Found, Kind, Linked, Outgoing,
    Purpose, Reach, SentMail,
};
use tbd_proto::{
    cv::v1::{DecideRequestRequest, GetAccessRequest, ListRequestsRequest, RequestAccessRequest},
    finance::v1::{
        CompleteConnectorRequest, StartConnectorRequest,
        finance_service_client::FinanceServiceClient,
    },
};
use tokio::{net::TcpListener, sync::oneshot};
use tonic::Request;

use crate::support::{self, OWNER, VISITOR, as_caller};

const MAILBOX: &str = "nevio@inorbit.hr";

/// A mailbox that can only send, and remembers what it sent.
#[derive(Debug, Clone)]
struct MockMailbox {
    sent: Arc<Mutex<Vec<Outgoing>>>,
}

#[async_trait]
impl Connector for MockMailbox {
    fn kind(&self) -> Kind {
        Kind {
            name: "mock",
            label: "Mock mailbox",
            description: "Sends, and keeps what it sent.",
            auth: Auth::Oauth,
            consent_note: "nothing",
            configured: true,
            purposes: &Purpose::ALL,
        }
    }
    async fn start(&self, ctx: &AuthContext) -> Result<String, ConnectorError> {
        Ok(format!("https://mock.test/auth?state={}", ctx.state))
    }
    async fn complete(&self, _ctx: &AuthContext, _code: &str) -> Result<Linked, ConnectorError> {
        Ok(Linked {
            credentials: json!({ "token": "SECRET", "scope": "send" }),
            external_id: MAILBOX.into(),
            label: MAILBOX.into(),
        })
    }
    async fn test(&self, _credentials: &Value) -> Result<String, ConnectorError> {
        Ok("ok".into())
    }
    async fn pull(
        &self,
        _credentials: &Value,
        _config: &Value,
        _since: DateTime<Utc>,
        _seen: &(dyn for<'a> Fn(&'a str) -> bool + Sync),
        _sink: tokio::sync::mpsc::Sender<Found>,
    ) -> Result<Reach, ConnectorError> {
        Ok(Reach::Complete)
    }
    fn capabilities(&self, _credentials: &Value) -> Capabilities {
        Capabilities {
            read: false,
            send: true,
        }
    }
    async fn send(
        &self,
        _credentials: &Value,
        mail: &Outgoing,
    ) -> Result<SentMail, ConnectorError> {
        let mut sent = self.sent.lock().unwrap();
        sent.push(mail.clone());
        let n = sent.len();
        Ok(SentMail {
            provider_id: format!("sent-{n}"),
            thread_key: format!("thread-{n}"),
            message_id: format!("<sent-{n}@example.test>"),
        })
    }
}

/// A finance server on `url`'s database with the mock mailbox, the owner and
/// the company seeded and the mailbox linked for sending; returns its address
/// and the record of what the mailbox sent.
#[allow(clippy::too_many_lines)]
async fn finance_with_mailbox(
    url: &str,
    pool: &sqlx::PgPool,
) -> (String, Arc<Mutex<Vec<Outgoing>>>, oneshot::Sender<()>) {
    let sent = Arc::new(Mutex::new(Vec::new()));
    let record = Arc::clone(&sent);
    let kinds: tbd_finance::connectors::KindsFactory = Arc::new(move || {
        vec![Box::new(MockMailbox {
            sent: Arc::clone(&record),
        }) as Box<dyn Connector>]
    });

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../configs/finance");
    let (mut config, _) = tbd_finance::Config::load(&dir, "local").unwrap();
    config.server.listen = addr;
    config.metrics.listen = None;
    config.store.url = url.to_owned();
    config.connectors.key =
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, [9u8; 32]);
    config.connectors.redirect_url = "https://finance.test/connectors/callback".into();
    let (stop, stopped) = oneshot::channel();
    tokio::spawn(async move {
        tbd_finance::serve_with_kinds(
            listener,
            config,
            tbd_finance::Runtime::default(),
            kinds,
            async {
                let _ = stopped.await;
            },
        )
        .await
        .unwrap();
    });

    // The owner and their company, the way the finance tests seed them.
    let owner: UserId = tbd_db::ensure_user(pool, OWNER.subject, OWNER.email, "Owner")
        .await
        .unwrap();
    let company = tbd_db::create_org(pool, "Inorbit d.o.o.", None, Some("HR"), true)
        .await
        .unwrap();
    tbd_db::grant(
        pool,
        owner,
        PartyId(company.0),
        Capability::Own,
        Some(owner),
        None,
    )
    .await
    .unwrap();

    // The owner links the mailbox for sending.
    let channel = tonic::transport::Channel::from_shared(format!("http://{addr}"))
        .unwrap()
        .connect()
        .await
        .unwrap();
    let mut f = FinanceServiceClient::new(channel);
    let started = f
        .start_connector(as_caller(
            &OWNER,
            StartConnectorRequest {
                party_id: company.0.to_string(),
                kind: "mock".into(),
                purpose: "send".into(),
            },
        ))
        .await
        .unwrap()
        .into_inner();
    let state = url::Url::parse(&started.url)
        .unwrap()
        .query_pairs()
        .find(|(k, _)| k == "state")
        .unwrap()
        .1
        .into_owned();
    let linked = f
        .complete_connector(as_caller(
            &OWNER,
            CompleteConnectorRequest {
                state,
                code: "ok".into(),
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .connector
        .unwrap();
    assert!(linked.can_send && !linked.can_read);
    assert_eq!(linked.external_id, MAILBOX);

    // What `cv grant` does: the service's subject may read the company.
    let service = tbd_db::ensure_user(pool, tbd_cv::notify::SERVICE_SUBJECT, None, "cv service")
        .await
        .unwrap();
    tbd_db::grant(
        pool,
        service,
        PartyId(company.0),
        Capability::Read,
        Some(owner),
        None,
    )
    .await
    .unwrap();

    (format!("http://{addr}"), sent, stop)
}

#[allow(clippy::too_many_lines)]
#[tokio::test]
async fn the_owner_is_told_through_the_linked_mailbox_and_the_stranger_is_not() {
    let (url, pool) = support::database().await;
    let (finance_url, sent, _stop_finance) = finance_with_mailbox(&url, &pool).await;

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../configs/cv");
    let (mut config, _) = tbd_cv::Config::load(&dir, "local").unwrap();
    config.server.listen = addr;
    config.metrics.listen = None;
    config.store.url = url;
    config.finance.url = finance_url;
    config.notify.from = MAILBOX.into();
    config.notify.to = MAILBOX.into();
    config.notify.url = "https://cv.test/".into();
    let (_stop, stopped) = oneshot::channel::<()>();
    tokio::spawn(async move {
        tbd_cv::serve_with(listener, config, tbd_cv::Runtime::default(), async {
            let _ = stopped.await;
        })
        .await
        .unwrap();
    });
    let channel = tonic::transport::Channel::from_shared(format!("http://{addr}"))
        .unwrap()
        .connect()
        .await
        .unwrap();
    let mut c = tbd_proto::cv::v1::cv_service_client::CvServiceClient::new(channel);

    let state = c
        .request_access(as_caller(
            &VISITOR,
            RequestAccessRequest {
                note: "Hiring for Rust.".into(),
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .state
        .unwrap();
    assert_eq!(state.status, "requested");
    assert!(state.notifications, "a mailbox is configured");

    // The mail goes out off the request; wait for it.
    let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
    loop {
        if sent.lock().unwrap().len() == 1 {
            break;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "the owner was never told"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    let mail = sent.lock().unwrap()[0].clone();
    assert_eq!(mail.to, vec![MAILBOX.to_owned()]);
    assert!(mail.subject.contains("Rita Reader"), "{}", mail.subject);
    assert!(mail.text.contains("Hiring for Rust."), "{}", mail.text);
    assert!(
        mail.text.contains("https://cv.test/admin/"),
        "{}",
        mail.text
    );

    // The row remembers it.
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    loop {
        let state = c
            .get_access(as_caller(&VISITOR, GetAccessRequest {}))
            .await
            .unwrap()
            .into_inner()
            .state
            .unwrap();
        if !state.notified_at.is_empty() {
            break;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "notified_at never set"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    // Approval mails the requester -- best effort: the local environment's
    // allow list refuses a stranger, so nothing more is sent, and the
    // decision stands.
    let id = c
        .list_requests(as_caller(&OWNER, ListRequestsRequest::default()))
        .await
        .unwrap()
        .into_inner()
        .requests[0]
        .id
        .clone();
    let decided = c
        .decide_request(as_caller(
            &OWNER,
            DecideRequestRequest {
                id,
                decision: "approve".into(),
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .request
        .unwrap();
    assert_eq!(decided.status, "approved");
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert_eq!(
        sent.lock().unwrap().len(),
        1,
        "the stranger is outside allow_to"
    );
}

/// Keeps the compiler honest about the request helper's metadata: a real
/// `Request` is what the finance client needs too.
#[allow(dead_code)]
fn _typed(r: Request<()>) -> Request<()> {
    r
}
