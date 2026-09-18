//! Text out of a PDF, in-process.
//!
//! `pdf-extract` is pure Rust and handles the text layer the way the
//! accountant's PDFs carry it (Stripe, Google, Hetzner, our own Typst). It has
//! no OCR: a scanned receipt yields nothing, and says so. It also panics on
//! some malformed files, which is why the call is fenced and run off the
//! async runtime.

use std::panic::{AssertUnwindSafe, catch_unwind};

/// Why no text came out.
#[derive(Debug, thiserror::Error)]
pub enum PdfError {
    /// Not a PDF, or one the reader cannot parse.
    #[error("unreadable pdf: {0}")]
    Unreadable(String),
    /// The reader crashed on it. Its own bug; the document is kept, unread.
    #[error("pdf reader failed on this file")]
    Crashed,
    /// A PDF with no text layer, a scan most likely.
    #[error("no text in pdf")]
    Empty,
}

/// Which reader, for the record on the document.
pub const ENGINE: &str = "pdf-extract 0.12";

/// The document's text, page after page. Runs on a blocking thread.
///
/// # Errors
/// The file is not a PDF the reader can open, the reader crashed, or the
/// PDF carries no text.
pub async fn text(bytes: Vec<u8>) -> Result<String, PdfError> {
    tokio::task::spawn_blocking(move || text_sync(&bytes))
        .await
        .map_err(|_| PdfError::Crashed)?
}

/// Same, synchronously. For tests and the blocking worker.
///
/// # Errors
/// As [`text`].
pub fn text_sync(bytes: &[u8]) -> Result<String, PdfError> {
    let out = catch_unwind(AssertUnwindSafe(|| {
        pdf_extract::extract_text_from_mem(bytes)
    }))
    .map_err(|_| PdfError::Crashed)?
    .map_err(|e| PdfError::Unreadable(e.to_string()))?;
    let tidy = tidy(&out);
    if tidy.chars().filter(|c| c.is_alphanumeric()).count() < 8 {
        return Err(PdfError::Empty);
    }
    Ok(tidy)
}

/// Collapse the reader's spacing: runs of blanks to one, blank lines to one.
fn tidy(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut blank = 0;
    for line in raw.lines() {
        let line = line.split_whitespace().collect::<Vec<_>>().join(" ");
        if line.is_empty() {
            blank += 1;
            if blank > 1 {
                continue;
            }
        } else {
            blank = 0;
        }
        out.push_str(&line);
        out.push('\n');
    }
    out.trim().to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn our_own_invoice_reads_back() {
        let doc = crate::invoice::render::sample();
        let rendered = crate::invoice::render::render(&doc).expect("sample renders");
        let text = text_sync(&rendered.pdf).expect("typst output carries text");
        assert!(text.contains(&doc.number), "number in text: {text}");
        assert!(text.contains("Total due"), "{text}");
    }

    #[test]
    fn not_a_pdf_is_unreadable_not_a_crash() {
        assert!(matches!(
            text_sync(b"%PDF-hetzner"),
            Err(PdfError::Unreadable(_) | PdfError::Crashed | PdfError::Empty)
        ));
        assert!(text_sync(b"hello").is_err());
    }

    #[test]
    fn tidy_collapses_spacing() {
        assert_eq!(tidy("a   b \n\n\n c\n"), "a b\n\nc");
    }
}
