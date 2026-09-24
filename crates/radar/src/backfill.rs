//! The backfill: past weeks written from the sources' archives (RFC 0007,
//! `docs/quietpager/README.md`). It imports every archive's items for the
//! range, then walks the weeks oldest first and, per language with enough
//! items and per reader language, writes a digest on the archive tier,
//! published at once and marked `archive`. It stops before the first week the
//! weekly run has written, never overwrites a digest (so a stopped run resumes),
//! runs `[archive] parallel` digests at a time, and retries a failed one with
//! backoff. One run at a time; its progress is readable while it runs.

use std::{
    pin::Pin,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use chrono::{DateTime, NaiveDate, TimeZone, Utc, Weekday};
use futures::{StreamExt, stream};
use tbd_common::metrics::names;

use crate::{
    archive::Archive,
    config::{Archive as ArchiveConfig, Digest as DigestConfig},
    digest::Writer,
    store::{ItemRow, Store},
};

/// Where a run is; a snapshot is what `GetBackfill` answers.
#[derive(Debug, Clone, Default)]
pub struct Progress {
    /// A run is in progress.
    pub running: bool,
    /// The range asked for.
    pub from_week: String,
    /// The range asked for.
    pub to_week: String,
    /// `"import"`, `"write"` or `"done"`.
    pub phase: String,
    /// Items seen for the first time while importing.
    pub items_imported: u64,
    /// Weeks to write, after the cut at the first live week.
    pub weeks_total: u32,
    /// Weeks finished.
    pub weeks_done: u32,
    /// The week being written.
    pub current_week: String,
    /// Digests written.
    pub written: u32,
    /// Digests not written: a thin week, or one already written.
    pub skipped: u32,
    /// Digests that failed after every retry.
    pub failed: u32,
    /// When the run started.
    pub started_at: Option<DateTime<Utc>>,
    /// When it finished.
    pub finished_at: Option<DateTime<Utc>>,
}

/// Why a backfill could not start.
#[derive(Debug, thiserror::Error)]
pub enum BackfillError {
    /// A week label is not `YYYY-Www`, or the range is backwards.
    #[error("{0}")]
    Range(String),
    /// No llm service is configured.
    #[error("no llm service configured")]
    NoWriter,
}

/// Monday 00:00 UTC of an ISO week label ("2025-W01").
///
/// # Errors
/// The label is not a valid ISO week.
pub fn week_start(label: &str) -> Result<DateTime<Utc>, BackfillError> {
    let bad = || BackfillError::Range(format!("{label:?} is not an ISO week like 2025-W01"));
    let (y, w) = label.split_once("-W").ok_or_else(bad)?;
    let d = NaiveDate::from_isoywd_opt(
        y.parse().map_err(|_| bad())?,
        w.parse().map_err(|_| bad())?,
        Weekday::Mon,
    )
    .ok_or_else(bad)?;
    Ok(Utc.from_utc_datetime(&d.and_hms_opt(0, 0, 0).unwrap_or_default()))
}

/// Every ISO week from `from` to `to`, inclusive, as (label, Monday).
///
/// # Errors
/// A label is invalid or the range is backwards.
pub fn weeks(from: &str, to: &str) -> Result<Vec<(String, DateTime<Utc>)>, BackfillError> {
    let (start, end) = (week_start(from)?, week_start(to)?);
    if start > end {
        return Err(BackfillError::Range(format!("{from} is after {to}")));
    }
    let mut out = Vec::new();
    let mut at = start;
    while at <= end {
        out.push((crate::digest::iso_week(at), at));
        at += chrono::Duration::days(7);
    }
    Ok(out)
}

/// The backfill over one store.
#[derive(Debug, Clone)]
pub struct Backfill {
    store: Store,
    archive: Archive,
    writer: Option<Writer>,
    config: ArchiveConfig,
    digest: DigestConfig,
    running: Arc<AtomicBool>,
    progress: Arc<Mutex<Progress>>,
}

impl Backfill {
    /// A backfill over `store`.
    #[must_use]
    pub fn new(
        store: Store,
        archive: Archive,
        writer: Option<Writer>,
        config: ArchiveConfig,
        digest: DigestConfig,
    ) -> Self {
        Self {
            store,
            archive,
            writer,
            config,
            digest,
            running: Arc::new(AtomicBool::new(false)),
            progress: Arc::new(Mutex::new(Progress::default())),
        }
    }

    /// A snapshot of the current or last run.
    #[must_use]
    pub fn progress(&self) -> Progress {
        self.progress.lock().map(|p| p.clone()).unwrap_or_default()
    }

    fn update(&self, f: impl FnOnce(&mut Progress)) {
        if let Ok(mut p) = self.progress.lock() {
            f(&mut p);
        }
    }

    /// Start a run over `[from, to]` in the background; `Ok(false)` when one
    /// is already running.
    ///
    /// # Errors
    /// The range is invalid, or no llm service is configured.
    pub fn start(&self, from: &str, to: &str) -> Result<bool, BackfillError> {
        let weeks = weeks(from, to)?;
        if self.writer.is_none() {
            return Err(BackfillError::NoWriter);
        }
        if self.running.swap(true, Ordering::SeqCst) {
            return Ok(false);
        }
        self.update(|p| {
            *p = Progress {
                running: true,
                from_week: from.to_owned(),
                to_week: to.to_owned(),
                phase: "import".to_owned(),
                started_at: Some(Utc::now()),
                ..Progress::default()
            };
        });
        let this = self.clone();
        tokio::spawn(async move {
            this.run(weeks).await;
            this.update(|p| {
                p.running = false;
                "done".clone_into(&mut p.phase);
                p.finished_at = Some(Utc::now());
            });
            this.running.store(false, Ordering::SeqCst);
        });
        Ok(true)
    }

    async fn run(&self, weeks: Vec<(String, DateTime<Utc>)>) {
        let Some(writer) = self.writer.clone() else {
            return;
        };
        let (Some(first), Some(last)) = (weeks.first(), weeks.last()) else {
            return;
        };
        let (from, to) = (first.1, last.1 + chrono::Duration::days(7));

        let items = self.archive.read(from, to).await;
        match self.store.upsert_items(&items).await {
            Ok(new) => {
                self.update(|p| p.items_imported = new);
                tracing::info!(read = items.len(), new, "radar archive imported");
            }
            Err(e) => {
                tracing::warn!(error = %e, "radar archive import failed; stopping");
                return;
            }
        }

        // Never write over, or next to, what the weekly run owns.
        let cut = self.store.earliest_live_week().await.ok().flatten();
        let weeks: Vec<_> = weeks
            .into_iter()
            .filter(|(w, _)| cut.as_ref().is_none_or(|c| w < c))
            .collect();
        let total = u32::try_from(weeks.len()).unwrap_or(u32::MAX);
        self.update(|p| {
            "write".clone_into(&mut p.phase);
            p.weeks_total = total;
        });
        tracing::info!(weeks = total, cut = ?cut, "radar backfill writing");

        let limit = i64::try_from(self.digest.max_items).unwrap_or(i64::MAX);
        for (week, monday) in weeks {
            self.update(|p| p.current_week.clone_from(&week));
            let mut jobs = Vec::new();
            for language in ["go", "rust"] {
                let items = match self
                    .store
                    .items_between(language, monday, monday + chrono::Duration::days(7), limit)
                    .await
                {
                    Ok(items) => items,
                    Err(e) => {
                        tracing::warn!(%week, language, error = %e, "items not read");
                        continue;
                    }
                };
                for lang in &self.digest.langs {
                    if items.len() < self.config.min_items {
                        self.count(language, lang, "thin");
                        continue;
                    }
                    if self
                        .store
                        .digest_exists(&week, language, lang)
                        .await
                        .unwrap_or(false)
                    {
                        self.count(language, lang, "exists");
                        continue;
                    }
                    jobs.push((language, lang.clone(), items.clone()));
                }
            }
            // Owned, boxed futures built before they are driven: the run is a
            // spawned task, and a closure mapped over the stream trips the
            // compiler's higher-ranked lifetime check (rust-lang/rust#102211).
            let futures: Vec<Pin<Box<dyn Future<Output = ()> + Send>>> = jobs
                .into_iter()
                .map(|(language, lang, items)| {
                    let (this, writer, week) = (self.clone(), writer.clone(), week.clone());
                    Box::pin(async move {
                        this.write_one(&writer, &week, language, &lang, &items)
                            .await;
                    }) as Pin<Box<dyn Future<Output = ()> + Send>>
                })
                .collect();
            stream::iter(futures)
                .buffer_unordered(self.config.parallel.max(1))
                .collect::<Vec<()>>()
                .await;
            self.update(|p| p.weeks_done += 1);
        }
    }

    fn count(&self, language: &str, lang: &str, outcome: &'static str) {
        metrics::counter!(names::RADAR_BACKFILL_TOTAL, "language" => language.to_owned(), "lang" => lang.to_owned(), "outcome" => outcome)
            .increment(1);
        self.update(|p| match outcome {
            "written" => p.written += 1,
            "failed" => p.failed += 1,
            _ => p.skipped += 1,
        });
    }

    async fn write_one(
        &self,
        writer: &Writer,
        week: &str,
        language: &str,
        lang: &str,
        items: &[ItemRow],
    ) {
        let mut attempt = 0;
        loop {
            match writer
                .write_on(&self.config.tier, week, language, lang, items)
                .await
            {
                Ok(d) => {
                    match self.store.put_archive_digest(&d).await {
                        Ok(Some(_)) => self.count(language, lang, "written"),
                        Ok(None) => self.count(language, lang, "exists"),
                        Err(e) => {
                            tracing::warn!(week, language, lang, error = %e, "archive digest not stored");
                            self.count(language, lang, "failed");
                        }
                    }
                    return;
                }
                Err(e) if attempt < self.config.retries => {
                    attempt += 1;
                    let wait = Duration::from_secs(5 * 3u64.pow(attempt - 1));
                    tracing::info!(week, language, lang, attempt, error = %e, wait_secs = wait.as_secs(), "archive digest retry");
                    tokio::time::sleep(wait).await;
                }
                Err(e) => {
                    tracing::warn!(week, language, lang, error = %e, "archive digest failed");
                    self.count(language, lang, "failed");
                    return;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weeks_span_the_range_inclusive_across_a_year() {
        let w = weeks("2025-W52", "2026-W02").unwrap();
        let labels: Vec<_> = w.iter().map(|(l, _)| l.as_str()).collect();
        assert_eq!(labels, ["2025-W52", "2026-W01", "2026-W02"]);
        assert_eq!(w[0].1.format("%Y-%m-%d").to_string(), "2025-12-22");
    }

    #[test]
    fn bad_ranges_are_refused() {
        assert!(weeks("2026-W02", "2025-W52").is_err());
        assert!(weeks("2025-01", "2025-W02").is_err());
        assert!(weeks("2025-W54", "2025-W55").is_err());
    }
}
