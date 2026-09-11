//! Plan and apply a set of registrations against a workspace, in two phases:
//! resolve everything first, write nothing unless everything resolves.

use std::path::PathBuf;

use crate::{
    edit::{Edit, Registration, Status},
    repo::{RepoError, Workspace},
};

/// One registration with its status in this workspace.
#[derive(Debug, Clone)]
pub struct Item {
    /// What.
    pub registration: Registration,
    /// Whether it is already there.
    pub status: Status,
}

impl Item {
    /// A generated file that exists with other content: the service evolved
    /// past its template. Informational; `check` never rewrites files.
    #[must_use]
    pub fn is_diverged(&self) -> bool {
        matches!(self.registration.edit, Edit::Create { .. })
            && matches!(self.status, Status::Conflict(_))
    }
}

/// The resolved plan.
#[derive(Debug, Clone, Default)]
pub struct Plan {
    /// Every registration, in apply order.
    pub items: Vec<Item>,
}

/// Why a plan could not be applied.
#[derive(Debug, thiserror::Error)]
pub enum ScaffoldError {
    /// Some registrations cannot be applied; nothing was written.
    #[error("{} registration(s) blocked; nothing written", .0.len())]
    Blocked(Vec<String>),
    /// An edit failed while applying.
    #[error("{id}: {reason}")]
    Apply {
        /// Registration id.
        id: String,
        /// Cause.
        reason: String,
    },
    /// The workspace could not be read or written.
    #[error(transparent)]
    Repo(#[from] RepoError),
}

/// What an application did.
#[derive(Debug, Default, Clone)]
pub struct Outcome {
    /// Files created.
    pub created: Vec<PathBuf>,
    /// Registrations applied to existing files.
    pub edited: Vec<String>,
    /// Registrations that were already present.
    pub skipped: Vec<String>,
}

impl Outcome {
    /// Nothing needed doing.
    #[must_use]
    pub fn nothing_to_do(&self) -> bool {
        self.created.is_empty() && self.edited.is_empty()
    }
}

/// Resolve every registration's status.
///
/// # Errors
/// A file cannot be read.
pub fn plan(ws: &mut Workspace, regs: Vec<Registration>) -> Result<Plan, RepoError> {
    let mut items = Vec::with_capacity(regs.len());
    for mut registration in regs {
        // Generated Rust is compared and written as rustfmt would leave it.
        if registration.path.extension().is_some_and(|e| e == "rs")
            && let Edit::Create { content } = &registration.edit
            && let Some(formatted) = crate::fmt::rustfmt(content)
        {
            registration.edit = Edit::Create { content: formatted };
        }
        let existing = ws.read(&registration.path)?;
        let status = registration.status(existing.as_deref());
        items.push(Item {
            registration,
            status,
        });
    }
    Ok(Plan { items })
}

impl Plan {
    /// Registrations that stop the plan: unresolvable anchors always, conflicts unless forced.
    #[must_use]
    pub fn blockers(&self, force: bool) -> Vec<String> {
        self.items
            .iter()
            .filter_map(|i| match &i.status {
                Status::Unresolvable(why) => Some(format!("{}: {why}", i.registration.id)),
                Status::Conflict(why) if !force => Some(format!("{}: {why}", i.registration.id)),
                _ => None,
            })
            .collect()
    }

    /// Every registration is present.
    #[must_use]
    pub fn all_present(&self) -> bool {
        self.items.iter().all(|i| i.status == Status::Present)
    }

    /// Present, missing, conflict and unresolvable counts. A diverged
    /// generated file counts as present: it is registered, just not verbatim.
    #[must_use]
    pub fn counts(&self) -> (usize, usize, usize, usize) {
        let mut c = (0, 0, 0, 0);
        for i in &self.items {
            if i.is_diverged() {
                c.0 += 1;
                continue;
            }
            match i.status {
                Status::Present => c.0 += 1,
                Status::Missing => c.1 += 1,
                Status::Conflict(_) => c.2 += 1,
                Status::Unresolvable(_) => c.3 += 1,
            }
        }
        c
    }
}

/// Apply every missing registration (and conflicting creates when `force`)
/// to the workspace cache. Nothing is written to disk; call
/// [`Workspace::commit`] for that.
///
/// # Errors
/// The plan has blockers, or an edit fails.
pub fn apply(
    ws: &mut Workspace,
    plan: &Plan,
    force: bool,
    creates: bool,
) -> Result<Outcome, ScaffoldError> {
    let blockers = plan.blockers(force);
    if !blockers.is_empty() {
        return Err(ScaffoldError::Blocked(blockers));
    }
    let mut outcome = Outcome::default();
    for item in &plan.items {
        let reg = &item.registration;
        let is_create = matches!(reg.edit, Edit::Create { .. });
        match &item.status {
            Status::Present => outcome.skipped.push(reg.id.clone()),
            Status::Missing | Status::Conflict(_) => {
                if is_create && !creates {
                    outcome.skipped.push(reg.id.clone());
                    continue;
                }
                let existing = ws.read(&reg.path)?;
                let mut new =
                    reg.apply(existing.as_deref())
                        .map_err(|reason| ScaffoldError::Apply {
                            id: reg.id.clone(),
                            reason,
                        })?;
                // An edited Rust file is left as rustfmt would (it sorts
                // `pub mod` lines), so `cargo fmt --check` stays green.
                if !is_create
                    && reg.path.extension().is_some_and(|e| e == "rs")
                    && let Some(formatted) = crate::fmt::rustfmt(&new)
                {
                    new = formatted;
                }
                ws.write(&reg.path, new);
                if is_create {
                    outcome.created.push(reg.path.clone());
                } else {
                    outcome.edited.push(reg.id.clone());
                }
            }
            Status::Unresolvable(_) => unreachable!("blockers checked above"),
        }
    }
    Ok(outcome)
}

/// Re-parse every TOML file the plan touched, so a broken edit never reaches disk.
///
/// # Errors
/// A file no longer parses.
pub fn verify_toml(ws: &mut Workspace) -> Result<(), ScaffoldError> {
    for path in ws.changed() {
        if path.extension().is_some_and(|e| e == "toml")
            && let Some(text) = ws.read(&path)?
            && let Err(e) = text.parse::<toml::Table>()
        {
            return Err(ScaffoldError::Apply {
                id: path.display().to_string(),
                reason: format!("no longer valid TOML: {e}"),
            });
        }
    }
    Ok(())
}
