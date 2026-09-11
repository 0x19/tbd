//! `tbd service check`: is every registration for a service present?

use std::path::Path;

use crate::{
    edit::Status,
    registry,
    repo::{RepoError, Workspace},
    scaffold,
    service::Service,
};

/// One line of a report.
#[derive(Debug, Clone)]
pub struct Line {
    /// Registration id.
    pub id: String,
    /// File.
    pub path: String,
    /// Status.
    pub status: Status,
    /// A generated file that exists with other content (see `Item::is_diverged`).
    pub diverged: bool,
}

/// The report for one service.
#[derive(Debug, Clone, Default)]
pub struct Report {
    /// One line per registration.
    pub lines: Vec<Line>,
}

impl Report {
    /// Lines that are not present. A diverged generated file is not a
    /// problem: the service has evolved past its template, which is expected.
    pub fn problems(&self) -> impl Iterator<Item = &Line> {
        self.lines
            .iter()
            .filter(|l| !l.diverged && l.status != Status::Present)
    }

    /// Generated files that exist with other content.
    pub fn diverged(&self) -> impl Iterator<Item = &Line> {
        self.lines.iter().filter(|l| l.diverged)
    }
}

/// Read the marker of a scaffolded crate.
///
/// # Errors
/// The crate's `CLAUDE.md` cannot be read.
pub fn service_from_marker(ws: &mut Workspace, name: &str) -> Result<Option<Service>, RepoError> {
    let path = Path::new("crates").join(name).join("CLAUDE.md");
    Ok(ws
        .read(&path)?
        .and_then(|text| text.lines().next().and_then(Service::from_marker)))
}

/// Check every registration.
///
/// # Errors
/// A template fails to render or a file cannot be read.
pub fn check(ws: &mut Workspace, service: &Service) -> Result<Report, scaffold::ScaffoldError> {
    let regs = registry::registrations(service).map_err(|e| scaffold::ScaffoldError::Apply {
        id: "template".into(),
        reason: e.to_string(),
    })?;
    let plan = scaffold::plan(ws, regs)?;
    Ok(Report {
        lines: plan
            .items
            .into_iter()
            .map(|i| Line {
                diverged: i.is_diverged(),
                id: i.registration.id,
                path: i.registration.path.display().to_string(),
                status: i.status,
            })
            .collect(),
    })
}
