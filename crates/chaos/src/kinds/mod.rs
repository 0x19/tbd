//! Service kinds as data. One module per kind, one line in [`ALL`]; the
//! topology tables (`[stack.<plural>.<name>]`), launchers, cross-checks,
//! validate targets and checks, runtime add and clone, CLI flags and the admin
//! UI's forms all derive from it. `tbd new service` writes a module and a line.

pub mod engine;
pub mod humans;
pub mod ledger;
pub mod protocol;

use std::{net::SocketAddr, sync::Arc};

use serde::{Serialize, de::DeserializeOwned};

use crate::{
    load,
    service::Service,
    stack::{Launcher, Stack, ephemeral},
    validate::Check,
};

/// Every kind, in display order (validate output, `chaos up`, error lists).
pub static ALL: &[&Kind] = &[
    &engine::KIND,
    &protocol::KIND,
    &ledger::KIND,
    &humans::KIND,
    // tbd:kinds-end (tbd new service inserts above this line; do not edit)
];

/// Static description of one kind. Behaviour hooks are fn pointers so the
/// whole registry is a `static`.
// The capability flags are independent yes/no facts, not a state machine.
#[allow(clippy::struct_excessive_bools)]
pub struct Kind {
    /// The kind everywhere: `Instance.kind`, `InstanceInfo.kind`, `POST /stack`.
    pub name: &'static str,
    /// Shown in the admin UI.
    pub label: &'static str,
    /// `[stack.<plural>.<name>]`.
    pub plural: &'static str,
    /// What `chaos up` prints next to the address.
    pub surface: &'static str,
    /// `chaos validate` has a URL target for this kind.
    pub target: Option<Target>,
    /// The keys of the kind's table besides `listen`: what `POST /stack`
    /// accepts and the UI asks for.
    pub fields: &'static [Field],
    /// `InstanceHandle::fault` is `Some`: `set_behavior` and `PUT /behavior` apply.
    pub fault: bool,
    /// `InstanceHandle::requests` is `Some`: `[assertions.services.<name>]` reads counters.
    pub counters: bool,
    /// Load runs target instances of this kind.
    pub load_target: bool,
    /// May be added to a running stack.
    pub addable: bool,
    /// Parse the table (minus `listen`) into the service.
    pub parse: fn(toml::Table) -> Result<Arc<dyn Service>, toml::de::Error>,
    /// Validate checks, in output order. Empty when `target` is `None`.
    pub checks: &'static [Check],
}

/// A validate target.
pub struct Target {
    /// One line for `--help` and the docs.
    pub help: &'static str,
    /// Used when `[targets]` has no entry for the kind.
    pub default_url: &'static str,
}

/// One key of a kind's table.
pub struct Field {
    /// Key name.
    pub name: &'static str,
    /// Shown in the admin UI.
    pub label: &'static str,
    /// Value shape.
    pub kind: FieldKind,
    /// Must be given.
    pub required: bool,
    /// Value when absent, as text.
    pub default: Option<&'static str>,
}

/// Value shape of a [`Field`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKind {
    /// Free text.
    Text,
    /// A humantime duration such as `1s`.
    Duration,
    /// The name of a running instance of another kind.
    InstanceOf(&'static str),
}

/// A dependency on an instance of another kind, from an [`FieldKind::InstanceOf`] field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dependency {
    /// The field naming it, e.g. `engine`.
    pub field: &'static str,
    /// The kind it must be.
    pub kind: &'static str,
}

impl Kind {
    /// The dependency field, if the kind has one.
    #[must_use]
    pub fn dependency(&self) -> Option<Dependency> {
        self.fields.iter().find_map(|f| match f.kind {
            FieldKind::InstanceOf(kind) => Some(Dependency {
                field: f.name,
                kind,
            }),
            FieldKind::Text | FieldKind::Duration => None,
        })
    }

    /// `CHAOS_<NAME>_URL`.
    #[must_use]
    pub fn env_var(&self) -> String {
        format!("CHAOS_{}_URL", self.name.to_ascii_uppercase())
    }

    /// A launcher for the generic stack; `listen` `None` means any free port.
    ///
    /// # Errors
    /// The kind rejects the table.
    pub fn launcher(
        &self,
        spec: toml::Table,
        listen: Option<SocketAddr>,
    ) -> Result<Launcher, toml::de::Error> {
        Ok(Launcher {
            service: (self.parse)(spec)?,
            listen: listen.unwrap_or_else(ephemeral),
        })
    }
}

impl PartialEq for Kind {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl std::fmt::Debug for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name)
    }
}

/// `parse` for a kind whose spec struct is its service.
///
/// # Errors
/// The table does not match the struct.
pub fn parse<S: Service + DeserializeOwned + 'static>(
    table: toml::Table,
) -> Result<Arc<dyn Service>, toml::de::Error> {
    Ok(Arc::new(table.try_into::<S>()?))
}

/// The kind named `name`.
#[must_use]
pub fn by_name(name: &str) -> Option<&'static Kind> {
    ALL.iter().copied().find(|k| k.name == name)
}

/// The kind whose topology table is `plural`.
#[must_use]
pub fn by_plural(plural: &str) -> Option<&'static Kind> {
    ALL.iter().copied().find(|k| k.plural == plural)
}

/// `engine, ledger, protocol`, for error messages.
#[must_use]
pub fn names() -> String {
    ALL.iter().map(|k| k.name).collect::<Vec<_>>().join(", ")
}

/// `engines, ledgers, protocols`, for error messages.
#[must_use]
pub fn plurals() -> String {
    ALL.iter().map(|k| k.plural).collect::<Vec<_>>().join(", ")
}

/// Every kind that has a validate target.
pub fn with_target() -> impl Iterator<Item = (&'static Kind, &'static Target)> {
    ALL.iter()
        .copied()
        .filter_map(|k| k.target.as_ref().map(|t| (k, t)))
}

/// Running instances of every `load_target` kind, as load targets.
#[must_use]
pub fn load_targets(stack: &Stack) -> Vec<load::Target> {
    stack
        .instances()
        .filter(|(_, i)| by_name(i.kind).is_some_and(|k| k.load_target))
        .map(|(name, i)| load::Target {
            name: name.to_owned(),
            http_url: i.http_url(),
        })
        .collect()
}

/// A kind as the API and the admin UI see it.
#[derive(Debug, Clone, Serialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct KindInfo {
    /// [`Kind::name`].
    pub name: &'static str,
    /// [`Kind::label`].
    pub label: &'static str,
    /// [`Kind::plural`].
    pub plural: &'static str,
    /// [`Kind::surface`].
    pub surface: &'static str,
    /// [`Kind::fault`].
    pub fault: bool,
    /// [`Kind::counters`].
    pub counters: bool,
    /// [`Kind::load_target`].
    pub load_target: bool,
    /// [`Kind::addable`].
    pub addable: bool,
    /// Has a validate target.
    pub target: bool,
    /// The kind a dependency field names, if any.
    pub dependency_kind: Option<&'static str>,
    /// The table's keys.
    pub fields: Vec<FieldInfo>,
}

/// A [`Field`] as the API sees it.
#[derive(Debug, Clone, Serialize)]
pub struct FieldInfo {
    /// Key name.
    pub name: &'static str,
    /// Label.
    pub label: &'static str,
    /// `text`, `duration` or `instance_of`.
    pub kind: &'static str,
    /// For `instance_of`, the kind.
    pub of_kind: Option<&'static str>,
    /// Must be given.
    pub required: bool,
    /// Default as text.
    pub default: Option<&'static str>,
}

/// Every kind, described.
#[must_use]
pub fn describe() -> Vec<KindInfo> {
    ALL.iter()
        .map(|k| KindInfo {
            name: k.name,
            label: k.label,
            plural: k.plural,
            surface: k.surface,
            fault: k.fault,
            counters: k.counters,
            load_target: k.load_target,
            addable: k.addable,
            target: k.target.is_some(),
            dependency_kind: k.dependency().map(|d| d.kind),
            fields: k
                .fields
                .iter()
                .map(|f| FieldInfo {
                    name: f.name,
                    label: f.label,
                    kind: match f.kind {
                        FieldKind::Text => "text",
                        FieldKind::Duration => "duration",
                        FieldKind::InstanceOf(_) => "instance_of",
                    },
                    of_kind: match f.kind {
                        FieldKind::InstanceOf(k) => Some(k),
                        FieldKind::Text | FieldKind::Duration => None,
                    },
                    required: f.required,
                    default: f.default,
                })
                .collect(),
        })
        .collect()
}

/// The registry and the check catalogue as Markdown, for `docs/chaos/kinds.md`.
#[must_use]
pub fn markdown() -> String {
    use std::fmt::Write as _;
    let mut out = String::new();
    let _ = writeln!(
        out,
        "| Kind | Table | Surface | Fault injection | Counters | Load target | Addable | Validate target (default, env) | Fields |"
    );
    let _ = writeln!(out, "|---|---|---|---|---|---|---|---|---|");
    for k in ALL {
        let yn = |b: bool| if b { "yes" } else { "no" };
        let target = k.target.as_ref().map_or("none".to_owned(), |t| {
            format!("`{}`, `{}`", t.default_url, k.env_var())
        });
        let fields = if k.fields.is_empty() {
            "none".to_owned()
        } else {
            k.fields
                .iter()
                .map(|f| {
                    let shape = match f.kind {
                        FieldKind::Text => "text".to_owned(),
                        FieldKind::Duration => "duration".to_owned(),
                        FieldKind::InstanceOf(of) => format!("a running {of}"),
                    };
                    let req = if f.required { ", required" } else { "" };
                    let def = f
                        .default
                        .map(|d| format!(", default `{d}`"))
                        .unwrap_or_default();
                    format!("`{}` ({shape}{req}{def})", f.name)
                })
                .collect::<Vec<_>>()
                .join("; ")
        };
        let _ = writeln!(
            out,
            "| `{}` | `[stack.{}.<name>]` | {} | {} | {} | {} | {} | {} | {} |",
            k.name,
            k.plural,
            k.surface,
            yn(k.fault),
            yn(k.counters),
            yn(k.load_target),
            yn(k.addable),
            target,
            fields
        );
    }
    let _ = writeln!(out);
    let _ = writeln!(out, "| Check | Surface | Kind | Passes when |");
    let _ = writeln!(out, "|---|---|---|---|");
    for k in ALL {
        for c in k.checks {
            let _ = writeln!(
                out,
                "| `{}` | {} | `{}` | {} |",
                c.name, c.surface, k.name, c.doc
            );
        }
    }
    out
}
