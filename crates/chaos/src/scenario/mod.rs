//! Scenarios: start a stack, apply load while a timeline perturbs it, assert.
//!
//! One executor runs every scenario; TOML is the only definition format for
//! now. The structure leaves room for Rust-defined scenarios later: anything
//! that yields a [`config::ScenarioFile`] can be executed.

pub mod assertions;
pub mod config;
pub mod executor;
pub mod report;
pub mod timeline;

pub use config::ScenarioFile;
pub use executor::{
    Hooks, RunEvent, ScenarioResult, run_file, run_file_with, run_scenario, run_scenario_with,
};
