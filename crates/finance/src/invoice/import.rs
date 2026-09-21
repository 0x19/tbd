//! Issued invoices from before this service: the PDFs, read back into the
//! books so the year's totals, the pre-fill and the counter start from what
//! was really issued. A one-shot from the CLI: every file is parsed and shown
//! first; nothing is written without `--apply`.
//!
//! The text comes from poppler's `pdftotext -bbox-layout`, because positions
//! are what tell a wrapped description which row it belongs to (the row
//! number sits at the vertical centre of its cell, the description's lines
//! above and below it), and the in-process reader has none. So this runs
//! where poppler is installed, on a laptop against the cluster's database,
//! never in the service. Three template generations are read the same way:
//! `LibreOffice` (2024-2025), Google Docs (2025-2026) and Typst (this service).

use std::path::Path;

use chrono::{NaiveDate, NaiveDateTime};
use regex::Regex;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use tbd_db::{Access, PartyId, map_err};
use uuid::Uuid;

use super::{
    VatTreatment,
    store::{ClientInput, ClientRow, InvoiceError, sql, upsert_client},
};

/// One line as printed: quantity, unit price and the amount the page shows.
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct PrintedLine {
    pub description: String,
    pub quantity_milli: i64,
    pub unit_price_minor: i64,
    pub amount_minor: i64,
}

/// Why a file could not be read as one of our invoices.
#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    /// `pdftotext` is missing or failed.
    #[error("pdftotext: {0}")]
    Tool(String),
    /// The file is not an invoice of ours, or a field could not be found.
    #[error("{0}")]
    Unreadable(String),
    /// The figures on the page do not add up.
    #[error("does not add up: {0}")]
    Arithmetic(String),
}

/// One word on the page, in PDF points.
#[derive(Debug, Clone)]
pub struct Word {
    /// Left edge.
    pub x0: f64,
    /// Right edge.
    pub x1: f64,
    /// The text.
    pub text: String,
}

/// One line of words, as poppler groups them.
#[derive(Debug, Clone)]
pub struct TextLine {
    /// The page, from 1.
    pub page: u32,
    /// Left edge.
    pub x0: f64,
    /// Top edge.
    pub y0: f64,
    /// Bottom edge.
    pub y1: f64,
    /// The words, left to right.
    pub words: Vec<Word>,
}

impl TextLine {
    fn text(&self) -> String {
        self.words
            .iter()
            .map(|w| w.text.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    }
    fn y(&self) -> f64 {
        f64::midpoint(self.y0, self.y1)
    }
}

/// A parsed invoice: the header, the client, the lines, the totals.
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct Parsed {
    pub source: String,
    pub sha256: String,
    pub bytes: Vec<u8>,
    pub ordinal: i32,
    pub premises: String,
    pub device: String,
    pub year: i32,
    /// `invoice` or `advance` (an advance invoice numbered in the series).
    pub kind: String,
    pub issued_at: NaiveDateTime,
    pub due_date: NaiveDate,
    pub delivery_date: NaiveDate,
    pub place_of_issue: String,
    pub client_name: String,
    pub client_address: Vec<String>,
    pub client_country: String,
    pub client_tax_id: String,
    pub currency: String,
    pub vat_treatment: VatTreatment,
    pub lines: Vec<PrintedLine>,
    pub subtotal_minor: i64,
    pub vat_minor: i64,
    pub total_minor: i64,
    pub note: String,
}

impl Parsed {
    /// The number as printed.
    #[must_use]
    pub fn number(&self) -> String {
        format!(
            "{}-{}-{}-{}",
            self.ordinal, self.premises, self.device, self.year
        )
    }
}

/// Run poppler on the file and return its positional XHTML.
///
/// # Errors
/// `pdftotext` missing, or failing on the file.
pub fn pdftotext_bbox(path: &Path) -> Result<String, ImportError> {
    let out = std::process::Command::new("pdftotext")
        .arg("-bbox-layout")
        .arg(path)
        .arg("-")
        .output()
        .map_err(|e| ImportError::Tool(format!("{e}; install poppler-utils")))?;
    if !out.status.success() {
        return Err(ImportError::Tool(
            String::from_utf8_lossy(&out.stderr).trim().to_owned(),
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// The lines of poppler's `-bbox-layout` output, in page order.
#[must_use]
pub fn lines_of_bbox(xml: &str) -> Vec<TextLine> {
    let page_re = Regex::new(r"<page ").unwrap_or_else(|_| unreachable!());
    let line_re = Regex::new(
        r#"<line xMin="([\d.]+)" yMin="([\d.]+)" xMax="[\d.]+" yMax="([\d.]+)">(.*?)</line>"#,
    )
    .unwrap_or_else(|_| unreachable!());
    let word_re = Regex::new(
        r#"<word xMin="([\d.]+)" yMin="[\d.]+" xMax="([\d.]+)" yMax="[\d.]+">(.*?)</word>"#,
    )
    .unwrap_or_else(|_| unreachable!());
    let flat = xml.replace('\n', " ");
    let mut out = Vec::new();
    let mut page = 0;
    // Pages and lines interleave; walk both by position.
    let mut pages: Vec<usize> = page_re.find_iter(&flat).map(|m| m.start()).collect();
    pages.reverse();
    for cap in line_re.captures_iter(&flat) {
        let at = cap.get(0).map_or(0, |m| m.start());
        while pages.last().is_some_and(|&p| p < at) {
            pages.pop();
            page += 1;
        }
        let words: Vec<Word> = word_re
            .captures_iter(&cap[4])
            .map(|w| Word {
                x0: w[1].parse().unwrap_or(0.0),
                x1: w[2].parse().unwrap_or(0.0),
                text: unescape(&w[3]),
            })
            .collect();
        if words.is_empty() {
            continue;
        }
        out.push(TextLine {
            page,
            x0: cap[1].parse().unwrap_or(0.0),
            y0: cap[2].parse().unwrap_or(0.0),
            y1: cap[3].parse().unwrap_or(0.0),
            words,
        });
    }
    out
}

fn unescape(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace('\u{200b}', "")
}

/// `9.167,00`, `13,750.00`, `303,89`, `0,00` as minor units. The last
/// separator with two digits after it is the decimal point.
///
/// # Errors
/// Not a money figure.
pub fn money_minor(s: &str) -> Result<i64, ImportError> {
    let s = s.trim().trim_start_matches('€').trim();
    let bad = || ImportError::Unreadable(format!("not an amount: {s:?}"));
    let negative = s.starts_with('-');
    let digits_only = s.trim_start_matches('-');
    let (int, frac) = match digits_only.rfind([',', '.']) {
        Some(i) if digits_only.len() - i - 1 == 2 => (&digits_only[..i], &digits_only[i + 1..]),
        _ => (digits_only, "00"),
    };
    let int: String = int.chars().filter(char::is_ascii_digit).collect();
    if int.is_empty() || !frac.chars().all(|c| c.is_ascii_digit()) {
        return Err(bad());
    }
    let minor: i64 = format!("{int}{frac}").parse().map_err(|_| bad())?;
    Ok(if negative { -minor } else { minor })
}

fn quantity_milli(s: &str) -> Result<i64, ImportError> {
    let s = s.trim().replace(',', ".");
    let (int, frac) = s.split_once('.').unwrap_or((&s, ""));
    let int: i64 = int
        .parse()
        .map_err(|_| ImportError::Unreadable(format!("not a quantity: {s:?}")))?;
    let frac = format!("{frac:0<3}");
    let frac: i64 = frac[..3]
        .parse()
        .map_err(|_| ImportError::Unreadable(format!("not a quantity: {s:?}")))?;
    Ok(int * 1000 + frac)
}

/// `dd.mm.yyyy`, with or without a trailing dot.
fn date(d: &str, m: &str, y: &str) -> Result<NaiveDate, ImportError> {
    NaiveDate::from_ymd_opt(
        y.parse().unwrap_or(0),
        m.parse().unwrap_or(0),
        d.parse().unwrap_or(0),
    )
    .ok_or_else(|| ImportError::Unreadable(format!("not a date: {d}.{m}.{y}")))
}

/// ISO 3166 for the countries a client of ours has been in; the name as
/// printed on the invoice.
fn country_code(name: &str) -> Option<&'static str> {
    Some(match name.trim().to_lowercase().as_str() {
        "croatia" | "hrvatska" => "HR",
        "finland" | "finska" => "FI",
        "serbia" | "srbija" => "RS",
        "germany" | "deutschland" | "njemačka" => "DE",
        "austria" | "österreich" => "AT",
        "slovenia" | "slovenija" => "SI",
        "italy" | "italia" => "IT",
        "netherlands" | "the netherlands" => "NL",
        "united kingdom" | "uk" => "GB",
        "united states" | "usa" => "US",
        "switzerland" => "CH",
        _ => return None,
    })
}

const EU: &[&str] = &[
    "AT", "BE", "BG", "CY", "CZ", "DE", "DK", "EE", "ES", "FI", "FR", "GR", "HU", "IE", "IT", "LT",
    "LU", "LV", "MT", "NL", "PL", "PT", "RO", "SE", "SI", "SK",
];

/// The treatment a client's country and tax id imply: standard at home,
/// reverse charge with an EU VAT id, outside scope beyond the EU.
fn treatment_for(country: &str, tax_id: &str) -> VatTreatment {
    if country == "HR" {
        VatTreatment::StandardHr
    } else if EU.contains(&country) && !tax_id.trim().is_empty() {
        VatTreatment::ReverseChargeEu
    } else {
        VatTreatment::OutsideScopeNonEu
    }
}

/// The lines that sit on the same row as `y`, to the right of `min_x`.
fn at_y(lines: &[TextLine], page: u32, y: f64, min_x: f64) -> Vec<&TextLine> {
    let mut v: Vec<&TextLine> = lines
        .iter()
        .filter(|l| l.page == page && (l.y() - y).abs() < 4.0 && l.x0 > min_x)
        .collect();
    v.sort_by(|a, b| a.x0.total_cmp(&b.x0));
    v
}

/// The number printed on the same row as a label, right of it.
fn figure_beside(lines: &[TextLine], label: &TextLine) -> Option<i64> {
    at_y(lines, label.page, label.y(), label.x0 + 1.0)
        .into_iter()
        .rev()
        .find_map(|l| l.words.last().and_then(|w| money_minor(&w.text).ok()))
}

/// The header of an invoice, from the joined text.
struct Header {
    ordinal: i32,
    premises: String,
    device: String,
    year: i32,
    kind: String,
    issued_at: NaiveDateTime,
    due_date: NaiveDate,
    delivery_date: NaiveDate,
    place_of_issue: String,
}

/// The text to the right of a label: the rest of the label's own line, then
/// every line on the same row further right. The three generations put the
/// value on the label's line, on its own line beside it, or a row lower in
/// the joined text, and only the position says which.
fn value_after(lines: &[TextLine], label: &Regex) -> Option<String> {
    let line = lines.iter().find(|l| label.is_match(&l.text()))?;
    let own = label.replace(&line.text(), "").trim().to_owned();
    let beside: Vec<String> = at_y(lines, line.page, line.y(), line.x0 + 1.0)
        .into_iter()
        .filter(|l| !std::ptr::eq(*l, line))
        .map(TextLine::text)
        .collect();
    let joined = std::iter::once(own)
        .chain(beside)
        .filter(|t| !t.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    (!joined.trim().is_empty()).then_some(joined)
}

fn header(lines: &[TextLine], text: &str) -> Result<Header, ImportError> {
    let re = |p: &str| Regex::new(p).unwrap_or_else(|_| unreachable!());
    let num = re(
        r"(?i)(advance invoice|invoice number|invoice no\.?|broj računa)\D{0,60}?(\d{1,5})-(\d{1,3})-(\d{1,3})-(20\d{2})\b",
    );
    let n = num
        .captures(text)
        .ok_or_else(|| ImportError::Unreadable("no invoice number".into()))?;
    let kind = if n[1].to_lowercase().starts_with("advance") {
        "advance"
    } else {
        "invoice"
    };
    let date_re = re(r"(\d{2})\.(\d{2})\.(\d{4})");
    let time_re = re(r"(\d{2})\.(\d{2})\.(\d{4})\.?\s*(\d{2}):(\d{2})");
    let issued_text = value_after(
        lines,
        &re(r"(?i)^date and time(?:\s*/\s*datum i vrijeme)?:?"),
    )
    .ok_or_else(|| ImportError::Unreadable("no date and time".into()))?;
    let issued = time_re
        .captures(&issued_text)
        .ok_or_else(|| ImportError::Unreadable(format!("no time in {issued_text:?}")))?;
    let issued_at = date(&issued[1], &issued[2], &issued[3])?
        .and_hms_opt(
            issued[4].parse().unwrap_or(0),
            issued[5].parse().unwrap_or(0),
            0,
        )
        .ok_or_else(|| ImportError::Unreadable("bad time".into()))?;
    let due_text = value_after(lines, &re(r"(?i)^due date(?:\s*/\s*rok dospijeća)?:?"))
        .ok_or_else(|| ImportError::Unreadable("no due date".into()))?;
    let due = date_re
        .captures(&due_text)
        .ok_or_else(|| ImportError::Unreadable(format!("no date in {due_text:?}")))?;
    let delivery = value_after(
        lines,
        &re(r"(?i)^delivery date(?:\s*/\s*datum isporuke)?:?"),
    )
    .and_then(|t| date_re.captures(&t).map(|c| date(&c[1], &c[2], &c[3])))
    .transpose()?
    .unwrap_or(issued_at.date());
    let place = value_after(
        lines,
        &re(r"(?i)^place of invoice issue(?:\s*/\s*mjesto izdavanja)?:?"),
    )
    .and_then(|t| t.split_whitespace().next().map(str::to_owned))
    .unwrap_or_else(|| "Viškovo".to_owned());
    Ok(Header {
        ordinal: n[2].parse().unwrap_or(0),
        premises: n[3].to_owned(),
        device: n[4].to_owned(),
        year: n[5].parse().unwrap_or(0),
        kind: kind.into(),
        issued_at,
        due_date: date(&due[1], &due[2], &due[3])?,
        delivery_date: delivery,
        place_of_issue: place,
    })
}

/// The client block: the left-column lines between the issuer's OIB and
/// the table header.
struct ClientBlock {
    name: String,
    address: Vec<String>,
    country: String,
    tax_id: String,
}

fn client_block(
    lines: &[TextLine],
    table_header: &TextLine,
    source: &str,
) -> Result<ClientBlock, ImportError> {
    let issuer_oib = lines
        .iter()
        .find(|l| l.text().contains("38846238650") && l.x0 < 200.0)
        .ok_or_else(|| ImportError::Unreadable("not an Inorbit invoice: no issuer OIB".into()))?;
    let tax_re = Regex::new(r"(?i)^(?:OIB|TID No\.?|VAT(?:-ID)?|Tax ID)[:.]?\s*(\S.*)$")
        .unwrap_or_else(|_| unreachable!());
    let block: Vec<String> = lines
        .iter()
        .filter(|l| {
            l.page == issuer_oib.page
                && l.y0 > issuer_oib.y1
                && l.y1 < table_header.y0
                && l.x0 < 200.0
        })
        .map(TextLine::text)
        .filter(|t| !t.trim().is_empty())
        .collect();
    let mut name = None;
    let mut address = Vec::new();
    let mut country = String::new();
    let mut tax_id = String::new();
    for line in block {
        if let Some(c) = tax_re.captures(&line) {
            c[1].trim().clone_into(&mut tax_id);
            continue;
        }
        if name.is_none() {
            name = Some(line);
            continue;
        }
        if let Some(code) = country_code(line.trim_end_matches(',')) {
            country = code.into();
        } else {
            address.push(line);
        }
    }
    // The Google Docs generation draws the client's box as a picture, so the
    // page carries no client text; the file name does (`…-9-1-1-tenderly.pdf`).
    let name = name.or_else(|| client_hint(source)).ok_or_else(|| {
        ImportError::Unreadable("no client on the page and none in the file name".into())
    })?;
    Ok(ClientBlock {
        name,
        address,
        country,
        tax_id,
    })
}

/// `inorbit-31-08-2026-9-1-1-tenderly.pdf` names its client last.
fn client_hint(source: &str) -> Option<String> {
    let stem = Path::new(source).file_stem()?.to_str()?;
    let last = stem.rsplit('-').next()?;
    if last.is_empty() || !last.chars().all(char::is_alphabetic) {
        return None;
    }
    let mut c = last.chars();
    let first = c.next()?.to_uppercase().collect::<String>();
    Some(format!("{first}{}", c.as_str()))
}

/// The table: the rows by the numbered cells, each with the qty, price and
/// amount on its row and the description lines nearest to it.
fn table(lines: &[TextLine], head: &TextLine, end_y: f64) -> Result<Vec<PrintedLine>, ImportError> {
    let page = head.page;
    let head_words = at_y(lines, page, head.y(), -1.0);
    let col_x = |needle: &str| -> Option<f64> {
        head_words
            .iter()
            .find(|l| l.text().to_lowercase().contains(needle))
            .map(|l| l.x0)
    };
    let qty_x = col_x("qty")
        .or_else(|| col_x("količina"))
        .ok_or_else(|| ImportError::Unreadable("no quantity column".into()))?;
    let hash_x = head_words.first().map_or(0.0, |l| l.x0);
    let in_table = |l: &&TextLine| l.page == page && l.y0 > head.y1 && l.y1 < end_y;
    // The numbered cells: a lone small integer in the leftmost column.
    let mut rows: Vec<(i32, f64)> = lines
        .iter()
        .filter(in_table)
        .filter(|l| l.x0 < qty_x - 40.0 && l.x0 < hash_x + 30.0)
        .filter_map(|l| {
            let first = l.words.first()?;
            let n: i32 = first.text.parse().ok()?;
            (n > 0 && n < 1000).then_some((n, l.y()))
        })
        .collect();
    rows.sort_by(|a, b| a.1.total_cmp(&b.1));
    if rows.is_empty() {
        return Err(ImportError::Unreadable("no table rows".into()));
    }
    let mut out = Vec::new();
    for (i, (_, y)) in rows.iter().enumerate() {
        // Figures on the row: the three rightmost numbers are qty, price, amount.
        let figures: Vec<String> = at_y(lines, page, *y, qty_x - 40.0)
            .into_iter()
            .flat_map(|l| l.words.iter().map(|w| w.text.clone()))
            .filter(|w| w.chars().any(|c| c.is_ascii_digit()))
            .collect();
        if figures.len() < 3 {
            return Err(ImportError::Unreadable(format!(
                "row {}: expected qty, price and amount, found {figures:?}",
                i + 1
            )));
        }
        let n = figures.len();
        let quantity_milli = quantity_milli(&figures[n - 3])?;
        let unit_price_minor = money_minor(&figures[n - 2])?;
        let amount_minor = money_minor(&figures[n - 1])?;
        let description = description_for(lines, &in_table, &rows, i, hash_x, qty_x);
        if description.is_empty() {
            return Err(ImportError::Unreadable(format!(
                "row {}: no description",
                i + 1
            )));
        }
        let expected = (quantity_milli * unit_price_minor + 500) / 1000;
        if expected != amount_minor {
            return Err(ImportError::Arithmetic(format!(
                "row {}: {quantity_milli} × {unit_price_minor} is {expected}, printed {amount_minor}",
                i + 1
            )));
        }
        out.push(PrintedLine {
            description,
            quantity_milli,
            unit_price_minor,
            amount_minor,
        });
    }
    Ok(out)
}

/// The description of row `i`: every line in the description column nearer
/// to this row than to any other, top to bottom, the row's own line too.
fn description_for(
    lines: &[TextLine],
    in_table: &dyn Fn(&&TextLine) -> bool,
    rows: &[(i32, f64)],
    i: usize,
    hash_x: f64,
    qty_x: f64,
) -> String {
    let mut desc: Vec<(f64, String)> = lines
        .iter()
        .filter(in_table)
        .filter_map(|l| {
            let words: Vec<&Word> = l
                .words
                .iter()
                .filter(|w| w.x0 >= hash_x + 12.0 && w.x0 < qty_x - 8.0)
                .filter(|w| {
                    !(l.x0 < hash_x + 30.0
                        && (w.x0 - l.x0).abs() < 0.01
                        && w.text.parse::<i32>().is_ok())
                })
                .collect();
            if words.is_empty() {
                return None;
            }
            let nearest = rows
                .iter()
                .enumerate()
                .min_by(|a, b| (a.1.1 - l.y()).abs().total_cmp(&(b.1.1 - l.y()).abs()))
                .map(|(j, _)| j)?;
            (nearest == i).then(|| {
                (
                    l.y(),
                    words
                        .iter()
                        .map(|w| w.text.as_str())
                        .collect::<Vec<_>>()
                        .join(" "),
                )
            })
        })
        .collect();
    desc.sort_by(|a, b| a.0.total_cmp(&b.0));
    desc.into_iter()
        .map(|(_, t)| t)
        .collect::<Vec<_>>()
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Read one invoice out of poppler's lines.
///
/// # Errors
/// Not one of our invoices, a field missing, or figures that do not add up.
#[allow(clippy::too_many_lines)]
pub fn parse(lines: &[TextLine], source: &str, bytes: &[u8]) -> Result<Parsed, ImportError> {
    let text = lines
        .iter()
        .map(TextLine::text)
        .collect::<Vec<_>>()
        .join("\n");
    if !lines
        .iter()
        .any(|l| l.text().contains("38846238650") && l.x0 < 200.0)
    {
        return Err(ImportError::Unreadable("not an Inorbit invoice".into()));
    }
    let h = header(lines, &text)?;
    let head = lines
        .iter()
        .find(|l| {
            let t = l.text().to_lowercase();
            t.starts_with('#') || (t.contains("service") && l.x0 < 200.0 && l.y0 > 150.0)
        })
        .ok_or_else(|| ImportError::Unreadable("no table header".into()))?;
    let label = |needle: &str| {
        lines.iter().find(|l| {
            l.page == head.page
                && l.y0 > head.y1
                && l.x0 > 250.0
                && l.text().to_lowercase().starts_with(needle)
        })
    };
    let subtotal_label =
        label("sub total").ok_or_else(|| ImportError::Unreadable("no subtotal".into()))?;
    let subtotal_minor = figure_beside(lines, subtotal_label)
        .or_else(|| {
            // LibreOffice may put the figure a line lower.
            lines
                .iter()
                .filter(|l| {
                    l.page == head.page
                        && l.y0 > subtotal_label.y0
                        && l.y0 < subtotal_label.y1 + 20.0
                        && l.x0 > 450.0
                })
                .find_map(|l| l.words.last().and_then(|w| money_minor(&w.text).ok()))
        })
        .ok_or_else(|| ImportError::Unreadable("no subtotal figure".into()))?;
    let vat_minor = label("vat")
        .and_then(|l| figure_beside(lines, l))
        .unwrap_or(0);
    let total_minor = label("total due")
        .and_then(|l| figure_beside(lines, l))
        .unwrap_or(subtotal_minor + vat_minor);
    let mut lines_in = table(lines, head, subtotal_label.y0)?;
    let mut sum: i64 = lines_in.iter().map(|l| l.amount_minor).sum();
    if sum != subtotal_minor {
        // An advance already paid is printed positive and subtracted.
        for l in &mut lines_in {
            let d = l.description.to_lowercase();
            if d.contains("advance") || d.contains("predujam") || d.contains("avans") {
                l.unit_price_minor = -l.unit_price_minor;
                l.amount_minor = -l.amount_minor;
            }
        }
        sum = lines_in.iter().map(|l| l.amount_minor).sum();
    }
    if sum != subtotal_minor {
        return Err(ImportError::Arithmetic(format!(
            "lines add to {sum}, subtotal printed {subtotal_minor}"
        )));
    }
    if subtotal_minor + vat_minor != total_minor {
        return Err(ImportError::Arithmetic(format!(
            "{subtotal_minor} + {vat_minor} is not {total_minor}"
        )));
    }
    let client = client_block(lines, head, source)?;
    let currency = if text.contains("(€)") || text.contains("EUR") {
        "EUR"
    } else if text.contains("(HRK)") {
        "HRK"
    } else {
        "EUR"
    };
    let note = lines
        .iter()
        .map(TextLine::text)
        .find(|t| t.starts_with("This is receipt") || t.starts_with("This is a receipt"))
        .unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Ok(Parsed {
        source: source.to_owned(),
        sha256: format!("{:x}", hasher.finalize()),
        bytes: bytes.to_vec(),
        ordinal: h.ordinal,
        premises: h.premises,
        device: h.device,
        year: h.year,
        kind: h.kind,
        issued_at: h.issued_at,
        due_date: h.due_date,
        delivery_date: h.delivery_date,
        place_of_issue: h.place_of_issue,
        vat_treatment: treatment_for(&client.client_country_or_empty(), &client.tax_id),
        client_name: client.name,
        client_address: client.address,
        client_country: client.country,
        client_tax_id: client.tax_id,
        currency: currency.into(),
        lines: lines_in,
        subtotal_minor,
        vat_minor,
        total_minor,
        note,
    })
}

impl ClientBlock {
    fn client_country_or_empty(&self) -> String {
        self.country.clone()
    }
}

/// Read one PDF from disk.
///
/// # Errors
/// The file, poppler, or the page.
pub fn read_file(path: &Path) -> Result<Parsed, ImportError> {
    let bytes =
        std::fs::read(path).map_err(|e| ImportError::Tool(format!("{}: {e}", path.display())))?;
    let xml = pdftotext_bbox(path)?;
    let lines = lines_of_bbox(&xml);
    parse(&lines, &path.display().to_string(), &bytes)
}

/// What the import did or refused, per invoice number.
#[derive(Debug, Default)]
pub struct Report {
    /// Written now.
    pub imported: Vec<String>,
    /// Already in the books, left alone.
    pub present: Vec<String>,
    /// Not written, and why.
    pub refused: Vec<(String, String)>,
}

/// Write the parsed invoices as approved invoices of `party`: the client
/// found by name or made, the PDF stored as the document, the lines and
/// totals as printed, an `imported` event naming the file, and the counter
/// raised past the highest number seen. A number already in the books is
/// left alone; two files claiming one number are both refused.
///
/// # Errors
/// The party is outside the grant (not found); the database.
#[allow(clippy::too_many_lines)]
pub async fn apply(
    pool: &PgPool,
    access: &Access,
    party: Uuid,
    parsed: &[Parsed],
) -> Result<Report, InvoiceError> {
    access.require(PartyId(party), "party")?;
    let mut report = Report::default();
    // Two files, one number: neither is written.
    let mut claims: std::collections::HashMap<String, Vec<&Parsed>> =
        std::collections::HashMap::new();
    for p in parsed {
        claims.entry(p.number()).or_default().push(p);
    }
    for p in parsed {
        let number = p.number();
        // The same invoice saved twice (a copy, or one with a receipt appended)
        // is one invoice: the smallest file is the one kept.
        let same = |q: &Parsed| {
            q.issued_at == p.issued_at
                && q.total_minor == p.total_minor
                && q.client_name.eq_ignore_ascii_case(&p.client_name)
                && q.lines.len() == p.lines.len()
        };
        if claims[&number]
            .iter()
            .any(|q| q.sha256 != p.sha256 && same(q) && q.bytes.len() < p.bytes.len())
        {
            continue;
        }
        let rivals: Vec<&&Parsed> = claims[&number]
            .iter()
            .filter(|q| q.sha256 != p.sha256 && !same(q))
            .collect();
        if !rivals.is_empty() {
            report.refused.push((
                number.clone(),
                format!(
                    "{} files claim this number: {}",
                    rivals.len() + 1,
                    std::iter::once(p.source.as_str())
                        .chain(rivals.iter().map(|q| q.source.as_str()))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            ));
            continue;
        }
        if report.imported.contains(&number) {
            // The same file twice (a copy): once is enough.
            continue;
        }
        let existing: Option<(Uuid,)> = sqlx::query_as(
            "select id from finance.invoices
              where party_id = $1 and year = $2 and premises = $3 and device = $4 and ordinal = $5",
        )
        .bind(party)
        .bind(p.year)
        .bind(&p.premises)
        .bind(&p.device)
        .bind(p.ordinal)
        .fetch_optional(pool)
        .await
        .map_err(map_err)?;
        if existing.is_some() {
            report.present.push(number);
            continue;
        }
        let client = client_for(pool, access, party, p).await?;
        write_one(pool, access, party, p, &client).await?;
        report.imported.push(number);
    }
    Ok(report)
}

/// The client by name within the party, or a new one from the invoice.
async fn client_for(
    pool: &PgPool,
    access: &Access,
    party: Uuid,
    p: &Parsed,
) -> Result<ClientRow, InvoiceError> {
    let found = sqlx::query_as::<_, ClientRow>(sql(&format!(
        "select {} from finance.clients where party_id = $1 and lower(name) = lower($2) limit 1",
        super::store::CLIENT_COLUMNS
    )))
    .bind(party)
    .bind(&p.client_name)
    .fetch_optional(pool)
    .await
    .map_err(map_err)?;
    if let Some(c) = found {
        return Ok(c);
    }
    upsert_client(
        pool,
        access,
        ClientInput {
            id: None,
            party_id: party,
            name: p.client_name.clone(),
            address_lines: p.client_address.clone(),
            country_code: if p.client_country.is_empty() {
                "XX".into()
            } else {
                p.client_country.clone()
            },
            tax_id: p.client_tax_id.clone(),
            vat_treatment: p.vat_treatment,
            recipients: Vec::new(),
            currency: p.currency.clone(),
        },
    )
    .await
}

/// The PDF as the invoice's document, once per content.
async fn store_pdf(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    party: Uuid,
    p: &Parsed,
) -> Result<Uuid, InvoiceError> {
    let existing: Option<(Uuid,)> =
        sqlx::query_as("select id from finance.documents where party_id = $1 and sha256 = $2")
            .bind(party)
            .bind(&p.sha256)
            .fetch_optional(&mut **tx)
            .await
            .map_err(map_err)?;
    if let Some((id,)) = existing {
        return Ok(id);
    }
    let id = Uuid::new_v4();
    let filename = Path::new(&p.source)
        .file_name()
        .map(|f| f.to_string_lossy().into_owned())
        .unwrap_or_default();
    sqlx::query(
        "insert into finance.documents (id, party_id, kind, sha256, content_type, size_bytes, filename)
         values ($1, $2, 'invoice', $3, 'application/pdf', $4, $5)",
    )
    .bind(id)
    .bind(party)
    .bind(&p.sha256)
    .bind(i64::try_from(p.bytes.len()).unwrap_or(i64::MAX))
    .bind(&filename)
    .execute(&mut **tx)
    .await
    .map_err(map_err)?;
    sqlx::query("insert into finance.document_blobs (document_id, bytes) values ($1, $2)")
        .bind(id)
        .bind(&p.bytes)
        .execute(&mut **tx)
        .await
        .map_err(map_err)?;
    Ok(id)
}

async fn write_one(
    pool: &PgPool,
    access: &Access,
    party: Uuid,
    p: &Parsed,
    client: &ClientRow,
) -> Result<(), InvoiceError> {
    let mut tx = pool.begin().await.map_err(map_err)?;
    let document_id = store_pdf(&mut tx, party, p).await?;
    let id = Uuid::new_v4();
    let issued = p.issued_at.and_utc();
    let note = if p.kind == "advance" && !p.note.contains("dvance") {
        format!("Advance invoice. {}", p.note).trim().to_owned()
    } else {
        p.note.clone()
    };
    sqlx::query(
        "insert into finance.invoices (id, party_id, client_id, status, year, ordinal, premises, device,
            issued_at, delivery_date, due_date, place_of_issue, currency, subtotal_minor, vat_minor,
            total_minor, vat_treatment, vat_note, note, approved_at, document_id, created_at, updated_at)
         values ($1,$2,$3,'approved',$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$8,$19,$8,$8)",
    )
    .bind(id)
    .bind(party)
    .bind(client.id)
    .bind(p.year)
    .bind(p.ordinal)
    .bind(&p.premises)
    .bind(&p.device)
    .bind(issued)
    .bind(p.delivery_date)
    .bind(p.due_date)
    .bind(&p.place_of_issue)
    .bind(&p.currency)
    .bind(p.subtotal_minor)
    .bind(p.vat_minor)
    .bind(p.total_minor)
    .bind(p.vat_treatment.as_str())
    .bind(p.vat_treatment.note())
    .bind(&note)
    .bind(document_id)
    .execute(&mut *tx)
    .await
    .map_err(map_err)?;
    for (i, l) in p.lines.iter().enumerate() {
        sqlx::query(
            "insert into finance.invoice_lines (id, invoice_id, position, description, quantity_milli, unit_price_minor, amount_minor)
             values ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(Uuid::new_v4())
        .bind(id)
        .bind(i32::try_from(i + 1).unwrap_or(i32::MAX))
        .bind(&l.description)
        .bind(l.quantity_milli)
        .bind(l.unit_price_minor)
        .bind(l.amount_minor)
        .execute(&mut *tx)
        .await
        .map_err(map_err)?;
    }
    sqlx::query(
        "insert into finance.invoice_numbers (party_id, year, premises, device, next_ordinal)
         values ($1, $2, $3, $4, $5)
         on conflict (party_id, year, premises, device)
         do update set next_ordinal = greatest(finance.invoice_numbers.next_ordinal, excluded.next_ordinal)",
    )
    .bind(party)
    .bind(p.year)
    .bind(&p.premises)
    .bind(&p.device)
    .bind(p.ordinal + 1)
    .execute(&mut *tx)
    .await
    .map_err(map_err)?;
    sqlx::query(
        "insert into finance.invoice_events (id, invoice_id, user_id, event, detail) values ($1, $2, $3, 'imported', $4)",
    )
    .bind(Uuid::new_v4())
    .bind(id)
    .bind(if access.user().0.is_nil() { None } else { Some(access.user().0) })
    .bind(serde_json::json!({ "source": p.source, "sha256": p.sha256, "kind": p.kind }))
    .execute(&mut *tx)
    .await
    .map_err(map_err)?;
    tx.commit().await.map_err(map_err)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(page: u32, x0: f64, y0: f64, words: &[(f64, &str)]) -> TextLine {
        TextLine {
            page,
            x0,
            y0,
            y1: y0 + 12.0,
            words: words
                .iter()
                .map(|(x, t)| Word {
                    x0: *x,
                    x1: x + 30.0,
                    text: (*t).to_owned(),
                })
                .collect(),
        }
    }

    /// The `LibreOffice` layout of January 2025: a two-line description
    /// around its row, a one-line one above its row, then a third row.
    #[allow(clippy::too_many_lines)]
    fn january() -> Vec<TextLine> {
        vec![
            line(
                1,
                338.0,
                13.0,
                &[(338.0, "Invoice"), (402.0, "number"), (473.0, "1-1-1-2025")],
            ),
            line(1, 55.0, 63.0, &[(55.0, "Inorbit"), (97.0, "d.o.o.")]),
            line(
                1,
                390.0,
                63.0,
                &[
                    (390.0, "Date"),
                    (420.0, "and"),
                    (445.0, "time"),
                    (500.0, "31.01.2025."),
                    (560.0, "09:15"),
                ],
            ),
            line(
                1,
                390.0,
                80.0,
                &[(390.0, "Due"), (420.0, "date"), (500.0, "15.02.2025.")],
            ),
            line(
                1,
                390.0,
                97.0,
                &[
                    (390.0, "Place"),
                    (420.0, "of"),
                    (440.0, "invoice"),
                    (480.0, "issue"),
                    (540.0, "Viškovo"),
                ],
            ),
            line(1, 55.0, 120.0, &[(55.0, "OIB:"), (80.0, "38846238650")]),
            line(1, 55.0, 160.0, &[(55.0, "Eiger"), (90.0, "Oy")]),
            line(
                1,
                55.0,
                175.0,
                &[
                    (55.0, "c/o"),
                    (75.0, "Equilibrium"),
                    (140.0, "Group"),
                    (170.0, "Oy"),
                ],
            ),
            line(
                1,
                55.0,
                190.0,
                &[
                    (55.0, "Meritullinkatu"),
                    (130.0, "1B"),
                    (150.0, "00170"),
                    (190.0, "Helsinki"),
                ],
            ),
            line(1, 55.0, 205.0, &[(55.0, "Finland")]),
            line(1, 55.0, 220.0, &[(55.0, "OIB:"), (80.0, "FI32746464")]),
            line(1, 57.0, 295.0, &[(57.0, "#")]),
            line(1, 85.0, 295.0, &[(85.0, "Service")]),
            line(1, 340.0, 295.0, &[(340.0, "Hrs/Qty")]),
            line(1, 423.0, 295.0, &[(423.0, "Rate/Price")]),
            line(1, 507.0, 295.0, &[(507.0, "Sub"), (532.0, "Total")]),
            line(
                1,
                85.0,
                320.0,
                &[
                    (85.0, "Software"),
                    (139.0, "development"),
                    (218.0, "services"),
                ],
            ),
            line(1, 57.0, 327.0, &[(57.0, "1")]),
            line(1, 357.0, 327.0, &[(357.0, "1")]),
            line(1, 435.0, 327.0, &[(435.0, "9.167,00")]),
            line(1, 514.0, 327.0, &[(514.0, "9.167,00")]),
            line(
                1,
                85.0,
                336.0,
                &[(85.0, "for"), (105.0, "January"), (154.0, "2025")],
            ),
            line(1, 85.0, 354.0, &[(85.0, "Bonus")]),
            line(1, 57.0, 361.0, &[(57.0, "2")]),
            line(1, 357.0, 361.0, &[(357.0, "1")]),
            line(1, 435.0, 361.0, &[(435.0, "7.000,00")]),
            line(1, 514.0, 361.0, &[(514.0, "7.000,00")]),
            line(
                1,
                85.0,
                386.0,
                &[
                    (85.0, "Flight"),
                    (117.0, "Reimbursement,"),
                    (205.0, "Zagreb"),
                    (245.0, "-"),
                    (251.0, "Lisbon"),
                ],
            ),
            line(1, 57.0, 392.0, &[(57.0, "3")]),
            line(1, 357.0, 392.0, &[(357.0, "1")]),
            line(1, 446.0, 392.0, &[(446.0, "303,89")]),
            line(1, 524.0, 392.0, &[(524.0, "303,89")]),
            line(1, 398.0, 436.0, &[(398.0, "Sub"), (424.0, "Total")]),
            line(1, 507.0, 436.0, &[(507.0, "16.470,89")]),
            line(1, 398.0, 452.0, &[(398.0, "VAT"), (424.0, "(PDV)")]),
            line(1, 540.0, 452.0, &[(540.0, "0,00")]),
            line(
                1,
                398.0,
                470.0,
                &[(398.0, "Total"), (424.0, "Due"), (450.0, "(€)")],
            ),
            line(1, 507.0, 470.0, &[(507.0, "16.470,89")]),
            line(
                1,
                55.0,
                520.0,
                &[
                    (55.0, "The"),
                    (80.0, "service"),
                    (120.0, "is"),
                    (140.0, "subject"),
                    (180.0, "to"),
                    (200.0, "reverse"),
                    (240.0, "charge"),
                ],
            ),
        ]
    }

    #[test]
    fn money_reads_both_conventions() {
        assert_eq!(money_minor("9.167,00").unwrap(), 916_700);
        assert_eq!(money_minor("13,750.00").unwrap(), 1_375_000);
        assert_eq!(money_minor("303,89").unwrap(), 30_389);
        assert_eq!(money_minor("0,00").unwrap(), 0);
        assert_eq!(money_minor("124.099,92").unwrap(), 12_409_992);
        assert_eq!(money_minor("1,450,082.00").unwrap(), 145_008_200);
        assert!(money_minor("Viškovo").is_err());
        assert_eq!(quantity_milli("1").unwrap(), 1000);
        assert_eq!(quantity_milli("1.5").unwrap(), 1500);
        assert_eq!(quantity_milli("0,25").unwrap(), 250);
    }

    #[test]
    fn the_january_invoice_reads_with_its_descriptions_whole() {
        let p = parse(&january(), "january.pdf", b"%PDF").unwrap();
        assert_eq!(p.number(), "1-1-1-2025");
        assert_eq!(p.kind, "invoice");
        assert_eq!(p.issued_at.to_string(), "2025-01-31 09:15:00");
        assert_eq!(p.due_date.to_string(), "2025-02-15");
        assert_eq!(p.delivery_date.to_string(), "2025-01-31");
        assert_eq!(p.place_of_issue, "Viškovo");
        assert_eq!(p.client_name, "Eiger Oy");
        assert_eq!(p.client_country, "FI");
        assert_eq!(p.client_tax_id, "FI32746464");
        assert_eq!(
            p.client_address,
            vec![
                "c/o Equilibrium Group Oy",
                "Meritullinkatu 1B 00170 Helsinki"
            ]
        );
        assert_eq!(p.vat_treatment, VatTreatment::ReverseChargeEu);
        let d: Vec<&str> = p.lines.iter().map(|l| l.description.as_str()).collect();
        assert_eq!(
            d,
            vec![
                "Software development services for January 2025",
                "Bonus",
                "Flight Reimbursement, Zagreb - Lisbon"
            ]
        );
        assert_eq!(p.lines[0].unit_price_minor, 916_700);
        assert_eq!(p.lines[2].amount_minor, 30_389);
        assert_eq!(p.subtotal_minor, 1_647_089);
        assert_eq!(p.vat_minor, 0);
        assert_eq!(p.total_minor, 1_647_089);
        assert_eq!(p.currency, "EUR");
    }

    #[test]
    fn figures_that_do_not_add_up_are_refused() {
        let mut lines = january();
        // The subtotal printed one cent off.
        let sub = lines
            .iter_mut()
            .find(|l| l.words[0].text == "16.470,89")
            .unwrap();
        sub.words[0].text = "16.470,88".into();
        let e = parse(&lines, "x.pdf", b"%PDF").unwrap_err();
        assert!(matches!(e, ImportError::Arithmetic(_)), "{e}");
    }

    #[test]
    fn the_bbox_xml_becomes_lines_with_pages() {
        let xml = r#"<doc><page width="612" height="1008"><flow><block>
            <line xMin="55.1" yMin="63.6" xMax="128.8" yMax="78.6">
              <word xMin="55.1" yMin="63.6" xMax="93.9" yMax="78.6">Inorbit</word>
              <word xMin="97.0" yMin="63.6" xMax="128.8" yMax="78.6">d.o.o.</word>
            </line></block></flow></page>
            <page width="612" height="1008"><flow><block>
            <line xMin="10" yMin="20" xMax="30" yMax="32"><word xMin="10" yMin="20" xMax="30" yMax="32">Two&amp;more</word></line>
            </block></flow></page></doc>"#;
        let lines = lines_of_bbox(xml);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].page, 1);
        assert_eq!(lines[0].text(), "Inorbit d.o.o.");
        assert_eq!(lines[1].page, 2);
        assert_eq!(lines[1].words[0].text, "Two&more");
    }
}
