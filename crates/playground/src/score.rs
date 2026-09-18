//! The fastest breaches, kept across restarts.
//!
//! The list is short and write-rarely: a breach happens at most once every few
//! minutes, so the whole thing is rewritten on each change rather than
//! appended to. A file that cannot be read starts an empty board rather than
//! failing the service — a lost leaderboard is not worth an outage.

use std::{io, path::PathBuf};

use serde::{Deserialize, Serialize};
use tbd_proto::playground::v1::Score as WireScore;
use tokio::sync::RwLock;

/// One breach worth remembering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    /// Who claimed it.
    pub actor: String,
    /// How long the breach took from the last healed moment.
    pub seconds: f64,
    /// How many faults were live when it broke.
    pub faults: u32,
    /// Unix seconds, so the file stays readable by eye.
    pub at: i64,
}

/// The board.
#[derive(Debug)]
pub struct Scores {
    entries: RwLock<Vec<Entry>>,
    keep: usize,
    path: Option<PathBuf>,
}

impl Scores {
    /// A board backed by a file, loaded now if it is there.
    pub async fn open(path: PathBuf, keep: usize) -> Self {
        let entries = match tokio::fs::read(&path).await {
            Ok(bytes) => serde_json::from_slice::<Vec<Entry>>(&bytes).unwrap_or_else(|error| {
                tracing::warn!(path = %path.display(), %error, "unreadable scoreboard, starting empty");
                Vec::new()
            }),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Vec::new(),
            Err(error) => {
                tracing::warn!(path = %path.display(), %error, "could not read the scoreboard");
                Vec::new()
            }
        };
        Self {
            entries: RwLock::new(entries),
            keep,
            path: Some(path),
        }
    }

    /// A board that forgets when the process does. Used by tests.
    #[must_use]
    pub fn in_memory(keep: usize) -> Self {
        Self {
            entries: RwLock::new(Vec::new()),
            keep,
            path: None,
        }
    }

    /// Record a breach, keeping the list sorted by how fast it was and no
    /// longer than `keep`.
    pub async fn record(&self, actor: &str, seconds: f64, faults: u32) {
        let entry = Entry {
            actor: actor.to_owned(),
            seconds,
            faults,
            at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(0)),
        };
        let snapshot = {
            let mut entries = self.entries.write().await;
            entries.push(entry);
            entries.sort_by(|a, b| a.seconds.total_cmp(&b.seconds));
            entries.truncate(self.keep);
            entries.clone()
        };
        self.persist(&snapshot).await;
    }

    /// The board, fastest first.
    pub async fn list(&self) -> Vec<WireScore> {
        self.entries
            .read()
            .await
            .iter()
            .map(|e| WireScore {
                actor: e.actor.clone(),
                seconds_to_breach: e.seconds,
                faults_used: e.faults,
                at: Some(prost_types::Timestamp {
                    seconds: e.at,
                    nanos: 0,
                }),
            })
            .collect()
    }

    /// Write the whole list. A failure is logged and dropped: the board in
    /// memory is still right, and the next breach tries again.
    async fn persist(&self, entries: &[Entry]) {
        let Some(path) = &self.path else { return };
        let Ok(json) = serde_json::to_vec_pretty(entries) else {
            return;
        };
        if let Some(parent) = path.parent()
            && let Err(error) = tokio::fs::create_dir_all(parent).await
        {
            tracing::warn!(path = %parent.display(), %error, "could not make room for the scoreboard");
            return;
        }
        if let Err(error) = tokio::fs::write(path, json).await {
            tracing::warn!(path = %path.display(), %error, "could not write the scoreboard");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn the_fastest_breach_comes_first_and_the_list_stays_short() {
        let scores = Scores::in_memory(3);
        scores.record("slow", 90.0, 1).await;
        scores.record("quick", 4.5, 3).await;
        scores.record("middling", 30.0, 2).await;
        scores.record("slowest", 120.0, 1).await;

        let list = scores.list().await;
        assert_eq!(list.len(), 3, "only the top three are kept");
        assert_eq!(list[0].actor, "quick");
        assert_eq!(list[2].actor, "slow");
        assert!(list.iter().all(|s| s.at.is_some()), "each one is dated");
    }

    #[tokio::test]
    async fn a_board_survives_a_restart() {
        let dir = tempfile::tempdir().expect("a temp dir");
        let path = dir.path().join("nested/scores.json");

        let first = Scores::open(path.clone(), 10).await;
        first.record("nevio", 12.25, 2).await;

        let second = Scores::open(path, 10).await;
        let list = second.list().await;
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].actor, "nevio");
        assert!((list[0].seconds_to_breach - 12.25).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn rubbish_on_disk_starts_an_empty_board_rather_than_failing() {
        let dir = tempfile::tempdir().expect("a temp dir");
        let path = dir.path().join("scores.json");
        tokio::fs::write(&path, b"not json at all")
            .await
            .expect("write");

        let scores = Scores::open(path, 10).await;
        assert!(scores.list().await.is_empty());
    }
}
