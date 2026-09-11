//! The path registry hook. The ledger knows nothing about people; which paths
//! exist and which sources may write them is injected by the embedder
//! (`docs/design/humans/000-facts-ledger.md`: "Unknown paths are rejected at
//! write time"). Phase 1 ships [`ShapeOnly`], which checks the shape and lets
//! every source write.

use super::{Source, validate};

/// Why the registry refused a write.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RegistryError {
    /// The path is not well-formed.
    #[error("malformed path {path:?}: {reason}")]
    Malformed {
        /// The path.
        path: String,
        /// Why.
        reason: String,
    },
    /// The path is well-formed but not registered.
    #[error("unknown path {0:?}")]
    Unknown(String),
    /// The source may not write this path.
    #[error("{writer} may not write {path:?}")]
    Forbidden {
        /// The path.
        path: String,
        /// The source that tried to write it.
        writer: Source,
    },
}

/// What may be written.
pub trait Registry: Send + Sync + 'static {
    /// The path is known and well-formed, and `source` may write it.
    ///
    /// # Errors
    /// See [`RegistryError`].
    fn check_write(&self, path: &str, source: Source) -> Result<(), RegistryError>;
}

/// Shape only: `[a-z0-9_]+(\.[a-z0-9_]+)+`, at most 256 bytes, every source
/// allowed. The real registry is the embedder's.
#[derive(Debug, Clone, Copy, Default)]
pub struct ShapeOnly;

impl Registry for ShapeOnly {
    fn check_write(&self, path: &str, _source: Source) -> Result<(), RegistryError> {
        validate::path_shape(path, 2).map_err(|reason| RegistryError::Malformed {
            path: path.to_owned(),
            reason,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shape_only_accepts_dotted_paths_and_rejects_the_rest() {
        assert!(
            ShapeOnly
                .check_write("traits.warmth", Source::Inferred)
                .is_ok()
        );
        assert!(
            ShapeOnly
                .check_write("relations.match.abc_1", Source::Observed)
                .is_ok()
        );
        for bad in [
            "traits", "Traits.x", "a..b", ".a", "a.", "a b.c", "", "a.b*",
        ] {
            assert!(
                matches!(
                    ShapeOnly.check_write(bad, Source::Declared),
                    Err(RegistryError::Malformed { .. })
                ),
                "{bad:?}"
            );
        }
        let long = format!("a.{}", "b".repeat(300));
        assert!(ShapeOnly.check_write(&long, Source::Declared).is_err());
    }
}
