//! The full CV as a PDF, rendered for one reader.
//!
//! The template and the public facts (`assets/cv.typ`, `assets/cv.json`, both
//! written from the site's data file by `mise run www:cv`) are compiled into
//! the binary; the private fields and the reader arrive as `sys.inputs` and
//! never touch a file. Without either the render is the public CV, byte for
//! byte what the site serves, which is the test that the two never drift.

use std::sync::LazyLock;

use chrono::{DateTime, Utc};
use tbd_render::{Engine, Template, pinned};
pub use tbd_render::{RenderError, Rendered};

use crate::private::Private;

static TEMPLATE: &str = include_str!("../assets/cv.typ");
static DATA: &str = include_str!("../assets/cv.json");

/// Built once: parsing the fonts is the expensive part.
static ENGINE: LazyLock<Engine> = LazyLock::new(|| {
    Engine::new(
        Template {
            name: "cv.typ",
            source: TEMPLATE,
        },
        [("cv.json", DATA.as_bytes())],
    )
});

/// Who the document is prepared for; on every page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reader {
    /// Their name, as the token carried it.
    pub name: String,
    /// Their e-mail address.
    pub email: String,
    /// The day, `2026-09-23`.
    pub date: String,
}

/// Render the CV: public when both are `None`, the full one for a reader
/// otherwise. CPU-bound: call from `spawn_blocking`.
///
/// # Errors
/// The template fails on this input, or the PDF cannot be written.
pub fn render(
    private: Option<&Private>,
    reader: Option<&Reader>,
    ident: &str,
    at: DateTime<Utc>,
) -> Result<Rendered, RenderError> {
    let mut inputs = serde_json::Map::new();
    if let Some(p) = private {
        // As a string: the template decodes it, which keeps one code path
        // for the library and for `typst --input` on the command line.
        inputs.insert(
            "private".into(),
            serde_json::Value::String(serde_json::to_string(p).unwrap_or_default()),
        );
    }
    if let Some(r) = reader {
        inputs.insert(
            "reader".into(),
            serde_json::json!({ "name": r.name, "email": r.email, "date": r.date }),
        );
    }
    ENGINE.render(&serde_json::Value::Object(inputs), &pinned(ident, at))
}

/// Parse the fonts and lay the public CV out once, so the first download
/// does not pay for it. The result is discarded.
pub fn warm() {
    if let Err(e) = render(None, None, "warm", Utc::now()) {
        tracing::warn!(error = %e, "the CV template does not render");
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use crate::private::{PrivateExperience, Reference};

    fn at() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-09-23T10:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    fn private() -> Private {
        Private {
            phone: "+385 99 000 0001".into(),
            address: "Benčani 15A, Saršoni".into(),
            references: vec![Reference {
                name: "Ann Example".into(),
                role: "CTO, Example Ltd".into(),
                contact: "ann@example.com".into(),
            }],
            experience: vec![PrivateExperience {
                company: "Tenderly".into(),
                body: "Built the widget balancer that fronts every network.".into(),
                highlights: vec!["It carries nine million widgets a second.".into()],
            }],
        }
    }

    fn text(pdf: &[u8]) -> String {
        pdf_extract::extract_text_from_mem(pdf)
            .unwrap()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    #[test]
    fn the_public_render_is_pinned_and_carries_nothing_private() {
        let a = render(None, None, "cv-public", at()).unwrap();
        let b = render(None, None, "cv-public", at()).unwrap();
        assert!(a.pdf.starts_with(b"%PDF-"));
        assert_eq!(a.pdf, b.pdf, "the public CV is a pure function of the data");
        assert!(a.pages >= 1);
        let t = text(&a.pdf);
        assert!(t.contains("Nevio Vesic"), "{t}");
        for absent in [
            "+385 99 000 0001",
            "Saršoni",
            "Ann Example",
            "Prepared for",
            "References",
            "widget balancer",
            "nine million widgets",
        ] {
            assert!(!t.contains(absent), "public render carries {absent:?}");
        }
    }

    /// A privacy claim is a test: the full render carries the private fields
    /// and names its reader on the page; the public one never does.
    #[test]
    fn the_full_render_names_the_reader_on_every_page_and_shows_the_private_fields() {
        let reader = Reader {
            name: "Rita Reader".into(),
            email: "rita@example.org".into(),
            date: "2026-09-23".into(),
        };
        let full = render(Some(&private()), Some(&reader), "cv-rita", at()).unwrap();
        let t = text(&full.pdf);
        for present in [
            "+385 99 000 0001",
            "Benčani 15A",
            "Ann Example",
            "CTO, Example Ltd",
            "Prepared for Rita Reader",
            "rita@example.org",
            "2026-09-23",
            "widget balancer",
            "nine million widgets",
        ] {
            assert!(t.contains(present), "full render lacks {present:?}: {t}");
        }
        assert_eq!(
            t.matches("Prepared for Rita Reader").count(),
            full.pages,
            "the reader's name is on every page"
        );
        assert!(full.pdf.len() < 1_000_000, "{} bytes", full.pdf.len());
    }
}
