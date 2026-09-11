//! Runs a set of service instances in this process.
//!
//! Generic over [`Service`]: the stack knows names, dependencies, addresses and
//! lifecycle, nothing about what the services are.

use std::{
    collections::{BTreeMap, BTreeSet},
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
    time::Duration,
};

use serde::{Deserialize, Serialize};
use tbd_common::fault::Behavior;

use crate::service::{Instance, Peers, RequestCounts, Service};

/// One launcher, running or not, as the API and the UI see it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceInfo {
    /// Name from the topology.
    pub name: String,
    /// Service kind.
    pub kind: String,
    /// Bound (or remembered) address.
    pub addr: SocketAddr,
    /// Currently running.
    pub running: bool,
    /// Names it needs running first.
    pub depends_on: Vec<String>,
    /// Current fault behaviour, when the service has fault injection.
    pub behavior: Option<Behavior>,
    /// Current store fault behaviour, when the service has a store to fail.
    #[serde(default)]
    pub store_behavior: Option<Behavior>,
    /// Request counters, when the service exposes them.
    pub requests: Option<RequestCounts>,
    /// Added at runtime (a replica or a new instance) rather than from the
    /// topology; only these can be removed.
    #[serde(default)]
    pub added: bool,
}

/// One entry in a topology: a service and where to bind it.
pub struct Launcher {
    /// How to start it.
    pub service: Arc<dyn Service>,
    /// Address to bind. Port 0 on first start means any free port; the port
    /// chosen is remembered so a restart lands on the same address.
    pub listen: SocketAddr,
}

/// Errors from running a stack.
#[derive(Debug, thiserror::Error)]
pub enum StackError {
    /// A dependency cycle or a reference to an unknown instance.
    #[error("cannot order start-up: unresolved dependencies for {0:?}")]
    Unresolvable(Vec<String>),
    /// A service failed to start.
    #[error("start {name}: {source}")]
    Start {
        /// Instance name.
        name: String,
        /// Cause.
        source: anyhow::Error,
    },
    /// A service started but never became ready.
    #[error("{name} at {addr} not ready after {timeout:?}")]
    NotReady {
        /// Instance name.
        name: String,
        /// Bound address.
        addr: SocketAddr,
        /// How long we waited.
        timeout: Duration,
    },
    /// Unknown instance name.
    #[error("no instance named {0:?}")]
    Unknown(String),
    /// Instance is already running.
    #[error("{0:?} is already running")]
    AlreadyRunning(String),
    /// Instance has no fault injection.
    #[error("{0:?} has no fault injection")]
    NoFaults(String),
    /// The instance's service has no store to fail.
    #[error("{0:?} has no store fault injection")]
    NoStoreFaults(String),
    /// An instance with that name already exists.
    #[error("{0:?} already exists")]
    Exists(String),
    /// A dependency is not running.
    #[error("{name:?} needs {dep:?} running first")]
    DependencyDown {
        /// Instance being started.
        name: String,
        /// The dependency.
        dep: String,
    },
    /// Others depend on it.
    #[error("{name:?} is needed by {by:?}; remove those first")]
    InUse {
        /// Instance being removed.
        name: String,
        /// Who depends on it.
        by: Vec<String>,
    },
    /// Only instances added at runtime can be removed.
    #[error("{0:?} comes from the topology; stop it instead of removing it")]
    FromTopology(String),
}

/// A running (or partially running) stack.
pub struct Stack {
    launchers: BTreeMap<String, Launcher>,
    instances: BTreeMap<String, Instance>,
    /// Names added after start (replicas, new instances).
    added: BTreeSet<String>,
    ready_timeout: Duration,
}

impl std::fmt::Debug for Stack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Stack")
            .field("instances", &self.instances)
            .finish_non_exhaustive()
    }
}

/// Any free port on loopback.
pub fn ephemeral() -> SocketAddr {
    SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0)
}

impl Stack {
    /// Start every launcher in dependency order and wait for readiness.
    pub async fn start(launchers: BTreeMap<String, Launcher>) -> Result<Self, StackError> {
        let mut stack = Self {
            launchers,
            instances: BTreeMap::new(),
            added: BTreeSet::new(),
            ready_timeout: Duration::from_secs(10),
        };
        let mut pending: Vec<String> = stack.launchers.keys().cloned().collect();
        while !pending.is_empty() {
            let startable: Vec<String> = pending
                .iter()
                .filter(|name| {
                    stack.launchers[*name]
                        .service
                        .depends_on()
                        .iter()
                        .all(|dep| stack.instances.contains_key(dep))
                })
                .cloned()
                .collect();
            if startable.is_empty() {
                return Err(StackError::Unresolvable(pending));
            }
            for name in &startable {
                stack.start_instance(name).await?;
            }
            pending.retain(|n| !startable.contains(n));
        }
        Ok(stack)
    }

    /// Start (or restart) one instance by name on its remembered address.
    pub async fn start_instance(&mut self, name: &str) -> Result<&Instance, StackError> {
        if self.instances.contains_key(name) {
            return Err(StackError::AlreadyRunning(name.to_owned()));
        }
        let launcher = self
            .launchers
            .get(name)
            .ok_or_else(|| StackError::Unknown(name.to_owned()))?;
        let service = Arc::clone(&launcher.service);
        let listen = launcher.listen;
        let instance = service
            .start(name, listen, &Peers(&self.instances))
            .await
            .map_err(|source| StackError::Start {
                name: name.to_owned(),
                source,
            })?;
        if !instance.wait_ready(self.ready_timeout).await {
            let addr = instance.addr;
            instance.stop().await;
            return Err(StackError::NotReady {
                name: name.to_owned(),
                addr,
                timeout: self.ready_timeout,
            });
        }
        tracing::info!(instance = name, kind = instance.kind, addr = %instance.addr, "ready");
        if let Some(l) = self.launchers.get_mut(name) {
            l.listen = instance.addr;
        }
        self.instances.insert(name.to_owned(), instance);
        Ok(&self.instances[name])
    }

    /// Stop one instance by name. Its address stays reserved for a restart.
    pub async fn stop_instance(&mut self, name: &str) -> Result<(), StackError> {
        let instance = self
            .instances
            .remove(name)
            .ok_or_else(|| StackError::Unknown(name.to_owned()))?;
        tracing::info!(instance = name, "stopping");
        instance.stop().await;
        Ok(())
    }

    /// Add a launcher at runtime and start it. The name must be new and every
    /// dependency running. Added instances are marked `added` and can be
    /// removed again.
    pub async fn add_instance(
        &mut self,
        name: &str,
        launcher: Launcher,
    ) -> Result<&Instance, StackError> {
        if self.launchers.contains_key(name) {
            return Err(StackError::Exists(name.to_owned()));
        }
        if let Some(dep) = launcher
            .service
            .depends_on()
            .into_iter()
            .find(|d| !self.instances.contains_key(d))
        {
            return Err(StackError::DependencyDown {
                name: name.to_owned(),
                dep,
            });
        }
        self.launchers.insert(name.to_owned(), launcher);
        self.added.insert(name.to_owned());
        match self.start_instance(name).await {
            Ok(_) => Ok(&self.instances[name]),
            Err(e) => {
                self.launchers.remove(name);
                self.added.remove(name);
                Err(e)
            }
        }
    }

    /// The next free replica name for `name`: `<base>-<n>` for the smallest
    /// free `n` from 2, where `<base>` is `name` without a trailing `-<digits>`
    /// (`engine-1` gives `engine-2`, `spare` gives `spare-2`).
    pub fn next_name(&self, name: &str) -> String {
        let base = name
            .rsplit_once('-')
            .filter(|(_, n)| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))
            .map_or(name, |(b, _)| b);
        (2..=10_000)
            .map(|n| format!("{base}-{n}"))
            .find(|candidate| !self.launchers.contains_key(candidate))
            .unwrap_or_else(|| format!("{base}-2"))
    }

    /// Whether a launcher with this name exists (running or not).
    pub fn has(&self, name: &str) -> bool {
        self.launchers.contains_key(name)
    }

    /// Stop and forget an instance added at runtime. Topology instances and
    /// instances others depend on are refused.
    pub async fn remove_instance(&mut self, name: &str) -> Result<(), StackError> {
        if !self.launchers.contains_key(name) {
            return Err(StackError::Unknown(name.to_owned()));
        }
        if !self.added.contains(name) {
            return Err(StackError::FromTopology(name.to_owned()));
        }
        let by: Vec<String> = self
            .launchers
            .iter()
            .filter(|(other, l)| {
                other.as_str() != name && l.service.depends_on().iter().any(|d| d == name)
            })
            .map(|(other, _)| other.clone())
            .collect();
        if !by.is_empty() {
            return Err(StackError::InUse {
                name: name.to_owned(),
                by,
            });
        }
        if self.instances.contains_key(name) {
            self.stop_instance(name).await?;
        }
        self.launchers.remove(name);
        self.added.remove(name);
        tracing::info!(instance = name, "removed");
        Ok(())
    }

    /// A running instance.
    pub fn get(&self, name: &str) -> Option<&Instance> {
        self.instances.get(name)
    }

    /// Every running instance, by name.
    pub fn instances(&self) -> impl Iterator<Item = (&str, &Instance)> {
        self.instances.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// Running instances of one kind.
    pub fn of_kind(&self, kind: &str) -> Vec<&Instance> {
        self.instances.values().filter(|i| i.kind == kind).collect()
    }

    /// Every launcher with its live state, sorted by name.
    pub fn describe(&self) -> Vec<InstanceInfo> {
        self.launchers
            .iter()
            .map(|(name, l)| {
                let instance = self.instances.get(name);
                InstanceInfo {
                    name: name.clone(),
                    kind: l.service.kind().to_owned(),
                    addr: instance.map_or(l.listen, |i| i.addr),
                    running: instance.is_some(),
                    depends_on: l.service.depends_on(),
                    behavior: instance.and_then(|i| i.fault().map(|f| f.get())),
                    store_behavior: instance.and_then(|i| i.store_fault().map(|f| f.get())),
                    requests: instance.and_then(Instance::requests),
                    added: self.added.contains(name),
                }
            })
            .collect()
    }

    /// Set the fault behaviour of a running instance.
    pub fn set_behavior(&self, name: &str, behavior: Behavior) -> Result<(), StackError> {
        let instance = self
            .instances
            .get(name)
            .ok_or_else(|| StackError::Unknown(name.to_owned()))?;
        let fault = instance
            .fault()
            .ok_or_else(|| StackError::NoFaults(name.to_owned()))?;
        fault.set(behavior);
        Ok(())
    }

    /// Set the store fault behaviour of a running instance: reads fail before
    /// they run, writes after they committed.
    pub fn set_store_behavior(&self, name: &str, behavior: Behavior) -> Result<(), StackError> {
        let instance = self
            .instances
            .get(name)
            .ok_or_else(|| StackError::Unknown(name.to_owned()))?;
        let fault = instance
            .store_fault()
            .ok_or_else(|| StackError::NoStoreFaults(name.to_owned()))?;
        fault.set(behavior);
        Ok(())
    }

    /// Request counters of every instance that exposes them.
    pub fn request_counts(&self) -> BTreeMap<String, RequestCounts> {
        self.instances
            .iter()
            .filter_map(|(n, i)| i.requests().map(|c| (n.clone(), c)))
            .collect()
    }

    /// Stop everything, dependents first.
    pub async fn shutdown(mut self) {
        while !self.instances.is_empty() {
            let running: Vec<String> = self.instances.keys().cloned().collect();
            // A leaf is an instance no other running instance depends on.
            let leaves: Vec<String> = running
                .iter()
                .filter(|name| {
                    !running.iter().any(|other| {
                        self.launchers
                            .get(other)
                            .is_some_and(|l| l.service.depends_on().contains(name))
                    })
                })
                .cloned()
                .collect();
            let batch = if leaves.is_empty() { running } else { leaves };
            for name in batch {
                let _ = self.stop_instance(&name).await;
            }
        }
    }
}
