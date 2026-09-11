//! Layered TOML configuration: `base.toml` plus one environment file.
//!
//! A service keeps its configuration in a directory such as `configs/chaos/`
//! holding `base.toml` and one file per environment (`local.toml`,
//! `dev.toml`, `production.toml`). [`load`] reads `base.toml`, then the
//! environment's file, deep-merges the second over the first and deserialises
//! the result. Tables merge key by key; every other value in the environment
//! file replaces the base value, arrays included.
//!
//! Precedence for a running binary is therefore: `base.toml` < `<env>.toml` <
//! command-line flags and their environment variables, which the binary
//! applies itself after loading.

use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;

/// The reserved name of the shared layer.
pub const BASE: &str = "base";
/// Environment variable naming the environment, shared by every binary.
pub const ENV_VAR: &str = "TBD_ENV";
/// The environment a binary assumes when [`ENV_VAR`] is unset.
pub const DEFAULT_ENV: &str = "local";

/// What went wrong while loading.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// A file could not be read.
    #[error("read {path}: {source}")]
    Read {
        /// File.
        path: PathBuf,
        /// Cause.
        source: std::io::Error,
    },
    /// A file is not valid TOML.
    #[error("parse {path}: {source}")]
    Parse {
        /// File.
        path: PathBuf,
        /// Cause.
        source: Box<toml::de::Error>,
    },
    /// The merged document does not fit the target type.
    #[error("config for env {env:?} from {files:?}: {source}")]
    Shape {
        /// Environment name.
        env: String,
        /// Files that were merged, in order.
        files: Vec<PathBuf>,
        /// Cause.
        source: Box<toml::de::Error>,
    },
    /// The environment file does not exist.
    #[error("no config for env {env:?}: {path} does not exist")]
    MissingEnv {
        /// Environment name.
        env: String,
        /// Expected file.
        path: PathBuf,
    },
    /// The environment name is `base` or not a plain file stem.
    #[error("invalid env name {0:?}: use a plain name such as local, dev or production")]
    InvalidEnv(String),
    /// The loaded values do not hold together (a binary's own cross-field check).
    #[error("invalid config: {0}")]
    Invalid(String),
}

/// A loaded configuration and where it came from.
#[derive(Debug, Clone)]
pub struct Loaded<T> {
    /// The merged, typed value.
    pub value: T,
    /// Environment name.
    pub env: String,
    /// Files merged, in order.
    pub files: Vec<PathBuf>,
}

/// Load `dir/base.toml` merged with `dir/<env>.toml` into `T`.
///
/// `base.toml` is required. The environment file is required too: a missing
/// one is an error rather than a silent fallback, so a typo in the environment
/// name cannot start a service with base defaults.
pub fn load<T: DeserializeOwned>(dir: &Path, env: &str) -> Result<Loaded<T>, ConfigError> {
    if env == BASE
        || env.is_empty()
        || !env
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(ConfigError::InvalidEnv(env.to_owned()));
    }
    let base_path = dir.join(format!("{BASE}.toml"));
    let env_path = dir.join(format!("{env}.toml"));
    if !env_path.is_file() {
        return Err(ConfigError::MissingEnv {
            env: env.to_owned(),
            path: env_path,
        });
    }
    let mut merged = read_table(&base_path)?;
    merge_into(&mut merged, read_table(&env_path)?);
    let files = vec![base_path, env_path];
    let value = merged.try_into().map_err(|source| ConfigError::Shape {
        env: env.to_owned(),
        files: files.clone(),
        source: Box::new(source),
    })?;
    Ok(Loaded {
        value,
        env: env.to_owned(),
        files,
    })
}

/// Merge two TOML documents the way [`load`] does. Public so a tool can show
/// the effective configuration without a target type.
pub fn merge(base: toml::Table, over: toml::Table) -> toml::Table {
    let mut out = base;
    merge_into(&mut out, over);
    out
}

fn read_table(path: &Path) -> Result<toml::Table, ConfigError> {
    let text = std::fs::read_to_string(path).map_err(|source| ConfigError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    text.parse::<toml::Table>()
        .map_err(|source| ConfigError::Parse {
            path: path.to_path_buf(),
            source: Box::new(source),
        })
}

fn merge_into(base: &mut toml::Table, over: toml::Table) {
    for (key, value) in over {
        match (base.get_mut(&key), value) {
            (Some(toml::Value::Table(b)), toml::Value::Table(o)) => merge_into(b, o),
            (slot, value) => {
                if let Some(slot) = slot {
                    *slot = value;
                } else {
                    base.insert(key, value);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[derive(Debug, serde::Deserialize, PartialEq)]
    #[serde(deny_unknown_fields)]
    struct Cfg {
        name: String,
        ports: Vec<u16>,
        links: Links,
    }

    #[derive(Debug, serde::Deserialize, PartialEq)]
    #[serde(deny_unknown_fields)]
    struct Links {
        grafana: String,
        #[serde(default)]
        tempo: String,
    }

    fn dir() -> tempdir::Dir {
        let d = tempdir::Dir::new();
        d.write(
            "base.toml",
            "name = \"base\"\nports = [1, 2]\n[links]\ngrafana = \"g\"\ntempo = \"t\"\n",
        );
        d.write(
            "local.toml",
            "name = \"local\"\nports = [3]\n[links]\ngrafana = \"lg\"\n",
        );
        d
    }

    #[test]
    fn env_layer_replaces_scalars_and_arrays_and_merges_tables() {
        let d = dir();
        let loaded: Loaded<Cfg> = load(d.path(), "local").unwrap();
        assert_eq!(loaded.value.name, "local");
        assert_eq!(loaded.value.ports, vec![3]);
        assert_eq!(loaded.value.links.grafana, "lg");
        assert_eq!(loaded.value.links.tempo, "t", "untouched keys survive");
        assert_eq!(loaded.files.len(), 2);
    }

    #[test]
    fn missing_env_and_reserved_names_are_errors() {
        let d = dir();
        assert!(matches!(
            load::<Cfg>(d.path(), "staging"),
            Err(ConfigError::MissingEnv { .. })
        ));
        assert!(matches!(
            load::<Cfg>(d.path(), "base"),
            Err(ConfigError::InvalidEnv(_))
        ));
        assert!(matches!(
            load::<Cfg>(d.path(), "../etc"),
            Err(ConfigError::InvalidEnv(_))
        ));
    }

    #[test]
    fn unknown_keys_fail_after_merging() {
        let d = dir();
        d.write("dev.toml", "bogus = 1\n");
        assert!(matches!(
            load::<Cfg>(d.path(), "dev"),
            Err(ConfigError::Shape { .. })
        ));
    }

    /// Minimal temp directory without a crate.
    mod tempdir {
        use std::path::{Path, PathBuf};

        pub struct Dir(PathBuf);

        impl Dir {
            pub fn new() -> Self {
                let p = std::env::temp_dir().join(format!(
                    "tbd-config-{}-{}",
                    std::process::id(),
                    rand::random::<u64>()
                ));
                std::fs::create_dir_all(&p).unwrap();
                Self(p)
            }
            pub fn path(&self) -> &Path {
                &self.0
            }
            pub fn write(&self, name: &str, text: &str) {
                std::fs::write(self.0.join(name), text).unwrap();
            }
        }

        impl Drop for Dir {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
    }
}
