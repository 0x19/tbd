//! Findings: one JSON file per finding under `[paths] findings`, an index in
//! memory, grouped by signature across runs. Mirrors [`super::runs::RunStore`].

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use tbd_stress::{Finding, FindingSummary};
use tokio::sync::Mutex;

/// What `GET /findings` filters on.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct FindingFilter {
    /// Findings of one run.
    pub run: Option<String>,
    /// Findings of one invariant.
    pub invariant: Option<String>,
    /// Findings of one campaign (by name).
    pub campaign: Option<String>,
}

impl FindingFilter {
    fn matches(&self, f: &Finding) -> bool {
        self.run
            .as_ref()
            .is_none_or(|r| f.run_id.as_ref() == Some(r))
            && self.invariant.as_ref().is_none_or(|i| f.invariant == *i)
            && self.campaign.as_ref().is_none_or(|c| f.campaign == *c)
    }
}

/// Findings that share a signature: the same rule broken the same way.
#[derive(Debug, Clone, Serialize)]
pub struct FindingGroup {
    /// The invariant.
    pub invariant: String,
    /// The signature.
    pub signature: String,
    /// How many findings carry it.
    pub count: usize,
    /// First seen.
    pub first: String,
    /// Last seen.
    pub last: String,
    /// Runs it appeared in.
    pub runs: Vec<String>,
    /// Campaigns it appeared in.
    pub campaigns: Vec<String>,
    /// The newest finding, as the list shows it.
    pub sample: FindingSummary,
}

/// The findings on disk and in memory.
pub struct FindingStore {
    dir: PathBuf,
    findings: Mutex<BTreeMap<String, Finding>>,
}

impl FindingStore {
    /// Open `dir`, creating it, and index every `*.json` in it.
    ///
    /// # Errors
    /// The directory cannot be created or read.
    pub fn open(dir: &Path) -> std::io::Result<Self> {
        std::fs::create_dir_all(dir)?;
        let mut findings = BTreeMap::new();
        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            if path.extension().is_none_or(|e| e != "json") {
                continue;
            }
            match std::fs::read_to_string(&path)
                .map_err(|e| e.to_string())
                .and_then(|t| serde_json::from_str::<Finding>(&t).map_err(|e| e.to_string()))
            {
                Ok(f) => {
                    findings.insert(f.id.clone(), f);
                }
                Err(error) => {
                    tracing::warn!(path = %path.display(), %error, "unreadable finding; skipped");
                }
            }
        }
        tracing::info!(dir = %dir.display(), findings = findings.len(), "findings loaded");
        Ok(Self {
            dir: dir.to_path_buf(),
            findings: Mutex::new(findings),
        })
    }

    /// Write a finding (new or updated) and index it.
    ///
    /// # Errors
    /// The file cannot be written.
    pub async fn put(&self, finding: &Finding) -> std::io::Result<()> {
        let path = self.dir.join(format!("{}.json", finding.id));
        let tmp = self.dir.join(format!("{}.json.tmp", finding.id));
        let text = serde_json::to_string_pretty(finding).map_err(std::io::Error::other)?;
        tokio::fs::write(&tmp, text).await?;
        tokio::fs::rename(&tmp, &path).await?;
        self.findings
            .lock()
            .await
            .insert(finding.id.clone(), finding.clone());
        Ok(())
    }

    /// One finding.
    pub async fn get(&self, id: &str) -> Option<Finding> {
        self.findings.lock().await.get(id).cloned()
    }

    /// Remove one; `Ok(false)` when there was none.
    ///
    /// # Errors
    /// The file cannot be removed.
    pub async fn delete(&self, id: &str) -> std::io::Result<bool> {
        if self.findings.lock().await.remove(id).is_none() {
            return Ok(false);
        }
        match tokio::fs::remove_file(self.dir.join(format!("{id}.json"))).await {
            Ok(()) => Ok(true),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(true),
            Err(e) => Err(e),
        }
    }

    /// Newest first, filtered, at most `limit`.
    pub async fn list(&self, limit: usize, filter: &FindingFilter) -> Vec<FindingSummary> {
        self.findings
            .lock()
            .await
            .values()
            .rev()
            .filter(|f| filter.matches(f))
            .take(limit)
            .map(Finding::summary)
            .collect()
    }

    /// Every finding grouped by signature, newest group first.
    pub async fn grouped(&self, filter: &FindingFilter) -> Vec<FindingGroup> {
        let findings = self.findings.lock().await;
        let mut groups: BTreeMap<String, FindingGroup> = BTreeMap::new();
        // Oldest first, so `first` and `sample` (the newest) fall out of the order.
        for f in findings.values().filter(|f| filter.matches(f)) {
            let g = groups
                .entry(f.signature.clone())
                .or_insert_with(|| FindingGroup {
                    invariant: f.invariant.clone(),
                    signature: f.signature.clone(),
                    count: 0,
                    first: f.found_at.clone(),
                    last: f.found_at.clone(),
                    runs: Vec::new(),
                    campaigns: Vec::new(),
                    sample: f.summary(),
                });
            g.count += 1;
            g.last.clone_from(&f.found_at);
            g.sample = f.summary();
            if let Some(run) = &f.run_id
                && !g.runs.contains(run)
            {
                g.runs.push(run.clone());
            }
            if !g.campaigns.contains(&f.campaign) {
                g.campaigns.push(f.campaign.clone());
            }
        }
        let mut out: Vec<FindingGroup> = groups.into_values().collect();
        out.sort_by(|a, b| b.last.cmp(&a.last));
        out
    }

    /// How many findings, and how many distinct signatures.
    pub async fn counts(&self) -> (usize, usize) {
        let findings = self.findings.lock().await;
        let signatures: std::collections::BTreeSet<&str> =
            findings.values().map(|f| f.signature.as_str()).collect();
        (findings.len(), signatures.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tbd_stress::{WorkerClass, trace::Violation};

    fn finding(invariant: &'static str, message: &str, run: &str) -> Finding {
        let v = Violation {
            invariant,
            message: message.into(),
            expected: serde_json::json!("x"),
            actual: serde_json::json!("y"),
        };
        let mut f = Finding::new(
            &v,
            vec![],
            uuid::Uuid::now_v7(),
            WorkerClass::Owner,
            "smoke",
            "ledger-1",
            Some("memory"),
        );
        f.run_id = Some(run.into());
        f
    }

    #[tokio::test]
    async fn findings_round_trip_and_group_by_signature() {
        let dir = std::env::temp_dir().join(format!("chaos-findings-{}", uuid::Uuid::now_v7()));
        let store = FindingStore::open(&dir).unwrap();
        store
            .put(&finding("history_cut", "row 12 missing", "r1"))
            .await
            .unwrap();
        store
            .put(&finding("history_cut", "row 99 missing", "r2"))
            .await
            .unwrap();
        store
            .put(&finding("idempotency", "replayed differently", "r2"))
            .await
            .unwrap();
        assert_eq!(store.counts().await, (3, 2));
        let all = store.list(10, &FindingFilter::default()).await;
        assert_eq!(all.len(), 3);
        let by_run = store
            .list(
                10,
                &FindingFilter {
                    run: Some("r2".into()),
                    ..FindingFilter::default()
                },
            )
            .await;
        assert_eq!(by_run.len(), 2);
        let groups = store.grouped(&FindingFilter::default()).await;
        assert_eq!(groups.len(), 2);
        let cut = groups
            .iter()
            .find(|g| g.invariant == "history_cut")
            .unwrap();
        assert_eq!(cut.count, 2);
        assert_eq!(cut.runs, vec!["r1".to_owned(), "r2".to_owned()]);
        // A second open reads the files back.
        let again = FindingStore::open(&dir).unwrap();
        assert_eq!(again.counts().await, (3, 2));
        let id = all[0].id.clone();
        assert!(again.delete(&id).await.unwrap());
        assert!(!again.delete(&id).await.unwrap());
        assert_eq!(again.counts().await.0, 2);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
