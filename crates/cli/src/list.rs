//! `tbd service list`: every binary crate, and how registered it is.

use crate::{
    check, registry,
    repo::{RepoError, Workspace},
    scaffold,
    service::Service,
};

/// One row.
#[derive(Debug, Clone)]
pub struct Row {
    /// Crate directory name.
    pub name: String,
    /// Package name from its manifest.
    pub package: String,
    /// The scaffolded service, when the crate carries a marker.
    pub managed: Option<Service>,
    /// Present registrations out of the total, for managed crates.
    pub registered: Option<(usize, usize)>,
}

/// Every crate with a `[[bin]]`, except the CLI itself.
///
/// # Errors
/// `crates/` cannot be listed or a manifest cannot be read.
pub fn rows(ws: &mut Workspace) -> Result<Vec<Row>, RepoError> {
    let mut rows = Vec::new();
    for manifest in ws.crate_manifests()? {
        let Some(text) = ws.read(&manifest)? else {
            continue;
        };
        if !text.contains("[[bin]]") {
            continue;
        }
        let name = manifest
            .parent()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if name == "cli" {
            continue;
        }
        let package = text
            .lines()
            .find_map(|l| l.strip_prefix("name = "))
            .map(|v| v.trim_matches('"').to_owned())
            .unwrap_or_default();
        let managed = check::service_from_marker(ws, &name)?;
        let registered = match &managed {
            Some(service) => registry::registrations(service)
                .ok()
                .and_then(|regs| scaffold::plan(ws, regs).ok())
                .map(|plan| {
                    let (present, ..) = plan.counts();
                    (present, plan.items.len())
                }),
            None => None,
        };
        rows.push(Row {
            name,
            package,
            managed,
            registered,
        });
    }
    Ok(rows)
}
