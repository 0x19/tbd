//! `tbd-db` against a real Postgres.
//!
//! Access control is the reason this suite exists. A leak is silent — nothing
//! errors, a query just returns one row too many — so the cases assert on
//! absence as much as on presence.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod access;
mod access_api;
mod rls;
mod roles;
mod schema;
mod support;
