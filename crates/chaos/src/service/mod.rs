//! The service extension point.
//!
//! A [`Service`] knows how to start one instance of something on an address
//! and hand back an [`Instance`] the stack can observe, perturb and stop.

use std::{collections::BTreeMap, net::SocketAddr, time::Duration};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tbd_common::fault::FaultHandle;

/// Something that can be started in-process.
#[async_trait]
pub trait Service: Send + Sync {
    /// Short kind name, e.g. `engine`.
    fn kind(&self) -> &'static str;

    /// Whether a stopped and restarted instance still holds what it held: a
    /// service on a real database says yes, one on an in-memory store says no.
    /// A campaign refuses to stop an instance that forgets.
    fn durable(&self) -> bool {
        true
    }

    /// Names of instances that must be running before this one starts.
    fn depends_on(&self) -> Vec<String> {
        Vec::new()
    }

    /// Start one instance bound to `listen`. Port 0 means any free port.
    async fn start(
        &self,
        name: &str,
        listen: SocketAddr,
        peers: &Peers<'_>,
    ) -> anyhow::Result<Instance>;
}

/// Already-running instances a service may need to find, by name.
pub struct Peers<'a>(pub(crate) &'a BTreeMap<String, Instance>);

impl Peers<'_> {
    /// Address of a running instance.
    pub fn addr(&self, name: &str) -> Option<SocketAddr> {
        self.0.get(name).map(|i| i.addr)
    }
}

/// Request counters an instance may expose.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestCounts {
    /// Requests received.
    pub total: u64,
    /// Requests that failed.
    pub failed: u64,
}

/// Control surface of a running instance. Implemented per service.
#[async_trait]
pub trait InstanceHandle: Send + Sync {
    /// True when the instance answers its own readiness check.
    async fn ready(&self) -> bool;

    /// Stop the instance and wait for it to exit.
    async fn stop(self: Box<Self>);

    /// Fault injection, when the service supports it.
    fn fault(&self) -> Option<FaultHandle> {
        None
    }

    /// Fault injection at the service's store, when it has one: reads fail
    /// before they run, writes after they committed (`set_store_behavior`).
    fn store_fault(&self) -> Option<FaultHandle> {
        None
    }

    /// Request counters, when the service exposes them.
    fn requests(&self) -> Option<RequestCounts> {
        None
    }
}

/// A running instance.
pub struct Instance {
    /// Name from the topology.
    pub name: String,
    /// Service kind.
    pub kind: &'static str,
    /// Bound address.
    pub addr: SocketAddr,
    handle: Box<dyn InstanceHandle>,
}

impl std::fmt::Debug for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Instance")
            .field("name", &self.name)
            .field("kind", &self.kind)
            .field("addr", &self.addr)
            .finish_non_exhaustive()
    }
}

impl Instance {
    /// Wrap a handle.
    pub fn new(
        name: &str,
        kind: &'static str,
        addr: SocketAddr,
        handle: impl InstanceHandle + 'static,
    ) -> Self {
        Self {
            name: name.to_owned(),
            kind,
            addr,
            handle: Box::new(handle),
        }
    }

    /// `http://host:port`.
    pub fn http_url(&self) -> String {
        format!("http://{}", self.addr)
    }

    /// Poll readiness until it succeeds or `timeout` passes.
    pub async fn wait_ready(&self, timeout: Duration) -> bool {
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            if self.handle.ready().await {
                return true;
            }
            if tokio::time::Instant::now() >= deadline {
                return false;
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    }

    /// See [`InstanceHandle::fault`].
    pub fn fault(&self) -> Option<FaultHandle> {
        self.handle.fault()
    }

    /// Store fault injection, when the service has a store.
    pub fn store_fault(&self) -> Option<FaultHandle> {
        self.handle.store_fault()
    }

    /// See [`InstanceHandle::requests`].
    pub fn requests(&self) -> Option<RequestCounts> {
        self.handle.requests()
    }

    /// Stop and wait.
    pub async fn stop(self) {
        self.handle.stop().await;
    }
}

/// Wait until `addr` accepts TCP connections or `timeout` passes.
pub async fn wait_for_port(addr: SocketAddr, timeout: Duration) -> bool {
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        if tokio::net::TcpStream::connect(addr).await.is_ok() {
            return true;
        }
        if tokio::time::Instant::now() >= deadline {
            return false;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

/// Shared shape for services that run a tokio task and stop through a oneshot.
pub(crate) struct TaskHandle {
    stop: Option<tokio::sync::oneshot::Sender<()>>,
    task: Option<tokio::task::JoinHandle<()>>,
}

impl TaskHandle {
    pub(crate) fn new(
        stop: tokio::sync::oneshot::Sender<()>,
        task: tokio::task::JoinHandle<()>,
    ) -> Self {
        Self {
            stop: Some(stop),
            task: Some(task),
        }
    }

    pub(crate) async fn stop(mut self) {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        if let Some(task) = self.task.take()
            && tokio::time::timeout(Duration::from_secs(5), task)
                .await
                .is_err()
        {
            tracing::warn!("instance did not stop within 5s");
        }
    }
}
