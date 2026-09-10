//! Fault injection, off by default and transport-free.
//!
//! A service holds a [`FaultHandle`] and calls [`FaultHandle::apply`] at the
//! start of each request. Production binaries never change it from
//! [`Behavior::Healthy`]. The chaos tool flips it at runtime to make a real
//! service slow, failing or hung, which is how scenarios inject faults without
//! a mock server. Each service maps [`ErrorKind`] onto its own error type.

use std::{
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};

/// Transport-neutral failure classes. gRPC maps these to status codes, HTTP to
/// statuses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    /// The backend is down or unreachable.
    Unavailable,
    /// A bug.
    Internal,
    /// Backpressure; the caller should slow down.
    Overloaded,
    /// The work took too long.
    Timeout,
}

/// A fault the service must turn into its own error.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("injected {kind:?}: {message}")]
pub struct Fault {
    /// Class of failure.
    pub kind: ErrorKind,
    /// Human-readable reason.
    pub message: String,
}

/// How a service misbehaves. Rates are probabilities in `0.0..=1.0`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Behavior {
    /// Normal operation.
    #[default]
    Healthy,
    /// Add latency to every request.
    Slow {
        /// Base latency.
        #[serde(with = "humantime_serde")]
        latency: Duration,
        /// Uniform random jitter added on top.
        #[serde(default, with = "humantime_serde")]
        jitter: Duration,
    },
    /// Never answer. The request hangs until the client gives up.
    Hang,
    /// Fail a fraction of requests.
    Error {
        /// Class of failure.
        kind: ErrorKind,
        /// Fraction of requests that fail.
        #[serde(default = "one")]
        rate: f64,
        /// Reason given to the caller.
        #[serde(default = "default_message")]
        message: String,
    },
    /// Behave healthily for a while, then switch to another behaviour.
    DelayedFailure {
        /// Healthy period, measured from when this behaviour was set.
        #[serde(with = "humantime_serde")]
        healthy_for: Duration,
        /// What happens afterwards.
        then: Box<Behavior>,
    },
}

fn one() -> f64 {
    1.0
}

fn default_message() -> String {
    "injected fault".to_owned()
}

struct Inner {
    behavior: Behavior,
    since: Instant,
}

/// Shared, runtime-mutable behaviour. Cheap to clone; all clones see updates.
#[derive(Clone)]
pub struct FaultHandle {
    inner: Arc<RwLock<Inner>>,
}

impl std::fmt::Debug for FaultHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FaultHandle")
            .field("behavior", &self.get())
            .finish()
    }
}

impl Default for FaultHandle {
    fn default() -> Self {
        Self::new(Behavior::Healthy)
    }
}

impl FaultHandle {
    /// A handle starting in `behavior`.
    pub fn new(behavior: Behavior) -> Self {
        Self {
            inner: Arc::new(RwLock::new(Inner {
                behavior,
                since: Instant::now(),
            })),
        }
    }

    /// Replace the behaviour. The clock for [`Behavior::DelayedFailure`] restarts.
    pub fn set(&self, behavior: Behavior) {
        let mut guard = self
            .inner
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        guard.behavior = behavior;
        guard.since = Instant::now();
    }

    /// The configured behaviour, as set.
    pub fn get(&self) -> Behavior {
        self.inner
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .behavior
            .clone()
    }

    /// The behaviour in force right now, with `DelayedFailure` collapsed to
    /// whichever phase is active.
    pub fn effective(&self) -> Behavior {
        let guard = self
            .inner
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        resolve(&guard.behavior, guard.since.elapsed())
    }

    /// Apply the fault at the start of a request: may delay, fail or hang.
    pub async fn apply(&self) -> Result<(), Fault> {
        match self.effective() {
            Behavior::Healthy | Behavior::DelayedFailure { .. } => Ok(()),
            Behavior::Slow { latency, jitter } => {
                tokio::time::sleep(latency + random_jitter(jitter)).await;
                Ok(())
            }
            Behavior::Hang => std::future::pending().await,
            Behavior::Error {
                kind,
                rate,
                message,
            } => {
                if fires(rate) {
                    Err(Fault { kind, message })
                } else {
                    Ok(())
                }
            }
        }
    }

    /// Apply the fault to one item of an outgoing stream. Only error
    /// behaviours fire here; delaying a stream is the producer's business.
    pub fn stream_error(&self) -> Option<Fault> {
        match self.effective() {
            Behavior::Error {
                kind,
                rate,
                message,
            } if fires(rate) => Some(Fault { kind, message }),
            _ => None,
        }
    }
}

fn resolve(behavior: &Behavior, elapsed: Duration) -> Behavior {
    match behavior {
        Behavior::DelayedFailure { healthy_for, then } => {
            if elapsed < *healthy_for {
                Behavior::Healthy
            } else {
                resolve(then, elapsed.saturating_sub(*healthy_for))
            }
        }
        other => other.clone(),
    }
}

fn fires(rate: f64) -> bool {
    rate >= 1.0 || (rate > 0.0 && rand::random::<f64>() < rate)
}

fn random_jitter(jitter: Duration) -> Duration {
    if jitter.is_zero() {
        Duration::ZERO
    } else {
        jitter.mul_f64(rand::random::<f64>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delayed_failure_resolves_by_elapsed_time() {
        let b = Behavior::DelayedFailure {
            healthy_for: Duration::from_secs(2),
            then: Box::new(Behavior::Hang),
        };
        assert_eq!(resolve(&b, Duration::from_secs(1)), Behavior::Healthy);
        assert_eq!(resolve(&b, Duration::from_secs(3)), Behavior::Hang);
    }

    #[test]
    fn behavior_parses_from_toml() {
        let b: Behavior =
            toml::from_str("type = \"error\"\nkind = \"unavailable\"\nrate = 0.5\n").unwrap();
        assert!(matches!(
            b,
            Behavior::Error {
                kind: ErrorKind::Unavailable,
                ..
            }
        ));
        let slow: Behavior =
            toml::from_str("type = \"slow\"\nlatency = \"200ms\"\njitter = \"50ms\"\n").unwrap();
        assert!(matches!(slow, Behavior::Slow { .. }));
        assert!(
            toml::from_str::<Behavior>("type = \"slow\"\nlatency = \"1s\"\nbogus = 1\n").is_err()
        );
    }

    #[tokio::test]
    async fn error_at_full_rate_always_fails_and_reset_heals() {
        let h = FaultHandle::new(Behavior::Error {
            kind: ErrorKind::Internal,
            rate: 1.0,
            message: "boom".into(),
        });
        assert_eq!(h.apply().await.unwrap_err().kind, ErrorKind::Internal);
        h.set(Behavior::Healthy);
        assert!(h.apply().await.is_ok());
    }
}
