//! Queued and scheduled runs.
//!
//! `chaos serve` runs one thing at a time (`RunStore::begin`). The queue is
//! what waits for that slot: "run all scenarios", a list pushed by a script,
//! and whatever a schedule fires. A schedule is a cron expression with a
//! [`Job`] attached; schedules live in one JSON file (`[paths] schedules`) so
//! they survive a restart, the queue is in memory and does not.

use std::{
    collections::{BTreeMap, VecDeque},
    path::{Path, PathBuf},
};

use chrono::{DateTime, SecondsFormat, Utc};
use croner::{
    Cron,
    parser::{CronParser, Seconds},
};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use super::{
    error::ApiError,
    runs::RunKind,
    state::{LoadRequest, ValidateRequest},
};

/// One unit of work, as carried by `POST /queue` and by a schedule.
///
/// JSON: `{"scenario": "<id>"}`, `"all_scenarios"`, `{"load": {…LoadRequest}}`,
/// `{"validate": {…ValidateRequest}}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Job {
    /// One scenario by id.
    Scenario(String),
    /// Every scenario that checks and is not skipped, in id order. Expanded
    /// into one queue item per scenario when queued.
    AllScenarios,
    /// An ad-hoc load run.
    Load(LoadRequest),
    /// A validate run.
    Validate(ValidateRequest),
}

impl Job {
    /// The kind of run it produces.
    pub fn kind(&self) -> RunKind {
        match self {
            Self::Scenario(_) | Self::AllScenarios => RunKind::Scenario,
            Self::Load(_) => RunKind::Load,
            Self::Validate(_) => RunKind::Validate,
        }
    }

    /// Name for lists, before the run exists.
    pub fn name(&self) -> String {
        match self {
            Self::Scenario(id) => id.clone(),
            Self::AllScenarios => "all scenarios".to_owned(),
            Self::Load(req) => req.name.clone().unwrap_or_else(|| "load".to_owned()),
            Self::Validate(_) => "validate".to_owned(),
        }
    }

    /// The scenario id, for scenario jobs.
    pub fn scenario_id(&self) -> Option<&str> {
        match self {
            Self::Scenario(id) => Some(id),
            _ => None,
        }
    }
}

/// A job waiting for the active slot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedRun {
    /// Queue item id (UUID v7).
    pub id: String,
    /// What runs.
    pub job: Job,
    /// Kind of run it will be.
    pub kind: RunKind,
    /// Name it will have.
    pub name: String,
    /// Scenario id, for scenario jobs.
    pub scenario_id: Option<String>,
    /// The schedule that queued it, when one did.
    pub schedule_id: Option<String>,
    /// RFC 3339.
    pub queued_at: String,
}

/// FIFO of jobs waiting for the slot. In memory only.
#[derive(Default)]
pub struct Queue {
    items: Mutex<VecDeque<QueuedRun>>,
}

impl Queue {
    /// Everything waiting, front first.
    pub async fn list(&self) -> Vec<QueuedRun> {
        self.items.lock().await.iter().cloned().collect()
    }

    /// Append jobs, in order.
    pub async fn push(&self, jobs: Vec<Job>, schedule_id: Option<&str>) -> Vec<QueuedRun> {
        let mut items = self.items.lock().await;
        let mut out = Vec::with_capacity(jobs.len());
        for job in jobs {
            let item = QueuedRun {
                id: uuid::Uuid::now_v7().to_string(),
                kind: job.kind(),
                name: job.name(),
                scenario_id: job.scenario_id().map(str::to_owned),
                schedule_id: schedule_id.map(str::to_owned),
                queued_at: now_string(),
                job,
            };
            items.push_back(item.clone());
            out.push(item);
        }
        out
    }

    /// Take the front item.
    pub async fn pop(&self) -> Option<QueuedRun> {
        self.items.lock().await.pop_front()
    }

    /// Put an item back at the front (the slot was taken meanwhile).
    pub async fn push_front(&self, item: QueuedRun) {
        self.items.lock().await.push_front(item);
    }

    /// Drop one item. `false` when it is not queued.
    pub async fn remove(&self, id: &str) -> bool {
        let mut items = self.items.lock().await;
        let before = items.len();
        items.retain(|i| i.id != id);
        items.len() != before
    }

    /// Drop everything.
    pub async fn clear(&self) {
        self.items.lock().await.clear();
    }

    /// Whether a schedule already has an item waiting.
    pub async fn has_schedule(&self, schedule_id: &str) -> bool {
        self.items
            .lock()
            .await
            .iter()
            .any(|i| i.schedule_id.as_deref() == Some(schedule_id))
    }
}

/// What a client sends to create or replace a schedule.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScheduleSpec {
    /// Shown in lists.
    pub name: String,
    /// Cron expression, UTC: five fields (`*/15 * * * *`), or six with leading
    /// seconds. Nicknames like `@hourly` are accepted.
    pub cron: String,
    /// What it queues.
    pub job: Job,
    /// Off means kept but never fired.
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

/// A schedule as stored and returned.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schedule {
    /// UUID v7.
    pub id: String,
    /// Name.
    pub name: String,
    /// Cron expression.
    pub cron: String,
    /// What it queues.
    pub job: Job,
    /// Fires when on.
    pub enabled: bool,
    /// RFC 3339.
    pub created_at: String,
    /// RFC 3339.
    pub updated_at: String,
    /// When it will fire next, RFC 3339; `null` when disabled.
    #[serde(default)]
    pub next_at: Option<String>,
    /// When it last queued its job.
    #[serde(default)]
    pub last_fired_at: Option<String>,
    /// When it last fell due while its previous job was still queued or
    /// running, and was skipped.
    #[serde(default)]
    pub last_skipped_at: Option<String>,
    /// Times it queued its job.
    #[serde(default)]
    pub fired: u64,
    /// Times it was skipped.
    #[serde(default)]
    pub skipped: u64,
}

/// Schedules on disk: one JSON file, rewritten on every change.
pub struct Schedules {
    path: PathBuf,
    items: Mutex<BTreeMap<String, Schedule>>,
}

impl Schedules {
    /// Load `path` when it exists. `next_at` is recomputed from now, so fires
    /// missed while serve was down are dropped, not replayed.
    pub fn open(path: &Path) -> std::io::Result<Self> {
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)?;
        }
        let mut items: BTreeMap<String, Schedule> = match std::fs::read_to_string(path) {
            Ok(text) => serde_json::from_str::<Vec<Schedule>>(&text)
                .map_err(|e| std::io::Error::other(format!("{}: {e}", path.display())))?
                .into_iter()
                .map(|s| (s.id.clone(), s))
                .collect(),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => BTreeMap::new(),
            Err(e) => return Err(e),
        };
        let now = Utc::now();
        for s in items.values_mut() {
            s.next_at = next_at(s, now);
        }
        Ok(Self {
            path: path.to_owned(),
            items: Mutex::new(items),
        })
    }

    /// Every schedule, by id.
    pub async fn list(&self) -> Vec<Schedule> {
        self.items.lock().await.values().cloned().collect()
    }

    /// One schedule.
    pub async fn get(&self, id: &str) -> Option<Schedule> {
        self.items.lock().await.get(id).cloned()
    }

    /// Validate and store a new schedule.
    pub async fn create(&self, spec: ScheduleSpec) -> Result<Schedule, ApiError> {
        check_spec(&spec)?;
        let now = Utc::now();
        let mut schedule = Schedule {
            id: uuid::Uuid::now_v7().to_string(),
            name: spec.name,
            cron: spec.cron,
            job: spec.job,
            enabled: spec.enabled,
            created_at: to_string(now),
            updated_at: to_string(now),
            next_at: None,
            last_fired_at: None,
            last_skipped_at: None,
            fired: 0,
            skipped: 0,
        };
        schedule.next_at = next_at(&schedule, now);
        let mut items = self.items.lock().await;
        items.insert(schedule.id.clone(), schedule.clone());
        self.save(&items).await?;
        Ok(schedule)
    }

    /// Validate and replace name, cron, job and enabled; counters stay.
    pub async fn update(&self, id: &str, spec: ScheduleSpec) -> Result<Schedule, ApiError> {
        check_spec(&spec)?;
        let now = Utc::now();
        let mut items = self.items.lock().await;
        let schedule = items
            .get_mut(id)
            .ok_or_else(|| ApiError::not_found(format!("no schedule {id}")))?;
        schedule.name = spec.name;
        schedule.cron = spec.cron;
        schedule.job = spec.job;
        schedule.enabled = spec.enabled;
        schedule.updated_at = to_string(now);
        schedule.next_at = next_at(schedule, now);
        let out = schedule.clone();
        self.save(&items).await?;
        Ok(out)
    }

    /// Remove a schedule.
    pub async fn delete(&self, id: &str) -> Result<(), ApiError> {
        let mut items = self.items.lock().await;
        if items.remove(id).is_none() {
            return Err(ApiError::not_found(format!("no schedule {id}")));
        }
        self.save(&items).await?;
        Ok(())
    }

    /// Enabled schedules whose `next_at` has passed, each advanced to its
    /// following occurrence. The caller queues or skips them and records which.
    pub async fn take_due(&self, now: DateTime<Utc>) -> Vec<Schedule> {
        let mut items = self.items.lock().await;
        let mut due = Vec::new();
        for s in items.values_mut() {
            if !s.enabled {
                continue;
            }
            let Some(at) = s.next_at.as_deref().and_then(parse_time) else {
                s.next_at = next_at(s, now);
                continue;
            };
            if at <= now {
                s.next_at = next_at(s, now);
                due.push(s.clone());
            }
        }
        due
    }

    /// Note that a due schedule queued its job (`fired`) or was skipped.
    pub async fn mark(&self, id: &str, fired: bool, now: DateTime<Utc>) {
        let mut items = self.items.lock().await;
        if let Some(s) = items.get_mut(id) {
            if fired {
                s.fired += 1;
                s.last_fired_at = Some(to_string(now));
            } else {
                s.skipped += 1;
                s.last_skipped_at = Some(to_string(now));
            }
        }
        if let Err(error) = self.save(&items).await {
            tracing::error!(%error, "could not write schedules");
        }
    }

    async fn save(&self, items: &BTreeMap<String, Schedule>) -> std::io::Result<()> {
        let list: Vec<&Schedule> = items.values().collect();
        let text = serde_json::to_string_pretty(&list)?;
        let tmp = self.path.with_extension("json.tmp");
        tokio::fs::write(&tmp, text).await?;
        tokio::fs::rename(&tmp, &self.path).await
    }
}

/// Parse a cron expression the way schedules accept it: five fields, or six
/// with leading seconds, UTC.
pub fn parse_cron(expr: &str) -> Result<Cron, ApiError> {
    CronParser::builder()
        .seconds(Seconds::Optional)
        .build()
        .parse(expr)
        .map_err(|e| ApiError::invalid(format!("cron {expr:?}: {e}")))
}

fn check_spec(spec: &ScheduleSpec) -> Result<(), ApiError> {
    if spec.name.trim().is_empty() {
        return Err(ApiError::invalid("schedule name is empty"));
    }
    parse_cron(&spec.cron)?;
    if let Job::Load(req) = &spec.job {
        req.load.check().map_err(ApiError::invalid)?;
    }
    Ok(())
}

/// The next fire time after `now`, or `None` when disabled or the expression
/// never matches again.
fn next_at(s: &Schedule, now: DateTime<Utc>) -> Option<String> {
    if !s.enabled {
        return None;
    }
    let cron = parse_cron(&s.cron).ok()?;
    cron.find_next_occurrence(&now, false).ok().map(to_string)
}

fn parse_time(s: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|t| t.with_timezone(&Utc))
}

fn to_string(t: DateTime<Utc>) -> String {
    t.to_rfc3339_opts(SecondsFormat::Millis, true)
}

fn now_string() -> String {
    to_string(Utc::now())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cron_accepts_five_and_six_fields_and_nicknames() {
        assert!(parse_cron("*/15 * * * *").is_ok());
        assert!(parse_cron("0 */5 * * * *").is_ok());
        assert!(parse_cron("@hourly").is_ok());
        assert!(parse_cron("every day").is_err());
        assert!(parse_cron("").is_err());
    }

    #[test]
    fn job_json_shapes() {
        let j: Job = serde_json::from_str(r#"{"scenario":"baseline"}"#).unwrap();
        assert!(matches!(j, Job::Scenario(ref id) if id == "baseline"));
        let j: Job = serde_json::from_str(r#""all_scenarios""#).unwrap();
        assert!(matches!(j, Job::AllScenarios));
        let j: Job = serde_json::from_str(r#"{"validate":{}}"#).unwrap();
        assert!(matches!(j, Job::Validate(_)));
        assert_eq!(
            serde_json::to_string(&Job::Scenario("x".into())).unwrap(),
            r#"{"scenario":"x"}"#
        );
    }

    #[tokio::test]
    async fn schedules_round_trip_through_the_file() {
        let dir = std::env::temp_dir().join(format!("chaos-sched-{}", uuid::Uuid::now_v7()));
        let path = dir.join("schedules.json");
        let store = Schedules::open(&path).unwrap();
        let s = store
            .create(ScheduleSpec {
                name: "nightly".into(),
                cron: "0 3 * * *".into(),
                job: Job::AllScenarios,
                enabled: true,
            })
            .await
            .unwrap();
        assert!(s.next_at.is_some());
        let bad = store
            .create(ScheduleSpec {
                name: "bad".into(),
                cron: "nope".into(),
                job: Job::AllScenarios,
                enabled: true,
            })
            .await;
        assert_eq!(bad.unwrap_err().status, 422);

        let again = Schedules::open(&path).unwrap();
        let list = again.list().await;
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "nightly");

        let off = again
            .update(
                &s.id,
                ScheduleSpec {
                    name: "nightly".into(),
                    cron: "0 3 * * *".into(),
                    job: Job::AllScenarios,
                    enabled: false,
                },
            )
            .await
            .unwrap();
        assert!(off.next_at.is_none());
        assert!(again.take_due(Utc::now()).await.is_empty());
        again.delete(&s.id).await.unwrap();
        assert!(again.delete(&s.id).await.is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn due_schedules_are_taken_once_and_advanced() {
        let dir = std::env::temp_dir().join(format!("chaos-due-{}", uuid::Uuid::now_v7()));
        let store = Schedules::open(&dir.join("s.json")).unwrap();
        let s = store
            .create(ScheduleSpec {
                name: "tick".into(),
                cron: "* * * * * *".into(),
                job: Job::AllScenarios,
                enabled: true,
            })
            .await
            .unwrap();
        let later = Utc::now() + chrono::Duration::seconds(2);
        let due = store.take_due(later).await;
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].id, s.id);
        assert!(store.take_due(later).await.is_empty());
        store.mark(&s.id, true, later).await;
        assert_eq!(store.get(&s.id).await.unwrap().fired, 1);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
