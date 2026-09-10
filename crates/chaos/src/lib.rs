//! Validation, load and fault-injection framework.
//!
//! Two extension points:
//!
//! - [`service::Service`]: how to start one instance of a service in-process.
//!   This project supplies the engine and the protocol service.
//! - [`load::ops::Operation`]: what one unit of load looks like against a
//!   target. This project supplies REST, GraphQL, WebSocket and gRPC ping.
//!
//! Everything else is generic: the [`stack`] runner, validation, load
//! generation, timelines, assertions and reports.

pub mod load;
pub mod scenario;
pub mod service;
pub mod stack;
pub mod tls;
pub mod topology;
pub mod validate;
