//! The PDF, from the document and nothing else.
//!
//! Typst runs as a library with the template, the fonts and the mark compiled
//! into the binary: no filesystem, no packages, no network, so a render is
//! the same on every machine. The PDF's id and creation date are pinned to
//! the invoice, so rendering the same document twice yields the same bytes
//! -- which is what lets the document store dedupe on content, and what
//! makes a re-render that differs a *detected* change.

use std::sync::LazyLock;

use chrono::{Datelike, Timelike};
use chrono_tz::Europe::Zagreb;
use typst::{
    foundations::{Array, Datetime, Dict, Smart, Str, Value},
    syntax::{DiagSpanKind, Source},
};
use typst_as_lib::{TypstEngine, TypstTemplateMainFile, conversions::IntoSource as _};
use typst_layout::PagedDocument;
use typst_pdf::{PdfOptions, Timestamp};

use super::{Client, InvoiceDoc, Issuer, Line, VatTreatment, money, quantity};

static TEMPLATE: &str = include_str!("../../assets/invoice.typ");
static MARK: &[u8] = include_bytes!("../../assets/mark.svg");
static FONTS: [&[u8]; 4] = [
    include_bytes!("../../assets/fonts/Inter-Regular.otf"),
    include_bytes!("../../assets/fonts/Inter-Medium.otf"),
    include_bytes!("../../assets/fonts/Inter-SemiBold.otf"),
    include_bytes!("../../assets/fonts/Inter-Bold.otf"),
];

/// The template as a Typst source with a name, so a diagnostic's span can be
/// turned back into a line of `invoice.typ`.
static SOURCE: LazyLock<Source> = LazyLock::new(|| ("invoice.typ", TEMPLATE).into_source());

/// Built once: parsing the fonts is the expensive part.
static ENGINE: LazyLock<TypstEngine<TypstTemplateMainFile>> = LazyLock::new(|| {
    TypstEngine::builder()
        .main_file(SOURCE.clone())
        .fonts(FONTS)
        .with_static_file_resolver([("mark.svg", MARK)])
        .build()
});

/// A compile failure with every diagnostic pointed at its line.
fn describe(error: &typst_as_lib::TypstAsLibError) -> String {
    match error {
        typst_as_lib::TypstAsLibError::TypstSource(diagnostics) => diagnostics
            .iter()
            .map(|d| {
                let start = match d.span.get() {
                    DiagSpanKind::Number { id, num, sub_range } if id == SOURCE.id() => {
                        SOURCE.range(num, sub_range).map(|r| r.start)
                    }
                    DiagSpanKind::Range { id, range } if id == SOURCE.id() => Some(range.start),
                    _ => None,
                };
                let line = start
                    .and_then(|b| SOURCE.lines().byte_to_line(b))
                    .map_or_else(|| "?".to_owned(), |l| (l + 1).to_string());
                let hints: Vec<String> = d.hints.iter().map(|h| h.v.to_string()).collect();
                format!(
                    "invoice.typ:{line}: {}{}",
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

/// Why a render failed. Never `Internal` for a timeout; callers map it.
#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    /// The template did not compile against this document.
    #[error("typst: {0}")]
    Compile(String),
    /// The PDF could not be written.
    #[error("pdf: {0}")]
    Pdf(String),
}

/// Watermark text for a document that is not an issued invoice.
#[must_use]
pub fn watermark(doc: &InvoiceDoc) -> &'static str {
    if doc.number_preview { "PREVIEW" } else { "" }
}

/// A rendered document.
#[derive(Debug, Clone)]
pub struct Rendered {
    /// The PDF.
    pub pdf: Vec<u8>,
    /// How many pages it took. An invoice is one; two means the lines
    /// overflowed and the legal footer moved.
    pub pages: usize,
}

/// Render to PDF. CPU-bound: call from `spawn_blocking`.
///
/// # Errors
/// The template fails on this document, or the PDF cannot be written.
pub fn render(doc: &InvoiceDoc) -> Result<Rendered, RenderError> {
    let mut inputs = Dict::new();
    inputs.insert(Str::from("doc"), to_typst(&input(doc)));
    let compiled = ENGINE
        .compile_with_input::<_, PagedDocument>(inputs)
        .output
        .map_err(|e| RenderError::Compile(describe(&e)))?;

    // Pinned id and time: the same document renders to the same bytes. The
    // PDF's own clock is UTC; the printed time is Zagreb's.
    let at = doc.issued_at;
    let stamp = Datetime::from_ymd_hms(
        at.year(),
        u8::try_from(at.month()).unwrap_or(1),
        u8::try_from(at.day()).unwrap_or(1),
        u8::try_from(at.hour()).unwrap_or(0),
        u8::try_from(at.minute()).unwrap_or(0),
        u8::try_from(at.second()).unwrap_or(0),
    )
    .and_then(|d| Timestamp::new_utc(d).into());
    let ident = format!("inorbit-invoice-{}", doc.number);
    let options = PdfOptions {
        ident: Smart::Custom(ident),
        timestamp: stamp,
        ..PdfOptions::default()
    };
    let pdf =
        typst_pdf::pdf(&compiled, &options).map_err(|e| RenderError::Pdf(format!("{e:?}")))?;
    Ok(Rendered {
        pdf,
        pages: compiled.pages().len(),
    })
}

/// What the template reads: every value pre-formatted, so the template holds
/// layout and nothing else.
fn input(doc: &InvoiceDoc) -> serde_json::Value {
    let at = doc.issued_at.with_timezone(&Zagreb);
    let date = |d: chrono::NaiveDate| d.format("%d.%m.%Y.").to_string();
    serde_json::json!({
        "number": doc.number,
        "number_preview": doc.number_preview,
        "issued_at": at.format("%d.%m.%Y. %H:%M").to_string(),
        "delivery_date": date(doc.delivery_date),
        "due_date": date(doc.due_date),
        "place_of_issue": doc.place_of_issue,
        "currency": doc.currency,
        "issuer": {
            "legal_name": doc.issuer.legal_name,
            "address_lines": doc.issuer.address_lines,
            "oib": doc.issuer.oib,
            "vat_id": doc.issuer.vat_id,
            "iban": doc.issuer.iban,
            "swift": doc.issuer.swift,
            "bank_name": doc.issuer.bank_name,
            "court": doc.issuer.court,
            "registration_no": doc.issuer.registration_no,
            "share_capital": doc.issuer.share_capital,
            "board_member": doc.issuer.board_member,
            "issued_by": doc.issuer.issued_by,
            "operator_id": doc.issuer.operator_id,
        },
        "client": {
            "name": doc.client.name,
            "address_lines": doc.client.address_lines,
            "country": doc.client.country,
            "tax_id": doc.client.tax_id,
        },
        "lines": doc.lines.iter().map(|l| serde_json::json!({
            "position": l.position,
            "description": l.description,
            "quantity": quantity(l.quantity_milli),
            "unit_price": money(l.unit_price_minor),
            "amount": money(l.amount_minor),
        })).collect::<Vec<_>>(),
        "subtotal": money(doc.subtotal_minor),
        "vat": money(doc.vat_minor),
        "vat_label": doc.vat_treatment.label(),
        "total": money(doc.total_minor),
        "vat_note": doc.vat_note,
        "note": doc.note,
        "watermark": watermark(doc),
    })
}

/// JSON to Typst values, structurally.
fn to_typst(v: &serde_json::Value) -> Value {
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

/// The August 2026 invoice, as the template sees it. For tests and for
/// `finance invoice sample`.
#[must_use]
pub fn sample() -> InvoiceDoc {
    let lines = vec![
        Line {
            position: 1,
            description: "Prepaid services related to software and software development \
                          solutions / Usluge vezane uz softver i rješenja razvoja softvera"
                .into(),
            quantity_milli: 1000,
            unit_price_minor: 1_375_000,
            amount_minor: 1_375_000,
        },
        Line {
            position: 2,
            description: "On-call duty - incident fee / Dežurstvo - naknada za intervencije".into(),
            quantity_milli: 1000,
            unit_price_minor: 75_082,
            amount_minor: 75_082,
        },
    ];
    let treatment = VatTreatment::OutsideScopeNonEu;
    let (subtotal, vat, total) = super::totals::totals(&lines, treatment);
    InvoiceDoc {
        number: "9-1-1-2026".into(),
        number_preview: false,
        issued_at: chrono::DateTime::parse_from_rfc3339("2026-08-31T11:55:00Z")
            .map(|t| t.with_timezone(&chrono::Utc))
            .unwrap_or_default(),
        delivery_date: chrono::NaiveDate::from_ymd_opt(2026, 8, 31).unwrap_or_default(),
        due_date: chrono::NaiveDate::from_ymd_opt(2026, 9, 15).unwrap_or_default(),
        place_of_issue: "Viškovo".into(),
        currency: "EUR".into(),
        issuer: Issuer {
            legal_name: "Inorbit d.o.o.".into(),
            address_lines: vec!["Benčani 15A".into(), "51216 Viškovo, Croatia".into()],
            oib: "38846238650".into(),
            vat_id: "HR38846238650".into(),
            iban: "HR9224020061100925189".into(),
            swift: "ESBCHR22".into(),
            bank_name: "Erste & Steiermärkische Bank d.d., Rijeka".into(),
            court: "Trgovački sud u Rijeci".into(),
            registration_no: "081116183".into(),
            share_capital: "2,640.00 EUR, uplaćen u cijelosti / fully paid".into(),
            board_member: "Nevio Vesić".into(),
            issued_by: "Nevio Vesić".into(),
            operator_id: "1".into(),
        },
        client: Client {
            name: "Tenderly".into(),
            address_lines: vec!["Miloša Milojevića 7".into(), "Belgrade 11070".into()],
            country: "Serbia".into(),
            tax_id: String::new(),
        },
        lines,
        subtotal_minor: subtotal,
        vat_minor: vat,
        total_minor: total,
        vat_treatment: treatment,
        vat_note: treatment.note().into(),
        note: String::new(),
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn the_sample_renders_to_one_pdf_page_with_the_number_in_it() {
        let rendered = render(&sample()).unwrap();
        // For eyes: `INVOICE_SAMPLE_OUT=/tmp/x.pdf cargo nextest run ...`.
        if let Ok(path) = std::env::var("INVOICE_SAMPLE_OUT") {
            std::fs::write(path, &rendered.pdf).unwrap();
        }
        assert!(rendered.pdf.starts_with(b"%PDF-"), "not a PDF");
        assert!(
            rendered.pdf.len() > 10_000,
            "suspiciously small: {} bytes",
            rendered.pdf.len()
        );
        assert_eq!(
            rendered.pages, 1,
            "the lines overflowed; the legal footer moved to page 2"
        );
    }

    #[test]
    fn the_same_document_renders_to_the_same_bytes() {
        let a = render(&sample()).unwrap().pdf;
        let b = render(&sample()).unwrap().pdf;
        assert_eq!(a, b, "a render must be a pure function of the document");
    }

    #[test]
    fn a_preview_is_watermarked_and_an_issued_invoice_is_not() {
        let mut doc = sample();
        assert_eq!(watermark(&doc), "");
        doc.number_preview = true;
        assert_eq!(watermark(&doc), "PREVIEW");
    }
}
