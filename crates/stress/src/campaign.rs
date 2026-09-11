//! A campaign file: what to run against the ledger, for how long, and what
//! counts as a failure. TOML, every table `deny_unknown_fields`, so a typo is
//! a check error and never a silently ignored key.
//!
//! `[stack]` and `[[timeline]]` are carried as opaque tables: this crate never
//! boots a stack or plays a timeline. The caller (chaos) parses them into its
//! own types and runs them around [`crate::run`].

use std::{collections::BTreeMap, time::Duration};

use serde::{Deserialize, Serialize};

use crate::model::invariants;

/// The whole file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Campaign {
    /// `[campaign]`.
    pub campaign: Meta,
    /// `[stack]`, a scenario's stack, opaque here.
    #[serde(default, skip_serializing_if = "toml::Table::is_empty")]
    pub stack: toml::Table,
    /// `[[timeline]]`, a scenario's timeline, opaque here.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub timeline: Vec<toml::Table>,
    /// `[workload]`.
    #[serde(default)]
    pub workload: Workload,
    /// `[faults]`.
    #[serde(default)]
    pub faults: Faults,
    /// `[invariants]`: `name = false` switches one off; every name must exist.
    #[serde(default)]
    pub invariants: BTreeMap<String, bool>,
    /// `[stop]`.
    #[serde(default)]
    pub stop: Stop,
}

/// `[campaign]`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Meta {
    /// Shown in reports and the UI.
    pub name: String,
    /// One or two sentences: what the campaign proves.
    #[serde(default)]
    pub description: String,
    /// Skipped when a directory is run; for the long ones.
    #[serde(default)]
    pub skip: bool,
    /// The measured phase.
    #[serde(default = "default_duration", with = "humantime_serde")]
    pub duration: Duration,
    /// Same workload first; its numbers are discarded.
    #[serde(default, with = "humantime_serde")]
    pub warmup: Option<Duration>,
    /// Every random choice derives from it.
    #[serde(default)]
    pub seed: u64,
    /// Per request.
    #[serde(default = "default_timeout", with = "humantime_serde")]
    pub timeout: Duration,
}

/// `[workload]`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Workload {
    /// Requests in flight across every worker.
    pub max_in_flight: usize,
    /// Paths facts are written under.
    pub paths: Vec<String>,
    /// Paths for facts that name another subject.
    pub relation_paths: Vec<String>,
    /// The consent vocabulary; a fact's consent is a non-empty subset.
    pub scopes: Vec<String>,
    /// `[workload.owner]`.
    pub owner: Owner,
}

impl Default for Workload {
    fn default() -> Self {
        Self {
            max_in_flight: 64,
            paths: [
                "profile.name",
                "profile.bio",
                "traits.warmth",
                "traits.novelty",
                "journal.entry",
                "readings.sun",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect(),
            relation_paths: ["relations.match.m1", "relations.match.m2"]
                .into_iter()
                .map(str::to_owned)
                .collect(),
            scopes: ["self", "engine.base", "tier2@persona-a"]
                .into_iter()
                .map(str::to_owned)
                .collect(),
            owner: Owner::default(),
        }
    }
}

/// `[workload.owner]`: workers with exclusive subjects and an exact model.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Owner {
    /// Concurrent workers.
    pub workers: u32,
    /// Subjects per worker; two or more enable pair relations and the cascade check.
    pub subjects: u32,
    /// Think time between operations.
    #[serde(with = "humantime_serde")]
    pub pace: Duration,
    /// Page size for reads; 0 draws one in 1..=1000 per read.
    pub limit: u32,
    /// `[workload.owner.mix]`.
    pub mix: OwnerMix,
}

impl Default for Owner {
    fn default() -> Self {
        Self {
            workers: 4,
            subjects: 2,
            pace: Duration::ZERO,
            limit: 0,
            mix: OwnerMix::default(),
        }
    }
}

/// Relative weights of the owner operations. Without the table the balanced
/// mix below runs; with it, a missing key is 0, so a campaign lists exactly
/// what it wants.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnerMix {
    /// One fact.
    #[serde(default)]
    pub append: f64,
    /// `Current`, walked through every page.
    #[serde(default)]
    pub current: f64,
    /// `History`, walked through every page.
    #[serde(default)]
    pub history: f64,
    /// `History` at an earlier instant.
    #[serde(default)]
    pub history_cut: f64,
    /// Retract a valued path, or a path with nothing valued.
    #[serde(default)]
    pub retract: f64,
    /// Re-send an earlier append with its key, as is and mutated.
    #[serde(default)]
    pub idempotent_replay: f64,
    /// A fact naming a peer subject.
    #[serde(default)]
    pub pair_relation: f64,
    /// Erase, denied reads, restore, and the cascade when the window fits.
    #[serde(default)]
    pub erase_cycle: f64,
    /// A fact that has expired or is about to.
    #[serde(default)]
    pub expiring: f64,
}

impl Default for OwnerMix {
    fn default() -> Self {
        Self {
            append: 6.0,
            current: 3.0,
            history: 3.0,
            history_cut: 2.0,
            retract: 1.0,
            idempotent_replay: 1.0,
            pair_relation: 1.0,
            erase_cycle: 0.2,
            expiring: 0.5,
        }
    }
}

impl OwnerMix {
    /// `(name, weight)` for every operation with weight above zero.
    #[must_use]
    pub fn weighted(&self) -> Vec<(OwnerOp, f64)> {
        [
            (OwnerOp::Append, self.append),
            (OwnerOp::Current, self.current),
            (OwnerOp::History, self.history),
            (OwnerOp::HistoryCut, self.history_cut),
            (OwnerOp::Retract, self.retract),
            (OwnerOp::IdempotentReplay, self.idempotent_replay),
            (OwnerOp::PairRelation, self.pair_relation),
            (OwnerOp::EraseCycle, self.erase_cycle),
            (OwnerOp::Expiring, self.expiring),
        ]
        .into_iter()
        .filter(|(_, w)| *w > 0.0)
        .collect()
    }

    fn any_negative(&self) -> bool {
        [
            self.append,
            self.current,
            self.history,
            self.history_cut,
            self.retract,
            self.idempotent_replay,
            self.pair_relation,
            self.erase_cycle,
            self.expiring,
        ]
        .iter()
        .any(|w| *w < 0.0 || w.is_nan())
    }
}

/// The owner operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OwnerOp {
    /// One fact.
    Append,
    /// `Current`, every page.
    Current,
    /// `History`, every page.
    History,
    /// `History` at an earlier instant.
    HistoryCut,
    /// A retraction.
    Retract,
    /// An idempotent replay and a conflicting one.
    IdempotentReplay,
    /// A fact naming a peer.
    PairRelation,
    /// The erasure cycle.
    EraseCycle,
    /// An expiring fact.
    Expiring,
}

impl OwnerOp {
    /// The operation name in metrics and reports.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Append => "append",
            Self::Current => "current",
            Self::History => "history",
            Self::HistoryCut => "history_cut",
            Self::Retract => "retract",
            Self::IdempotentReplay => "idempotent_replay",
            Self::PairRelation => "pair_relation",
            Self::EraseCycle => "erase_cycle",
            Self::Expiring => "expiring",
        }
    }
}

/// `[faults]`: what the workload tolerates while faults are injected.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Faults {
    /// Error classes a request may draw without being a finding
    /// (`unavailable`, `deadline_exceeded`, `transport`, `timeout`, ...). Counted,
    /// and a write that drew one is re-driven through its idempotency key.
    pub tolerate: Vec<String>,
    /// How long to wait for a cascade when the announced grace fits in it.
    #[serde(with = "humantime_serde")]
    pub settle: Duration,
    /// An expiry within this of now is not compared.
    #[serde(with = "humantime_serde")]
    pub clock_skew: Duration,
}

impl Default for Faults {
    fn default() -> Self {
        Self {
            tolerate: Vec::new(),
            settle: Duration::from_millis(1500),
            clock_skew: Duration::from_millis(500),
        }
    }
}

/// `[stop]`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Stop {
    /// Stop early after this many findings; 0 = never.
    pub max_findings: u64,
    /// Shrink every finding after the run.
    pub shrink: bool,
    /// Replays a shrink may spend per finding.
    pub shrink_attempts: u32,
    /// Wall time a shrink may spend per finding.
    #[serde(with = "humantime_serde")]
    pub shrink_timeout: Duration,
}

impl Default for Stop {
    fn default() -> Self {
        Self {
            max_findings: 0,
            shrink: true,
            shrink_attempts: 200,
            shrink_timeout: Duration::from_secs(60),
        }
    }
}

fn default_duration() -> Duration {
    Duration::from_secs(3)
}

fn default_timeout() -> Duration {
    Duration::from_secs(5)
}

/// A file that does not parse or does not hold together.
#[derive(Debug, thiserror::Error)]
pub enum CampaignError {
    /// TOML or an unknown key.
    #[error("{0}")]
    Parse(String),
    /// A cross-check failed.
    #[error("{0}")]
    Check(String),
}

impl Campaign {
    /// Parse and check.
    ///
    /// # Errors
    /// [`CampaignError`] with the first problem.
    pub fn parse(text: &str) -> Result<Self, CampaignError> {
        let campaign: Self =
            toml::from_str(text).map_err(|e| CampaignError::Parse(e.to_string()))?;
        campaign.check().map_err(CampaignError::Check)?;
        Ok(campaign)
    }

    /// Every check that needs no ledger.
    ///
    /// # Errors
    /// The first problem, as a sentence.
    pub fn check(&self) -> Result<(), String> {
        if self.campaign.name.trim().is_empty() {
            return Err("campaign.name is empty".into());
        }
        if self.campaign.duration.is_zero() {
            return Err("campaign.duration must be above zero".into());
        }
        if self.campaign.timeout.is_zero() {
            return Err("campaign.timeout must be above zero".into());
        }
        let w = &self.workload;
        if w.max_in_flight == 0 {
            return Err("workload.max_in_flight must be above zero".into());
        }
        if w.paths.is_empty() {
            return Err("workload.paths is empty".into());
        }
        for p in w.paths.iter().chain(&w.relation_paths) {
            if !valid_path(p) {
                return Err(format!(
                    "workload path {p:?}: lower-case segments joined by dots, at least two"
                ));
            }
        }
        if w.scopes.is_empty() {
            return Err("workload.scopes is empty".into());
        }
        for s in &w.scopes {
            if !valid_scope(s) {
                return Err(format!("workload scope {s:?}: [a-z0-9_.@-], 1 to 64 bytes"));
            }
        }
        if w.owner.workers == 0 {
            return Err("workload.owner.workers must be above zero".into());
        }
        if w.owner.subjects == 0 {
            return Err("workload.owner.subjects must be above zero".into());
        }
        if w.owner.limit > 1000 {
            return Err("workload.owner.limit: at most 1000 (the ledger's page cap)".into());
        }
        if w.owner.mix.any_negative() {
            return Err("workload.owner.mix: weights are zero or above".into());
        }
        if w.owner.mix.weighted().is_empty() {
            return Err("workload.owner.mix: give at least one operation a weight".into());
        }
        if w.owner.mix.pair_relation > 0.0 && w.owner.subjects < 2 {
            return Err("workload.owner.mix.pair_relation needs owner.subjects >= 2".into());
        }
        if w.owner.mix.pair_relation > 0.0 && w.relation_paths.is_empty() {
            return Err("workload.owner.mix.pair_relation needs workload.relation_paths".into());
        }
        for name in self.invariants.keys() {
            if !invariants::ALL.contains(&name.as_str()) {
                return Err(format!(
                    "invariants.{name}: unknown; one of {}",
                    invariants::ALL.join(", ")
                ));
            }
        }
        for class in &self.faults.tolerate {
            if !crate::client::KNOWN_CLASSES.contains(&class.as_str()) {
                return Err(format!(
                    "faults.tolerate {class:?}: unknown class; one of {}",
                    crate::client::KNOWN_CLASSES.join(", ")
                ));
            }
        }
        Ok(())
    }

    /// Whether an invariant is on.
    #[must_use]
    pub fn invariant_on(&self, name: &str) -> bool {
        self.invariants.get(name).copied().unwrap_or(true)
    }

    /// The invariants that are on, in catalogue order.
    #[must_use]
    pub fn enabled_invariants(&self) -> Vec<&'static str> {
        invariants::ALL
            .iter()
            .copied()
            .filter(|n| self.invariant_on(n))
            .collect()
    }

    /// Whether the campaign has a stack of its own.
    #[must_use]
    pub fn has_stack(&self) -> bool {
        !self.stack.is_empty()
    }
}

fn valid_path(p: &str) -> bool {
    let segments: Vec<&str> = p.split('.').collect();
    segments.len() >= 2
        && segments.iter().all(|s| {
            !s.is_empty()
                && s.bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
        })
}

fn valid_scope(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes().all(|b| {
            b.is_ascii_lowercase()
                || b.is_ascii_digit()
                || b == b'_'
                || b == b'.'
                || b == b'@'
                || b == b'-'
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL: &str = "[campaign]\nname = \"t\"\n";

    #[test]
    fn defaults_hold_together() {
        let c = Campaign::parse(MINIMAL).unwrap();
        assert_eq!(c.campaign.duration, Duration::from_secs(3));
        assert_eq!(c.workload.owner.workers, 4);
        assert_eq!(c.enabled_invariants().len(), invariants::ALL.len());
        assert!(!c.has_stack());
    }

    #[test]
    fn unknown_keys_and_names_are_refused() {
        let e = Campaign::parse("[campaign]\nname = \"t\"\nnope = 1\n").unwrap_err();
        assert!(matches!(e, CampaignError::Parse(_)), "{e}");
        let e = Campaign::parse(&format!("{MINIMAL}[invariants]\nnope = false\n")).unwrap_err();
        assert!(e.to_string().contains("invariants.nope"), "{e}");
        let e =
            Campaign::parse(&format!("{MINIMAL}[faults]\ntolerate = [\"boom\"]\n")).unwrap_err();
        assert!(e.to_string().contains("faults.tolerate"), "{e}");
        let e = Campaign::parse(&format!(
            "{MINIMAL}[workload.owner]\nsubjects = 1\n[workload.owner.mix]\npair_relation = 1\n"
        ))
        .unwrap_err();
        assert!(e.to_string().contains("owner.subjects >= 2"), "{e}");
        let e = Campaign::parse(&format!("{MINIMAL}[workload]\npaths = [\"Bad\"]\n")).unwrap_err();
        assert!(e.to_string().contains("path"), "{e}");
    }

    #[test]
    fn a_present_mix_lists_exactly_what_runs() {
        let c = Campaign::parse(&format!(
            "{MINIMAL}[workload.owner.mix]\nappend = 2\nhistory = 1\n"
        ))
        .unwrap();
        let ops: Vec<_> = c
            .workload
            .owner
            .mix
            .weighted()
            .into_iter()
            .map(|(o, _)| o)
            .collect();
        assert_eq!(ops, [OwnerOp::Append, OwnerOp::History]);
        assert_eq!(
            Campaign::parse(MINIMAL)
                .unwrap()
                .workload
                .owner
                .mix
                .weighted()
                .len(),
            9
        );
    }

    #[test]
    fn stack_and_timeline_pass_through() {
        let c = Campaign::parse(
            "[campaign]\nname = \"t\"\n[stack.ledgers.l]\ngrace = \"0s\"\n[[timeline]]\nat = \"1s\"\naction = \"log\"\nmessage = \"hi\"\n",
        )
        .unwrap();
        assert!(c.has_stack());
        assert_eq!(c.timeline.len(), 1);
        // Round-trips through JSON for the API's `parsed`.
        let json = serde_json::to_value(&c).unwrap();
        assert_eq!(json["stack"]["ledgers"]["l"]["grace"], "0s");
    }
}
