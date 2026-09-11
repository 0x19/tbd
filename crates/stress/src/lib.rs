//! Stress campaigns against the ledger.
//!
//! A campaign (`campaign`, a TOML file) puts closed-loop workers on a ledger.
//! Each owner worker holds a few subjects nobody else writes to and a
//! client-side [`model::SubjectModel`] of every one, built only from the
//! requests it sent and the answers it got; every read is judged against that
//! model by the checkers in [`model::invariants`]. A broken rule is a
//! [`finding::Finding`]: the subject, the trace of symbolic requests and
//! responses that led there, expected and actual.
//!
//! The crate never boots a ledger and never depends on `tbd-ledger`: the model
//! must not share code with the thing it checks. Whoever calls [`run`] gives it
//! [`client::Target`]s (a channel and an optional bearer) and receives a
//! [`report::CampaignResult`]; chaos does the stack, the timeline and the API
//! around it.

pub mod campaign;
pub mod client;
pub mod executor;
pub mod finding;
pub mod hooks;
pub mod metrics;
pub mod model;
pub mod replay;
pub mod report;
pub mod shrink;
pub mod trace;
pub mod workers;

pub use campaign::{Campaign, CampaignError};
pub use client::{CallError, GrpcLedger, LedgerClient, Target};
pub use executor::{PROGRESS_INTERVAL, run, run_with_clients};
pub use finding::{Finding, FindingSummary, ReplayOutcome, WorkerClass};
pub use hooks::{Hooks, StressEvent};
pub use metrics::{Latency, LoadSnapshot, Metrics, OpSnapshot, TargetCounts};
pub use replay::{Replayed, replay, replay_finding};
pub use report::{CampaignResult, CheckCount, StressSnapshot, render, summary};
pub use shrink::{Budget, shrink};
