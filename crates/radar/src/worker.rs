//! The radar's two jobs, shared by the timers and the admin RPCs: read every
//! source (`refresh`), and write the week's digests (`digest`).

use std::{
    collections::BTreeSet,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use chrono::{DateTime, Datelike, Timelike, Utc};
use tbd_common::metrics::names;
use tokio_util::sync::CancellationToken;

use crate::{
    config::{Digest, Fetch, SourceSpec},
    digest::{DigestError, Writer, iso_week},
    fetch::Fetcher,
    store::{DigestRow, Store, StoreError},
};

/// What one refresh did.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Refreshed {
    /// Sources read.
    pub sources: u32,
    /// Sources that failed.
    pub failed: u32,
    /// Items seen for the first time.
    pub new_items: u64,
}

/// Why a digest run stopped.
#[derive(Debug, thiserror::Error)]
pub enum RunError {
    /// The store failed.
    #[error(transparent)]
    Store(#[from] StoreError),
    /// No llm service is configured.
    #[error("no llm service configured")]
    NoWriter,
}

/// The jobs over one store.
#[derive(Debug, Clone)]
pub struct Worker {
    store: Store,
    fetcher: Fetcher,
    writer: Option<Writer>,
    sources: Arc<Vec<SourceSpec>>,
    fetch: Fetch,
    digest: Digest,
    /// One digest run at a time, whoever started it (the schedule, an admin).
    running: Arc<AtomicBool>,
}

/// Clears the running flag when a run ends, however it ends.
struct Running(Arc<AtomicBool>);

impl Drop for Running {
    fn drop(&mut self) {
        self.0.store(false, Ordering::SeqCst);
    }
}

impl Worker {
    /// A worker over `store`.
    #[must_use]
    pub fn new(
        store: Store,
        fetcher: Fetcher,
        writer: Option<Writer>,
        sources: Vec<SourceSpec>,
        fetch: Fetch,
        digest: Digest,
    ) -> Self {
        Self {
            store,
            fetcher,
            writer,
            sources: Arc::new(sources),
            fetch,
            digest,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Whether an llm service is configured to write with.
    #[must_use]
    pub fn has_writer(&self) -> bool {
        self.writer.is_some()
    }

    fn claim(&self) -> Option<Running> {
        (!self.running.swap(true, Ordering::SeqCst)).then(|| Running(self.running.clone()))
    }

    /// Run [`Worker::digest`] now and wait for it, unless a run is already in
    /// progress (`Ok(None)`).
    ///
    /// # Errors
    /// As [`Worker::digest`].
    pub async fn digest_now(
        &self,
        now: DateTime<Utc>,
        force: bool,
    ) -> Result<Option<(String, Vec<DigestRow>)>, RunError> {
        let Some(_running) = self.claim() else {
            return Ok(None);
        };
        self.digest(now, force).await.map(Some)
    }

    /// Start [`Worker::digest`] in the background; false when a run is
    /// already in progress. The outcome is logged and counted.
    #[must_use]
    pub fn start_digest(&self, now: DateTime<Utc>, force: bool) -> bool {
        let Some(running) = self.claim() else {
            return false;
        };
        let worker = self.clone();
        tokio::spawn(async move {
            let _running = running;
            match worker.digest(now, force).await {
                Ok((week, written)) => {
                    tracing::info!(%week, written = written.len(), "radar digest run done");
                }
                Err(e) => tracing::warn!(error = %e, "radar digest run failed"),
            }
        });
        true
    }

    /// Read every source once; a failing source is counted and logged, never
    /// fatal.
    ///
    /// # Errors
    /// Storing the items failed.
    pub async fn refresh(&self, now: DateTime<Utc>) -> Result<Refreshed, StoreError> {
        let mut out = Refreshed::default();
        for source in self.sources.iter() {
            out.sources += 1;
            match self.fetcher.read(source, now).await {
                Ok(items) => {
                    let new = self.store.upsert_items(&items).await?;
                    out.new_items += new;
                    metrics::counter!(names::RADAR_FETCHES_TOTAL, "source" => source.name.clone(), "outcome" => "ok")
                        .increment(1);
                    metrics::counter!(names::RADAR_ITEMS_NEW_TOTAL, "source" => source.name.clone())
                        .increment(new);
                    tracing::info!(source = %source.name, read = items.len(), new, "radar source read");
                }
                Err(e) => {
                    out.failed += 1;
                    metrics::counter!(names::RADAR_FETCHES_TOTAL, "source" => source.name.clone(), "outcome" => "failed")
                        .increment(1);
                    tracing::warn!(source = %source.name, error = %e, "radar source failed");
                }
            }
        }
        Ok(out)
    }

    /// The week a run at `now` writes: the week of the day before, so a Monday
    /// morning run is last week's digest.
    #[must_use]
    pub fn week_of(now: DateTime<Utc>) -> String {
        iso_week(now - chrono::Duration::days(1))
    }

    /// Write the digests for the week of `now`: one per language the sources
    /// cover and per reader language, from the items of the seven days before
    /// `now`. Existing ones are kept unless `force`. A language with no items
    /// that week gets no digest; a failed digest is counted and logged, and the
    /// others still run.
    ///
    /// # Errors
    /// No llm service, or the store failed.
    pub async fn digest(
        &self,
        now: DateTime<Utc>,
        force: bool,
    ) -> Result<(String, Vec<DigestRow>), RunError> {
        let writer = self.writer.as_ref().ok_or(RunError::NoWriter)?;
        let week = Self::week_of(now);
        let from = now - chrono::Duration::days(7);
        let languages: BTreeSet<&str> = self.sources.iter().map(|s| s.language.as_str()).collect();
        let limit = i64::try_from(self.digest.max_items).unwrap_or(i64::MAX);
        let mut written = Vec::new();
        for language in languages {
            let items = self.store.items_between(language, from, now, limit).await?;
            for lang in &self.digest.langs {
                if !force && self.store.digest_exists(&week, language, lang).await? {
                    continue;
                }
                if items.is_empty() {
                    metrics::counter!(names::RADAR_DIGESTS_TOTAL, "language" => language.to_owned(), "lang" => lang.clone(), "outcome" => "no_items")
                        .increment(1);
                    tracing::info!(%week, language, lang, "no items this week; no digest");
                    continue;
                }
                match writer.write(&week, language, lang, &items).await {
                    Ok(d) => {
                        let stored = self.store.put_digest(&d).await?;
                        metrics::counter!(names::RADAR_DIGESTS_TOTAL, "language" => language.to_owned(), "lang" => lang.clone(), "outcome" => "written")
                            .increment(1);
                        tracing::info!(%week, language, lang, items = items.len(), model = %stored.model, "digest written");
                        written.push(stored);
                    }
                    Err(e) => {
                        metrics::counter!(names::RADAR_DIGESTS_TOTAL, "language" => language.to_owned(), "lang" => lang.clone(), "outcome" => "failed")
                            .increment(1);
                        log_failure(&week, language, lang, &e);
                    }
                }
            }
        }
        Ok((week, written))
    }

    /// Whether `now` is inside the weekly writing window.
    #[must_use]
    pub fn due(&self, now: DateTime<Utc>) -> bool {
        now.weekday().number_from_monday() == self.digest.weekday && now.hour() >= self.digest.hour
    }

    /// Run the fetch timer and the digest schedule until `cancel`.
    pub async fn run(self, cancel: CancellationToken) {
        let fetch = self.clone();
        let fetch_cancel = cancel.clone();
        let fetching = tokio::spawn(async move { fetch.fetch_loop(fetch_cancel).await });
        self.digest_loop(cancel).await;
        let _ = fetching.await;
    }

    async fn fetch_loop(&self, cancel: CancellationToken) {
        if self.fetch.interval_secs == 0 {
            return;
        }
        let mut wait = Duration::from_secs(self.fetch.initial_delay_secs);
        loop {
            tokio::select! {
                () = cancel.cancelled() => return,
                () = tokio::time::sleep(wait) => {}
            }
            if let Err(e) = self.refresh(Utc::now()).await {
                tracing::warn!(error = %e, "radar refresh failed");
            }
            wait = Duration::from_secs(self.fetch.interval_secs);
        }
    }

    async fn digest_loop(&self, cancel: CancellationToken) {
        if self.digest.check_secs == 0 || self.writer.is_none() {
            cancel.cancelled().await;
            return;
        }
        let every = Duration::from_secs(self.digest.check_secs);
        loop {
            tokio::select! {
                () = cancel.cancelled() => return,
                () = tokio::time::sleep(every) => {}
            }
            let now = Utc::now();
            if self.due(now)
                && let Err(e) = self.digest_now(now, false).await
            {
                tracing::warn!(error = %e, "radar digest run failed");
            }
        }
    }
}

fn log_failure(week: &str, language: &str, lang: &str, e: &DigestError) {
    tracing::warn!(week, language, lang, error = %e, "digest not written");
}
