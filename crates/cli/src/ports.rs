//! Port bookkeeping: which listen and metrics ports the repository already
//! uses, and the next free ones.

use std::{collections::BTreeSet, path::Path};

use crate::repo::{RepoError, Workspace};

/// Ports in use, by role.
#[derive(Debug, Default, Clone)]
pub struct Ports {
    /// Service listen ports (`*_LISTEN_ADDR`, Envoy `port_value`).
    pub listen: BTreeSet<u16>,
    /// Prometheus ports on the developer's machine (`*_METRICS_ADDR`).
    pub metrics: BTreeSet<u16>,
}

const FILES: &[&str] = &[
    ".env.example",
    "devops/envoy/envoy.yaml",
    "devops/k8s/base/configmap.yaml",
    "compose.yaml",
];

fn trailing_port(s: &str) -> Option<u16> {
    let s = s.trim().trim_end_matches(['"', '\'', ',', '}']).trim();
    s.rsplit(':').next()?.trim().parse().ok()
}

impl Ports {
    /// Scan the files that mention ports.
    ///
    /// # Errors
    /// A file cannot be read.
    pub fn scan(ws: &mut Workspace) -> Result<Self, RepoError> {
        let mut ports = Self::default();
        for file in FILES {
            let Some(text) = ws.read(Path::new(file))? else {
                continue;
            };
            for line in text.lines() {
                let line = line.trim_start().trim_start_matches('#').trim_start();
                if let Some((key, value)) = line.split_once(['=', ':'])
                    && key.trim().ends_with("_LISTEN_ADDR")
                    && let Some(p) = trailing_port(value)
                {
                    ports.listen.insert(p);
                }
                if let Some((key, value)) = line.split_once(['=', ':'])
                    && key.trim().ends_with("_METRICS_ADDR")
                    && let Some(p) = trailing_port(value)
                {
                    ports.metrics.insert(p);
                }
                if let Some(rest) = line.split("port_value:").nth(1)
                    && let Some(p) = trailing_port(rest)
                {
                    ports.listen.insert(p);
                }
            }
        }
        Ok(ports)
    }

    /// The next listen port above every port at or above 50000.
    #[must_use]
    pub fn next_listen(&self) -> u16 {
        self.listen
            .iter()
            .filter(|&&p| p >= 50000)
            .max()
            .map_or(50052, |p| p + 1)
    }

    /// The next metrics port above every port at or above 9464.
    #[must_use]
    pub fn next_metrics(&self) -> u16 {
        self.metrics
            .iter()
            .filter(|&&p| p >= 9464)
            .max()
            .map_or(9466, |p| p + 1)
    }

    /// Whether a listen port is taken.
    #[must_use]
    pub fn listen_in_use(&self, port: u16) -> bool {
        self.listen.contains(&port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ports_from_env_yaml_and_envoy_lines() {
        assert_eq!(trailing_port("0.0.0.0:50051"), Some(50051));
        assert_eq!(trailing_port(" 127.0.0.1:9464\""), Some(9464));
        assert_eq!(trailing_port(" 50051 }"), Some(50051));
        assert_eq!(trailing_port("nope"), None);
    }
}
