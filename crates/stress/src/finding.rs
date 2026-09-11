//! A finding: one broken invariant, with everything needed to read it and to
//! run it again.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::trace::{Step, Violation};

/// Which worker class found it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkerClass {
    /// Exclusive subjects, exact model.
    Owner,
    /// Shared subjects, order-free model.
    Contention,
    /// Hostile requests.
    Fuzz,
}

impl WorkerClass {
    /// Lower-case name.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Owner => "owner",
            Self::Contention => "contention",
            Self::Fuzz => "fuzz",
        }
    }
}

/// One replay of a finding's trace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayOutcome {
    /// When.
    pub at: String,
    /// Against which target.
    pub target: String,
    /// Whether the same invariant broke again.
    pub reproduced: bool,
    /// The message of the reproduced violation, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Steps run before the verdict.
    pub steps_run: usize,
}

/// The record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// UUID v7.
    pub id: String,
    /// From [`crate::model::invariants::ALL`].
    pub invariant: String,
    /// Dedup key across runs: the invariant and the message with ids and stamps blanked.
    pub signature: String,
    /// One sentence.
    pub message: String,
    /// What the model expected.
    pub expected: serde_json::Value,
    /// What the ledger answered.
    pub actual: serde_json::Value,
    /// The subject the trace is bound to.
    pub subject: String,
    /// Which worker class.
    pub worker: WorkerClass,
    /// The campaign's name.
    pub campaign: String,
    /// The chaos run it belongs to, when chaos ran it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    /// The target it was found on.
    pub target: String,
    /// The store the target reported on ping.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub store: Option<String>,
    /// RFC 3339.
    pub found_at: String,
    /// The steps on the subject up to and including the violating one.
    pub trace: Vec<Step>,
    /// How long the trace was before shrinking.
    pub original_len: usize,
    /// Whether the trace is the shortest that still reproduces.
    pub shrunk: bool,
    /// Why it was not shrunk, or what shrinking found.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shrink_note: Option<String>,
    /// Replays since.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub replays: Vec<ReplayOutcome>,
}

/// The list view: a finding without its trace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindingSummary {
    /// Id.
    pub id: String,
    /// Invariant.
    pub invariant: String,
    /// Signature.
    pub signature: String,
    /// Message.
    pub message: String,
    /// Subject.
    pub subject: String,
    /// Worker class.
    pub worker: WorkerClass,
    /// Campaign.
    pub campaign: String,
    /// Run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    /// Target.
    pub target: String,
    /// When.
    pub found_at: String,
    /// Steps in the trace.
    pub trace_len: usize,
    /// Shrunk.
    pub shrunk: bool,
}

impl Finding {
    /// From a violation and the trace that led to it.
    #[must_use]
    pub fn new(
        v: &Violation,
        trace: Vec<Step>,
        subject: uuid::Uuid,
        worker: WorkerClass,
        campaign: &str,
        target: &str,
        store: Option<&str>,
    ) -> Self {
        let original_len = trace.len();
        Self {
            id: uuid::Uuid::now_v7().to_string(),
            invariant: v.invariant.to_owned(),
            signature: signature(v.invariant, &v.message),
            message: v.message.clone(),
            expected: v.expected.clone(),
            actual: v.actual.clone(),
            subject: subject.to_string(),
            worker,
            campaign: campaign.to_owned(),
            run_id: None,
            target: target.to_owned(),
            store: store.map(str::to_owned),
            found_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            trace,
            original_len,
            shrunk: false,
            shrink_note: None,
            replays: Vec::new(),
        }
    }

    /// The list view.
    #[must_use]
    pub fn summary(&self) -> FindingSummary {
        FindingSummary {
            id: self.id.clone(),
            invariant: self.invariant.clone(),
            signature: self.signature.clone(),
            message: self.message.clone(),
            subject: self.subject.clone(),
            worker: self.worker,
            campaign: self.campaign.clone(),
            run_id: self.run_id.clone(),
            target: self.target.clone(),
            found_at: self.found_at.clone(),
            trace_len: self.trace.len(),
            shrunk: self.shrunk,
        }
    }
}

/// The dedup key: SHA-256 over the invariant and the message with UUIDs,
/// RFC 3339 stamps and numbers of four digits or more replaced by `#`, first
/// twelve hex characters. Two runs that break the same rule the same way share
/// it; the ids they happened on do not matter.
#[must_use]
pub fn signature(invariant: &str, message: &str) -> String {
    let mut h = Sha256::new();
    h.update(invariant.as_bytes());
    h.update(b"\n");
    h.update(normalise(message).as_bytes());
    let hex = format!("{:x}", h.finalize());
    hex[..12].to_owned()
}

fn normalise(message: &str) -> String {
    let mut out = String::with_capacity(message.len());
    let mut digits = 0usize;
    let mut buf = String::new();
    let flush = |out: &mut String, buf: &mut String, digits: usize| {
        if digits >= 4 || looks_like_id(buf) {
            out.push('#');
        } else {
            out.push_str(buf);
        }
        buf.clear();
    };
    for ch in message.chars() {
        if ch.is_ascii_alphanumeric()
            || ch == '-'
            || ch == ':'
            || ch == '.'
            || ch == 'T'
            || ch == 'Z'
        {
            if ch.is_ascii_digit() {
                digits += 1;
            }
            buf.push(ch);
        } else {
            if !buf.is_empty() {
                flush(&mut out, &mut buf, digits);
            }
            digits = 0;
            out.push(ch);
        }
    }
    if !buf.is_empty() {
        flush(&mut out, &mut buf, digits);
    }
    out
}

/// A token that is a UUID or a timestamp: mostly digits with dashes or colons.
fn looks_like_id(token: &str) -> bool {
    let digits = token.chars().filter(char::is_ascii_digit).count();
    digits >= 8 && (token.contains('-') || token.contains(':'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signatures_ignore_ids_and_stamps() {
        let a = signature(
            "history_cut",
            "history at a cut: the model holds a fact profile.name at id 4123 that the ledger did not return",
        );
        let b = signature(
            "history_cut",
            "history at a cut: the model holds a fact profile.name at id 9981 that the ledger did not return",
        );
        assert_eq!(a, b);
        let c = signature(
            "history_cut",
            "history at a cut: the model holds a fact profile.bio at id 1 that the ledger did not return",
        );
        assert_ne!(a, c);
        let u1 = signature(
            "x",
            "subject 01a08fdf-4b86-7842-b074-c08f918ecd03 at 2026-09-11T10:00:00Z",
        );
        let u2 = signature(
            "x",
            "subject 01a08fdf-9999-7842-b074-c08f918ecd03 at 2026-09-12T11:30:00Z",
        );
        assert_eq!(u1, u2);
        assert_eq!(a.len(), 12);
    }
}
