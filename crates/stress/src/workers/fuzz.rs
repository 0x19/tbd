//! The fuzz worker: hostile requests on subjects of its own, each seeded with
//! one valid fact first, and every answer judged by the case's expected codes
//! (`clean_refusal`). The cases are [`crate::fuzz::FuzzCase`]; a run with the
//! same seed sends the same sequence.

use std::sync::Arc;

use rand::{Rng, rngs::StdRng};
use uuid::Uuid;

use super::{Context, Lane, judge, send};
use crate::{
    finding::WorkerClass,
    fuzz::FuzzCase,
    model::{generate, invariants},
    replay::{Outcome, call},
    trace::{self, Binding, Concrete, Request, SubjectRef, TimeRef},
};

/// Steps kept per fuzz subject: the seeding append plus the last cases.
const TRACE_DEPTH: usize = 20;

/// One fuzz worker.
pub struct Fuzz {
    ctx: Arc<Context>,
    rng: StdRng,
    lanes: Vec<Lane>,
    seeded: Vec<bool>,
}

impl Fuzz {
    /// A worker with `subjects` fresh subjects.
    #[must_use]
    pub fn new(ctx: Arc<Context>, seed: u64, subjects: u32) -> Self {
        Self {
            ctx,
            rng: crate::fuzz::rng(seed),
            lanes: (0..subjects.max(1))
                .map(|_| Lane::new(Uuid::now_v7(), TRACE_DEPTH))
                .collect(),
            seeded: vec![false; subjects.max(1) as usize],
        }
    }

    /// Subjects this worker owns.
    #[must_use]
    pub fn subject_count(&self) -> usize {
        self.lanes.len()
    }

    /// Drive the target until cancelled.
    pub async fn run(mut self) {
        let pace = self.ctx.campaign.workload.fuzz.pace;
        while !self.ctx.cancel.is_cancelled() {
            let si = self.rng.random_range(0..self.lanes.len());
            if !self.seeded[si] {
                self.seed(si).await;
                continue;
            }
            self.case(si).await;
            if !pace.is_zero() {
                tokio::select! {
                    () = tokio::time::sleep(pace) => {}
                    () = self.ctx.cancel.cancelled() => break,
                }
            }
        }
    }

    fn binding(&self, si: usize) -> Binding {
        Binding {
            own: self.lanes[si].subject,
            peers: vec![],
        }
    }

    /// One valid fact, so the subject exists and `valid_read` has something to see.
    async fn seed(&mut self, si: usize) {
        let request = Request::Append {
            subject: SubjectRef::Own,
            path: "profile.name".into(),
            source: 2,
            value: generate::value(&mut self.rng),
            origin: serde_json::json!({"by": "stress", "class": "fuzz"}),
            confidence: Some(1.0),
            counterparty: None,
            observed_at: TimeRef::At(chrono::Utc::now()),
            expires_at: None,
            consent: vec!["self".into()],
            stub: false,
            idempotency_key: Some(generate::key(&mut self.rng)),
        };
        let Ok(Concrete::Append(req)) = request.materialise(&self.binding(si), &[]) else {
            return;
        };
        let client = Arc::clone(&self.ctx.client);
        let (result, _) = send(
            &self.ctx,
            &mut self.lanes[si],
            WorkerClass::Fuzz,
            request,
            async move { client.append(req).await },
            trace::append_json,
        )
        .await;
        self.seeded[si] = result.is_ok();
    }

    async fn case(&mut self, si: usize) {
        let paths = self.ctx.campaign.workload.paths.clone();
        let case = FuzzCase::draw(&mut self.rng, &paths);
        let request = Request::Fuzz { case };
        let Ok(concrete) = request.materialise(&self.binding(si), &[]) else {
            return;
        };
        let Concrete::Fuzz {
            case: name,
            inner,
            expect,
        } = concrete
        else {
            return;
        };
        let client = Arc::clone(&self.ctx.client);
        let (result, tolerated) = send(
            &self.ctx,
            &mut self.lanes[si],
            WorkerClass::Fuzz,
            request,
            async move {
                match call(client.as_ref(), *inner).await {
                    Outcome::Err(e) => Err(e),
                    other => Ok(other),
                }
            },
            |o| o.json().unwrap_or_default(),
        )
        .await;
        if tolerated {
            return;
        }
        let v = invariants::clean_refusal(name, expect, result.as_ref().map(|_| ()));
        judge(
            &self.ctx,
            &self.lanes[si],
            WorkerClass::Fuzz,
            &["clean_refusal"],
            v,
        );
    }
}
