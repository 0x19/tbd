//! The bench: how a service is started, held, driven and reached.
//!
//! Everything here is about services in general and knows nothing about which
//! ones this repository has. That is the point — it depends on no service
//! crate, so any crate that drives services can depend on it without the cycle
//! that a dependency on [`tbd-chaos`](../tbd_chaos/index.html) would create.
//! `tbd-chaos` builds its kinds, topologies, scenarios and HTTP API on top of
//! it; `tbd-playground` builds a public sandbox on the same pieces.
//!
//! - [`service::Service`]: how to start one instance of a service in-process.
//! - [`stack`]: several of them at once, with dependencies, faults and counters.
//! - [`load`]: an open-loop generator and the operations it runs.
//! - [`enginelb`]: a balancer in front of several replicas, where a deployed
//!   environment has Envoy.
//! - [`tls`] and [`auth`]: reaching a deployed stack over TLS with a token.

pub mod auth;
pub mod check;
pub mod enginelb;
pub mod kind;
pub mod kinds;
pub mod load;
pub mod service;
pub mod stack;
pub mod tls;
