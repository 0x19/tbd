//! The sandbox engine (RFC 0010, `docs/sandbox/README.md`): compile and run one
//! Go or Rust program in one throwaway gVisor container, and say what it did.
//!
//! A run is a fixed recipe, built here and nowhere else: a container started
//! with every restriction ([`recipe::container_args`]), the source written into
//! its scratch space, the compiler run with a deadline, the program run with a
//! deadline and its input, and the container removed whatever happened. A
//! request carries three things (the language, the source, the input) and none
//! of them ever becomes an option of the container or a word of a command: the
//! language picks a row of a fixed table, the source and the input only ever
//! travel on a process's standard input.
//!
//! Every limit is enforced from outside the sandbox: the deadline by removing
//! the container, the output caps by the reader here, which stops reading and
//! removes the container once a stream passes its cap.

pub mod config;
pub mod recipe;
pub mod run;
pub mod server;

pub use config::Config;
pub use run::{Killed, Outcome, RunRequest, RunResponse, Step};
