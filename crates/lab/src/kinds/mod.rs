//! The kinds every deployment of this project has.
//!
//! One module per service, each declaring how to start it in-process, what its
//! topology table accepts, and the validate checks it ships with. The list is
//! assembled by whoever depends on this crate: `tbd-chaos` adds the playground
//! to these and calls the result [`crate::kind`]'s registry.

pub mod engine;
pub mod finance;
pub mod humans;
pub mod ledger;
pub mod protocol;

use crate::kind::Kind;

/// The kinds defined here, in display order.
pub static CORE: &[&Kind] = &[
    &engine::KIND,
    &protocol::KIND,
    &ledger::KIND,
    &humans::KIND,
    &finance::KIND,
];
