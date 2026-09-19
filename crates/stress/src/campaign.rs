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
    /// `[sweep]`: run the campaign once per value of one parameter.
    #[serde(default)]
    pub sweep: Option<Sweep>,
    /// `[stop]`.
    #[serde(default)]
    pub stop: Stop,
}

/// `[sweep]`: the same campaign at every value of one parameter, repeated, so
/// a difference between two points can be told from noise.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sweep {
    /// What varies, one of [`SWEEP_PARAMETERS`].
    pub parameter: String,
    /// The values, in the order they run.
    pub values: Vec<u64>,
    /// Measurements per value; every repeat is a fresh set of workers and
    /// subjects, and their latencies pool into the point's interval.
    #[serde(default = "one")]
    pub repeat: u32,
    /// The measured phase of one repeat; replaces `[campaign] duration`.
    #[serde(default = "default_point_duration", with = "humantime_serde")]
    pub point_duration: Duration,
    /// `[sweep.knee]`: where the curve is called broken.
    #[serde(default)]
    pub knee: Knee,
}

/// `[sweep.knee]`: the first point that crosses either bound is the knee.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Knee {
    /// A p99 above this, in milliseconds.
    pub p99_ms: Option<f64>,
    /// A p99 at least this many times the previous point's.
    pub factor: Option<f64>,
}

/// The parameters a sweep may vary. `owner.pace` and `contention.pace` take
/// milliseconds; everything else is a count.
pub const SWEEP_PARAMETERS: &[&str] = &[
    "owner.workers",
    "owner.subjects",
    "owner.pace",
    "contention.workers",
    "contention.subjects",
    "contention.pace",
    "fuzz.workers",
    "max_in_flight",
];

fn one() -> u32 {
    1
}

fn default_point_duration() -> Duration {
    Duration::from_secs(5)
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
    /// `[workload.contention]`.
    pub contention: Contention,
    /// `[workload.fuzz]`.
    pub fuzz: Fuzz,
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
            contention: Contention::default(),
            fuzz: Fuzz::default(),
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

/// `[workload.contention]`: workers that share subjects. No worker knows the
/// whole truth about a shared subject, so the model is order-free: what I was
/// acknowledged must be visible, and what `Current` shows must be the newest
/// row the snapshot holds or newer than the snapshot. Off unless `workers` is
/// set; every contention worker runs against the first target.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Contention {
    /// Concurrent workers; 0 switches the class off.
    pub workers: u32,
    /// Shared subjects, the same set for every contention worker.
    pub subjects: u32,
    /// Think time between operations.
    #[serde(with = "humantime_serde")]
    pub pace: Duration,
    /// Page size for reads; 0 draws one per read.
    pub limit: u32,
    /// `[workload.contention.mix]`.
    pub mix: ContentionMix,
}

impl Default for Contention {
    fn default() -> Self {
        Self {
            workers: 0,
            subjects: 4,
            pace: Duration::ZERO,
            limit: 0,
            mix: ContentionMix::default(),
        }
    }
}

/// Relative weights of the contention operations; a missing key is 0 when the
/// table is present.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentionMix {
    /// One fact, keyed.
    #[serde(default)]
    pub append: f64,
    /// `History` then `Current`, judged for consistency.
    #[serde(default)]
    pub current: f64,
    /// `History`, judged for the worker's own acknowledged facts.
    #[serde(default)]
    pub history: f64,
    /// A retraction of a shared key.
    #[serde(default)]
    pub retract: f64,
}

impl Default for ContentionMix {
    fn default() -> Self {
        Self {
            append: 5.0,
            current: 2.0,
            history: 2.0,
            retract: 1.0,
        }
    }
}

impl ContentionMix {
    /// `(operation, weight)` for every weight above zero.
    #[must_use]
    pub fn weighted(&self) -> Vec<(ContentionOp, f64)> {
        [
            (ContentionOp::Append, self.append),
            (ContentionOp::Current, self.current),
            (ContentionOp::History, self.history),
            (ContentionOp::Retract, self.retract),
        ]
        .into_iter()
        .filter(|(_, w)| *w > 0.0)
        .collect()
    }

    fn any_negative(&self) -> bool {
        [self.append, self.current, self.history, self.retract]
            .iter()
            .any(|w| *w < 0.0 || w.is_nan())
    }
}

/// The contention operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentionOp {
    /// One fact.
    Append,
    /// `History` then `Current`.
    Current,
    /// `History`.
    History,
    /// A retraction.
    Retract,
}

/// `[workload.fuzz]`: workers that send hostile requests on subjects of their
/// own and expect a clean refusal every time. Off unless `workers` is set.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Fuzz {
    /// Concurrent workers; 0 switches the class off.
    pub workers: u32,
    /// Subjects per worker, each seeded with one valid fact.
    pub subjects: u32,
    /// Think time between requests.
    #[serde(with = "humantime_serde")]
    pub pace: Duration,
}

impl Default for Fuzz {
    fn default() -> Self {
        Self {
            workers: 0,
            subjects: 2,
            pace: Duration::ZERO,
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
        if w.owner.workers == 0 && w.contention.workers == 0 && w.fuzz.workers == 0 {
            return Err("workload: give owner, contention or fuzz some workers".into());
        }
        if w.owner.workers > 0 {
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
                return Err(
                    "workload.owner.mix.pair_relation needs workload.relation_paths".into(),
                );
            }
        }
        if w.contention.workers > 0 {
            if w.contention.subjects == 0 {
                return Err("workload.contention.subjects must be above zero".into());
            }
            if w.contention.limit > 1000 {
                return Err(
                    "workload.contention.limit: at most 1000 (the ledger's page cap)".into(),
                );
            }
            if w.contention.mix.any_negative() {
                return Err("workload.contention.mix: weights are zero or above".into());
            }
            if w.contention.mix.weighted().is_empty() {
                return Err("workload.contention.mix: give at least one operation a weight".into());
            }
        }
        if w.fuzz.workers > 0 && w.fuzz.subjects == 0 {
            return Err("workload.fuzz.subjects must be above zero".into());
        }
        for name in self.invariants.keys() {
            if !invariants::ALL.contains(&name.as_str()) {
                return Err(format!(
                    "invariants.{name}: unknown; one of {}",
                    invariants::ALL.join(", ")
                ));
            }
        }
        self.check_sweep()?;
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

    /// `[sweep]`, when there is one: the parameter exists, the values suit it,
    /// and the class it varies is doing work.
    fn check_sweep(&self) -> Result<(), String> {
        let Some(s) = &self.sweep else {
            return Ok(());
        };
        let w = &self.workload;
        if !SWEEP_PARAMETERS.contains(&s.parameter.as_str()) {
            return Err(format!(
                "sweep.parameter {:?}: unknown; one of {}",
                s.parameter,
                SWEEP_PARAMETERS.join(", ")
            ));
        }
        if s.values.is_empty() {
            return Err("sweep.values is empty".into());
        }
        if s.repeat == 0 {
            return Err("sweep.repeat must be above zero".into());
        }
        if s.point_duration.is_zero() {
            return Err("sweep.point_duration must be above zero".into());
        }
        let counts = !s.parameter.ends_with("pace");
        if counts && s.values.contains(&0) {
            return Err(format!(
                "sweep.values: {} counts from one; only a pace may be zero",
                s.parameter
            ));
        }
        if s.knee.factor.is_some_and(|f| f <= 1.0) {
            return Err("sweep.knee.factor must be above one".into());
        }
        if s.knee.p99_ms.is_some_and(|m| m <= 0.0) {
            return Err("sweep.knee.p99_ms must be above zero".into());
        }
        // A swept class must be the one doing the work, so the numbers move.
        let workers = match s.parameter.as_str() {
            "contention.subjects" | "contention.pace" => w.contention.workers > 0,
            "owner.subjects" | "owner.pace" => w.owner.workers > 0,
            _ => true,
        };
        if !workers {
            return Err(format!(
                "sweep.parameter {:?}: that class has no workers",
                s.parameter
            ));
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

    /// This campaign with the sweep's parameter set to `value` and the measured
    /// phase cut to one point: what a single sweep point runs.
    #[must_use]
    pub fn at_point(&self, value: u64) -> Self {
        let mut c = self.clone();
        let Some(sweep) = &self.sweep else { return c };
        c.campaign.duration = sweep.point_duration;
        let count = u32::try_from(value).unwrap_or(u32::MAX);
        match sweep.parameter.as_str() {
            "owner.workers" => c.workload.owner.workers = count,
            "owner.subjects" => c.workload.owner.subjects = count,
            "owner.pace" => c.workload.owner.pace = Duration::from_millis(value),
            "contention.workers" => c.workload.contention.workers = count,
            "contention.subjects" => c.workload.contention.subjects = count,
            "contention.pace" => c.workload.contention.pace = Duration::from_millis(value),
            "fuzz.workers" => c.workload.fuzz.workers = count,
            "max_in_flight" => {
                c.workload.max_in_flight = usize::try_from(value).unwrap_or(usize::MAX);
            }
            _ => {}
        }
        c
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
    fn classes_are_off_until_given_workers_and_one_must_run() {
        let c = Campaign::parse(MINIMAL).unwrap();
        assert_eq!(c.workload.contention.workers, 0);
        assert_eq!(c.workload.fuzz.workers, 0);
        let e = Campaign::parse(&format!("{MINIMAL}[workload.owner]\nworkers = 0\n")).unwrap_err();
        assert!(e.to_string().contains("some workers"), "{e}");
        // A fuzz-only campaign needs no owner mix to hold together.
        let c = Campaign::parse(&format!(
            "{MINIMAL}[workload.owner]\nworkers = 0\n[workload.fuzz]\nworkers = 2\n"
        ))
        .unwrap();
        assert_eq!(c.workload.fuzz.workers, 2);
        let e = Campaign::parse(&format!(
            "{MINIMAL}[workload.contention]\nworkers = 2\nsubjects = 0\n"
        ))
        .unwrap_err();
        assert!(e.to_string().contains("contention.subjects"), "{e}");
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
    fn a_sweep_point_is_the_campaign_with_one_value_changed() {
        let c = Campaign::parse(&format!(
            "{MINIMAL}[sweep]\nparameter = \"owner.workers\"\nvalues = [1, 4]\nrepeat = 2\npoint_duration = \"500ms\"\n[sweep.knee]\np99_ms = 50\n"
        ))
        .unwrap();
        let s = c.sweep.as_ref().unwrap();
        assert_eq!(s.values, [1, 4]);
        assert_eq!(s.repeat, 2);
        assert_eq!(s.knee.p99_ms, Some(50.0));
        let point = c.at_point(4);
        assert_eq!(point.workload.owner.workers, 4);
        assert_eq!(point.campaign.duration, Duration::from_millis(500));
        // Everything else is the campaign as written.
        assert_eq!(point.workload.owner.subjects, c.workload.owner.subjects);
        // A pace sweeps in milliseconds.
        let paced = Campaign::parse(&format!(
            "{MINIMAL}[sweep]\nparameter = \"owner.pace\"\nvalues = [0, 20]\n"
        ))
        .unwrap();
        assert_eq!(
            paced.at_point(20).workload.owner.pace,
            Duration::from_millis(20)
        );
    }

    #[test]
    fn a_sweep_is_checked_like_everything_else() {
        let bad = |sweep: &str| {
            Campaign::parse(&format!("{MINIMAL}[sweep]\n{sweep}"))
                .unwrap_err()
                .to_string()
        };
        assert!(bad("parameter = \"nope\"\nvalues = [1]\n").contains("sweep.parameter"));
        assert!(
            bad("parameter = \"owner.workers\"\nvalues = []\n").contains("sweep.values is empty")
        );
        assert!(
            bad("parameter = \"owner.workers\"\nvalues = [0, 1]\n").contains("counts from one")
        );
        assert!(
            bad("parameter = \"owner.workers\"\nvalues = [1]\nrepeat = 0\n")
                .contains("sweep.repeat")
        );
        assert!(
            bad("parameter = \"owner.workers\"\nvalues = [1]\n[sweep.knee]\nfactor = 1.0\n")
                .contains("factor must be above one")
        );
        // A pace may be zero, and a sweep of a class with no workers is refused.
        assert!(
            Campaign::parse(&format!(
                "{MINIMAL}[sweep]\nparameter = \"owner.pace\"\nvalues = [0]\n"
            ))
            .is_ok()
        );
        assert!(bad("parameter = \"contention.subjects\"\nvalues = [2]\n").contains("no workers"));
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
