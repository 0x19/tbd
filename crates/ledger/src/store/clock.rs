//! Where the in-memory store gets its time from, so tests can move it.

use std::sync::Mutex;

use super::Timestamp;

/// A source of `now`.
pub trait Clock: Send + Sync + 'static {
    /// The current instant.
    fn now(&self) -> Timestamp;
}

/// The system clock.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Timestamp {
        chrono::Utc::now()
    }
}

/// A clock tests move by hand.
#[derive(Debug)]
pub struct ManualClock {
    now: Mutex<Timestamp>,
}

impl ManualClock {
    /// Start at `at`.
    #[must_use]
    pub fn at(at: Timestamp) -> Self {
        Self {
            now: Mutex::new(at),
        }
    }

    /// Start at the current system time.
    #[must_use]
    pub fn now() -> Self {
        Self::at(chrono::Utc::now())
    }

    /// Move forward.
    pub fn advance(&self, by: std::time::Duration) {
        let mut now = self
            .now
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *now += chrono::TimeDelta::from_std(by).unwrap_or(chrono::TimeDelta::MAX);
    }

    /// Set the instant.
    pub fn set(&self, at: Timestamp) {
        *self
            .now
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = at;
    }
}

impl Clock for ManualClock {
    fn now(&self) -> Timestamp {
        *self
            .now
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}
