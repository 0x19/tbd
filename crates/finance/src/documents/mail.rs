//! A receipt that came as a mail with no attachment: the mail itself,
//! printed to PDF.
//!
//! Medium, `OpenAI`, Audible, `PlayStation` and Namecheap send no file; the
//! receipt is the message. The accountant files paper, so the message
//! becomes a page: sender, subject, date and the text, set with the same
//! engine and fonts as the invoice. HTML is reduced to its text first; a
//! mail's layout is for a screen, and its words are what matter.

use std::sync::LazyLock;

use chrono::{DateTime, Utc};
use regex::Regex;
use typst::foundations::{Dict, Str, Value};
use typst_as_lib::{TypstEngine, TypstTemplateMainFile, conversions::IntoSource as _};
use typst_layout::PagedDocument;
use typst_pdf::PdfOptions;

use crate::invoice::render::FONTS;

static TEMPLATE: &str = include_str!("../../assets/mail.typ");

static ENGINE: LazyLock<TypstEngine<TypstTemplateMainFile>> = LazyLock::new(|| {
    TypstEngine::builder()
        .main_file(("mail.typ", TEMPLATE).into_source())
        .fonts(FONTS)
        .build()
});

/// More than this is not a receipt; it is a newsletter, and the page keeps
/// the head of it.
const MAX_CHARS: usize = 20_000;

/// A mail as the printer needs it.
#[derive(Debug, Clone)]
pub struct Mail {
    /// The provider's id, for the footer.
    pub message_id: String,
    /// The mailbox it arrived in.
    pub mailbox: String,
    /// `Name <addr>`, as the mailbox gave it.
    pub from: String,
    /// The subject line.
    pub subject: String,
    /// When it arrived.
    pub received: Option<DateTime<Utc>>,
    /// The body as text; HTML already reduced.
    pub text: String,
}

/// Why a mail did not print.
#[derive(Debug, thiserror::Error)]
pub enum MailError {
    /// The template did not compile against this mail.
    #[error("mail template: {0}")]
    Compile(String),
    /// The PDF could not be written.
    #[error("mail pdf: {0}")]
    Pdf(String),
}

/// Print the mail. CPU-bound: call from `spawn_blocking`.
///
/// # Errors
/// The template fails on this mail, or the PDF cannot be written.
pub fn print(mail: &Mail) -> Result<Vec<u8>, MailError> {
    let text: String = mail.text.chars().take(MAX_CHARS).collect();
    let lines: Vec<Value> = text
        .lines()
        .map(|l| Value::Str(Str::from(l.trim_end())))
        .collect();
    let mut doc = Dict::new();
    let put = |d: &mut Dict, k: &str, v: &str| {
        d.insert(Str::from(k), Value::Str(Str::from(v)));
    };
    put(&mut doc, "message_id", &mail.message_id);
    put(&mut doc, "mailbox", &mail.mailbox);
    put(&mut doc, "from", &mail.from);
    put(&mut doc, "subject", &mail.subject);
    put(
        &mut doc,
        "received",
        &mail
            .received
            .map(|t| t.format("%Y-%m-%d %H:%M UTC").to_string())
            .unwrap_or_default(),
    );
    doc.insert(
        Str::from("lines"),
        Value::Array(lines.into_iter().collect()),
    );
    let mut inputs = Dict::new();
    inputs.insert(Str::from("doc"), Value::Dict(doc));
    let compiled = ENGINE
        .compile_with_input::<_, PagedDocument>(inputs)
        .output
        .map_err(|e| MailError::Compile(format!("{e:?}")))?;
    typst_pdf::pdf(&compiled, &PdfOptions::default()).map_err(|e| MailError::Pdf(format!("{e:?}")))
}

static BLOCKS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?is)<script\b[^>]*>.*?</script\s*>|<style\b[^>]*>.*?</style\s*>|<head\b[^>]*>.*?</head\s*>")
        .unwrap_or_else(|e| panic!("{e}"))
});
static BREAKS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)<\s*(br|/p|/div|/tr|/li|/h[1-6]|/table|/blockquote|p|div|tr|li|h[1-6]|table)\b[^>]*>",
    )
    .unwrap_or_else(|e| panic!("{e}"))
});
static CELLS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)<\s*/t[dh]\b[^>]*>").unwrap_or_else(|e| panic!("{e}")));
static TAGS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)<[^>]*>").unwrap_or_else(|e| panic!("{e}")));
static ENTITY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"&(#x[0-9a-fA-F]+|#\d+|[a-zA-Z]+);").unwrap_or_else(|e| panic!("{e}"))
});
static SPACES: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[ \t\u{a0}]+").unwrap_or_else(|e| panic!("{e}")));

/// The words of an HTML mail, one block per line, blank lines collapsed.
#[must_use]
pub fn html_to_text(html: &str) -> String {
    let s = BLOCKS.replace_all(html, "");
    let s = BREAKS.replace_all(&s, "\n");
    let s = CELLS.replace_all(&s, "\t");
    let s = TAGS.replace_all(&s, "");
    let s = ENTITY.replace_all(&s, |c: &regex::Captures<'_>| entity(&c[1]));
    tidy(&s)
}

/// Runs of spaces to one, trailing spaces gone, more than one blank line
/// to one.
#[must_use]
pub fn tidy(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut blank = 0;
    for line in text.lines() {
        let line = SPACES.replace_all(line, " ");
        let line = line.trim();
        if line.is_empty() {
            blank += 1;
            if blank == 1 {
                out.push('\n');
            }
            continue;
        }
        blank = 0;
        out.push_str(line);
        out.push('\n');
    }
    out.trim().to_owned()
}

fn entity(name: &str) -> String {
    if let Some(hex) = name.strip_prefix("#x") {
        return u32::from_str_radix(hex, 16)
            .ok()
            .and_then(char::from_u32)
            .map(String::from)
            .unwrap_or_default();
    }
    if let Some(dec) = name.strip_prefix('#') {
        return dec
            .parse::<u32>()
            .ok()
            .and_then(char::from_u32)
            .map(String::from)
            .unwrap_or_default();
    }
    match name {
        "amp" => "&",
        "lt" => "<",
        "gt" => ">",
        "quot" => "\"",
        "apos" => "'",
        "nbsp" => " ",
        "euro" => "€",
        "pound" => "£",
        "copy" => "©",
        "reg" => "®",
        "trade" => "™",
        "ndash" => "–",
        "mdash" => "—",
        "hellip" => "…",
        "rsquo" => "’",
        "lsquo" => "‘",
        "rdquo" => "”",
        "ldquo" => "“",
        _ => "",
    }
    .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn html_becomes_its_words_in_order() {
        let html = r"<html><head><style>p{color:red}</style></head><body>
            <table><tr><td>Total</td><td>$5.00&nbsp;USD</td></tr></table>
            <p>Thanks &amp; see you</p><div>Invoice #&#8203;1234</div><script>x()</script>
            <p>Paid on 06/10/26</p></body></html>";
        let text = html_to_text(html);
        assert_eq!(
            text,
            "Total $5.00 USD\n\nThanks & see you\n\nInvoice #\u{200b}1234\n\nPaid on 06/10/26"
        );
    }

    #[test]
    fn a_mail_prints_to_a_pdf_the_reader_can_read() {
        let mail = Mail {
            message_id: "18f2".into(),
            mailbox: "nevio@inorbit.hr".into(),
            from: "Medium <noreply@medium.com>".into(),
            subject: "Your Medium receipt".into(),
            received: Some("2026-08-10T08:55:00Z".parse().unwrap()),
            text: "Medium Monthly Membership\n\nTotal paid $5.00 USD\nPayment date: 08/10/26\n"
                .into(),
        };
        let pdf = print(&mail).unwrap();
        assert!(pdf.starts_with(b"%PDF"), "a pdf");
        assert!(pdf.len() > 1000);
    }
}
