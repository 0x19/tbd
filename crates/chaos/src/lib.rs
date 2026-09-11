//! Validation, load and fault-injection framework.
//!
//! Two extension points:
//!
//! - [`service::Service`]: how to start one instance of a service in-process.
//!   This project's services are the *kinds* in [`kinds`]: one module each,
//!   registered in [`kinds::ALL`], from which topology tables, validate
//!   targets and checks, runtime add and the admin UI's forms all derive.
//! - [`load::ops::Operation`]: what one unit of load looks like against a
//!   target. This project supplies REST, GraphQL, WebSocket and gRPC ping.
//!
//! Everything else is generic: the [`stack`] runner, validation, load
//! generation, timelines, assertions and reports. [`stress`] runs the
//! `tbd-stress` campaigns around the same stack and timeline.

pub mod api;
pub mod auth;
pub mod config;
pub mod kinds;
pub mod load;
pub mod scenario;
pub mod service;
pub mod stack;
pub mod stress;
pub mod tls;
pub mod topology;
pub mod validate;
