//! The fields the full CV has and the public one does not.
//!
//! They come from a Kubernetes Secret (`CV_PRIVATE_JSON`) or a file outside
//! the repository, never from git, and they never appear in a log line: the
//! `Debug` form says how many references there are and nothing else.

use serde::{Deserialize, Serialize};

/// What only an approved reader sees.
#[derive(Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Private {
    /// A phone number, as it should be dialled.
    #[serde(default)]
    pub phone: String,
    /// A postal address, one line.
    #[serde(default)]
    pub address: String,
    /// People who will vouch, with how to reach them.
    #[serde(default)]
    pub references: Vec<Reference>,
    /// What the full CV says about a position beyond the public paragraph:
    /// matched to the public entry by company name.
    #[serde(default)]
    pub experience: Vec<PrivateExperience>,
    /// A summary that replaces the public one in the full CV; empty keeps
    /// the public one.
    #[serde(default)]
    pub summary: String,
    /// Lines added to the public "Selected work" in the full CV.
    #[serde(default)]
    pub achievements: Vec<String>,
}

/// The private half of one position.
#[derive(Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PrivateExperience {
    /// The company, exactly as the public entry names it.
    pub company: String,
    /// A paragraph under the public one; may be empty.
    #[serde(default)]
    pub body: String,
    /// Lines under the public highlights.
    #[serde(default)]
    pub highlights: Vec<String>,
}

impl std::fmt::Debug for PrivateExperience {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PrivateExperience")
            .field("company", &self.company)
            .field("highlights", &self.highlights.len())
            .finish_non_exhaustive()
    }
}

/// One reference.
#[derive(Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Reference {
    /// Their name.
    pub name: String,
    /// Who they are to the candidate.
    #[serde(default)]
    pub role: String,
    /// How to reach them.
    #[serde(default)]
    pub contact: String,
}

impl std::fmt::Debug for Private {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Private")
            .field("phone", &if self.phone.is_empty() { "" } else { "<set>" })
            .field(
                "address",
                &if self.address.is_empty() { "" } else { "<set>" },
            )
            .field("references", &self.references.len())
            .field("experience", &self.experience.len())
            .field(
                "summary",
                &if self.summary.is_empty() { "" } else { "<set>" },
            )
            .field("achievements", &self.achievements.len())
            .finish()
    }
}

impl std::fmt::Debug for Reference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Reference")
            .field("name", &"<set>")
            .finish_non_exhaustive()
    }
}

/// Why the private fields did not load.
#[derive(Debug, thiserror::Error)]
pub enum PrivateError {
    /// The JSON does not have the expected shape.
    #[error("private fields: {0}")]
    Parse(#[from] serde_json::Error),
    /// The file could not be read.
    #[error("private fields at {path}: {source}")]
    Read {
        /// The path that was tried.
        path: String,
        /// The I/O error.
        source: std::io::Error,
    },
}

/// The fields from the inline JSON when given, else from the file at `path`,
/// else none at all (the full CV then differs from the public one only by
/// the reader's name on it).
///
/// # Errors
/// The JSON is malformed, or the file cannot be read.
pub fn load(json: &str, path: &str) -> Result<Option<Private>, PrivateError> {
    let text = if !json.trim().is_empty() {
        json.to_owned()
    } else if !path.trim().is_empty() {
        std::fs::read_to_string(path).map_err(|source| PrivateError::Read {
            path: path.to_owned(),
            source,
        })?
    } else {
        return Ok(None);
    };
    Ok(Some(serde_json::from_str(&text)?))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn inline_json_wins_over_the_path_and_nothing_is_none() {
        let p = load(
            r#"{"phone":"+385 99 1","address":"Somewhere 1","references":[{"name":"A"}]}"#,
            "/nope",
        )
        .unwrap()
        .unwrap();
        assert_eq!(p.phone, "+385 99 1");
        assert_eq!(p.references.len(), 1);
        assert!(load("", "").unwrap().is_none());
        assert!(matches!(
            load("", "/definitely/not/here.json"),
            Err(PrivateError::Read { .. })
        ));
        assert!(matches!(load("{", ""), Err(PrivateError::Parse(_))));
    }

    /// A privacy claim is a test: the debug form of the private fields never
    /// carries a value, so a log line cannot leak one.
    #[test]
    fn the_debug_form_says_nothing_private() {
        let p = Private {
            phone: "+385 99 123 4567".into(),
            address: "Benčani 15A".into(),
            references: vec![Reference {
                name: "Ann Example".into(),
                role: "CTO".into(),
                contact: "ann@example.com".into(),
            }],
            experience: vec![PrivateExperience {
                company: "Acme".into(),
                body: "Ran the secret project.".into(),
                highlights: vec!["Carried 9 million widgets a second.".into()],
            }],
            summary: "Twenty years of secret projects.".into(),
            achievements: vec!["The secret widget.".into()],
        };
        let text = format!("{p:?}");
        for secret in [
            "secret",
            "4567",
            "Benčani",
            "Ann",
            "example.com",
            "secret project",
            "widgets",
        ] {
            assert!(!text.contains(secret), "{text}");
        }
    }
}
