//! Test harness: the service on an ephemeral port, both tiers on one engine
//! fixture. A fixture is the in-process stub or a wiremock playing Ollama or
//! llama.cpp on the wire; all three honour the same prompt markers, which is
//! what lets one conformance suite run against every engine.

#![allow(clippy::unwrap_used, clippy::expect_used, dead_code)]

use std::{net::SocketAddr, time::Duration};

use base64::Engine as _;
use futures::StreamExt as _;
use sqlx::PgPool;
use tbd_llm::{
    Config, Runtime,
    config::{EngineKind, Tier},
};
use tbd_proto::llm::v1::{GenerateResponse, llm_service_client::LlmServiceClient};
use testcontainers::{
    ContainerAsync, GenericImage, ImageExt as _, core::IntoContainerPort as _, core::WaitFor,
    runners::AsyncRunner as _,
};
use tokio::{net::TcpListener, sync::oneshot};
use tonic::{Request, Status, transport::Channel};
use tonic_health::pb::health_client::HealthClient;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{body_string_contains, method, path},
};

pub struct Server {
    pub addr: SocketAddr,
    pub runtime: Runtime,
    _stop: oneshot::Sender<()>,
    _mocks: Vec<MockServer>,
    container: Option<ContainerAsync<GenericImage>>,
}

const IMAGE: (&str, &str) = ("pgvector/pgvector", "0.8.6-pg17-trixie");

/// A fresh, migrated database: its URL and the pool the test reads with.
///
/// The server comes from `TBD_TEST_DATABASE_URL` or, when unset, a container
/// this test starts. With neither the test fails and says so.
pub async fn database() -> (String, PgPool, Option<ContainerAsync<GenericImage>>) {
    let (admin_url, container) = admin_url().await;
    let admin = PgPool::connect(&admin_url).await.unwrap();
    let database = format!("llm_test_{}", uuid::Uuid::now_v7().simple());
    sqlx::query(sqlx::AssertSqlSafe(format!("create database {database}")))
        .execute(&admin)
        .await
        .unwrap();
    drop(admin);
    let url = {
        let mut u = url::Url::parse(&admin_url).unwrap();
        u.set_path(&database);
        u.to_string()
    };
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(8)
        .connect(&url)
        .await
        .unwrap();
    tbd_db::migrate(&pool).await.unwrap();
    (url, pool, container)
}

async fn admin_url() -> (String, Option<ContainerAsync<GenericImage>>) {
    if let Ok(url) = std::env::var("TBD_TEST_DATABASE_URL")
        && !url.trim().is_empty()
    {
        return (url, None);
    }
    let container = GenericImage::new(IMAGE.0, IMAGE.1)
        .with_exposed_port(5432.tcp())
        .with_wait_for(WaitFor::message_on_stderr(
            "database system is ready to accept connections",
        ))
        .with_env_var("POSTGRES_USER", "test")
        .with_env_var("POSTGRES_PASSWORD", "test")
        .with_env_var("POSTGRES_DB", "postgres")
        .start()
        .await
        .unwrap_or_else(|e| {
            panic!(
                "these tests need Docker (to start {}:{}) or TBD_TEST_DATABASE_URL: {e}",
                IMAGE.0, IMAGE.1
            )
        });
    let port = container.get_host_port_ipv4(5432).await.unwrap();
    let host = container.get_host().await.unwrap();
    (
        format!("postgres://test:test@{host}:{port}/postgres?sslmode=disable"),
        Some(container),
    )
}

/// Both tiers on `fixture`, recording into a fresh migrated database.
pub async fn start_with_store(
    fixture: Fixture,
    adjust: impl FnOnce(&mut Config),
) -> (Server, PgPool) {
    let (url, pool, container) = database().await;
    let mut server = start_fixtures(fixture, fixture, Runtime::default(), |c| {
        c.store.url = url;
        adjust(c);
    })
    .await;
    server.container = container;
    (server, pool)
}

/// The usage every fixture reports on a completed generation.
pub const PROMPT_TOKENS: u32 = 7;
pub const COMPLETION_TOKENS: u32 = 3;

/// What runs behind a tier in a test.
#[derive(Debug, Clone, Copy)]
pub enum Fixture {
    Stub,
    Ollama,
    Llamacpp,
}

impl Fixture {
    pub fn kind(self) -> EngineKind {
        match self {
            Fixture::Stub => EngineKind::Stub,
            Fixture::Ollama => EngineKind::Ollama,
            Fixture::Llamacpp => EngineKind::Llamacpp,
        }
    }

    /// The engine name the wire reports.
    pub fn name(self) -> &'static str {
        self.kind().as_str()
    }

    /// A mock playing this engine, or none for the stub.
    async fn mock(self) -> Option<MockServer> {
        match self {
            Fixture::Stub => None,
            Fixture::Ollama => Some(ollama_mock().await),
            Fixture::Llamacpp => Some(llamacpp_mock().await),
        }
    }
}

pub async fn start() -> Server {
    start_with(Runtime::default()).await
}

pub async fn start_with(runtime: Runtime) -> Server {
    start_fixtures(Fixture::Stub, Fixture::Stub, runtime, |_| {}).await
}

/// Both tiers on `fixture`.
pub async fn start_on(fixture: Fixture, adjust: impl FnOnce(&mut Config)) -> Server {
    start_fixtures(fixture, fixture, Runtime::default(), adjust).await
}

/// The fast tier on one fixture and the deep tier on another.
pub async fn start_fixtures(
    fast: Fixture,
    deep: Fixture,
    runtime: Runtime,
    adjust: impl FnOnce(&mut Config),
) -> Server {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let mut config = Config::stub(addr);
    let mut mocks = Vec::new();
    for (tier, fixture) in [(Tier::Fast, fast), (Tier::Deep, deep)] {
        let cfg = match tier {
            Tier::Fast => &mut config.engines.fast,
            Tier::Deep => &mut config.engines.deep,
        };
        cfg.kind = fixture.kind();
        if let Some(mock) = fixture.mock().await {
            cfg.url = mock.uri();
            cfg.model = format!("{}-model", fixture.name());
            mocks.push(mock);
        }
    }
    adjust(&mut config);
    let (stop, stopped) = oneshot::channel();
    let rt = runtime.clone();
    tokio::spawn(async move {
        tbd_llm::serve_with(listener, config, rt, async {
            let _ = stopped.await;
        })
        .await
        .unwrap();
    });
    Server {
        addr,
        runtime,
        _stop: stop,
        _mocks: mocks,
        container: None,
    }
}

/// Ollama on the wire: `/api/chat` as NDJSON, `/api/tags`, `/api/embed`.
async fn ollama_mock() -> MockServer {
    let server = MockServer::start().await;
    let thinking = "{\"model\":\"ollama-model\",\"message\":{\"role\":\"assistant\",\"content\":\"\",\"thinking\":\"hm\"},\"done\":false}\n";
    let line = |content: &str, done: bool| {
        if done {
            format!(
                "{{\"model\":\"ollama-model\",\"message\":{{\"role\":\"assistant\",\"content\":\"\"}},\"done\":true,\"prompt_eval_count\":{PROMPT_TOKENS},\"eval_count\":{COMPLETION_TOKENS}}}\n"
            )
        } else {
            format!(
                "{{\"model\":\"ollama-model\",\"message\":{{\"role\":\"assistant\",\"content\":\"{content}\"}},\"done\":false}}\n"
            )
        }
    };
    let ndjson = "application/x-ndjson";
    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .and(body_string_contains("<<error>>"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            format!(
                "{}{}{{\"error\":\"engine exploded\"}}\n",
                line("Hello", false),
                line(" there", false)
            ),
            ndjson,
        ))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .and(body_string_contains("<<hang>>"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(Duration::from_secs(30))
                .set_body_raw(line("late", false), ndjson),
        )
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .and(body_string_contains("<<refuse>>"))
        .respond_with(
            ResponseTemplate::new(404).set_body_string(r#"{"error":"model 'x' not found"}"#),
        )
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            format!(
                "{thinking}{}{}{}{}",
                line("Hello", false),
                line(" there", false),
                line(" world", false),
                line("", true)
            ),
            ndjson,
        ))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/tags"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(r#"{"models":[{"name":"ollama-model"}]}"#),
        )
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/embed"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"{"model":"ollama-model","embeddings":[[0.1,0.2,0.3],[0.4,0.5,0.6]]}"#,
        ))
        .mount(&server)
        .await;
    server
}

/// llama-server on the wire: `/v1/chat/completions` as SSE, `/v1/models`,
/// `/v1/embeddings`.
async fn llamacpp_mock() -> MockServer {
    let server = MockServer::start().await;
    let delta = |content: &str| {
        format!(
            "data: {{\"model\":\"llamacpp-model\",\"choices\":[{{\"index\":0,\"delta\":{{\"content\":\"{content}\"}},\"finish_reason\":null}}]}}\n\n"
        )
    };
    let reasoning = "data: {\"model\":\"llamacpp-model\",\"choices\":[{\"index\":0,\"delta\":{\"reasoning_content\":\"hm\"},\"finish_reason\":null}]}\n\n";
    let finish = "data: {\"choices\":[{\"index\":0,\"delta\":{},\"finish_reason\":\"stop\"}]}\n\n";
    let usage = format!(
        "data: {{\"choices\":[],\"usage\":{{\"prompt_tokens\":{PROMPT_TOKENS},\"completion_tokens\":{COMPLETION_TOKENS}}}}}\n\n"
    );
    let done = "data: [DONE]\n\n";
    let sse = "text/event-stream";
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(body_string_contains("<<error>>"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            format!(
                "{}{}data: {{\"error\":{{\"message\":\"engine exploded\"}}}}\n\n",
                delta("Hello"),
                delta(" there")
            ),
            sse,
        ))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(body_string_contains("<<hang>>"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(Duration::from_secs(30))
                .set_body_raw(delta("late"), sse),
        )
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(body_string_contains("<<refuse>>"))
        .respond_with(
            ResponseTemplate::new(400)
                .set_body_string(r#"{"error":{"message":"no slot available"}}"#),
        )
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            format!(
                "{reasoning}{}{}{}{finish}{usage}{done}",
                delta("Hello"),
                delta(" there"),
                delta(" world")
            ),
            sse,
        ))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(r#"{"data":[{"id":"llamacpp-model"}]}"#),
        )
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/embeddings"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"{"data":[{"index":1,"embedding":[0.4,0.5,0.6]},{"index":0,"embedding":[0.1,0.2,0.3]}]}"#,
        ))
        .mount(&server)
        .await;
    server
}

impl Server {
    pub async fn channel(&self) -> Channel {
        Channel::from_shared(format!("http://{}", self.addr))
            .unwrap()
            .connect()
            .await
            .unwrap()
    }

    pub async fn client(&self) -> LlmServiceClient<Channel> {
        LlmServiceClient::new(self.channel().await)
    }

    pub async fn health(&self) -> HealthClient<Channel> {
        HealthClient::new(self.channel().await)
    }
}

/// A person, as Envoy's verified payload describes them.
#[derive(Debug, Clone)]
pub struct Person {
    pub subject: &'static str,
    pub email: Option<&'static str>,
    pub role: Option<&'static str>,
}

pub const VISITOR: Person = Person {
    subject: "visitor-1",
    email: Some("rita@example.org"),
    role: Some("viewer"),
};

pub const OTHER: Person = Person {
    subject: "visitor-2",
    email: Some("sam@example.org"),
    role: Some("viewer"),
};

/// A request as `who`, with the payload Envoy would have forwarded.
pub fn as_caller<T>(who: &Person, message: T) -> Request<T> {
    let mut claims = serde_json::json!({ "sub": who.subject, "aud": ["tbd-ui"] });
    if let Some(e) = who.email {
        claims["email"] = e.into();
    }
    if let Some(r) = who.role {
        claims["role"] = r.into();
    }
    let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(claims.to_string());
    let mut request = Request::new(message);
    request
        .metadata_mut()
        .insert("x-jwt-payload", payload.parse().unwrap());
    request
}

/// Every chunk until the stream ends, and the error that ended it if one did.
pub async fn collect(
    stream: tonic::Streaming<GenerateResponse>,
) -> (Vec<GenerateResponse>, Option<Status>) {
    let mut chunks = Vec::new();
    let mut error = None;
    let mut stream = stream;
    while let Some(item) = stream.next().await {
        match item {
            Ok(c) => chunks.push(c),
            Err(s) => {
                error = Some(s);
                break;
            }
        }
    }
    (chunks, error)
}
