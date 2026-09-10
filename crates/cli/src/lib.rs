//! The `tbd` command: scaffolds a service from embedded templates and registers
//! it in every shared file of the repository, idempotently.
//!
//! Nothing here runs at service runtime. The crate is a set of pure text
//! operations ([`edit`]) over an in-memory view of the workspace ([`repo`]),
//! driven by a data table of registrations ([`registry`]). `main.rs` is the only
//! module that prints.

pub mod check;
pub mod edit;
pub mod fmt;
pub mod list;
pub mod ports;
pub mod registry;
pub mod repo;
pub mod scaffold;
pub mod service;
pub mod template;
pub mod templates;

/// Version of the CLI, written into the marker of every service it generates.
pub const VERSION: &str = tbd_common::VERSION;
