//! Instances added to the serve stack at runtime (replicas, new instances of
//! any kind), kept in one JSON file so they come back after a restart.
//!
//! The stack itself is generic and forgets everything on exit; this is the
//! record of what to re-add, in the same terms as the topology's
//! `[stack.<plural>.<name>]` tables: `kind` plus the kind's own keys.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::{kinds, stack::Launcher, topology::StackConfig};

/// What to start: the kind and its table (minus `listen`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddedSpec {
    /// The kind name.
    pub kind: String,
    /// The kind's own keys, as in `[stack.<plural>.<name>]`.
    #[serde(flatten)]
    pub spec: toml::Table,
}

impl AddedSpec {
    /// The spec of a topology instance, so a replica of it can be recorded.
    #[must_use]
    pub fn of_topology(config: &StackConfig, name: &str) -> Option<Self> {
        config.get(name).map(|i| Self {
            kind: i.kind.name.to_owned(),
            spec: i.spec.clone(),
        })
    }

    /// The registered kind.
    #[must_use]
    pub fn kind(&self) -> Option<&'static kinds::Kind> {
        kinds::by_name(&self.kind)
    }

    /// A launcher for the generic stack on any free port.
    ///
    /// # Errors
    /// The kind is not registered, or it rejects the table.
    pub fn launcher(&self) -> Result<Launcher, String> {
        let kind = self.kind().ok_or_else(|| {
            format!(
                "kind {:?} is not registered; kinds: {}",
                self.kind,
                kinds::names()
            )
        })?;
        kind.launcher(self.spec.clone(), None)
            .map_err(|e| format!("{}: {e}", self.kind))
    }
}

/// One added instance as stored.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddedInstance {
    /// Instance name.
    pub name: String,
    /// What it is.
    #[serde(flatten)]
    pub spec: AddedSpec,
    /// The instance it was cloned from, when it is a replica.
    #[serde(default)]
    pub replica_of: Option<String>,
    /// RFC 3339.
    pub added_at: String,
}

/// The file of added instances.
pub struct AddedStore {
    path: PathBuf,
    items: Mutex<BTreeMap<String, AddedInstance>>,
}

impl AddedStore {
    /// Load `path` when it exists.
    ///
    /// # Errors
    /// The file exists but cannot be read or parsed, or its directory cannot
    /// be created.
    pub fn open(path: &Path) -> std::io::Result<Self> {
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)?;
        }
        let items = match std::fs::read_to_string(path) {
            Ok(text) => serde_json::from_str::<Vec<AddedInstance>>(&text)
                .map_err(|e| std::io::Error::other(format!("{}: {e}", path.display())))?
                .into_iter()
                .map(|a| (a.name.clone(), a))
                .collect(),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => BTreeMap::new(),
            Err(e) => return Err(e),
        };
        Ok(Self {
            path: path.to_owned(),
            items: Mutex::new(items),
        })
    }

    /// Everything stored, oldest first. Re-adding walks this in dependency
    /// order; see `AppState::new`.
    pub async fn list(&self) -> Vec<AddedInstance> {
        let mut out: Vec<AddedInstance> = self.items.lock().await.values().cloned().collect();
        out.sort_by(|a, b| {
            a.added_at
                .cmp(&b.added_at)
                .then_with(|| a.name.cmp(&b.name))
        });
        out
    }

    /// One stored instance.
    pub async fn get(&self, name: &str) -> Option<AddedInstance> {
        self.items.lock().await.get(name).cloned()
    }

    /// Record an instance and write the file.
    ///
    /// # Errors
    /// The file cannot be written.
    pub async fn insert(&self, item: AddedInstance) -> std::io::Result<()> {
        let mut items = self.items.lock().await;
        items.insert(item.name.clone(), item);
        self.save(&items).await
    }

    /// Forget an instance and write the file.
    ///
    /// # Errors
    /// The file cannot be written.
    pub async fn remove(&self, name: &str) -> std::io::Result<()> {
        let mut items = self.items.lock().await;
        items.remove(name);
        self.save(&items).await
    }

    async fn save(&self, items: &BTreeMap<String, AddedInstance>) -> std::io::Result<()> {
        let list: Vec<&AddedInstance> = items.values().collect();
        let text = serde_json::to_string_pretty(&list)?;
        let tmp = self.path.with_extension("json.tmp");
        tokio::fs::write(&tmp, text).await?;
        tokio::fs::rename(&tmp, &self.path).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_json_uses_the_topology_keys() {
        let e: AddedInstance = serde_json::from_str(
            r#"{"name":"engine-2","kind":"engine","heartbeat":"100ms","added_at":"2026-09-10T00:00:00Z"}"#,
        )
        .unwrap();
        assert_eq!(e.spec.kind, "engine");
        assert_eq!(e.spec.spec["heartbeat"].as_str(), Some("100ms"));
        assert!(e.spec.launcher().is_ok());
        let p: AddedInstance = serde_json::from_str(
            r#"{"name":"protocol-2","kind":"protocol","engine":"engine-1","replica_of":"protocol-1","added_at":"2026-09-10T00:00:00Z"}"#,
        )
        .unwrap();
        assert_eq!(p.spec.spec["engine"].as_str(), Some("engine-1"));
        let json = serde_json::to_value(&p).unwrap();
        assert_eq!(json["kind"], "protocol");
        assert_eq!(json["engine"], "engine-1");
        assert_eq!(json["replica_of"], "protocol-1");
        let bad: AddedInstance = serde_json::from_str(
            r#"{"name":"w","kind":"widget","added_at":"2026-09-10T00:00:00Z"}"#,
        )
        .unwrap();
        assert!(
            bad.spec
                .launcher()
                .err()
                .unwrap()
                .contains("not registered")
        );
        let bogus: AddedInstance = serde_json::from_str(
            r#"{"name":"e","kind":"engine","bogus":1,"added_at":"2026-09-10T00:00:00Z"}"#,
        )
        .unwrap();
        assert!(bogus.spec.launcher().is_err());
    }

    #[tokio::test]
    async fn store_round_trips_oldest_first() {
        let dir = std::env::temp_dir().join(format!("chaos-added-{}", uuid::Uuid::now_v7()));
        let path = dir.join("stack.json");
        let store = AddedStore::open(&path).unwrap();
        let table = |engine: &str| {
            let mut t = toml::Table::new();
            t.insert("engine".into(), toml::Value::String(engine.into()));
            t
        };
        store
            .insert(AddedInstance {
                name: "protocol-2".into(),
                spec: AddedSpec {
                    kind: "protocol".into(),
                    spec: table("engine-2"),
                },
                replica_of: None,
                added_at: "2026-09-10T00:00:02Z".into(),
            })
            .await
            .unwrap();
        store
            .insert(AddedInstance {
                name: "engine-2".into(),
                spec: AddedSpec {
                    kind: "engine".into(),
                    spec: toml::Table::new(),
                },
                replica_of: Some("engine-1".into()),
                added_at: "2026-09-10T00:00:01Z".into(),
            })
            .await
            .unwrap();
        let again = AddedStore::open(&path).unwrap();
        let names: Vec<String> = again.list().await.into_iter().map(|a| a.name).collect();
        assert_eq!(names, ["engine-2", "protocol-2"]);
        again.remove("engine-2").await.unwrap();
        assert!(again.get("engine-2").await.is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
