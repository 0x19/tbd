//! Normalise generated Rust through `rustfmt`, when it is on the path, so what
//! the scaffold writes is what `cargo fmt --check` expects, and a re-run
//! recognises the file it wrote.

use std::{
    io::Write,
    process::{Command, Stdio},
};

/// `src` formatted by `rustfmt --edition 2024`, or `None` when rustfmt is
/// unavailable or rejects the input.
#[must_use]
pub fn rustfmt(src: &str) -> Option<String> {
    let mut child = Command::new("rustfmt")
        .args(["--edition", "2024", "--emit", "stdout", "--quiet"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    child.stdin.take()?.write_all(src.as_bytes()).ok()?;
    let out = child.wait_with_output().ok()?;
    if !out.status.success() {
        return None;
    }
    String::from_utf8(out.stdout).ok()
}
