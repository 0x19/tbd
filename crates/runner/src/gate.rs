//! Before a run reaches the sandbox: a slot among the runs at once (waiting in
//! a short line for one), and the caller's daily allowance.

use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex, PoisonError,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

use tbd_common::metrics::names;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

use crate::config::{Admission, Budget};

/// Runs at once and the line behind them.
#[derive(Debug, Clone)]
pub struct Gate {
    slots: Arc<Semaphore>,
    max: usize,
    max_queued: usize,
    timeout: Duration,
    waiting: Arc<AtomicUsize>,
}

/// A run's turn; given back when dropped.
#[derive(Debug)]
pub struct Slot {
    _permit: OwnedSemaphorePermit,
    gate: Gate,
}

impl Drop for Slot {
    fn drop(&mut self) {
        // The permit is released after this body; publish what it will be.
        #[allow(clippy::cast_precision_loss)] // runs in the tens
        metrics::gauge!(names::RUNNER_IN_FLIGHT)
            .set(self.gate.in_flight().saturating_sub(1) as f64);
    }
}

impl Gate {
    /// A gate from `[admission]`.
    #[must_use]
    pub fn new(a: &Admission) -> Self {
        Self {
            slots: Arc::new(Semaphore::new(a.max_in_flight)),
            max: a.max_in_flight,
            max_queued: a.max_queued,
            timeout: a.queue_timeout,
            waiting: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Runs now.
    #[must_use]
    pub fn in_flight(&self) -> usize {
        self.max.saturating_sub(self.slots.available_permits())
    }

    /// The most at once.
    #[must_use]
    pub fn max(&self) -> usize {
        self.max
    }

    /// A turn now, after a wait in line, or the reason there is none.
    ///
    /// # Errors
    /// The line is full, or the wait ran out.
    pub async fn enter(&self) -> Result<Slot, String> {
        let permit = if let Ok(p) = Arc::clone(&self.slots).try_acquire_owned() {
            p
        } else {
            let ahead = self.waiting.fetch_add(1, Ordering::SeqCst);
            if ahead >= self.max_queued {
                self.waiting.fetch_sub(1, Ordering::SeqCst);
                return Err(format!(
                    "busy: {} running and {ahead} waiting; try again shortly",
                    self.in_flight()
                ));
            }
            let started = Instant::now();
            let got =
                tokio::time::timeout(self.timeout, Arc::clone(&self.slots).acquire_owned()).await;
            self.waiting.fetch_sub(1, Ordering::SeqCst);
            match got {
                Ok(Ok(p)) => p,
                _ => {
                    return Err(format!(
                        "busy: no free slot after waiting {:.1}s; try again shortly",
                        started.elapsed().as_secs_f64()
                    ));
                }
            }
        };
        #[allow(clippy::cast_precision_loss)]
        metrics::gauge!(names::RUNNER_IN_FLIGHT).set(self.in_flight() as f64);
        Ok(Slot {
            _permit: permit,
            gate: self.clone(),
        })
    }
}

/// Each caller's runs today, in memory: a restart resets it (RFC 0010).
#[derive(Debug, Clone)]
pub struct Allowance {
    per_day: u32,
    used: Arc<Mutex<HashMap<String, (String, u32)>>>,
}

impl Allowance {
    /// From `[budget]`.
    #[must_use]
    pub fn new(b: &Budget) -> Self {
        Self {
            per_day: b.runs_per_day,
            used: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn today() -> String {
        chrono::Utc::now().format("%Y-%m-%d").to_string()
    }

    /// Runs the caller has left today; `None` when there is no limit.
    #[must_use]
    pub fn left(&self, subject: &str) -> Option<u32> {
        if self.per_day == 0 {
            return None;
        }
        let used = self.used.lock().unwrap_or_else(PoisonError::into_inner);
        let today = Self::today();
        let n = used
            .get(subject)
            .filter(|(day, _)| *day == today)
            .map_or(0, |(_, n)| *n);
        Some(self.per_day.saturating_sub(n))
    }

    /// Take one run of the caller's allowance, or say it is spent.
    ///
    /// # Errors
    /// Nothing is left today.
    pub fn take(&self, subject: &str) -> Result<Option<u32>, String> {
        if self.per_day == 0 {
            return Ok(None);
        }
        let mut used = self.used.lock().unwrap_or_else(PoisonError::into_inner);
        let today = Self::today();
        let entry = used
            .entry(subject.to_owned())
            .or_insert_with(|| (today.clone(), 0));
        if entry.0 != today {
            *entry = (today, 0);
        }
        if entry.1 >= self.per_day {
            return Err(format!(
                "daily run allowance spent: {0} of {0}; it resets at 00:00 UTC",
                self.per_day
            ));
        }
        entry.1 += 1;
        Ok(Some(self.per_day - entry.1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn a_full_line_is_refused_at_once_and_a_slot_comes_back() {
        let g = Gate::new(&Admission {
            max_in_flight: 1,
            max_queued: 0,
            queue_timeout: Duration::from_secs(5),
        });
        let held = g.enter().await.unwrap();
        assert!(g.enter().await.unwrap_err().starts_with("busy:"));
        drop(held);
        assert!(g.enter().await.is_ok());
    }

    #[test]
    fn the_allowance_counts_down_and_refuses_at_zero() {
        let a = Allowance::new(&Budget { runs_per_day: 2 });
        assert_eq!(a.take("p").unwrap(), Some(1));
        assert_eq!(a.take("p").unwrap(), Some(0));
        assert!(a.take("p").unwrap_err().contains("resets at 00:00 UTC"));
        assert_eq!(a.left("someone-else"), Some(2), "per caller");
        assert_eq!(
            Allowance::new(&Budget { runs_per_day: 0 })
                .take("p")
                .unwrap(),
            None
        );
    }
}
