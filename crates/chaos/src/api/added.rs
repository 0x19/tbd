//! Instances added to the serve stack at runtime (replicas, new engines and
//! protocols), kept in one JSON file so they come back after a restart.
//!
//! The stack itself is generic and forgets everything on exit; this is the
//! project-specific record of what to re-add, in the same terms as the
//! topology's `[stack.engines.X]` / `[stack.protocols.X]` tables.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    time::Duration,
};

use serde::{Deserialize, Serialize};
use tbd_common::fault::Behavior;
use tokio::sync::Mutex;

use crate::{
    stack::Launcher,
    topology::{EngineSpec, ProtocolSpec, StackConfig},
};

/// What to start: the topology's per-kind keys.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AddedSpec {
    /// An engine.
    Engine {
        /// Heartbeat interval on streams.
        #[serde(with = "humantime_serde")]
        heartbeat: Duration,
        /// Initial behaviour; a restart comes back healthy unless this says otherwise.
        #[serde(default)]
        behavior: Behavior,
    },
    /// A protocol forwarding to a named engine.
    Protocol {
        /// The engine.
        engine: String,
    },
}

impl AddedSpec {
    /// The spec of a topology instance, so a replica of it can be recorded.
    pub fn of_topology(config: &StackConfig, name: &str) -> Option<Self> {
        if let Some(e) = config.engines.get(name) {
            return Some(Self::Engine {
                heartbeat: e.heartbeat,
                behavior: e.behavior.clone(),
            });
        }
        config.protocols.get(name).map(|p| Self::Protocol {
            engine: p.engine.clone(),
        })
    }

    /// `engine` or `protocol`.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Engine { .. } => "engine",
            Self::Protocol { .. } => "protocol",
        }
    }

    /// A launcher for the generic stack, built the way the topology builds its own.
    pub fn launcher(&self, name: &str) -> Option<Launcher> {
        let mut config = StackConfig::default();
        match self {
            Self::Engine {
                heartbeat,
                behavior,
            } => {
                config.engines.insert(
                    name.to_owned(),
                    EngineSpec {
                        listen: None,
                        behavior: behavior.clone(),
                        heartbeat: *heartbeat,
                    },
                );
            }
            Self::Protocol { engine } => {
                config.protocols.insert(
                    name.to_owned(),
                    ProtocolSpec {
                        listen: None,
                        engine: engine.clone(),
                    },
                );
            }
        }
        config.launchers().into_values().next()
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

    /// Everything stored, engines before protocols, oldest first: the order
    /// to re-add them in.
    pub async fn list(&self) -> Vec<AddedInstance> {
        let mut out: Vec<AddedInstance> = self.items.lock().await.values().cloned().collect();
        out.sort_by(|a, b| {
            (a.spec.kind() != "engine", &a.added_at).cmp(&(b.spec.kind() != "engine", &b.added_at))
        });
        out
    }

    /// One stored instance.
    pub async fn get(&self, name: &str) -> Option<AddedInstance> {
        self.items.lock().await.get(name).cloned()
    }

    /// Record an instance and write the file.
    pub async fn insert(&self, item: AddedInstance) -> std::io::Result<()> {
        let mut items = self.items.lock().await;
        items.insert(item.name.clone(), item);
        self.save(&items).await
    }

    /// Forget an instance and write the file.
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
        assert!(
            matches!(e.spec, AddedSpec::Engine { heartbeat, .. } if heartbeat == Duration::from_millis(100))
        );
        let p: AddedInstance = serde_json::from_str(
            r#"{"name":"protocol-2","kind":"protocol","engine":"engine-1","replica_of":"protocol-1","added_at":"2026-09-10T00:00:00Z"}"#,
        )
        .unwrap();
        assert!(matches!(&p.spec, AddedSpec::Protocol { engine } if engine == "engine-1"));
        assert_eq!(serde_json::to_value(&p).unwrap()["kind"], "protocol");
    }

    #[tokio::test]
    async fn store_round_trips_and_orders_engines_first() {
        let dir = std::env::temp_dir().join(format!("chaos-added-{}", uuid::Uuid::now_v7()));
        let path = dir.join("stack.json");
        let store = AddedStore::open(&path).unwrap();
        store
            .insert(AddedInstance {
                name: "protocol-2".into(),
                spec: AddedSpec::Protocol {
                    engine: "engine-2".into(),
                },
                replica_of: None,
                added_at: "2026-09-10T00:00:01Z".into(),
            })
            .await
            .unwrap();
        store
            .insert(AddedInstance {
                name: "engine-2".into(),
                spec: AddedSpec::Engine {
                    heartbeat: Duration::from_secs(1),
                    behavior: Behavior::Healthy,
                },
                replica_of: Some("engine-1".into()),
                added_at: "2026-09-10T00:00:02Z".into(),
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
