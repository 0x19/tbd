//! Load operations against the ledger kind: append, current, history and
//! retract on a pool of subjects; `ledger_lifecycle`, one full assertion of
//! the retraction rule per request; `ledger_erase_cycle`, the erasure rule;
//! and `ledger_fuzz`, hostile requests that must never draw an internal
//! error. Every op holds its own state for the run (`OpKind::build` makes a
//! fresh instance per run), so a scenario is self-contained.

use std::{
    sync::{Arc, Mutex, PoisonError},
    time::Duration,
};

use async_trait::async_trait;
use rand::{Rng, SeedableRng, rngs::StdRng};
use tbd_proto::ledger::v1::{
    AppendRequest, CurrentRequest, Envelope, EraseRequest, HistoryRequest, RestoreRequest,
    RetractRequest, Source, ledger_service_client::LedgerServiceClient,
};
use tonic::Code;

use super::ops::{Clients, OpError, Operation, Target};

/// Paths the pool writes; small on purpose so retractions find something.
const PATHS: &[&str] = &[
    "profile.name",
    "profile.bio",
    "traits.warmth",
    "traits.novelty",
    "journal.entry",
    "readings.sun",
];

/// Subjects shared by the ledger ops of one run, and what has been written
/// to them, so reads and retractions hit facts that exist: a `NotFound` is
/// then a real failure, never a race of the pool with itself.
#[derive(Debug)]
pub struct Pool {
    subjects: Vec<uuid::Uuid>,
    rng: Mutex<StdRng>,
    written: Mutex<Vec<(uuid::Uuid, &'static str)>>,
}

impl Pool {
    /// `n` fresh subjects and a seeded generator.
    pub fn new(n: usize, seed: u64) -> Arc<Self> {
        Arc::new(Self {
            subjects: (0..n.max(1)).map(|_| uuid::Uuid::now_v7()).collect(),
            rng: Mutex::new(StdRng::seed_from_u64(seed)),
            written: Mutex::new(Vec::new()),
        })
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, StdRng> {
        self.rng.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn subject(&self) -> uuid::Uuid {
        let i = self.lock().random_range(0..self.subjects.len());
        self.subjects[i]
    }

    /// Remember a written `(subject, path)`.
    fn wrote(&self, subject: uuid::Uuid, path: &'static str) {
        let mut w = self.written.lock().unwrap_or_else(PoisonError::into_inner);
        if !w.contains(&(subject, path)) {
            w.push((subject, path));
        }
    }

    /// A subject with at least one fact, if any.
    fn written_subject(&self) -> Option<uuid::Uuid> {
        let w = self.written.lock().unwrap_or_else(PoisonError::into_inner);
        if w.is_empty() {
            return None;
        }
        let i = self.lock().random_range(0..w.len());
        Some(w[i].0)
    }

    /// Take a written `(subject, path)` out of the pool, for a retraction.
    fn take_written(&self) -> Option<(uuid::Uuid, &'static str)> {
        let mut w = self.written.lock().unwrap_or_else(PoisonError::into_inner);
        if w.is_empty() {
            return None;
        }
        let i = self.lock().random_range(0..w.len());
        Some(w.swap_remove(i))
    }

    fn path(&self) -> &'static str {
        let i = self.lock().random_range(0..PATHS.len());
        PATHS[i]
    }

    fn u64(&self) -> u64 {
        self.lock().random()
    }
}

fn now() -> prost_types::Timestamp {
    prost_types::Timestamp::from(std::time::SystemTime::now())
}

fn json(v: &serde_json::Value) -> Envelope {
    Envelope {
        version: 0,
        bytes: serde_json::to_vec(v).unwrap_or_default(),
    }
}

fn append_request(subject: uuid::Uuid, path: &str, value: u64) -> AppendRequest {
    AppendRequest {
        subject_id: subject.to_string(),
        path: path.to_owned(),
        source: Source::Declared as i32,
        value: Some(json(&serde_json::json!({ "v": value }))),
        origin: Some(json(&serde_json::json!({ "by": "chaos" }))),
        confidence: Some(1.0),
        counterparty_id: None,
        observed_at: Some(now()),
        expires_at: None,
        consent: vec!["self".into()],
        stub: false,
        idempotency_key: String::new(),
    }
}

fn current_request(subject: uuid::Uuid) -> CurrentRequest {
    CurrentRequest {
        subject_id: subject.to_string(),
        paths: vec![],
        sources: vec![],
        scopes: vec!["self".into()],
        cursor: String::new(),
        limit: 0,
    }
}

/// Map a status onto the report's classes. Unavailable is the transport (or
/// an injected fault); `Internal`, `Unknown` and `DataLoss` mean the ledger
/// broke its contract.
fn classify(s: &tonic::Status) -> OpError {
    match s.code() {
        Code::Unavailable => OpError::Transport,
        Code::Internal | Code::Unknown | Code::DataLoss => {
            OpError::Contract(format!("{:?}: {}", s.code(), s.message()))
        }
        code => OpError::Grpc(format!("{code:?}")),
    }
}

async fn client(
    clients: &Clients,
    target: &Target,
) -> Result<LedgerServiceClient<crate::tls::Grpc>, OpError> {
    Ok(LedgerServiceClient::new(clients.grpc(target).await?))
}

/// One append on a pooled subject.
pub struct Append(pub Arc<Pool>);

#[async_trait]
impl Operation for Append {
    fn name(&self) -> &'static str {
        "ledger_append"
    }

    async fn run(&self, clients: &Clients, target: &Target) -> Result<(), OpError> {
        let mut c = client(clients, target).await?;
        let resp = c
            .append(append_request(
                self.0.subject(),
                self.0.path(),
                self.0.u64(),
            ))
            .await
            .map_err(|s| classify(&s))?
            .into_inner();
        if resp.fact.is_some_and(|f| f.id > 0) {
            Ok(())
        } else {
            Err(OpError::Contract("append answered without a fact".into()))
        }
    }
}

/// A subject that has facts: one from the pool's written set, or a fresh
/// append when nothing has been written yet.
async fn readable_subject(
    pool: &Pool,
    c: &mut LedgerServiceClient<crate::tls::Grpc>,
) -> Result<uuid::Uuid, OpError> {
    if let Some(s) = pool.written_subject() {
        return Ok(s);
    }
    let (subject, path) = (pool.subject(), pool.path());
    c.append(append_request(subject, path, pool.u64()))
        .await
        .map_err(|s| classify(&s))?;
    pool.wrote(subject, path);
    Ok(subject)
}

/// `Current` on a subject that has facts.
pub struct Current(pub Arc<Pool>);

#[async_trait]
impl Operation for Current {
    fn name(&self) -> &'static str {
        "ledger_current"
    }

    async fn run(&self, clients: &Clients, target: &Target) -> Result<(), OpError> {
        let mut c = client(clients, target).await?;
        let subject = readable_subject(&self.0, &mut c).await?;
        c.current(current_request(subject))
            .await
            .map(|_| ())
            .map_err(|s| classify(&s))
    }
}

/// `History` with a random cut and page size.
pub struct History(pub Arc<Pool>);

#[async_trait]
impl Operation for History {
    fn name(&self) -> &'static str {
        "ledger_history"
    }

    async fn run(&self, clients: &Clients, target: &Target) -> Result<(), OpError> {
        let mut c = client(clients, target).await?;
        let subject = readable_subject(&self.0, &mut c).await?;
        let limit = u32::try_from(self.0.u64() % 50).unwrap_or(10);
        let at = self.0.u64().is_multiple_of(2).then(now);
        let req = HistoryRequest {
            subject_id: subject.to_string(),
            paths: vec![],
            sources: vec![],
            scopes: vec!["self".into()],
            cursor: String::new(),
            limit,
            at,
        };
        c.history(req).await.map(|_| ()).map_err(|s| classify(&s))
    }
}

/// Retract a written `(subject, path)` from the pool, appending one first
/// when the pool holds nothing.
pub struct Retract(pub Arc<Pool>);

#[async_trait]
impl Operation for Retract {
    fn name(&self) -> &'static str {
        "ledger_retract"
    }

    async fn run(&self, clients: &Clients, target: &Target) -> Result<(), OpError> {
        let mut c = client(clients, target).await?;
        let (subject, path) = if let Some(pair) = self.0.take_written() {
            pair
        } else {
            let (subject, path) = (self.0.subject(), self.0.path());
            c.append(append_request(subject, path, self.0.u64()))
                .await
                .map_err(|s| classify(&s))?;
            (subject, path)
        };
        let req = RetractRequest {
            subject_id: subject.to_string(),
            path: path.to_owned(),
            source: Source::Declared as i32,
            origin: Some(json(&serde_json::json!({ "by": "chaos" }))),
        };
        c.retract(req).await.map(|_| ()).map_err(|s| classify(&s))
    }
}

/// Append → current shows it → retract → history at the earlier instant does
/// not: the retraction rule, asserted once per request, on a fresh subject.
pub struct Lifecycle;

#[async_trait]
impl Operation for Lifecycle {
    fn name(&self) -> &'static str {
        "ledger_lifecycle"
    }

    async fn run(&self, clients: &Clients, target: &Target) -> Result<(), OpError> {
        let mut c = client(clients, target).await?;
        let subject = uuid::Uuid::now_v7();
        let value = rand::random::<u64>();
        let appended = c
            .append(append_request(subject, "journal.entry", value))
            .await
            .map_err(|s| classify(&s))?
            .into_inner();
        let fact = appended
            .fact
            .ok_or_else(|| OpError::Contract("append answered without a fact".into()))?;
        let page = c
            .current(current_request(subject))
            .await
            .map_err(|s| classify(&s))?
            .into_inner();
        if page.facts.len() != 1 || page.facts[0].id != fact.id {
            return Err(OpError::Contract(
                "current does not show the appended fact".into(),
            ));
        }
        c.retract(RetractRequest {
            subject_id: subject.to_string(),
            path: "journal.entry".into(),
            source: Source::Declared as i32,
            origin: Some(json(&serde_json::json!({ "by": "chaos" }))),
        })
        .await
        .map_err(|s| classify(&s))?;
        let cut = c
            .history(HistoryRequest {
                subject_id: subject.to_string(),
                paths: vec![],
                sources: vec![],
                scopes: vec!["self".into()],
                cursor: String::new(),
                limit: 0,
                at: fact.recorded_at,
            })
            .await
            .map_err(|s| classify(&s))?
            .into_inner();
        if !cut.facts.is_empty() {
            return Err(OpError::Contract(
                "a retracted value is visible in a cut of history".into(),
            ));
        }
        let page = c
            .current(current_request(subject))
            .await
            .map_err(|s| classify(&s))?
            .into_inner();
        if !page.facts.is_empty() {
            return Err(OpError::Contract(
                "current still shows a retracted fact".into(),
            ));
        }
        Ok(())
    }
}

/// Append → erase → reads denied → restore → reads back → erase again → wait
/// past the window → the subject is gone. Needs a ledger with a short grace
/// (`[stack.ledgers.X] grace = "0s"`) and its sweeper running.
pub struct EraseCycle {
    /// How long to wait for the sweeper after the second erasure.
    pub settle: Duration,
}

#[async_trait]
impl Operation for EraseCycle {
    fn name(&self) -> &'static str {
        "ledger_erase_cycle"
    }

    async fn run(&self, clients: &Clients, target: &Target) -> Result<(), OpError> {
        let mut c = client(clients, target).await?;
        let subject = uuid::Uuid::now_v7();
        let erase = || EraseRequest {
            subject_id: subject.to_string(),
        };
        c.append(append_request(subject, "profile.name", 1))
            .await
            .map_err(|s| classify(&s))?;
        c.erase(erase()).await.map_err(|s| classify(&s))?;
        match c.current(current_request(subject)).await {
            Err(s) if s.code() == Code::FailedPrecondition => {}
            Ok(_) => return Err(OpError::Contract("reads allowed while erased".into())),
            Err(s) => return Err(classify(&s)),
        }
        c.restore(RestoreRequest {
            subject_id: subject.to_string(),
        })
        .await
        .map_err(|s| classify(&s))?;
        let page = c
            .current(current_request(subject))
            .await
            .map_err(|s| classify(&s))?
            .into_inner();
        if page.facts.len() != 1 {
            return Err(OpError::Contract(
                "restore did not reopen the subject".into(),
            ));
        }
        c.erase(erase()).await.map_err(|s| classify(&s))?;
        tokio::time::sleep(self.settle).await;
        match c.current(current_request(subject)).await {
            Err(s) if s.code() == Code::FailedPrecondition => {}
            Ok(_) => return Err(OpError::Contract("reads allowed after the window".into())),
            Err(s) => return Err(classify(&s)),
        }
        match c
            .restore(RestoreRequest {
                subject_id: subject.to_string(),
            })
            .await
        {
            Err(s) if s.code() == Code::NotFound => Ok(()),
            Ok(_) => Err(OpError::Contract(
                "restore succeeded after the cascade".into(),
            )),
            Err(s) => Err(classify(&s)),
        }
    }
}

/// Hostile requests from a seeded generator. Success is a clean refusal
/// (`InvalidArgument`, `NotFound`, `FailedPrecondition`, `ResourceExhausted`)
/// or a clean answer; `Internal`, `Unknown` or a dropped connection is a
/// contract failure.
pub struct Fuzz(pub Arc<Pool>);

impl Fuzz {
    fn hostile_path(&self) -> String {
        match self.0.u64() % 9 {
            0 => String::new(),
            1 => "traits".into(),
            2 => "Traits.Warmth".into(),
            3 => "a..b".into(),
            4 => "x".repeat(10_000),
            5 => "..".into(),
            6 => "traits.*".into(),
            7 => "✨.☠".into(),
            _ => "profile.name\0".into(),
        }
    }

    fn hostile_append(&self) -> AppendRequest {
        let mut r = append_request(self.0.subject(), "profile.name", self.0.u64());
        match self.0.u64() % 12 {
            0 => r.subject_id = "not-a-uuid".into(),
            1 => r.path = self.hostile_path(),
            2 => r.source = Source::Unspecified as i32,
            3 => r.source = 99,
            4 => {
                r.value = Some(Envelope {
                    version: 7,
                    bytes: vec![0xff; 32],
                });
            }
            5 => {
                r.value = Some(Envelope {
                    version: 0,
                    bytes: b"{not json".to_vec(),
                });
            }
            6 => r.value = Some(json(&serde_json::json!("x".repeat(1_000_000)))),
            7 => {
                r.confidence = Some(if self.0.u64().is_multiple_of(2) {
                    -1.0
                } else {
                    f32::NAN
                });
            }
            8 => r.consent = (0..5000).map(|i| format!("scope-{i}")).collect(),
            9 => r.consent = vec!["Bad Scope!".into()],
            10 => r.counterparty_id = Some(uuid::Uuid::now_v7().to_string()),
            _ => {
                r.observed_at = None;
                r.expires_at = Some(prost_types::Timestamp {
                    seconds: i64::MAX,
                    nanos: -1,
                });
            }
        }
        r
    }

    fn hostile_current(&self) -> CurrentRequest {
        let mut r = current_request(self.0.subject());
        match self.0.u64() % 6 {
            0 => r.cursor = "garbage".into(),
            1 => r.limit = u32::MAX,
            2 => r.paths = vec![self.hostile_path()],
            3 => r.scopes.clear(),
            4 => r.sources = vec![42],
            _ => r.subject_id = uuid::Uuid::now_v7().to_string(),
        }
        r
    }
}

#[async_trait]
impl Operation for Fuzz {
    fn name(&self) -> &'static str {
        "ledger_fuzz"
    }

    async fn run(&self, clients: &Clients, target: &Target) -> Result<(), OpError> {
        let mut c = client(clients, target).await?;
        let outcome = match self.0.u64() % 3 {
            0 => c.append(self.hostile_append()).await.map(|_| ()),
            1 => c.current(self.hostile_current()).await.map(|_| ()),
            _ => {
                let cur = self.hostile_current();
                c.history(HistoryRequest {
                    subject_id: cur.subject_id,
                    paths: cur.paths,
                    sources: cur.sources,
                    scopes: cur.scopes,
                    cursor: cur.cursor,
                    limit: cur.limit,
                    at: Some(prost_types::Timestamp {
                        seconds: -62_135_596_800 - 1,
                        nanos: 0,
                    }),
                })
                .await
                .map(|_| ())
            }
        };
        match outcome {
            Ok(()) => Ok(()),
            Err(s)
                if matches!(
                    s.code(),
                    Code::InvalidArgument
                        | Code::NotFound
                        | Code::FailedPrecondition
                        | Code::ResourceExhausted
                        | Code::Aborted
                ) =>
            {
                Ok(())
            }
            Err(s) => Err(classify(&s)),
        }
    }
}
