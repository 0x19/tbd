//! The workspace as an in-memory cache of files, written only on `commit`.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
};

/// The workspace could not be found or read.
#[derive(Debug, thiserror::Error)]
pub enum RepoError {
    /// No `Cargo.toml` with a `[workspace]` table above `start`.
    #[error("no workspace root above {0}")]
    NotFound(PathBuf),
    /// I/O on a file.
    #[error("{path}: {source}")]
    Io {
        /// File.
        path: PathBuf,
        /// Cause.
        source: io::Error,
    },
}

/// A lazily cached view of the repository. Reads go through the cache; writes
/// stay in the cache until [`Workspace::commit`].
#[derive(Debug)]
pub struct Workspace {
    root: PathBuf,
    cache: BTreeMap<PathBuf, Option<String>>,
    dirty: BTreeSet<PathBuf>,
}

impl Workspace {
    /// Walk up from `start` to the directory whose `Cargo.toml` declares a workspace.
    ///
    /// # Errors
    /// No such directory.
    pub fn discover(start: &Path) -> Result<Self, RepoError> {
        let mut dir = Some(start);
        while let Some(d) = dir {
            let manifest = d.join("Cargo.toml");
            if manifest.is_file() {
                let text = fs::read_to_string(&manifest).map_err(|source| RepoError::Io {
                    path: manifest.clone(),
                    source,
                })?;
                if text.contains("[workspace]") {
                    return Ok(Self::open(d));
                }
            }
            dir = d.parent();
        }
        Err(RepoError::NotFound(start.to_path_buf()))
    }

    /// A workspace at a known root.
    #[must_use]
    pub fn open(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
            cache: BTreeMap::new(),
            dirty: BTreeSet::new(),
        }
    }

    /// The root directory.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// A file's content, `None` when it does not exist.
    ///
    /// # Errors
    /// The file exists but cannot be read.
    pub fn read(&mut self, rel: &Path) -> Result<Option<String>, RepoError> {
        if let Some(cached) = self.cache.get(rel) {
            return Ok(cached.clone());
        }
        let full = self.root.join(rel);
        let content = match fs::read_to_string(&full) {
            Ok(s) => Some(s),
            Err(e) if e.kind() == io::ErrorKind::NotFound => None,
            Err(source) => return Err(RepoError::Io { path: full, source }),
        };
        self.cache.insert(rel.to_path_buf(), content.clone());
        Ok(content)
    }

    /// Replace a file's content in the cache.
    pub fn write(&mut self, rel: &Path, content: String) {
        self.cache.insert(rel.to_path_buf(), Some(content));
        self.dirty.insert(rel.to_path_buf());
    }

    /// Files written since the last commit.
    #[must_use]
    pub fn changed(&self) -> Vec<PathBuf> {
        self.dirty.iter().cloned().collect()
    }

    /// Write every changed file to disk, creating directories as needed.
    ///
    /// # Errors
    /// A write fails; the files written before it are listed in the error's
    /// context by the caller.
    pub fn commit(&mut self) -> Result<Vec<PathBuf>, RepoError> {
        let mut written = Vec::new();
        for rel in std::mem::take(&mut self.dirty) {
            let Some(Some(content)) = self.cache.get(&rel) else {
                continue;
            };
            let full = self.root.join(&rel);
            if let Some(parent) = full.parent() {
                fs::create_dir_all(parent).map_err(|source| RepoError::Io {
                    path: parent.to_path_buf(),
                    source,
                })?;
            }
            fs::write(&full, content).map_err(|source| RepoError::Io {
                path: full.clone(),
                source,
            })?;
            written.push(rel);
        }
        Ok(written)
    }

    /// Relative paths of `crates/*/Cargo.toml` on disk.
    ///
    /// # Errors
    /// `crates/` cannot be listed.
    pub fn crate_manifests(&self) -> Result<Vec<PathBuf>, RepoError> {
        let dir = self.root.join("crates");
        let entries = fs::read_dir(&dir).map_err(|source| RepoError::Io {
            path: dir.clone(),
            source,
        })?;
        let mut out = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|source| RepoError::Io {
                path: dir.clone(),
                source,
            })?;
            let manifest = entry.path().join("Cargo.toml");
            if manifest.is_file() {
                out.push(
                    PathBuf::from("crates")
                        .join(entry.file_name())
                        .join("Cargo.toml"),
                );
            }
        }
        out.sort();
        Ok(out)
    }
}
