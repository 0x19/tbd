//! What a scaffolded service is: a validated name, a kind, its ports, and every
//! identifier derived from the name. The marker line in the generated
//! `CLAUDE.md` is how `check` and `list` rebuild it later without flags.

use std::fmt;

/// Names a service may not take: existing crates, infrastructure, and words
/// that would collide with paths the scaffold writes.
pub const RESERVED: &[&str] = &[
    "engine", "protocol", "chaos", "common", "proto", "cli", "envoy", "auth", "base", "tbd",
    "humans",
];

/// Why a name was rejected.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum NameError {
    /// Not `^[a-z][a-z0-9]{1,23}$`.
    #[error(
        "{0:?}: a service name is 2 to 24 lowercase letters and digits, starting with a letter"
    )]
    Shape(String),
    /// One of [`RESERVED`].
    #[error("{0:?} is reserved")]
    Reserved(String),
}

/// A validated service name.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ServiceName(String);

impl ServiceName {
    /// Validate a name.
    ///
    /// # Errors
    /// The name has the wrong shape or is reserved.
    pub fn parse(name: &str) -> Result<Self, NameError> {
        let ok_shape = (2..=24).contains(&name.len())
            && name.starts_with(|c: char| c.is_ascii_lowercase())
            && name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit());
        if !ok_shape {
            return Err(NameError::Shape(name.to_owned()));
        }
        if RESERVED.contains(&name) {
            return Err(NameError::Reserved(name.to_owned()));
        }
        Ok(Self(name.to_owned()))
    }

    /// The name as written.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// `ledger` to `Ledger`.
    #[must_use]
    pub fn pascal(&self) -> String {
        let mut chars = self.0.chars();
        match chars.next() {
            Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
            None => String::new(),
        }
    }

    /// `ledger` to `LEDGER`.
    #[must_use]
    pub fn upper(&self) -> String {
        self.0.to_ascii_uppercase()
    }
}

impl fmt::Display for ServiceName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// What kind of service the template renders.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Kind {
    /// A tonic gRPC server with health, reflection, metrics and a `Ping` RPC.
    Grpc,
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Grpc => f.write_str("grpc"),
        }
    }
}

/// A service to scaffold or check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Service {
    /// Validated name.
    pub name: ServiceName,
    /// Template kind.
    pub kind: Kind,
    /// The port the service listens on.
    pub port: u16,
    /// The Prometheus port published on the developer's machine (containers use 9464).
    pub metrics_port: u16,
    /// The bacon keybinding.
    pub bacon_key: char,
}

impl Service {
    /// Cargo package name, `tbd-<name>`.
    #[must_use]
    pub fn package(&self) -> String {
        format!("tbd-{}", self.name)
    }

    /// Rust crate identifier, `tbd_<name>`.
    #[must_use]
    pub fn crate_ident(&self) -> String {
        format!("tbd_{}", self.name)
    }

    /// Protobuf package, `tbd.<name>.v1`.
    #[must_use]
    pub fn proto_package(&self) -> String {
        format!("tbd.{}.v1", self.name)
    }

    /// Fully qualified gRPC service, `tbd.<name>.v1.<Name>Service`.
    #[must_use]
    pub fn grpc_service(&self) -> String {
        format!("{}.{}Service", self.proto_package(), self.name.pascal())
    }

    /// Proto file path under `/proto`.
    #[must_use]
    pub fn proto_path(&self) -> String {
        format!("tbd/{n}/v1/{n}.proto", n = self.name)
    }

    /// Container image name.
    #[must_use]
    pub fn image(&self) -> String {
        format!("ghcr.io/0x19/tbd-{}", self.name)
    }

    /// The marker written as the first line of the generated `CLAUDE.md`.
    #[must_use]
    pub fn marker(&self) -> String {
        format!(
            "<!-- tbd new service {} --kind {} --port {} --metrics-port {} --bacon-key {} (tbd-cli {}) -->",
            self.name,
            self.kind,
            self.port,
            self.metrics_port,
            self.bacon_key,
            crate::VERSION
        )
    }

    /// Rebuild a service from its marker line, if the line is one.
    #[must_use]
    pub fn from_marker(line: &str) -> Option<Self> {
        let line = line.trim();
        let inner = line
            .strip_prefix("<!-- tbd new service ")?
            .strip_suffix("-->")?
            .trim();
        let mut words = inner.split_whitespace();
        let name = ServiceName::parse(words.next()?).ok()?;
        let mut kind = Kind::Grpc;
        let mut port = None;
        let mut metrics_port = None;
        let mut bacon_key = None;
        while let Some(flag) = words.next() {
            match flag {
                "--kind" => {
                    kind = match words.next()? {
                        "grpc" => Kind::Grpc,
                        _ => return None,
                    };
                }
                "--port" => port = words.next()?.parse().ok(),
                "--metrics-port" => metrics_port = words.next()?.parse().ok(),
                "--bacon-key" => bacon_key = words.next()?.chars().next(),
                _ => break,
            }
        }
        Some(Self {
            bacon_key: bacon_key.unwrap_or_else(|| name.as_str().chars().next().unwrap_or('x')),
            name,
            kind,
            port: port?,
            metrics_port: metrics_port?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_validated() {
        assert!(ServiceName::parse("ledger").is_ok());
        assert!(ServiceName::parse("a1").is_ok());
        assert_eq!(
            ServiceName::parse("Ledger"),
            Err(NameError::Shape("Ledger".into()))
        );
        assert_eq!(ServiceName::parse("l"), Err(NameError::Shape("l".into())));
        assert_eq!(
            ServiceName::parse("my-svc"),
            Err(NameError::Shape("my-svc".into()))
        );
        assert_eq!(
            ServiceName::parse("engine"),
            Err(NameError::Reserved("engine".into()))
        );
    }

    #[test]
    fn marker_round_trips() {
        let s = Service {
            name: ServiceName::parse("ledger").unwrap(),
            kind: Kind::Grpc,
            port: 50052,
            metrics_port: 9466,
            bacon_key: 'l',
        };
        assert_eq!(Service::from_marker(&s.marker()), Some(s.clone()));
        assert_eq!(s.grpc_service(), "tbd.ledger.v1.LedgerService");
        assert_eq!(s.proto_path(), "tbd/ledger/v1/ledger.proto");
        assert_eq!(s.crate_ident(), "tbd_ledger");
        assert_eq!(Service::from_marker("# crates/ledger"), None);
    }
}
