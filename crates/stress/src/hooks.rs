//! How a running campaign reports progress to whoever started it.

use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::{finding::FindingSummary, metrics::LoadSnapshot, report::StressSnapshot};

/// What a campaign emits while it runs, in order.
#[derive(Debug, Clone)]
pub enum StressEvent {
    /// A phase began: `warmup`, `run`, `shrink`, `done`.
    Phase {
        /// The phase.
        name: String,
    },
    /// The load numbers, once a second.
    Load {
        /// Snapshot.
        snapshot: LoadSnapshot,
    },
    /// The checks and findings, once a second.
    Stress {
        /// Snapshot.
        snapshot: StressSnapshot,
    },
    /// A finding, as found.
    Finding {
        /// The list view.
        finding: FindingSummary,
    },
}

/// Progress sink and cancellation.
#[derive(Debug, Clone, Default)]
pub struct Hooks {
    /// Where events go; `None` discards them.
    pub events: Option<mpsc::UnboundedSender<StressEvent>>,
    /// Cancel: workers stop, the result is what was measured so far.
    pub cancel: CancellationToken,
}

impl Hooks {
    /// Send if anyone listens.
    pub fn emit(&self, event: StressEvent) {
        if let Some(tx) = &self.events {
            let _ = tx.send(event);
        }
    }
}
