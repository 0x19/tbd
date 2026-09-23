//! Typst as an embedded PDF engine, shared by every service that sets a page.
//!
//! A template is compiled into the binary together with the fonts and any
//! file it reads (a mark, a data file), so a render needs no filesystem, no
//! package and no network, and is the same on every machine. Data reaches the
//! template as `sys.inputs`, built from JSON so a template holds layout and
//! nothing else. The PDF's id and creation date can be pinned to the
//! document, which makes rendering the same document twice yield the same
//! bytes -- a store can then dedupe on content and a re-render that differs
//! is a *detected* change.
//!
//! The finance invoice and the mail printer were the first users; the CV is
//! the third. Rendering is CPU-bound: call it from `spawn_blocking`.

use chrono::{DateTime, Datelike as _, Timelike as _, Utc};
use typst::{
    foundations::{Array, Datetime, Dict, Smart, Str, Value},
    syntax::{DiagSpanKind, Source},
};
use typst_as_lib::{TypstEngine, TypstTemplateMainFile, conversions::IntoSource as _};
use typst_layout::PagedDocument;
pub use typst_pdf::PdfOptions;
use typst_pdf::Timestamp;

/// Inter, the four weights every document is set in. OFL; the licence is
/// beside the files.
pub static FONTS: [&[u8]; 4] = [
    include_bytes!("../assets/fonts/Inter-Regular.otf"),
    include_bytes!("../assets/fonts/Inter-Medium.otf"),
    include_bytes!("../assets/fonts/Inter-SemiBold.otf"),
    include_bytes!("../assets/fonts/Inter-Bold.otf"),
];

/// A template: its name (so a diagnostic can point at a line of it) and its
/// source, `include_str!`ed by the caller.
#[derive(Debug, Clone, Copy)]
pub struct Template {
    /// The file name a diagnostic names, `invoice.typ` for instance.
    pub name: &'static str,
    /// The Typst source.
    pub source: &'static str,
}

/// Why a render failed. Never `Internal` for a timeout; callers map it.
#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    /// The template did not compile against this input.
    #[error("typst: {0}")]
    Compile(String),
    /// The PDF could not be written.
    #[error("pdf: {0}")]
    Pdf(String),
}

/// A rendered document.
#[derive(Debug, Clone)]
pub struct Rendered {
    /// The PDF.
    pub pdf: Vec<u8>,
    /// How many pages it took.
    pub pages: usize,
}

/// One template, ready to render. Build it once (`LazyLock`): parsing the
/// fonts is the expensive part.
pub struct Engine {
    name: &'static str,
    source: Source,
    inner: TypstEngine<TypstTemplateMainFile>,
}

impl std::fmt::Debug for Engine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Engine")
            .field("template", &self.name)
            .finish_non_exhaustive()
    }
}

impl Engine {
    /// An engine for `template`, with `files` the template may read by name
    /// (`image("mark.svg")`, `json("cv.json")`) and the fonts compiled in.
    #[must_use]
    pub fn new(
        template: Template,
        files: impl IntoIterator<Item = (&'static str, &'static [u8])>,
    ) -> Self {
        let source: Source = (template.name, template.source).into_source();
        let engine = TypstEngine::builder()
            .main_file(source.clone())
            .fonts(FONTS)
            .with_static_file_resolver(files)
            .build();
        Self {
            name: template.name,
            source,
            inner: engine,
        }
    }

    /// Render `inputs` (a JSON object; each key becomes one of `sys.inputs`)
    /// to a PDF with `options` -- [`pinned`] for a document that must render
    /// to the same bytes every time, [`unpinned`] otherwise.
    ///
    /// # Errors
    /// The template fails on this input, or the PDF cannot be written.
    pub fn render(
        &self,
        inputs: &serde_json::Value,
        options: &PdfOptions,
    ) -> Result<Rendered, RenderError> {
        let dict = match to_typst(inputs) {
            Value::Dict(d) => d,
            other => {
                let mut d = Dict::new();
                d.insert(Str::from("doc"), other);
                d
            }
        };
        let compiled = self
            .inner
            .compile_with_input::<_, PagedDocument>(dict)
            .output
            .map_err(|e| RenderError::Compile(self.describe(&e)))?;
        let pdf =
            typst_pdf::pdf(&compiled, options).map_err(|e| RenderError::Pdf(format!("{e:?}")))?;
        Ok(Rendered {
            pdf,
            pages: compiled.pages().len(),
        })
    }

    /// A compile failure with every diagnostic pointed at its line.
    fn describe(&self, error: &typst_as_lib::TypstAsLibError) -> String {
        match error {
            typst_as_lib::TypstAsLibError::TypstSource(diagnostics) => diagnostics
                .iter()
                .map(|d| {
                    let start = match d.span.get() {
                        DiagSpanKind::Number { id, num, sub_range } if id == self.source.id() => {
                            self.source.range(num, sub_range).map(|r| r.start)
                        }
                        DiagSpanKind::Range { id, range } if id == self.source.id() => {
                            Some(range.start)
                        }
                        _ => None,
                    };
                    let line = start
                        .and_then(|b| self.source.lines().byte_to_line(b))
                        .map_or_else(|| "?".to_owned(), |l| (l + 1).to_string());
                    let hints: Vec<String> = d.hints.iter().map(|h| h.v.to_string()).collect();
                    format!(
                        "{}:{line}: {}{}",
                        self.name,
                        d.message,
                        if hints.is_empty() {
                            String::new()
                        } else {
                            format!(" ({})", hints.join("; "))
                        }
                    )
                })
                .collect::<Vec<_>>()
                .join("\n"),
            other => other.to_string(),
        }
    }
}

/// Options that pin the PDF's id and creation time to the document, so the
/// same document renders to the same bytes. The PDF's own clock is UTC.
#[must_use]
pub fn pinned(ident: &str, at: DateTime<Utc>) -> PdfOptions {
    let stamp = Datetime::from_ymd_hms(
        at.year(),
        u8::try_from(at.month()).unwrap_or(1),
        u8::try_from(at.day()).unwrap_or(1),
        u8::try_from(at.hour()).unwrap_or(0),
        u8::try_from(at.minute()).unwrap_or(0),
        u8::try_from(at.second()).unwrap_or(0),
    )
    .and_then(|d| Timestamp::new_utc(d).into());
    PdfOptions {
        ident: Smart::Custom(ident.to_owned()),
        timestamp: stamp,
        ..PdfOptions::default()
    }
}

/// Options for a document that need not be reproducible.
#[must_use]
pub fn unpinned() -> PdfOptions {
    PdfOptions::default()
}

/// JSON to Typst values, structurally.
#[must_use]
pub fn to_typst(v: &serde_json::Value) -> Value {
    match v {
        serde_json::Value::Null => Value::None,
        serde_json::Value::Bool(b) => Value::Bool(*b),
        serde_json::Value::Number(n) => n
            .as_i64()
            .map_or_else(|| Value::Float(n.as_f64().unwrap_or(0.0)), Value::Int),
        serde_json::Value::String(s) => Value::Str(Str::from(s.as_str())),
        serde_json::Value::Array(a) => Value::Array(a.iter().map(to_typst).collect::<Array>()),
        serde_json::Value::Object(o) => Value::Dict(
            o.iter()
                .map(|(k, v)| (Str::from(k.as_str()), to_typst(v)))
                .collect::<Dict>(),
        ),
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use std::sync::LazyLock;

    use super::*;

    static PAGE: LazyLock<Engine> = LazyLock::new(|| {
        Engine::new(
            Template {
                name: "page.typ",
                source: "#set text(font: \"Inter\")\n#let d = sys.inputs.doc\n= #d.title\n#json(\"data.json\").at(\"line\")\n",
            },
            [("data.json", br#"{"line":"from the file"}"# as &[u8])],
        )
    });

    #[test]
    fn a_template_reads_its_inputs_and_its_files_and_pins_to_the_same_bytes() {
        let inputs = serde_json::json!({"doc": {"title": "Hello"}});
        let at = DateTime::parse_from_rfc3339("2026-09-23T10:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let a = PAGE.render(&inputs, &pinned("test-1", at)).unwrap();
        let b = PAGE.render(&inputs, &pinned("test-1", at)).unwrap();
        assert!(a.pdf.starts_with(b"%PDF-"));
        assert_eq!(a.pages, 1);
        assert_eq!(
            a.pdf, b.pdf,
            "a pinned render is a pure function of its input"
        );
    }

    #[test]
    fn a_diagnostic_names_the_template_and_the_line() {
        let inputs = serde_json::json!({"doc": {"no_title": 1}});
        let err = PAGE.render(&inputs, &unpinned()).unwrap_err();
        let text = err.to_string();
        assert!(text.contains("page.typ:3:"), "{text}");
    }

    #[test]
    fn json_maps_to_typst_structurally() {
        let v = to_typst(&serde_json::json!({"a": [1, 2.5, "x", null, true]}));
        let Value::Dict(d) = v else {
            panic!("not a dict")
        };
        let Some(Value::Array(a)) = d.get("a").ok().cloned() else {
            panic!("no array")
        };
        assert_eq!(a.len(), 5);
    }
}
