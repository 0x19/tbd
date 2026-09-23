//! Service kinds as data: which ones exist, and everything derived from that.
//!
//! The *shape* of a kind and the five core kinds live in `tbd-lab`, which
//! depends on no tool and so can be depended on by anything that drives
//! services. This module is where the list is assembled, because the list
//! includes the playground, and the playground depends on the lab — putting
//! its kind in the lab would be a cycle.
//!
//! The topology tables (`[stack.<plural>.<name>]`), launchers, cross-checks,
//! validate targets and checks, runtime add and clone, CLI flags and the admin
//! UI's forms all derive from [`ALL`]. `tbd new service` writes a module and a
//! line.

pub mod cv;
pub mod llm;
pub mod playground;

pub use tbd_lab::kind::{Dependency, Field, FieldInfo, FieldKind, Kind, KindInfo, Target, parse};
pub use tbd_lab::kinds::{engine, finance, humans, ledger, protocol};

use crate::{load, stack::Stack};

/// Every kind, in display order (validate output, `chaos up`, error lists).
pub static ALL: &[&Kind] = &[
    &engine::KIND,
    &protocol::KIND,
    &ledger::KIND,
    &humans::KIND,
    &finance::KIND,
    &playground::KIND,
    &cv::KIND,
    &llm::KIND,
    // tbd:kinds-end (tbd new service inserts above this line; do not edit)
];

/// The kind named `name`.
#[must_use]
pub fn by_name(name: &str) -> Option<&'static Kind> {
    ALL.iter().copied().find(|k| k.name == name)
}

/// The kind whose topology table is `plural`.
#[must_use]
pub fn by_plural(plural: &str) -> Option<&'static Kind> {
    ALL.iter().copied().find(|k| k.plural == plural)
}

/// `engine, ledger, protocol`, for error messages.
#[must_use]
pub fn names() -> String {
    ALL.iter().map(|k| k.name).collect::<Vec<_>>().join(", ")
}

/// `engines, ledgers, protocols`, for error messages.
#[must_use]
pub fn plurals() -> String {
    ALL.iter().map(|k| k.plural).collect::<Vec<_>>().join(", ")
}

/// Every kind that has a validate target.
pub fn with_target() -> impl Iterator<Item = (&'static Kind, &'static Target)> {
    ALL.iter()
        .copied()
        .filter_map(|k| k.target.as_ref().map(|t| (k, t)))
}

/// Running instances of every `load_target` kind, as load targets.
#[must_use]
pub fn load_targets(stack: &Stack) -> Vec<load::Target> {
    tbd_lab::kind::load_targets(stack, ALL)
}

/// Every kind, described.
#[must_use]
pub fn describe() -> Vec<KindInfo> {
    tbd_lab::kind::describe(ALL)
}

/// The registry and the check catalogue as Markdown, for `docs/chaos/kinds.md`.
#[must_use]
pub fn markdown() -> String {
    tbd_lab::kind::markdown(ALL)
}
