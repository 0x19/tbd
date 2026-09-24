//! Admission control: how many requests a tier runs at once, how many may
//! wait for a turn, and for how long.
//!
//! An engine on one card runs a fixed number of requests in parallel; past
//! that it queues them itself, invisibly, until a caller gives up. The L2
//! makes that queue its own, bounded and measured: a request takes a slot or
//! waits in line for one, a full line is refused at once, and a wait that
//! runs out is refused with how long it waited. Either refusal is
//! `RESOURCE_EXHAUSTED` with a reason, so a crowd meets a named "busy" in
//! milliseconds rather than a minute of nothing.
//!
//! A slot is held for as long as the answer streams and is given back when
//! the stream ends, fails or its caller goes away (the [`Slot`] is dropped).
//! The line is first come, first served (tokio's semaphore is fair).

use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

use tbd_common::metrics::names;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

use crate::config::{EngineConfig, Tier};

/// One tier's slots and line.
#[derive(Debug, Clone)]
pub struct Admission {
    tier: Tier,
    slots: Arc<Semaphore>,
    max_in_flight: usize,
    max_queued: usize,
    queue_timeout: Duration,
    waiting: Arc<AtomicUsize>,
}

/// Why a request was not admitted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// Every slot taken and the line full: refused without waiting.
    QueueFull {
        /// Requests running.
        in_flight: usize,
        /// Requests already waiting.
        waiting: usize,
    },
    /// Waited the whole `queue_timeout` without a slot freeing.
    Waited(Duration),
}

impl Refusal {
    /// The `reason` label of the refusal counter.
    #[must_use]
    pub fn reason(&self) -> &'static str {
        match self {
            Self::QueueFull { .. } => "queue_full",
            Self::Waited(_) => "queue_timeout",
        }
    }

    /// The status message: what was full, in words a caller can act on.
    #[must_use]
    pub fn message(&self, tier: Tier) -> String {
        match self {
            Self::QueueFull { in_flight, waiting } => format!(
                "busy: tier {tier} is running {in_flight} and {waiting} are waiting; try again shortly"
            ),
            Self::Waited(waited) => format!(
                "busy: tier {tier} had no free slot after waiting {:.1}s; try again shortly",
                waited.as_secs_f64()
            ),
        }
    }
}

/// A request's turn on the tier. Give it back by dropping it.
#[derive(Debug)]
pub struct Slot {
    permit: Option<OwnedSemaphorePermit>,
    admission: Admission,
}

impl Drop for Slot {
    fn drop(&mut self) {
        drop(self.permit.take());
        self.admission.publish();
    }
}

impl Admission {
    /// The tier's admission from its `[engines.<tier>]` keys.
    #[must_use]
    pub fn new(tier: Tier, config: &EngineConfig) -> Self {
        let admission = Self {
            tier,
            slots: Arc::new(Semaphore::new(config.max_in_flight)),
            max_in_flight: config.max_in_flight,
            max_queued: config.max_queued,
            queue_timeout: config.queue_timeout,
            waiting: Arc::new(AtomicUsize::new(0)),
        };
        admission.publish();
        admission
    }

    /// Take a slot now, or wait in line for one, or be refused.
    ///
    /// # Errors
    /// The line is full, or the wait ran out.
    pub async fn enter(&self) -> Result<Slot, Refusal> {
        let started = Instant::now();
        let outcome = self.take().await;
        let waited = started.elapsed();
        match &outcome {
            Ok(_) => {
                metrics::histogram!(names::LLM_QUEUE_WAIT, "tier" => self.tier.as_str())
                    .record(waited.as_secs_f64());
            }
            Err(refusal) => {
                metrics::counter!(names::LLM_REFUSED_TOTAL, "tier" => self.tier.as_str(), "reason" => refusal.reason())
                    .increment(1);
            }
        }
        self.publish();
        outcome
    }

    async fn take(&self) -> Result<Slot, Refusal> {
        if let Ok(permit) = Arc::clone(&self.slots).try_acquire_owned() {
            return Ok(self.slot(permit));
        }
        // Count ourselves into the line first, so two arrivals cannot both
        // see one free place in it.
        let ahead = self.waiting.fetch_add(1, Ordering::SeqCst);
        if ahead >= self.max_queued {
            self.waiting.fetch_sub(1, Ordering::SeqCst);
            return Err(Refusal::QueueFull {
                in_flight: self.in_flight(),
                waiting: ahead,
            });
        }
        self.publish();
        let started = Instant::now();
        let wait =
            tokio::time::timeout(self.queue_timeout, Arc::clone(&self.slots).acquire_owned()).await;
        self.waiting.fetch_sub(1, Ordering::SeqCst);
        match wait {
            Ok(Ok(permit)) => Ok(self.slot(permit)),
            // The semaphore is never closed; treat it as a wait that ran out.
            Ok(Err(_)) | Err(_) => Err(Refusal::Waited(started.elapsed())),
        }
    }

    fn slot(&self, permit: OwnedSemaphorePermit) -> Slot {
        Slot {
            permit: Some(permit),
            admission: self.clone(),
        }
    }

    /// Requests running on the tier now.
    #[must_use]
    pub fn in_flight(&self) -> usize {
        self.max_in_flight
            .saturating_sub(self.slots.available_permits())
    }

    /// Requests waiting for a slot now.
    #[must_use]
    pub fn waiting(&self) -> usize {
        self.waiting.load(Ordering::SeqCst)
    }

    /// Requests the tier runs at once.
    #[must_use]
    pub fn max_in_flight(&self) -> usize {
        self.max_in_flight
    }

    /// Set the tier's gauges from the current counts.
    #[allow(clippy::cast_precision_loss)] // counts in the tens
    fn publish(&self) {
        metrics::gauge!(names::LLM_IN_FLIGHT, "tier" => self.tier.as_str())
            .set(self.in_flight() as f64);
        metrics::gauge!(names::LLM_QUEUED, "tier" => self.tier.as_str()).set(self.waiting() as f64);
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use crate::config::Engines;

    fn admission(in_flight: usize, queued: usize, timeout_ms: u64) -> Admission {
        let mut config = Engines::default().fast;
        config.max_in_flight = in_flight;
        config.max_queued = queued;
        config.queue_timeout = Duration::from_millis(timeout_ms);
        Admission::new(Tier::Fast, &config)
    }

    #[tokio::test]
    async fn a_full_line_is_refused_at_once() {
        let a = admission(1, 0, 10_000);
        let _held = a.enter().await.unwrap();
        let started = Instant::now();
        let refusal = a.enter().await.unwrap_err();
        assert_eq!(
            refusal,
            Refusal::QueueFull {
                in_flight: 1,
                waiting: 0
            }
        );
        assert!(started.elapsed() < Duration::from_millis(50));
    }

    #[tokio::test]
    async fn a_wait_that_runs_out_is_refused_with_how_long() {
        let a = admission(1, 1, 50);
        let _held = a.enter().await.unwrap();
        let refusal = a.enter().await.unwrap_err();
        assert!(matches!(refusal, Refusal::Waited(w) if w >= Duration::from_millis(50)));
        assert_eq!(a.waiting(), 0, "the line is left as it was");
    }

    #[tokio::test]
    async fn a_waiter_gets_the_slot_when_it_is_given_back() {
        let a = admission(1, 1, 5_000);
        let held = a.enter().await.unwrap();
        let next = tokio::spawn({
            let a = a.clone();
            async move { a.enter().await.map(|_| ()) }
        });
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert_eq!(a.waiting(), 1);
        drop(held);
        next.await.unwrap().unwrap();
        assert_eq!(a.in_flight(), 0, "the waiter's slot was given back too");
    }

    #[test]
    fn a_refusal_names_the_tier_and_says_what_to_do() {
        let m = Refusal::QueueFull {
            in_flight: 2,
            waiting: 8,
        }
        .message(Tier::Deep);
        assert!(m.starts_with("busy: tier deep"), "{m}");
        assert!(m.contains("try again"), "{m}");
    }
}
