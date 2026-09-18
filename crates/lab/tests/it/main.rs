//! Integration tests for the bench itself.
//!
//! These start real services on ephemeral ports — no mocks, per the repository's
//! rule — and prove the claims the playground's game rests on.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod enginelb;
