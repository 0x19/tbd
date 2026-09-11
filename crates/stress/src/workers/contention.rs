//! The contention worker: many workers on the same few subjects. Nobody holds
//! the truth about a shared subject, so the model is order-free: what I was
//! acknowledged is visible (or retracted later by someone), `Current` agrees
//! with a `History` snapshot taken just before it or is newer than the
//! snapshot, pages are ordered and paginate, retractions answer a tombstone or
//! `NotFound`, and nothing fails unclean. Findings carry this worker's own
//! steps and are kept whole: replaying them alone rarely reproduces a race,
//! and the trace says what this worker saw.

use std::sync::Arc;

use rand::{Rng, SeedableRng, rngs::StdRng, seq::IndexedRandom};
use tbd_proto::ledger::v1::Fact;
use uuid::Uuid;

use super::{Context, Lane, judge, send};
use crate::{
    campaign::ContentionOp,
    finding::WorkerClass,
    model::{
        generate,
        invariants::{self, Acked},
    },
    trace::{self, Binding, Concrete, Request, SubjectRef, TimeRef},
};

/// Steps kept per shared subject.
const TRACE_DEPTH: usize = 200;
/// Pages a read walks before giving up on the cursor chain.
const MAX_PAGES: usize = 200;

/// One contention worker.
pub struct Contention {
    ctx: Arc<Context>,
    rng: StdRng,
    lanes: Vec<Lane>,
    /// My acknowledged appends, per shared subject.
    acked: Vec<Vec<Acked>>,
    scopes: Vec<String>,
}

impl Contention {
    /// A worker over `subjects`, the set every contention worker shares.
    #[must_use]
    pub fn new(ctx: Arc<Context>, seed: u64, subjects: &[Uuid]) -> Self {
        let scopes = ctx.campaign.workload.scopes.clone();
        Self {
            ctx,
            rng: StdRng::seed_from_u64(seed),
            lanes: subjects
                .iter()
                .map(|s| Lane::new(*s, TRACE_DEPTH))
                .collect(),
            acked: subjects.iter().map(|_| Vec::new()).collect(),
            scopes,
        }
    }

    /// Drive the target until cancelled.
    pub async fn run(mut self) {
        let mix = self.ctx.campaign.workload.contention.mix.weighted();
        let total: f64 = mix.iter().map(|(_, w)| w).sum();
        let pace = self.ctx.campaign.workload.contention.pace;
        while !self.ctx.cancel.is_cancelled() {
            let mut pick = self.rng.random_range(0.0..total);
            let mut op = mix[0].0;
            for (o, w) in &mix {
                if pick < *w {
                    op = *o;
                    break;
                }
                pick -= w;
            }
            let si = self.rng.random_range(0..self.lanes.len());
            match op {
                ContentionOp::Append => self.op_append(si).await,
                ContentionOp::Current => self.op_current(si).await,
                ContentionOp::History => self.op_history(si).await,
                ContentionOp::Retract => self.op_retract(si).await,
            }
            if !pace.is_zero() {
                tokio::select! {
                    () = tokio::time::sleep(pace) => {}
                    () = self.ctx.cancel.cancelled() => break,
                }
            }
        }
    }

    /// Shared subject `si` as a symbolic reference: the first is `Own`, the
    /// rest peers, so a trace binds to the same set on replay.
    fn subject_ref(si: usize) -> SubjectRef {
        if si == 0 {
            SubjectRef::Own
        } else {
            SubjectRef::Peer(u8::try_from(si - 1).unwrap_or(u8::MAX))
        }
    }

    fn binding(&self) -> Binding {
        Binding {
            own: self.lanes[0].subject,
            peers: self.lanes[1..].iter().map(|l| l.subject).collect(),
        }
    }

    fn page_limit(&mut self) -> u32 {
        let cfg = self.ctx.campaign.workload.contention.limit;
        if cfg > 0 {
            return cfg;
        }
        *[1u32, 3, 10, 50, 1000]
            .choose(&mut self.rng)
            .unwrap_or(&1000)
    }

    fn judge(&self, si: usize, evaluated: &[&'static str], v: Vec<trace::Violation>) {
        judge(
            &self.ctx,
            &self.lanes[si],
            WorkerClass::Contention,
            evaluated,
            v,
        );
    }

    async fn op_append(&mut self, si: usize) {
        let paths = self.ctx.campaign.workload.paths.clone();
        let path = paths
            .choose(&mut self.rng)
            .cloned()
            .unwrap_or_else(|| "profile.name".into());
        let (source, confidence) = generate::source(&mut self.rng);
        let request = Request::Append {
            subject: Self::subject_ref(si),
            path,
            source,
            value: generate::value(&mut self.rng),
            origin: serde_json::json!({"by": "stress", "class": "contention"}),
            confidence,
            counterparty: None,
            observed_at: TimeRef::At(chrono::Utc::now()),
            expires_at: None,
            consent: generate::consent(&mut self.rng, &self.scopes),
            stub: false,
            idempotency_key: Some(generate::key(&mut self.rng)),
        };
        let Ok(Concrete::Append(req)) = request.materialise(&self.binding(), &[]) else {
            return;
        };
        let client = Arc::clone(&self.ctx.client);
        let sent = req.clone();
        let (result, _) = send(
            &self.ctx,
            &mut self.lanes[si],
            WorkerClass::Contention,
            request,
            async move { client.append(sent).await },
            trace::append_json,
        )
        .await;
        if let Ok(resp) = result {
            let v = invariants::append_echo_shared(&req, &resp);
            self.judge(si, &["append_echo"], v);
            if let Some(f) = &resp.fact
                && let Some(stamp) = invariants::wire_stamp(f)
            {
                self.acked[si].push(Acked {
                    id: f.id,
                    stamp,
                    key: (f.path.clone(), f.source),
                });
            }
        }
    }

    /// Every page of `History` (or `Current`), judged for order and pagination.
    async fn walk(&mut self, si: usize, current: bool, limit: u32) -> Option<Vec<Fact>> {
        let mut pages = Vec::new();
        let mut nexts = Vec::new();
        let mut cursor_from = None;
        let binding = self.binding();
        for _ in 0..MAX_PAGES {
            let request = if current {
                Request::Current {
                    subject: Self::subject_ref(si),
                    paths: vec![],
                    sources: vec![],
                    scopes: self.scopes.clone(),
                    limit,
                    cursor_from,
                }
            } else {
                Request::History {
                    subject: Self::subject_ref(si),
                    paths: vec![],
                    sources: vec![],
                    scopes: self.scopes.clone(),
                    limit,
                    cursor_from,
                    at: None,
                }
            };
            let steps = self.lanes[si].steps();
            let Ok(concrete) = request.materialise(&binding, &steps) else {
                return None;
            };
            let index = self.lanes[si].next_index();
            let client = Arc::clone(&self.ctx.client);
            let (facts, next) = match concrete {
                Concrete::Current(req) => {
                    let (r, _) = send(
                        &self.ctx,
                        &mut self.lanes[si],
                        WorkerClass::Contention,
                        request,
                        async move { client.current(req).await },
                        trace::current_json,
                    )
                    .await;
                    match r {
                        Ok(r) => (r.facts, r.next),
                        Err(e) => return self.read_failed(si, &e),
                    }
                }
                Concrete::History(req) => {
                    let (r, _) = send(
                        &self.ctx,
                        &mut self.lanes[si],
                        WorkerClass::Contention,
                        request,
                        async move { client.history(req).await },
                        trace::history_json,
                    )
                    .await;
                    match r {
                        Ok(r) => (r.facts, r.next),
                        Err(e) => return self.read_failed(si, &e),
                    }
                }
                _ => return None,
            };
            let done = next.is_empty();
            nexts.push(next);
            pages.push(facts);
            if done {
                break;
            }
            cursor_from = Some(index);
        }
        let all: Vec<Fact> = pages.iter().flatten().cloned().collect();
        let mut v = invariants::pagination(&pages, &nexts, limit);
        v.extend(invariants::page_ordered(&all));
        self.judge(si, &["pagination", "recorded_at_monotonic"], v);
        Some(all)
    }

    /// A read failed: `NotFound` is the contract while nobody has written the
    /// subject yet; anything else outside the tolerated classes is a finding.
    fn read_failed(&self, si: usize, e: &crate::client::CallError) -> Option<Vec<Fact>> {
        let tolerate = &self.ctx.campaign.faults.tolerate;
        if tolerate.iter().any(|t| *t == e.class()) || e.is_unclean() {
            return None;
        }
        if self.acked[si].is_empty() && e.code() == Some(tonic::Code::NotFound) {
            return None;
        }
        let v = trace::Violation {
            invariant: "acked_visible",
            message: format!("a read on a shared subject failed with {}", e.class()),
            expected: serde_json::json!("a page"),
            actual: serde_json::to_value(e).unwrap_or_default(),
        };
        self.judge(si, &[], vec![v]);
        None
    }

    async fn op_history(&mut self, si: usize) {
        let limit = self.page_limit();
        let Some(history) = self.walk(si, false, limit).await else {
            return;
        };
        let v = invariants::acked_visible(&self.acked[si], &history);
        self.judge(si, &["acked_visible"], v);
    }

    async fn op_current(&mut self, si: usize) {
        let limit = self.page_limit();
        let Some(snapshot) = self.walk(si, false, limit).await else {
            return;
        };
        let Some(current) = self.walk(si, true, limit).await else {
            return;
        };
        let v = invariants::current_consistent(&snapshot, &current);
        self.judge(si, &["current_consistent"], v);
    }

    async fn op_retract(&mut self, si: usize) {
        // A key I wrote, or any key: someone else may have written or retracted it.
        let (path, source) = if let Some(a) = self.acked[si].choose(&mut self.rng)
            && self.rng.random_bool(0.7)
        {
            a.key.clone()
        } else {
            let paths = self.ctx.campaign.workload.paths.clone();
            (
                paths
                    .choose(&mut self.rng)
                    .cloned()
                    .unwrap_or_else(|| "profile.name".into()),
                generate::source(&mut self.rng).0,
            )
        };
        let request = Request::Retract {
            subject: Self::subject_ref(si),
            path: path.clone(),
            source,
            origin: serde_json::json!({"by": "stress", "class": "contention"}),
        };
        let Ok(Concrete::Retract(req)) = request.materialise(&self.binding(), &[]) else {
            return;
        };
        let client = Arc::clone(&self.ctx.client);
        let (result, tolerated) = send(
            &self.ctx,
            &mut self.lanes[si],
            WorkerClass::Contention,
            request,
            async move { client.retract(req).await },
            trace::retract_json,
        )
        .await;
        if tolerated {
            return;
        }
        let v = invariants::tombstone_shape(&path, source, result.as_ref());
        self.judge(si, &["retract_semantics"], v);
    }

    /// Subjects this worker writes to.
    #[must_use]
    pub fn subject_count(&self) -> usize {
        self.lanes.len()
    }
}
