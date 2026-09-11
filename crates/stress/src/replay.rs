//! Run a trace again. The requests are symbolic, so they are bound to fresh
//! subjects and sent in order; every answer goes through the same model and
//! the same checkers the owner worker used, so a replay says whether the rule
//! breaks again, on any target.

use std::{collections::BTreeMap, sync::Arc, time::Duration};

use chrono::Utc;
use tbd_proto::ledger::v1::Fact;
use tonic::Code;
use uuid::Uuid;

use crate::{
    client::{CallError, LedgerClient},
    finding::ReplayOutcome,
    model::{Erased, SubjectModel, generate, invariants},
    trace::{self, Binding, Concrete, Request, Step, SubjectRef, Violation},
};

/// What one replay produced.
#[derive(Debug, Clone)]
pub struct Replayed {
    /// The steps as sent this time, responses included.
    pub steps: Vec<Step>,
    /// Every rule that broke, in order.
    pub violations: Vec<Violation>,
}

impl Replayed {
    /// Whether `invariant` broke with the same signature.
    #[must_use]
    pub fn reproduces(&self, invariant: &str, signature: &str) -> bool {
        self.violations.iter().any(|v| {
            v.invariant == invariant
                && crate::finding::signature(v.invariant, &v.message) == signature
        })
    }

    /// The first violation of `invariant`, any signature.
    #[must_use]
    pub fn first_of(&self, invariant: &str) -> Option<&Violation> {
        self.violations.iter().find(|v| v.invariant == invariant)
    }
}

/// A page walk in progress: consecutive reads with the same filters until an
/// empty `next`.
#[derive(Default)]
struct Walk {
    facts: Vec<Fact>,
    pages: Vec<Vec<Fact>>,
    nexts: Vec<String>,
    limit: u32,
}

/// The model and the judgement, step by step. Owner workers decide what to
/// send; this decides what an answer must have been, from the request alone.
pub struct Interpreter {
    /// One model per subject reference.
    models: BTreeMap<SubjectRef, SubjectModel>,
    walks: BTreeMap<(SubjectRef, String), Walk>,
    skew: chrono::Duration,
    tolerate: Vec<String>,
}

impl Interpreter {
    /// Fresh models for `binding`.
    #[must_use]
    pub fn new(binding: &Binding, skew: Duration, tolerate: Vec<String>) -> Self {
        let mut models = BTreeMap::new();
        models.insert(SubjectRef::Own, SubjectModel::new(binding.own));
        for (i, p) in binding.peers.iter().enumerate() {
            models.insert(
                SubjectRef::Peer(u8::try_from(i).unwrap_or(u8::MAX)),
                SubjectModel::new(*p),
            );
        }
        Self {
            models,
            walks: BTreeMap::new(),
            skew: chrono::Duration::from_std(skew)
                .unwrap_or_else(|_| chrono::Duration::milliseconds(500)),
            tolerate,
        }
    }

    fn model(&mut self, s: SubjectRef) -> &mut SubjectModel {
        self.models
            .entry(s)
            .or_insert_with(|| SubjectModel::new(Uuid::now_v7()))
    }

    /// Judge one answer and update the model.
    #[allow(clippy::too_many_lines)]
    pub fn observe(
        &mut self,
        request: &Request,
        concrete: &Concrete,
        outcome: &Outcome,
        steps: &[Step],
    ) -> Vec<Violation> {
        let mut out = Vec::new();
        if let Outcome::Err(e) = outcome
            && !self.tolerate.iter().any(|t| *t == e.class())
            && let Some(v) = invariants::clean_errors(e, &self.tolerate)
        {
            out.push(v);
        }
        let Some(subject) = request.subject() else {
            return out;
        };
        let erased = self.model(subject).erased.clone();
        match (concrete, outcome) {
            (Concrete::Append(req), Outcome::Append(resp)) => {
                let model = self.model(subject);
                let key = &req.idempotency_key;
                let known = (!key.is_empty())
                    .then(|| model.keys.get(key).cloned())
                    .flatten();
                match (&erased, known) {
                    (Erased::No, Some(use_)) => {
                        let kind = if use_.retracted {
                            invariants::ReplayKind::AfterRetract
                        } else if use_.fingerprint == generate::fingerprint(req) {
                            invariants::ReplayKind::Same {
                                fact_id: use_.fact_id.unwrap_or(-1),
                            }
                        } else {
                            invariants::ReplayKind::Conflict
                        };
                        out.extend(invariants::idempotency(kind, Ok(resp)));
                    }
                    (Erased::No, None) => {
                        out.extend(invariants::append_echo(req, resp, model));
                        model.apply_append(req, resp);
                    }
                    (Erased::Pending { .. } | Erased::Executed, _) => {
                        out.extend(invariants::denied("append", Ok(resp)));
                    }
                }
            }
            (Concrete::Append(req), Outcome::Err(e)) => {
                let model = self.model(subject);
                let key = &req.idempotency_key;
                let known = (!key.is_empty())
                    .then(|| model.keys.get(key).cloned())
                    .flatten();
                match (&erased, known) {
                    (Erased::Pending { .. } | Erased::Executed, _) => {
                        out.extend(invariants::denied("append", Err::<&(), _>(e)));
                    }
                    (Erased::No, Some(use_)) => {
                        let kind = if use_.retracted {
                            invariants::ReplayKind::AfterRetract
                        } else if use_.fingerprint == generate::fingerprint(req) {
                            invariants::ReplayKind::Same {
                                fact_id: use_.fact_id.unwrap_or(-1),
                            }
                        } else {
                            invariants::ReplayKind::Conflict
                        };
                        out.extend(invariants::idempotency(kind, Err(e)));
                    }
                    (Erased::No, None) => {
                        // A relation to a missing or erased peer is the counterparty
                        // check answering; anything else on an open subject is not.
                        let peer_gone = req.counterparty_id.is_some()
                            && matches!(e.code(), Some(Code::NotFound | Code::FailedPrecondition));
                        if !peer_gone
                            && !self.tolerate.iter().any(|t| *t == e.class())
                            && !e.is_unclean()
                        {
                            out.push(Violation {
                                invariant: "append_echo",
                                message: format!(
                                    "append on an open subject failed with {}",
                                    e.class()
                                ),
                                expected: serde_json::json!("a fact"),
                                actual: serde_json::to_value(e).unwrap_or_default(),
                            });
                        }
                    }
                }
            }
            (Concrete::Current(req), Outcome::Current(resp)) => {
                if erased == Erased::No {
                    let key = (
                        subject,
                        format!("current:{:?}:{:?}:{:?}", req.paths, req.sources, req.scopes),
                    );
                    let walk = self.walks.entry(key.clone()).or_default();
                    walk.limit = req.limit;
                    walk.facts.extend(resp.facts.iter().cloned());
                    walk.pages.push(resp.facts.clone());
                    walk.nexts.push(resp.next.clone());
                    if resp.next.is_empty() {
                        let walk = self.walks.remove(&key).unwrap_or_default();
                        out.extend(invariants::pagination(&walk.pages, &walk.nexts, walk.limit));
                        let model = &self.models[&subject];
                        let now = Utc::now();
                        if req.paths.is_empty() && req.sources.is_empty() {
                            out.extend(invariants::consent_filter(
                                model,
                                now,
                                self.skew,
                                &req.scopes,
                                &walk.facts,
                            ));
                        } else {
                            out.extend(invariants::path_source_filter(
                                model,
                                now,
                                self.skew,
                                &req.paths,
                                &req.sources,
                                &walk.facts,
                            ));
                        }
                    }
                } else {
                    out.extend(invariants::denied("current", Ok(resp)));
                }
            }
            (Concrete::History(req), Outcome::History(resp)) => {
                if erased == Erased::No {
                    let key = (subject, format!("history:{:?}", req.at));
                    let walk = self.walks.entry(key.clone()).or_default();
                    walk.limit = req.limit;
                    walk.facts.extend(resp.facts.iter().cloned());
                    walk.pages.push(resp.facts.clone());
                    walk.nexts.push(resp.next.clone());
                    if resp.next.is_empty() {
                        let walk = self.walks.remove(&key).unwrap_or_default();
                        out.extend(invariants::pagination(&walk.pages, &walk.nexts, walk.limit));
                        let model = self.model(subject);
                        match req.at.as_ref().and_then(trace::datetime) {
                            None => {
                                out.extend(invariants::history_is_everything(model, &walk.facts));
                                if model.has_unknown_stamps() || !model.pending.is_empty() {
                                    model.learn(&walk.facts);
                                }
                            }
                            Some(at) => {
                                if let Some(v) = invariants::history_cut(model, at, &walk.facts) {
                                    out.extend(v);
                                }
                            }
                        }
                    }
                } else {
                    out.extend(invariants::denied("history", Ok(resp)));
                }
            }
            (Concrete::Current(_) | Concrete::History(_), Outcome::Err(e)) => {
                let what = if matches!(concrete, Concrete::Current(_)) {
                    "current"
                } else {
                    "history"
                };
                match erased {
                    Erased::No => {
                        let exists = self.models[&subject].exists();
                        if exists
                            && !self.tolerate.iter().any(|t| *t == e.class())
                            && !e.is_unclean()
                        {
                            out.push(Violation {
                                invariant: if what == "history" {
                                    "history_is_everything"
                                } else {
                                    "current_is_latest"
                                },
                                message: format!(
                                    "{what} on an open subject failed with {}",
                                    e.class()
                                ),
                                expected: serde_json::json!("a page"),
                                actual: serde_json::to_value(e).unwrap_or_default(),
                            });
                        } else if !exists && e.code() != Some(Code::NotFound) && !e.is_unclean() {
                            out.push(Violation {
                                invariant: "current_is_latest",
                                message: format!("{what} on an unknown subject failed with {} instead of NotFound", e.class()),
                                expected: serde_json::json!("NotFound"),
                                actual: serde_json::to_value(e).unwrap_or_default(),
                            });
                        }
                    }
                    Erased::Pending { .. } | Erased::Executed => {
                        out.extend(invariants::denied::<()>(what, Err(e)));
                    }
                }
            }
            (Concrete::Retract(req), outcome @ (Outcome::Retract(_) | Outcome::Err(_))) => {
                let model = self.model(subject);
                let result = match outcome {
                    Outcome::Retract(r) => Ok(r),
                    Outcome::Err(e) => Err(e),
                    _ => return out,
                };
                if erased == Erased::No {
                    out.extend(invariants::retract_semantics(
                        model, &req.path, req.source, result,
                    ));
                    if let Ok(resp) = result {
                        model.apply_retract(&req.path, req.source, resp);
                    }
                } else {
                    out.extend(invariants::denied("retract", result));
                }
            }
            (Concrete::Erase(_), Outcome::Erase(resp)) => {
                let model = self.model(subject);
                match &erased {
                    Erased::No => model.apply_erase(resp),
                    Erased::Pending { requested_at, .. } => {
                        if resp
                            .requested_at
                            .as_ref()
                            .and_then(trace::datetime)
                            .map(generate::micros)
                            != requested_at.map(generate::micros)
                        {
                            out.push(Violation {
                                invariant: "erasure_denies",
                                message: "a second erase inside the window opened a new one".into(),
                                expected: serde_json::json!({"requested_at": requested_at.map(|t| t.to_rfc3339())}),
                                actual: trace::erase_json(resp),
                            });
                        }
                    }
                    Erased::Executed => out.push(Violation {
                        invariant: "erasure_executes",
                        message: "erase after the cascade succeeded".into(),
                        expected: serde_json::json!("NotFound"),
                        actual: trace::erase_json(resp),
                    }),
                }
            }
            (Concrete::Erase(_), Outcome::Err(e)) => {
                if erased == Erased::No
                    && !self.tolerate.iter().any(|t| *t == e.class())
                    && !e.is_unclean()
                {
                    out.push(Violation {
                        invariant: "erasure_denies",
                        message: format!("erase of an open subject failed with {}", e.class()),
                        expected: serde_json::json!("requested_at and executes_after"),
                        actual: serde_json::to_value(e).unwrap_or_default(),
                    });
                }
            }
            (Concrete::Restore(_), Outcome::Restore) => {
                let model = self.model(subject);
                match erased {
                    Erased::Pending { .. } => model.apply_restore(),
                    Erased::No => {}
                    Erased::Executed => out.push(Violation {
                        invariant: "erasure_executes",
                        message: "restore after the cascade succeeded".into(),
                        expected: serde_json::json!("NotFound"),
                        actual: serde_json::json!("ok"),
                    }),
                }
            }
            (Concrete::Restore(_), Outcome::Err(e)) => match erased {
                Erased::Pending { .. } if e.code() == Some(Code::NotFound) => {
                    // The window closed first: the subject is gone.
                    self.model(subject).apply_executed();
                }
                Erased::Pending { .. } => {
                    if !self.tolerate.iter().any(|t| *t == e.class()) && !e.is_unclean() {
                        out.push(Violation {
                            invariant: "erasure_denies",
                            message: format!("restore inside the window failed with {}", e.class()),
                            expected: serde_json::json!("ok"),
                            actual: serde_json::to_value(e).unwrap_or_default(),
                        });
                    }
                }
                Erased::Executed | Erased::No => {}
            },
            _ => {}
        }
        // A read after a sleep that followed an erase: the cascade ran.
        if let Concrete::Sleep(_) = concrete
            && let Erased::Pending { .. } = erased
            && steps
                .iter()
                .rev()
                .nth(1)
                .is_some_and(|s| matches!(s.request, Request::Erase { .. }))
        {
            self.model(subject).apply_executed();
        }
        out
    }
}

/// What a call answered.
#[derive(Debug, Clone)]
pub enum Outcome {
    /// Append.
    Append(tbd_proto::ledger::v1::AppendResponse),
    /// Current.
    Current(tbd_proto::ledger::v1::CurrentResponse),
    /// History.
    History(tbd_proto::ledger::v1::HistoryResponse),
    /// Retract.
    Retract(tbd_proto::ledger::v1::RetractResponse),
    /// Erase.
    Erase(tbd_proto::ledger::v1::EraseResponse),
    /// Restore.
    Restore,
    /// Slept.
    Slept,
    /// Failed.
    Err(CallError),
}

impl Outcome {
    fn json(&self) -> Option<serde_json::Value> {
        Some(match self {
            Self::Append(r) => trace::append_json(r),
            Self::Current(r) => trace::current_json(r),
            Self::History(r) => trace::history_json(r),
            Self::Retract(r) => trace::retract_json(r),
            Self::Erase(r) => trace::erase_json(r),
            Self::Restore => serde_json::json!({}),
            Self::Slept | Self::Err(_) => return None,
        })
    }
}

/// Send one concrete request.
pub async fn call(client: &dyn LedgerClient, c: Concrete) -> Outcome {
    match c {
        Concrete::Append(r) => client
            .append(r)
            .await
            .map_or_else(Outcome::Err, Outcome::Append),
        Concrete::Current(r) => client
            .current(r)
            .await
            .map_or_else(Outcome::Err, Outcome::Current),
        Concrete::History(r) => client
            .history(r)
            .await
            .map_or_else(Outcome::Err, Outcome::History),
        Concrete::Retract(r) => client
            .retract(r)
            .await
            .map_or_else(Outcome::Err, Outcome::Retract),
        Concrete::Erase(r) => client
            .erase(r)
            .await
            .map_or_else(Outcome::Err, Outcome::Erase),
        Concrete::Restore(r) => client
            .restore(r)
            .await
            .map_or_else(Outcome::Err, |_| Outcome::Restore),
        Concrete::Sleep(d) => {
            tokio::time::sleep(d).await;
            Outcome::Slept
        }
    }
}

/// How many peers a trace refers to.
fn peers_of(trace: &[Step]) -> usize {
    trace
        .iter()
        .flat_map(|s| {
            let mut refs = Vec::new();
            if let Some(SubjectRef::Peer(k)) = s.request.subject() {
                refs.push(usize::from(k) + 1);
            }
            if let Request::Append {
                counterparty: Some(SubjectRef::Peer(k)),
                ..
            } = &s.request
            {
                refs.push(usize::from(*k) + 1);
            }
            refs
        })
        .max()
        .unwrap_or(0)
}

/// Replay `trace` on fresh subjects against `client`.
pub async fn replay(
    trace: &[Step],
    client: &dyn LedgerClient,
    skew: Duration,
    tolerate: &[String],
) -> Replayed {
    let binding = Binding {
        own: Uuid::now_v7(),
        peers: (0..peers_of(trace)).map(|_| Uuid::now_v7()).collect(),
    };
    let mut interpreter = Interpreter::new(&binding, skew, tolerate.to_vec());
    let mut steps: Vec<Step> = Vec::with_capacity(trace.len());
    let mut violations = Vec::new();
    let started = std::time::Instant::now();
    for original in trace {
        let request = original.request.clone();
        let Ok(concrete) = request.materialise(&binding, &steps) else {
            continue;
        };
        let outcome = call(client, concrete.clone()).await;
        let step = Step {
            index: original.index,
            request: request.clone(),
            response: outcome.json(),
            error: match &outcome {
                Outcome::Err(e) => Some(e.clone()),
                _ => None,
            },
            at_ms: started.elapsed().as_secs_f64() * 1000.0,
            tolerated: false,
        };
        steps.push(step);
        violations.extend(interpreter.observe(&request, &concrete, &outcome, &steps));
    }
    Replayed { steps, violations }
}

/// Replay a finding's trace `attempts` times and report.
pub async fn replay_finding(
    trace: &[Step],
    invariant: &str,
    signature: &str,
    client: Arc<dyn LedgerClient>,
    target: &str,
    attempts: u32,
    skew: Duration,
    tolerate: &[String],
) -> ReplayOutcome {
    let mut reproduced = false;
    let mut message = None;
    let mut steps_run = 0;
    for _ in 0..attempts.max(1) {
        let r = replay(trace, client.as_ref(), skew, tolerate).await;
        steps_run = r.steps.len();
        if r.reproduces(invariant, signature) {
            reproduced = true;
            message = r.first_of(invariant).map(|v| v.message.clone());
            break;
        }
        if message.is_none() {
            message = r.first_of(invariant).map(|v| v.message.clone());
        }
    }
    ReplayOutcome {
        at: Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        target: target.to_owned(),
        reproduced,
        message,
        steps_run,
    }
}
