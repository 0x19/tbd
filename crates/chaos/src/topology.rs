//! The `[stack]` tables: `[stack.<plural>.<name>]`, one table per kind in
//! [`crate::kinds`]. The plural names the kind, `listen` is generic, the rest
//! of the table is the kind's own spec.

use std::{collections::BTreeMap, net::SocketAddr, sync::Arc};

use serde::{Deserialize, Deserializer, Serialize, de::Error as _};

use crate::{
    kinds::{self, Kind},
    service::Service,
    stack::{Launcher, Stack, ephemeral},
};

/// `[stack]` in a scenario or topology file.
#[derive(Debug, Clone, Default)]
pub struct StackConfig {
    /// Every instance by name, whatever its kind.
    pub instances: BTreeMap<String, InstanceSpec>,
}

/// One `[stack.<plural>.<name>]` table, parsed.
#[derive(Clone)]
pub struct InstanceSpec {
    /// The kind (from the plural).
    pub kind: &'static Kind,
    /// Bind address. `None` for any free loopback port.
    pub listen: Option<SocketAddr>,
    /// The table minus `listen`.
    pub spec: toml::Table,
    /// The table, parsed by the kind.
    pub service: Arc<dyn Service>,
}

impl std::fmt::Debug for InstanceSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InstanceSpec")
            .field("kind", &self.kind.name)
            .field("listen", &self.listen)
            .field("spec", &self.spec)
            .finish_non_exhaustive()
    }
}

impl InstanceSpec {
    /// Parse a table for a kind; `listen` is peeled off first.
    ///
    /// # Errors
    /// `listen` is not an address, or the kind rejects the rest.
    pub fn new(kind: &'static Kind, mut table: toml::Table) -> Result<Self, toml::de::Error> {
        let listen = match table.remove("listen") {
            None => None,
            Some(v) => Some(v.try_into::<SocketAddr>()?),
        };
        let service = (kind.parse)(table.clone())?;
        Ok(Self {
            kind,
            listen,
            spec: table,
            service,
        })
    }

    /// The table with `listen` back in, as written.
    #[must_use]
    pub fn table(&self) -> toml::Table {
        let mut t = self.spec.clone();
        if let Some(l) = self.listen {
            t.insert("listen".to_owned(), toml::Value::String(l.to_string()));
        }
        t
    }

    /// The value of the kind's dependency field, if it has one.
    #[must_use]
    pub fn dependency(&self) -> Option<(kinds::Dependency, String)> {
        let dep = self.kind.dependency()?;
        let value = self.spec.get(dep.field)?.as_str()?.to_owned();
        Some((dep, value))
    }
}

impl<'de> Deserialize<'de> for StackConfig {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let tables: BTreeMap<String, BTreeMap<String, toml::Table>> = Deserialize::deserialize(d)?;
        let mut instances: BTreeMap<String, InstanceSpec> = BTreeMap::new();
        for (plural, entries) in tables {
            let kind = kinds::by_plural(&plural).ok_or_else(|| {
                D::Error::custom(format!(
                    "unknown table `stack.{plural}`; expected one of: {}",
                    kinds::plurals()
                ))
            })?;
            for (name, table) in entries {
                let spec = InstanceSpec::new(kind, table)
                    .map_err(|e| D::Error::custom(format!("stack.{plural}.{name}: {e}")))?;
                if let Some(other) = instances.get(&name) {
                    return Err(D::Error::custom(format!(
                        "{name:?} is both {} and {}",
                        article(other.kind.name),
                        article(kind.name)
                    )));
                }
                instances.insert(name, spec);
            }
        }
        Ok(Self { instances })
    }
}

fn article(kind: &str) -> String {
    let a = if kind.starts_with(['a', 'e', 'i', 'o', 'u']) {
        "an"
    } else {
        "a"
    };
    format!("{a} {kind}")
}

impl Serialize for StackConfig {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut out: BTreeMap<&str, BTreeMap<&str, toml::Table>> = BTreeMap::new();
        for (name, i) in &self.instances {
            out.entry(i.kind.plural)
                .or_default()
                .insert(name.as_str(), i.table());
        }
        out.serialize(s)
    }
}

impl StackConfig {
    /// Add one instance from a table (the same keys as the file).
    ///
    /// # Errors
    /// The kind rejects the table.
    pub fn insert(
        &mut self,
        name: &str,
        kind: &'static Kind,
        table: toml::Table,
    ) -> Result<(), toml::de::Error> {
        let spec = InstanceSpec::new(kind, table)?;
        self.instances.insert(name.to_owned(), spec);
        Ok(())
    }

    /// One instance.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&InstanceSpec> {
        self.instances.get(name)
    }

    /// No instances at all.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.instances.is_empty()
    }

    /// Names of the instances of one kind.
    pub fn of_kind<'a>(&'a self, kind: &'a str) -> impl Iterator<Item = &'a str> + 'a {
        self.instances
            .iter()
            .filter(move |(_, i)| i.kind.name == kind)
            .map(|(n, _)| n.as_str())
    }

    /// Turn the config into launchers the generic [`Stack`] can run.
    #[must_use]
    pub fn launchers(&self) -> BTreeMap<String, Launcher> {
        self.instances
            .iter()
            .map(|(name, i)| {
                (
                    name.clone(),
                    Launcher {
                        service: Arc::clone(&i.service),
                        listen: i.listen.unwrap_or_else(ephemeral),
                    },
                )
            })
            .collect()
    }

    /// Start the whole stack.
    ///
    /// # Errors
    /// See [`Stack::start`].
    pub async fn start(&self) -> Result<Stack, crate::stack::StackError> {
        Stack::start(self.launchers()).await
    }

    /// Referential check without starting anything: every dependency names
    /// an instance of the expected kind.
    ///
    /// # Errors
    /// A dangling or mistyped reference, or an empty stack.
    pub fn check(&self) -> Result<(), String> {
        for (name, i) in &self.instances {
            let Some(dep) = i.kind.dependency() else {
                continue;
            };
            let Some(value) = i.spec.get(dep.field).and_then(toml::Value::as_str) else {
                return Err(format!("{} {name:?} has no `{}`", i.kind.name, dep.field));
            };
            match self.instances.get(value) {
                None => {
                    return Err(format!(
                        "{} {name:?} references unknown {} {value:?}",
                        i.kind.name, dep.kind
                    ));
                }
                Some(target) if target.kind.name != dep.kind => {
                    return Err(format!(
                        "{} {name:?}: `{}` names {value:?}, which is {} and not {}",
                        i.kind.name,
                        dep.field,
                        article(target.kind.name),
                        article(dep.kind)
                    ));
                }
                Some(_) => {}
            }
        }
        if self.instances.is_empty() {
            return Err("stack has no instances".to_owned());
        }
        Ok(())
    }
}

/// A file with only a stack in it, for `chaos up`.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TopologyFile {
    /// Optional description.
    #[serde(default)]
    pub scenario: Option<toml::Value>,
    /// The stack.
    pub stack: StackConfig,
    /// Ignored here; scenarios use them.
    #[serde(default)]
    pub load: Option<toml::Value>,
    /// Ignored here; scenarios use them.
    #[serde(default)]
    pub timeline: Option<toml::Value>,
    /// Ignored here; scenarios use them.
    #[serde(default)]
    pub assertions: Option<toml::Value>,
}
