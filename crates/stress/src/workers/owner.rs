//! The owner worker: a few subjects nobody else writes to, an exact model of
//! each, and every answer judged against it. This is where most findings come
//! from, because here the model knows the truth.

use std::{collections::VecDeque, sync::Arc, time::Duration};

use chrono::Utc;
use rand::{Rng, SeedableRng, rngs::StdRng, seq::IndexedRandom};
use tbd_proto::ledger::v1::{
    AppendResponse, CurrentResponse, EraseResponse, Fact, HistoryResponse, RestoreResponse,
    RetractResponse,
};
use tonic::Code;
use uuid::Uuid;

use super::Context;
use crate::{
    campaign::OwnerOp,
    client::CallError,
    finding::{Finding, WorkerClass},
    model::{
        SubjectModel, generate,
        invariants::{self, ReplayKind},
    },
    trace::{
        self, Binding, Concrete, Request, Step, SubjectRef, TimeRef, Violation, recorded_at_of,
    },
};

/// Steps kept per subject; a finding's trace is at most this long.
pub const TRACE_DEPTH: usize = 400;
/// Pages a read walks before giving up on the cursor chain.
const MAX_PAGES: usize = 200;
/// Re-drives of a write after a tolerated failure.
const REDRIVES: usize = 8;
/// An erasure window at least this long is treated as real: restore must work
/// inside it, and the cascade is not awaited.
const SAFE_WINDOW: Duration = Duration::from_secs(2);

struct Subject {
    model: SubjectModel,
    trace: VecDeque<Step>,
    next_index: usize,
    /// Indexes of acknowledged keyed appends, for replays.
    appends: Vec<usize>,
}

impl Subject {
    fn new() -> Self {
        Self {
            model: SubjectModel::new(Uuid::now_v7()),
            trace: VecDeque::new(),
            next_index: 0,
            appends: Vec::new(),
        }
    }

    fn steps(&self) -> Vec<Step> {
        self.trace.iter().cloned().collect()
    }

    fn step(&self, index: usize) -> Option<&Step> {
        self.trace.iter().find(|s| s.index == index)
    }
}

/// What a re-driven write settled to.
enum Settled<T> {
    /// An answer, from the first attempt or a re-drive.
    Answer(Result<T, CallError>),
    /// Every attempt drew a tolerated failure.
    Unknown,
}

/// One owner worker.
pub struct Owner {
    ctx: Arc<Context>,
    rng: StdRng,
    subjects: Vec<Subject>,
    scopes: Vec<String>,
    tolerate: Vec<String>,
    skew: chrono::Duration,
}

impl Owner {
    /// A worker with `subjects` fresh subjects, seeded.
    #[must_use]
    pub fn new(ctx: Arc<Context>, seed: u64, subjects: u32) -> Self {
        let scopes = ctx.campaign.workload.scopes.clone();
        let tolerate = ctx.campaign.faults.tolerate.clone();
        let skew = chrono::Duration::from_std(ctx.campaign.faults.clock_skew)
            .unwrap_or_else(|_| chrono::Duration::milliseconds(500));
        Self {
            ctx,
            rng: StdRng::seed_from_u64(seed),
            subjects: (0..subjects.max(1)).map(|_| Subject::new()).collect(),
            scopes,
            tolerate,
            skew,
        }
    }

    /// Drive the target until cancelled.
    pub async fn run(mut self) {
        let mix = self.ctx.campaign.workload.owner.mix.weighted();
        let total: f64 = mix.iter().map(|(_, w)| w).sum();
        let pace = self.ctx.campaign.workload.owner.pace;
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
            let si = self.rng.random_range(0..self.subjects.len());
            self.execute(op, si).await;
            if !pace.is_zero() {
                tokio::select! {
                    () = tokio::time::sleep(pace) => {}
                    () = self.ctx.cancel.cancelled() => break,
                }
            }
        }
    }

    /// Subjects this worker owns.
    #[must_use]
    pub fn subject_count(&self) -> usize {
        self.subjects.len()
    }

    async fn execute(&mut self, op: OwnerOp, si: usize) {
        match op {
            OwnerOp::Append => {
                self.op_append(si, false).await;
            }
            OwnerOp::Expiring => {
                self.op_append(si, true).await;
            }
            OwnerOp::Current => self.op_current(si).await,
            OwnerOp::History => self.op_history(si).await,
            OwnerOp::HistoryCut => self.op_history_cut(si).await,
            OwnerOp::Retract => self.op_retract(si).await,
            OwnerOp::IdempotentReplay => self.op_replay(si).await,
            OwnerOp::PairRelation => self.op_pair(si).await,
            OwnerOp::EraseCycle => self.op_erase_cycle(si).await,
        }
    }

    // ------------------------------------------------------------ plumbing

    fn binding(&self, si: usize) -> Binding {
        Binding {
            own: self.subjects[si].model.id,
            peers: self
                .subjects
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != si)
                .map(|(_, s)| s.model.id)
                .collect(),
        }
    }

    /// The `Peer(k)` of subject `pj` as seen from subject `si`.
    fn peer_ref(&self, si: usize, pj: usize) -> SubjectRef {
        let k = self
            .subjects
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != si)
            .position(|(i, _)| i == pj)
            .unwrap_or(0);
        SubjectRef::Peer(u8::try_from(k).unwrap_or(u8::MAX))
    }

    fn push_step(
        &mut self,
        si: usize,
        request: Request,
        response: Option<serde_json::Value>,
        error: Option<CallError>,
        tolerated: bool,
    ) -> usize {
        let at_ms = self.ctx.started.elapsed().as_secs_f64() * 1000.0;
        let s = &mut self.subjects[si];
        let index = s.next_index;
        s.next_index += 1;
        s.trace.push_back(Step {
            index,
            request,
            response,
            error,
            at_ms,
            tolerated,
        });
        while s.trace.len() > TRACE_DEPTH {
            s.trace.pop_front();
        }
        index
    }

    /// Count an evaluation and turn every violation into a finding.
    fn judge(&mut self, si: usize, evaluated: &[&'static str], violations: Vec<Violation>) {
        for name in evaluated {
            if !violations.iter().any(|v| v.invariant == *name) {
                self.ctx.checks.pass(name);
            }
        }
        for v in violations {
            if !self.ctx.campaign.invariant_on(v.invariant) {
                continue;
            }
            self.ctx.checks.violate(v.invariant);
            let f = Finding::new(
                &v,
                self.subjects[si].steps(),
                self.subjects[si].model.id,
                WorkerClass::Owner,
                &self.ctx.campaign.campaign.name,
                &self.ctx.target,
                self.ctx.store.as_deref(),
            );
            tracing::warn!(invariant = v.invariant, message = %v.message, subject = %f.subject, "finding");
            let _ = self.ctx.findings.send(f);
        }
    }

    /// One call: materialise, send, time, record, and judge the failure class.
    fn call(&self, si: usize, request: &Request) -> Result<Concrete, ()> {
        let binding = self.binding(si);
        let steps = self.subjects[si].steps();
        request.materialise(&binding, &steps).map_err(|e| {
            tracing::debug!(%e, "request not materialisable; skipped");
        })
    }

    async fn send<T>(
        &mut self,
        si: usize,
        request: Request,
        fut: impl Future<Output = Result<T, CallError>>,
        json: impl FnOnce(&T) -> serde_json::Value,
    ) -> Result<T, CallError> {
        let permit = self.ctx.semaphore.clone().acquire_owned().await;
        let started = std::time::Instant::now();
        let result = fut.await;
        drop(permit);
        let op = request.op();
        self.ctx.metrics.record(
            &self.ctx.target,
            op,
            result.as_ref().map(|_| ()).map_err(CallError::class),
            started.elapsed(),
        );
        let tolerated = result
            .as_ref()
            .err()
            .is_some_and(|e| self.tolerate.iter().any(|t| *t == e.class()));
        if tolerated {
            self.ctx.checks.tolerated();
        }
        let (response, error) = match &result {
            Ok(v) => (Some(json(v)), None),
            Err(e) => (None, Some(e.clone())),
        };
        self.push_step(si, request, response, error, tolerated);
        if let Err(e) = &result
            && !tolerated
            && let Some(v) = invariants::clean_errors(e, &self.tolerate)
        {
            self.judge(si, &["clean_errors"], vec![v]);
        } else if result.is_ok() {
            self.ctx.checks.pass("clean_errors");
        }
        result
    }

    async fn append(
        &mut self,
        si: usize,
        request: Request,
    ) -> Option<Result<AppendResponse, CallError>> {
        let Ok(Concrete::Append(req)) = self.call(si, &request) else {
            return None;
        };
        Some(self.send_append(si, request, req).await)
    }

    /// Send an already materialised append, so a re-drive sends the same bytes.
    async fn send_append(
        &mut self,
        si: usize,
        mut request: Request,
        req: tbd_proto::ledger::v1::AppendRequest,
    ) -> Result<AppendResponse, CallError> {
        // The trace keeps the instants that were sent, so a replay (idempotency
        // or a finding's) sends the same bytes.
        if let Request::Append {
            observed_at,
            expires_at,
            ..
        } = &mut request
        {
            if let Some(t) = req.observed_at.as_ref().and_then(trace::datetime) {
                *observed_at = TimeRef::At(t);
            }
            if let Some(t) = req.expires_at.as_ref().and_then(trace::datetime) {
                *expires_at = Some(TimeRef::At(t));
            }
        }
        let client = Arc::clone(&self.ctx.client);
        self.send(
            si,
            request,
            async move { client.append(req).await },
            trace::append_json,
        )
        .await
    }

    async fn current(
        &mut self,
        si: usize,
        request: Request,
    ) -> Option<Result<CurrentResponse, CallError>> {
        let Ok(Concrete::Current(req)) = self.call(si, &request) else {
            return None;
        };
        Some({
            let client = Arc::clone(&self.ctx.client);
            self.send(
                si,
                request,
                async move { client.current(req).await },
                trace::current_json,
            )
            .await
        })
    }

    async fn history(
        &mut self,
        si: usize,
        request: Request,
    ) -> Option<Result<HistoryResponse, CallError>> {
        let Ok(Concrete::History(req)) = self.call(si, &request) else {
            return None;
        };
        Some({
            let client = Arc::clone(&self.ctx.client);
            self.send(
                si,
                request,
                async move { client.history(req).await },
                trace::history_json,
            )
            .await
        })
    }

    async fn retract(
        &mut self,
        si: usize,
        request: Request,
    ) -> Option<Result<RetractResponse, CallError>> {
        let Ok(Concrete::Retract(req)) = self.call(si, &request) else {
            return None;
        };
        Some({
            let client = Arc::clone(&self.ctx.client);
            self.send(
                si,
                request,
                async move { client.retract(req).await },
                trace::retract_json,
            )
            .await
        })
    }

    async fn erase(
        &mut self,
        si: usize,
        request: Request,
    ) -> Option<Result<EraseResponse, CallError>> {
        let Ok(Concrete::Erase(req)) = self.call(si, &request) else {
            return None;
        };
        Some({
            let client = Arc::clone(&self.ctx.client);
            self.send(
                si,
                request,
                async move { client.erase(req).await },
                trace::erase_json,
            )
            .await
        })
    }

    async fn restore(
        &mut self,
        si: usize,
        request: Request,
    ) -> Option<Result<RestoreResponse, CallError>> {
        let Ok(Concrete::Restore(req)) = self.call(si, &request) else {
            return None;
        };
        Some({
            let client = Arc::clone(&self.ctx.client);
            self.send(
                si,
                request,
                async move { client.restore(req).await },
                |_| serde_json::json!({}),
            )
            .await
        })
    }

    async fn sleep(&mut self, si: usize, d: Duration) {
        self.push_step(
            si,
            Request::Sleep {
                ms: u64::try_from(d.as_millis()).unwrap_or(u64::MAX),
            },
            None,
            None,
            false,
        );
        tokio::select! {
            () = tokio::time::sleep(d) => {}
            () = self.ctx.cancel.cancelled() => {}
        }
    }

    /// Send an append; on a tolerated failure re-send the same request (same
    /// key), so the outcome is known: replayed means the first one landed.
    async fn append_settled(
        &mut self,
        si: usize,
        request: Request,
    ) -> Option<(
        tbd_proto::ledger::v1::AppendRequest,
        Settled<AppendResponse>,
    )> {
        let Ok(Concrete::Append(req)) = self.call(si, &request) else {
            return None;
        };
        let mut attempts = 0;
        loop {
            let result = self.send_append(si, request.clone(), req.clone()).await;
            match result {
                Err(e) if self.tolerate.iter().any(|t| *t == e.class()) && attempts < REDRIVES => {
                    attempts += 1;
                    self.ctx.checks.redriven();
                }
                Err(e) if self.tolerate.iter().any(|t| *t == e.class()) => {
                    return Some((req, Settled::Unknown));
                }
                // A re-drive tells the truth about the first attempt: `replayed`
                // means it committed; either way the model now holds exactly what
                // the ledger holds.
                r => return Some((req, Settled::Answer(r))),
            }
        }
    }

    /// Send a retract; on a tolerated failure re-send it. `NotFound` after a
    /// failure means the first attempt did it.
    async fn retract_settled(
        &mut self,
        si: usize,
        request: Request,
        had_value: bool,
    ) -> Option<(Settled<RetractResponse>, bool)> {
        let mut attempts = 0;
        let mut failed_before = false;
        loop {
            let result = self.retract(si, request.clone()).await?;
            match result {
                Err(e) if self.tolerate.iter().any(|t| *t == e.class()) && attempts < REDRIVES => {
                    attempts += 1;
                    failed_before = true;
                    self.ctx.checks.redriven();
                }
                Err(e) if self.tolerate.iter().any(|t| *t == e.class()) => {
                    return Some((Settled::Unknown, failed_before));
                }
                Err(e) if failed_before && had_value && e.code() == Some(Code::NotFound) => {
                    // The lost acknowledgement was for a retract that ran.
                    return Some((Settled::Unknown, true));
                }
                r => return Some((Settled::Answer(r), failed_before)),
            }
        }
    }

    /// Walk a `Current` through every page.
    async fn walk_current(
        &mut self,
        si: usize,
        paths: Vec<String>,
        sources: Vec<i32>,
        scopes: Vec<String>,
        limit: u32,
    ) -> Option<Vec<Vec<Fact>>> {
        let mut pages = Vec::new();
        let mut nexts = Vec::new();
        let mut cursor_from = None;
        for _ in 0..MAX_PAGES {
            let req = Request::Current {
                subject: SubjectRef::Own,
                paths: paths.clone(),
                sources: sources.clone(),
                scopes: scopes.clone(),
                limit,
                cursor_from,
            };
            let index = self.subjects[si].next_index;
            let resp = self.current(si, req).await?;
            match resp {
                Ok(r) => {
                    let done = r.next.is_empty();
                    nexts.push(r.next.clone());
                    pages.push(r.facts);
                    if done {
                        break;
                    }
                    cursor_from = Some(index);
                }
                Err(e) => {
                    self.read_failed(si, "current", &e);
                    return None;
                }
            }
        }
        self.judge(
            si,
            &["pagination"],
            invariants::pagination(&pages, &nexts, limit),
        );
        Some(pages)
    }

    /// Walk a `History` through every page.
    async fn walk_history(
        &mut self,
        si: usize,
        at: Option<TimeRef>,
        limit: u32,
    ) -> Option<Vec<Vec<Fact>>> {
        let mut pages = Vec::new();
        let mut nexts = Vec::new();
        let mut cursor_from = None;
        for _ in 0..MAX_PAGES {
            let req = Request::History {
                subject: SubjectRef::Own,
                paths: vec![],
                sources: vec![],
                scopes: self.scopes.clone(),
                limit,
                cursor_from,
                at,
            };
            let index = self.subjects[si].next_index;
            let resp = self.history(si, req).await?;
            match resp {
                Ok(r) => {
                    let done = r.next.is_empty();
                    nexts.push(r.next.clone());
                    pages.push(r.facts);
                    if done {
                        break;
                    }
                    cursor_from = Some(index);
                }
                Err(e) => {
                    self.read_failed(si, "history", &e);
                    return None;
                }
            }
        }
        self.judge(
            si,
            &["pagination"],
            invariants::pagination(&pages, &nexts, limit),
        );
        Some(pages)
    }

    /// A read on an open, existing subject failed: `NotFound` on a subject the
    /// model knows exists, `FailedPrecondition` when it is not erased, are
    /// contract violations; tolerated classes were counted by `send`.
    fn read_failed(&mut self, si: usize, what: &str, e: &CallError) {
        if self.tolerate.iter().any(|t| *t == e.class()) || e.is_unclean() {
            return;
        }
        let exists = self.subjects[si].model.exists();
        if !exists && e.code() == Some(Code::NotFound) {
            self.ctx.checks.pass("current_is_latest");
            return;
        }
        let v = Violation {
            invariant: if what == "history" {
                "history_is_everything"
            } else {
                "current_is_latest"
            },
            message: format!("{what} on an open subject failed with {}", e.class()),
            expected: serde_json::json!("a page"),
            actual: serde_json::to_value(e).unwrap_or_default(),
        };
        self.judge(si, &[], vec![v]);
    }

    fn page_limit(&mut self) -> u32 {
        let cfg = self.ctx.campaign.workload.owner.limit;
        if cfg > 0 {
            return cfg;
        }
        *[1u32, 2, 3, 5, 10, 50, 1000]
            .choose(&mut self.rng)
            .unwrap_or(&1000)
    }

    fn fresh_append(&mut self, si: usize, expiring: bool) -> Request {
        let paths = self.ctx.campaign.workload.paths.clone();
        let path = paths
            .choose(&mut self.rng)
            .cloned()
            .unwrap_or_else(|| "profile.name".into());
        let (source, confidence) = generate::source(&mut self.rng);
        let expires_at = expiring.then(|| {
            let ms = *[-2000i64, -60_000, 3000, 60_000]
                .choose(&mut self.rng)
                .unwrap_or(&60_000);
            TimeRef::OffsetFromNow { ms }
        });
        // The ledger requires `expires_at > observed_at`: a fact that has already
        // expired was observed before that.
        let observed_at = TimeRef::At(Utc::now() - chrono::Duration::milliseconds(120_000));
        let _ = si;
        Request::Append {
            subject: SubjectRef::Own,
            path,
            source,
            value: generate::value(&mut self.rng),
            origin: serde_json::json!({"by": "stress"}),
            confidence,
            counterparty: None,
            observed_at,
            expires_at,
            consent: generate::consent(&mut self.rng, &self.scopes),
            stub: self.rng.random_bool(0.05),
            idempotency_key: Some(generate::key(&mut self.rng)),
        }
    }

    // ------------------------------------------------------------ operations

    /// Append one fact; returns whether it was acknowledged.
    async fn op_append(&mut self, si: usize, expiring: bool) -> bool {
        let request = self.fresh_append(si, expiring);
        self.append_request(si, request).await
    }

    async fn append_request(&mut self, si: usize, request: Request) -> bool {
        let Some((concrete, settled)) = self.append_settled(si, request).await else {
            return false;
        };
        match settled {
            Settled::Answer(Ok(resp)) => {
                let v = invariants::append_echo(&concrete, &resp, &self.subjects[si].model);
                self.judge(si, &["append_echo", "recorded_at_monotonic"], v);
                let index = self.subjects[si].next_index - 1;
                self.subjects[si].model.apply_append(&concrete, &resp);
                // Relations are never replayed: the ledger checks the counterparty
                // before the key, so once the peer is erased the same key answers
                // `NotFound`, which is the contract and not an idempotency failure.
                if !resp.replayed
                    && !concrete.idempotency_key.is_empty()
                    && concrete.counterparty_id.is_none()
                {
                    self.subjects[si].appends.push(index);
                }
                true
            }
            Settled::Answer(Err(e)) => {
                if !self.tolerate.iter().any(|t| *t == e.class()) && !e.is_unclean() {
                    let v = Violation {
                        invariant: "append_echo",
                        message: format!("append on an open subject failed with {}", e.class()),
                        expected: serde_json::json!("a fact"),
                        actual: serde_json::to_value(&e).unwrap_or_default(),
                    };
                    self.judge(si, &[], vec![v]);
                }
                false
            }
            Settled::Unknown => {
                self.subjects[si].model.apply_pending(&concrete);
                false
            }
        }
    }

    async fn op_current(&mut self, si: usize) {
        if !self.subjects[si].model.exists() {
            return;
        }
        let limit = self.page_limit();
        let Some(pages) = self
            .walk_current(si, vec![], vec![], self.scopes.clone(), limit)
            .await
        else {
            return;
        };
        let facts: Vec<Fact> = pages.into_iter().flatten().collect();
        let now = Utc::now();
        let has_expiry = self.subjects[si]
            .model
            .facts
            .iter()
            .any(|f| f.expires_at.is_some());
        let v = invariants::current_is_latest(&self.subjects[si].model, now, self.skew, &facts);
        let mut evaluated = vec!["current_is_latest", "recorded_at_monotonic"];
        if has_expiry {
            evaluated.push("expiry");
        }
        self.judge(si, &evaluated, v);

        self.op_current_filters(si).await;
    }

    /// Reads under a scope subset and under path and source filters.
    async fn op_current_filters(&mut self, si: usize) {
        if self.rng.random_bool(0.5) && self.scopes.len() > 1 {
            let subset: Vec<String> = {
                let mut s = generate::consent(&mut self.rng, &self.scopes);
                if s.len() == self.scopes.len() {
                    s.pop();
                }
                s
            };
            if !subset.is_empty() {
                let limit = self.page_limit();
                if let Some(pages) = self
                    .walk_current(si, vec![], vec![], subset.clone(), limit)
                    .await
                {
                    let facts: Vec<Fact> = pages.into_iter().flatten().collect();
                    let v = invariants::consent_filter(
                        &self.subjects[si].model,
                        Utc::now(),
                        self.skew,
                        &subset,
                        &facts,
                    );
                    self.judge(si, &["consent_filter"], v);
                }
            }
        }
        if self.rng.random_bool(0.3) {
            let paths: Vec<String> = if self.rng.random_bool(0.5) {
                let p = self
                    .ctx
                    .campaign
                    .workload
                    .paths
                    .choose(&mut self.rng)
                    .cloned()
                    .unwrap_or_default();
                vec![p]
            } else {
                let p = self
                    .ctx
                    .campaign
                    .workload
                    .paths
                    .choose(&mut self.rng)
                    .cloned()
                    .unwrap_or_default();
                let prefix = p.rsplit_once('.').map_or(p.clone(), |(a, _)| a.to_owned());
                vec![format!("{prefix}.*")]
            };
            let sources: Vec<i32> = if self.rng.random_bool(0.5) {
                vec![generate::SOURCES[self.rng.random_range(0..generate::SOURCES.len())].0]
            } else {
                vec![]
            };
            let limit = self.page_limit();
            if let Some(pages) = self
                .walk_current(
                    si,
                    paths.clone(),
                    sources.clone(),
                    self.scopes.clone(),
                    limit,
                )
                .await
            {
                let facts: Vec<Fact> = pages.into_iter().flatten().collect();
                let v = invariants::path_source_filter(
                    &self.subjects[si].model,
                    Utc::now(),
                    self.skew,
                    &paths,
                    &sources,
                    &facts,
                );
                self.judge(si, &["path_source_filter"], v);
            }
        }
    }

    async fn op_history(&mut self, si: usize) {
        if !self.subjects[si].model.exists() {
            return;
        }
        let limit = self.page_limit();
        let Some(pages) = self.walk_history(si, None, limit).await else {
            return;
        };
        let facts: Vec<Fact> = pages.into_iter().flatten().collect();
        let v = invariants::history_is_everything(&self.subjects[si].model, &facts);
        self.judge(si, &["history_is_everything", "recorded_at_monotonic"], v);
        let m = &mut self.subjects[si].model;
        if m.has_unknown_stamps() || !m.pending.is_empty() {
            m.learn(&facts);
        }
    }

    async fn op_history_cut(&mut self, si: usize) {
        let candidates: Vec<usize> = self.subjects[si]
            .trace
            .iter()
            .filter(|s| recorded_at_of(s).is_some())
            .map(|s| s.index)
            .collect();
        let Some(&index) = candidates.choose(&mut self.rng) else {
            return;
        };
        let Some(at) = self.subjects[si].step(index).and_then(recorded_at_of) else {
            return;
        };
        let limit = self.page_limit();
        let Some(pages) = self
            .walk_history(si, Some(TimeRef::RecordedAtOf(index)), limit)
            .await
        else {
            return;
        };
        let facts: Vec<Fact> = pages.into_iter().flatten().collect();
        if let Some(v) = invariants::history_cut(&self.subjects[si].model, at, &facts) {
            self.judge(si, &["history_cut", "recorded_at_monotonic"], v);
        }
    }

    async fn op_retract(&mut self, si: usize) {
        if !self.subjects[si].model.exists() {
            return;
        }
        let keys = self.subjects[si].model.valued_keys();
        let (path, source) = if !keys.is_empty() && self.rng.random_bool(0.7) {
            keys.choose(&mut self.rng).cloned().unwrap_or_default()
        } else {
            let p = self
                .ctx
                .campaign
                .workload
                .paths
                .choose(&mut self.rng)
                .cloned()
                .unwrap_or_default();
            (
                p,
                generate::SOURCES[self.rng.random_range(0..generate::SOURCES.len())].0,
            )
        };
        let had_value = self.subjects[si].model.valued(&path, source).is_some();
        let origin = serde_json::json!({"by": "stress", "reason": "retract"});
        let request = Request::Retract {
            subject: SubjectRef::Own,
            path: path.clone(),
            source,
            origin: origin.clone(),
        };
        let Some((settled, _)) = self.retract_settled(si, request, had_value).await else {
            return;
        };
        match settled {
            Settled::Answer(r) => {
                let v = invariants::retract_semantics(
                    &self.subjects[si].model,
                    &path,
                    source,
                    r.as_ref(),
                );
                self.judge(si, &["retract_semantics", "recorded_at_monotonic"], v);
                if let Ok(resp) = r {
                    self.subjects[si].model.apply_retract(&path, source, &resp);
                }
            }
            Settled::Unknown => {
                if had_value {
                    self.subjects[si]
                        .model
                        .apply_retract_unknown(&path, source, origin);
                }
            }
        }
    }

    async fn op_replay(&mut self, si: usize) {
        let candidates: Vec<usize> = self.subjects[si]
            .appends
            .iter()
            .copied()
            .filter(|i| self.subjects[si].step(*i).is_some())
            .collect();
        let Some(&index) = candidates.choose(&mut self.rng) else {
            return;
        };
        let Some(step) = self.subjects[si].step(index).cloned() else {
            return;
        };
        let (
            Request::Append {
                idempotency_key: Some(key),
                ..
            },
            Some(fact_id),
        ) = (
            &step.request,
            step.response
                .as_ref()
                .and_then(|r| r.pointer("/fact/id"))
                .and_then(serde_json::Value::as_i64),
        )
        else {
            return;
        };
        let retracted = self.subjects[si]
            .model
            .keys
            .get(key)
            .is_some_and(|k| k.retracted);
        let conflict = self.rng.random_bool(0.3);
        let mut request = step.request.clone();
        if conflict && let Request::Append { value, .. } = &mut request {
            *value = serde_json::json!({"mutated": generate::value(&mut self.rng)});
        }
        let kind = if retracted {
            ReplayKind::AfterRetract
        } else if conflict {
            ReplayKind::Conflict
        } else {
            ReplayKind::Same { fact_id }
        };
        let Some(result) = self.append(si, request).await else {
            return;
        };
        if let Err(e) = &result
            && self.tolerate.iter().any(|t| *t == e.class())
        {
            return;
        }
        let v = invariants::idempotency(kind, result.as_ref());
        self.judge(si, &["idempotency"], v);
    }

    async fn op_pair(&mut self, si: usize) {
        if self.subjects.len() < 2 {
            return;
        }
        let peers: Vec<usize> = (0..self.subjects.len())
            .filter(|j| {
                *j != si
                    && self.subjects[*j].model.exists()
                    && self.subjects[*j].model.pending.is_empty()
            })
            .collect();
        let Some(&pj) = peers.choose(&mut self.rng) else {
            return;
        };
        let path = self
            .ctx
            .campaign
            .workload
            .relation_paths
            .choose(&mut self.rng)
            .cloned()
            .unwrap_or_default();
        let request = Request::Append {
            subject: SubjectRef::Own,
            path,
            source: 2,
            value: generate::value(&mut self.rng),
            origin: serde_json::json!({"by": "stress", "relation": true}),
            confidence: Some(1.0),
            counterparty: Some(self.peer_ref(si, pj)),
            observed_at: TimeRef::At(Utc::now()),
            expires_at: None,
            consent: generate::consent(&mut self.rng, &self.scopes),
            stub: false,
            idempotency_key: Some(generate::key(&mut self.rng)),
        };
        self.append_request(si, request).await;
    }

    async fn op_erase_cycle(&mut self, si: usize) {
        if !self.subjects[si].model.exists() && !self.op_append(si, false).await {
            return;
        }
        if !self.subjects[si].model.pending.is_empty() {
            return;
        }
        let own = Request::Erase {
            subject: SubjectRef::Own,
        };
        let Some(first) = self.erase(si, own.clone()).await else {
            return;
        };
        let first = match first {
            Ok(r) => r,
            Err(e) => {
                if !self.tolerate.iter().any(|t| *t == e.class()) && !e.is_unclean() {
                    self.judge(
                        si,
                        &[],
                        vec![Violation {
                            invariant: "erasure_denies",
                            message: format!("erase of an open subject failed with {}", e.class()),
                            expected: serde_json::json!("requested_at and executes_after"),
                            actual: serde_json::to_value(&e).unwrap_or_default(),
                        }],
                    );
                }
                return;
            }
        };
        self.subjects[si].model.apply_erase(&first);
        // A window shorter than this may close while the denials are checked
        // (the sweeper runs on its own clock), so restore is then allowed to
        // find the subject gone. A longer window is a real one: restore must work,
        // and the cascade cannot be awaited.
        let window = window_of(&first);
        let real_window = window.is_none_or(|w| w >= SAFE_WINDOW);
        self.erase_denials(si, &first, real_window).await;

        let restore = Request::Restore {
            subject: SubjectRef::Own,
        };
        match self.restore(si, restore).await {
            Some(Ok(_)) => {
                self.subjects[si].model.apply_restore();
                let limit = self.page_limit();
                if let Some(pages) = self
                    .walk_current(si, vec![], vec![], self.scopes.clone(), limit)
                    .await
                {
                    let facts: Vec<Fact> = pages.into_iter().flatten().collect();
                    let mut v = invariants::current_is_latest(
                        &self.subjects[si].model,
                        Utc::now(),
                        self.skew,
                        &facts,
                    );
                    for x in &mut v {
                        x.message = format!("after restore: {}", x.message);
                    }
                    self.judge(si, &["current_is_latest"], v);
                }
                if real_window {
                    return;
                }
                let Some(Ok(again)) = self.erase(si, own).await else {
                    return;
                };
                self.subjects[si].model.apply_erase(&again);
                self.cascade(si, window.unwrap_or_default()).await;
            }
            Some(Err(e)) if !real_window && e.code() == Some(Code::NotFound) => {
                // The sweeper executed the erasure before the restore arrived.
                self.cascade(si, window.unwrap_or_default()).await;
            }
            Some(Err(e)) if !self.tolerate.iter().any(|t| *t == e.class()) && !e.is_unclean() => {
                self.judge(
                    si,
                    &[],
                    vec![Violation {
                        invariant: "erasure_denies",
                        message: format!("restore inside the window failed with {}", e.class()),
                        expected: serde_json::json!("ok"),
                        actual: serde_json::to_value(&e).unwrap_or_default(),
                    }],
                );
            }
            Some(Err(_)) | None => {}
        }
    }

    /// Inside the window every call is denied and a relation naming the subject
    /// from a peer is denied too; with a real window a second erase is the same
    /// one (a short window may have closed already, so that is not judged).
    async fn erase_denials(&mut self, si: usize, first: &EraseResponse, real_window: bool) {
        let read = self
            .current(
                si,
                Request::Current {
                    subject: SubjectRef::Own,
                    paths: vec![],
                    sources: vec![],
                    scopes: self.scopes.clone(),
                    limit: 0,
                    cursor_from: None,
                },
            )
            .await;
        let write_req = self.fresh_append(si, false);
        let write = self.append(si, write_req).await;
        let mut v = Vec::new();
        if let Some(r) = &read {
            v.extend(invariants::denied("current", r.as_ref()));
        }
        if let Some(w) = &write {
            v.extend(invariants::denied("append", w.as_ref()));
        }
        if real_window {
            let own = Request::Erase {
                subject: SubjectRef::Own,
            };
            if let Some(s) = self.erase(si, own).await {
                v.extend(invariants::erase_idempotent(first, s.as_ref()));
            }
        }
        if self.subjects.len() > 1 {
            let pj = (si + 1) % self.subjects.len();
            if self.subjects[pj].model.exists() && self.subjects[pj].model.pending.is_empty() {
                let path = self
                    .ctx
                    .campaign
                    .workload
                    .relation_paths
                    .choose(&mut self.rng)
                    .cloned()
                    .unwrap_or_default();
                let rel = Request::Append {
                    subject: SubjectRef::Own,
                    path,
                    source: 2,
                    value: generate::value(&mut self.rng),
                    origin: serde_json::json!({"by": "stress", "relation": true}),
                    confidence: Some(1.0),
                    counterparty: Some(self.peer_ref(pj, si)),
                    observed_at: TimeRef::At(Utc::now()),
                    expires_at: None,
                    consent: vec![self.scopes[0].clone()],
                    stub: false,
                    idempotency_key: None,
                };
                if let Some(r) = self.append(pj, rel).await {
                    v.extend(invariants::denied(
                        "a relation naming the erased subject",
                        r.as_ref(),
                    ));
                }
            }
        }
        self.judge(si, &["erasure_denies"], v);
    }

    /// The subject is erased and its window is short: wait past it and the
    /// sweeper's interval, then the subject is gone for good and every peer
    /// that named it carries a tombstone per relation. The worker moves on to a
    /// fresh subject.
    async fn cascade(&mut self, si: usize, window: Duration) {
        let settle = self.ctx.campaign.faults.settle;
        self.sleep(si, window + settle).await;
        if self.ctx.cancel.is_cancelled() {
            return;
        }
        let read = self
            .current(
                si,
                Request::Current {
                    subject: SubjectRef::Own,
                    paths: vec![],
                    sources: vec![],
                    scopes: self.scopes.clone(),
                    limit: 0,
                    cursor_from: None,
                },
            )
            .await;
        let restore = self
            .restore(
                si,
                Request::Restore {
                    subject: SubjectRef::Own,
                },
            )
            .await;
        let write_req = self.fresh_append(si, false);
        let write = self.append(si, write_req).await;
        if let (Some(read), Some(restore), Some(write)) = (read, restore, write) {
            let v = invariants::executed(read.as_ref(), restore.as_ref(), write.as_ref());
            self.judge(si, &["erasure_executes"], v);
        }
        let erased = self.subjects[si].model.id;
        self.subjects[si].model.apply_executed();

        for pj in 0..self.subjects.len() {
            if pj == si {
                continue;
            }
            let names_it = self.subjects[pj]
                .model
                .facts
                .iter()
                .any(|f| !f.is_tombstone() && f.counterparty == Some(erased));
            if !names_it {
                continue;
            }
            let limit = self.page_limit();
            let Some(pages) = self.walk_history(pj, None, limit).await else {
                continue;
            };
            let facts: Vec<Fact> = pages.into_iter().flatten().collect();
            let v = invariants::cascade_tombstones_counterparty(
                &self.subjects[pj].model,
                erased,
                &facts,
            );
            self.judge(pj, &["cascade_tombstones_counterparty"], v);
            let m = &mut self.subjects[pj].model;
            m.apply_counterparty_erased(erased);
            m.learn(&facts);
        }
        self.subjects[si] = Subject::new();
    }
}

/// `executes_after - requested_at`, when both are present and ordered.
#[must_use]
pub fn window_of(e: &EraseResponse) -> Option<Duration> {
    let requested = trace::datetime(e.requested_at.as_ref()?)?;
    let executes = trace::datetime(e.executes_after.as_ref()?)?;
    (executes - requested).to_std().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_window_is_the_announced_difference() {
        let e = EraseResponse {
            requested_at: Some(prost_types::Timestamp {
                seconds: 100,
                nanos: 0,
            }),
            executes_after: Some(prost_types::Timestamp {
                seconds: 101,
                nanos: 500_000_000,
            }),
        };
        assert_eq!(window_of(&e), Some(Duration::from_millis(1500)));
        let inverted = EraseResponse {
            requested_at: Some(prost_types::Timestamp {
                seconds: 100,
                nanos: 0,
            }),
            executes_after: Some(prost_types::Timestamp {
                seconds: 99,
                nanos: 0,
            }),
        };
        assert_eq!(window_of(&inverted), None);
    }
}
