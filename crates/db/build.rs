//! Rebuild when a migration is added or edited.
//!
//! `sqlx::migrate!` embeds `/migrations` at compile time but does not, on its
//! own, make cargo watch the directory. Without this, adding a migration
//! leaves a stale binary that applies the old set and fails in a way that looks
//! like the SQL is wrong rather than like the build is old.

fn main() {
    println!("cargo:rerun-if-changed=../../migrations");
}
