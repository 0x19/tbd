//! The edit primitives every registration reduces to, on plain text.
//!
//! A [`Registration`] is one thing a service needs in one file: a file to
//! create, or lines to insert next to an [`Anchor`], or text to splice into an
//! anchored line. Its [`Registration::status`] says whether it is already
//! there, so applying a plan twice is a no-op by construction.

use std::{fmt, path::PathBuf};

/// How a line is found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Match {
    /// The whole line, exactly.
    Exact(String),
    /// The line starts with this.
    Prefix(String),
    /// The line contains this.
    Contains(String),
    /// The last line that starts with this.
    LastPrefix(String),
}

impl Match {
    fn hits(&self, line: &str) -> bool {
        match self {
            Self::Exact(s) => line == s,
            Self::Prefix(s) | Self::LastPrefix(s) => line.starts_with(s),
            Self::Contains(s) => line.contains(s),
        }
    }
}

impl fmt::Display for Match {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Exact(s) => write!(f, "line {s:?}"),
            Self::Prefix(s) => write!(f, "line starting {s:?}"),
            Self::Contains(s) => write!(f, "line containing {s:?}"),
            Self::LastPrefix(s) => write!(f, "last line starting {s:?}"),
        }
    }
}

/// Where an edit lands: a line, searched only after `scope` when one is given.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Anchor {
    /// Only lines after the first line matching this are considered.
    pub scope: Option<Match>,
    /// The line itself.
    pub line: Match,
}

impl Anchor {
    /// An unscoped anchor. Unique in the file, unless it is a [`Match::LastPrefix`].
    #[must_use]
    pub fn line(line: Match) -> Self {
        Self { scope: None, line }
    }

    /// The first matching line after the scope line.
    #[must_use]
    pub fn scoped(scope: Match, line: Match) -> Self {
        Self {
            scope: Some(scope),
            line,
        }
    }

    /// Index of the first line the scope allows searching from.
    fn start(&self, lines: &[String]) -> Result<usize, String> {
        match &self.scope {
            None => Ok(0),
            Some(s) => lines
                .iter()
                .position(|l| s.hits(l))
                .map(|i| i + 1)
                .ok_or_else(|| format!("scope {s} not found")),
        }
    }

    /// Resolve to a line index.
    ///
    /// # Errors
    /// The scope or the line is not found, or an unscoped line matches more
    /// than once.
    pub fn resolve(&self, lines: &[String]) -> Result<usize, String> {
        let start = self.start(lines)?;
        let hits: Vec<usize> = (start..lines.len())
            .filter(|&i| self.line.hits(&lines[i]))
            .collect();
        match (hits.as_slice(), &self.line, self.scope.is_some()) {
            ([], ..) => Err(format!("{} not found", self.line)),
            (all, Match::LastPrefix(_), _) => Ok(all[all.len() - 1]),
            ([one], ..) => Ok(*one),
            (first, _, true) => Ok(first[0]),
            (many, _, false) => Err(format!("{} matches {} lines", self.line, many.len())),
        }
    }
}

/// Where in an anchored line a splice goes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpliceAt {
    /// Append at the end of the line.
    End,
    /// Insert before the first occurrence of this text in the line.
    Before(String),
    /// Insert after the first occurrence of this text in the line.
    After(String),
}

/// One edit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Edit {
    /// Create a file with this content.
    Create {
        /// Whole file.
        content: String,
    },
    /// Insert lines after the anchor.
    InsertAfter {
        /// Where.
        anchor: Anchor,
        /// What, one or more lines.
        lines: String,
    },
    /// Insert lines before the anchor.
    InsertBefore {
        /// Where.
        anchor: Anchor,
        /// What, one or more lines.
        lines: String,
    },
    /// Insert text into the anchored line.
    Splice {
        /// Which line.
        anchor: Anchor,
        /// Where in it.
        at: SpliceAt,
        /// What.
        text: String,
    },
    /// Append lines at the end of the file.
    AppendEof {
        /// What, one or more lines.
        lines: String,
    },
}

impl Edit {
    fn anchor(&self) -> Option<&Anchor> {
        match self {
            Self::InsertAfter { anchor, .. }
            | Self::InsertBefore { anchor, .. }
            | Self::Splice { anchor, .. } => Some(anchor),
            Self::Create { .. } | Self::AppendEof { .. } => None,
        }
    }

    /// The lines an insert adds, for dry-run output.
    #[must_use]
    pub fn added(&self) -> Vec<String> {
        match self {
            Self::Create { content } => vec![format!("({} lines)", content.lines().count())],
            Self::InsertAfter { lines, .. }
            | Self::InsertBefore { lines, .. }
            | Self::AppendEof { lines } => split_lines(lines),
            Self::Splice { text, .. } => vec![format!("…{text}")],
        }
    }
}

/// One thing a service needs in one file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Registration {
    /// Stable id, e.g. `envoy:cluster`.
    pub id: String,
    /// File, relative to the workspace root.
    pub path: PathBuf,
    /// Text whose presence (in the anchor's scope, if any) means the edit was
    /// applied; ending in a newline, it must match a whole line. Must not
    /// depend on the port. Splices need none.
    pub needle: Option<String>,
    /// The edit.
    pub edit: Edit,
}

/// Whether a registration is applied.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    /// Already there.
    Present,
    /// Not there, and the edit can be applied.
    Missing,
    /// A file to create exists with different content.
    Conflict(String),
    /// The anchor could not be found.
    Unresolvable(String),
}

fn split_lines(s: &str) -> Vec<String> {
    s.strip_suffix('\n')
        .unwrap_or(s)
        .split('\n')
        .map(str::to_owned)
        .collect()
}

fn lines_of(text: &str) -> Vec<String> {
    text.lines().map(str::to_owned).collect()
}

fn join(lines: &[String], trailing_newline: bool) -> String {
    let mut out = lines.join("\n");
    if trailing_newline {
        out.push('\n');
    }
    out
}

impl Registration {
    /// Status against the file's current content (`None` when the file does not exist).
    #[must_use]
    pub fn status(&self, existing: Option<&str>) -> Status {
        if let Edit::Create { content } = &self.edit {
            return match existing {
                None => Status::Missing,
                Some(e) if e == content => Status::Present,
                Some(_) => Status::Conflict("exists with different content".into()),
            };
        }
        let Some(text) = existing else {
            return Status::Unresolvable("file does not exist".into());
        };
        let lines = lines_of(text);
        if self.is_present(&lines) {
            return Status::Present;
        }
        match self.edit.anchor() {
            Some(a) => match a.resolve(&lines) {
                Ok(_) => Status::Missing,
                Err(e) => Status::Unresolvable(e),
            },
            None => Status::Missing,
        }
    }

    fn is_present(&self, lines: &[String]) -> bool {
        if let Edit::Splice { anchor, text, .. } = &self.edit {
            return anchor
                .resolve(lines)
                .is_ok_and(|i| lines[i].contains(text.as_str()));
        }
        let Some(needle) = &self.needle else {
            return false;
        };
        let start = self
            .edit
            .anchor()
            .and_then(|a| a.start(lines).ok())
            .unwrap_or(0);
        // A needle ending in a newline must match a whole line, so `- name: x`
        // is not found inside `- name: x-lb`.
        match needle.strip_suffix('\n') {
            Some(whole) => lines[start..].iter().any(|l| l == whole),
            None => lines[start..].iter().any(|l| l.contains(needle.as_str())),
        }
    }

    /// Apply the edit to the file's current content and return the new content.
    ///
    /// # Errors
    /// The anchor could not be resolved, or the splice text is absent from the line.
    pub fn apply(&self, existing: Option<&str>) -> Result<String, String> {
        match &self.edit {
            Edit::Create { content } => Ok(content.clone()),
            Edit::AppendEof { lines } => {
                let mut text = existing.unwrap_or_default().to_owned();
                if !text.is_empty() && !text.ends_with('\n') {
                    text.push('\n');
                }
                text.push_str(lines);
                if !text.ends_with('\n') {
                    text.push('\n');
                }
                Ok(text)
            }
            Edit::InsertAfter { anchor, lines } | Edit::InsertBefore { anchor, lines } => {
                let text = existing.ok_or("file does not exist")?;
                let mut all = lines_of(text);
                let i = anchor.resolve(&all)?;
                let at = if matches!(self.edit, Edit::InsertAfter { .. }) {
                    i + 1
                } else {
                    i
                };
                let new = split_lines(lines);
                all.splice(at..at, new);
                Ok(join(&all, text.ends_with('\n')))
            }
            Edit::Splice {
                anchor,
                at,
                text: t,
            } => {
                let text = existing.ok_or("file does not exist")?;
                let mut all = lines_of(text);
                let i = anchor.resolve(&all)?;
                let line = &all[i];
                let new = match at {
                    SpliceAt::End => format!("{line}{t}"),
                    SpliceAt::Before(m) => {
                        let p = line
                            .find(m.as_str())
                            .ok_or_else(|| format!("{m:?} not in line {i}"))?;
                        format!("{}{t}{}", &line[..p], &line[p..])
                    }
                    SpliceAt::After(m) => {
                        let p = line
                            .find(m.as_str())
                            .ok_or_else(|| format!("{m:?} not in line {i}"))?
                            + m.len();
                        format!("{}{t}{}", &line[..p], &line[p..])
                    }
                };
                all[i] = new;
                Ok(join(&all, text.ends_with('\n')))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reg(edit: Edit, needle: Option<&str>) -> Registration {
        Registration {
            id: "t".into(),
            path: PathBuf::from("f"),
            needle: needle.map(str::to_owned),
            edit,
        }
    }

    #[test]
    fn insert_after_is_idempotent() {
        let r = reg(
            Edit::InsertAfter {
                anchor: Anchor::line(Match::Exact("  - protocol".into())),
                lines: "  - ledger\n".into(),
            },
            Some("  - ledger"),
        );
        let text = "resources:\n  - engine\n  - protocol\n  - envoy\n";
        assert_eq!(r.status(Some(text)), Status::Missing);
        let once = r.apply(Some(text)).unwrap();
        assert_eq!(
            once,
            "resources:\n  - engine\n  - protocol\n  - ledger\n  - envoy\n"
        );
        assert_eq!(r.status(Some(&once)), Status::Present);
    }

    #[test]
    fn scoped_anchor_takes_first_match_after_scope() {
        let text = "a:\n  - x\nb:\n  - x\n";
        let a = Anchor::scoped(Match::Exact("b:".into()), Match::Exact("  - x".into()));
        assert_eq!(a.resolve(&lines_of(text)), Ok(3));
        let unscoped = Anchor::line(Match::Exact("  - x".into()));
        assert!(unscoped.resolve(&lines_of(text)).is_err());
        let last = Anchor::line(Match::LastPrefix("  - x".into()));
        assert_eq!(last.resolve(&lines_of(text)), Ok(3));
    }

    #[test]
    fn splice_is_present_when_the_line_has_the_text() {
        let r = reg(
            Edit::Splice {
                anchor: Anchor::line(Match::Prefix("default-members = [".into())),
                at: SpliceAt::Before("]".into()),
                text: ", \"crates/ledger\"".into(),
            },
            None,
        );
        let text = "default-members = [\"crates/engine\"]\n";
        let out = r.apply(Some(text)).unwrap();
        assert_eq!(
            out,
            "default-members = [\"crates/engine\", \"crates/ledger\"]\n"
        );
        assert_eq!(r.status(Some(&out)), Status::Present);
        assert_eq!(
            r.status(Some("nothing\n")),
            Status::Unresolvable("line starting \"default-members = [\" not found".into())
        );
    }

    #[test]
    fn create_reports_conflicts_and_append_ends_with_newline() {
        let c = reg(
            Edit::Create {
                content: "x\n".into(),
            },
            None,
        );
        assert_eq!(c.status(None), Status::Missing);
        assert_eq!(c.status(Some("x\n")), Status::Present);
        assert!(matches!(c.status(Some("y\n")), Status::Conflict(_)));
        let a = reg(
            Edit::AppendEof {
                lines: "K=1".into(),
            },
            Some("K="),
        );
        assert_eq!(a.apply(Some("A=0")).unwrap(), "A=0\nK=1\n");
        assert_eq!(a.status(Some("A=0\nK=1\n")), Status::Present);
    }
}
