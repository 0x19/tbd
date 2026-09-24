//! What a receipt says, read out of its text: who, when, how much, which.
//!
//! Rules, not a model. Supplier invoices from the handful of vendors a small
//! company pays (Stripe-billed `SaaS`, Google, Hetzner, telecoms) label their
//! totals and dates in a few predictable ways, and a rule that names the
//! label is one the person can read when it is wrong. Every field says how
//! it was found ([`By`]), so the page can show a guess as a guess, and a
//! person's correction (`declared`) always wins over a re-read.

use std::sync::LazyLock;

use chrono::NaiveDate;
use regex::Regex;

/// A pattern written here, compiled once. A pattern that does not compile
/// is a bug in this file, caught by its tests, not a runtime condition.
fn re(pattern: &str) -> Regex {
    Regex::new(pattern).unwrap_or_else(|e| panic!("regex {pattern}: {e}"))
}

/// How a field was found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum By {
    /// Beside a label naming it ("Amount due", "Date of issue").
    Label,
    /// From the mailbox sender: their display name or domain.
    Sender,
    /// The first plausible thing in the text: a date, a company line.
    First,
    /// The mail's receipt time, for want of a date in the document.
    Received,
}

impl By {
    /// The wire word.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Label => "label",
            Self::Sender => "sender",
            Self::First => "first",
            Self::Received => "received",
        }
    }
}

/// What was read. Each field carries how.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Fields {
    /// The supplier.
    pub vendor: Option<(String, By)>,
    /// The document's own date.
    pub date: Option<(NaiveDate, By)>,
    /// The amount due, in minor units, with its ISO currency.
    pub amount: Option<(i64, String, By)>,
    /// The supplier's number for the document.
    pub invoice_no: Option<(String, By)>,
}

/// Read the fields from `text`, helped by the mail it came in: `sender` as
/// the mailbox gave it (`Name <addr>` or `addr`), `received` for a date of
/// last resort, and `filename`, whose own date (`Hetzner_2026-08-11_…`)
/// settles which way round a slash date is written.
#[must_use]
pub fn read(text: &str, sender: &str, received: Option<NaiveDate>, filename: &str) -> Fields {
    let lines: Vec<&str> = text.lines().map(str::trim).collect();
    // The anchor a slash date is read against: the file's own date, else
    // the mail's. "11/08/2026" is August to a German biller and November
    // to an American one; the nearer reading to the anchor wins.
    let anchor = DATE_ISO
        .captures(filename)
        .and_then(|c| NaiveDate::from_ymd_opt(c[1].parse().ok()?, num(&c[2])?, num(&c[3])?))
        .or(received);
    Fields {
        vendor: vendor(&lines, sender),
        date: date(&lines, anchor).or_else(|| received.map(|d| (d, By::Received))),
        amount: amount(&lines),
        invoice_no: invoice_no(&lines),
    }
}

// ---------------------------------------------------------------- amount

/// An amount with a currency mark before or after it. The number may group
/// thousands with `,`, `.` or a space and carry up to two decimals either way.
/// A trailing mark must not be the leading mark of the next figure: in
/// "August 29, 2026 $75.00" and "Dec 24, 2025€ 1,328.02" the year is not
/// money. `best_money` drops a trailing mark that a figure follows. A code
/// is a whole word: `EUROPA` after a date is not euros.
static AMOUNT: LazyLock<Regex> = LazyLock::new(|| {
    re(r"(?x)
        (?:(?P<pre>€|\$|£|\b(?:EUR|USD|GBP|CHF|HRK|kn)\b)\s?)?
        (?P<num>\d{1,3}(?:[.,\ ]\d{3})+(?:[.,]\d{1,2})?|\d+(?:[.,]\d{1,2})?)\b
        (?:\s?(?P<post>€|\$|£|\b(?:EUR|USD|GBP|CHF|HRK|kn)\b))?")
});

/// Labels that name the figure we want, best first. A line's best label
/// decides its rank; among equals the last line wins, since a summary
/// block closes an invoice.
const AMOUNT_LABELS: &[(&str, u8)] = &[
    ("amount due", 5),
    ("total due", 5),
    ("total paid", 5),
    ("amount paid", 5),
    ("za platiti", 5),
    ("grand total", 5),
    ("gesamtbetrag", 5),
    ("zu zahlen", 5),
    ("total in ", 4),
    ("total (incl", 4),
    ("total incl", 4),
    ("total", 3),
    ("ukupno", 3),
    ("gesamt", 3),
    ("summe", 3),
    ("betrag", 3),
    ("iznos", 2),
    ("amount", 2),
    (" due ", 2),
];

fn amount(lines: &[&str]) -> Option<(i64, String, By)> {
    let mut best: Option<(u8, i64, String)> = None;
    for (i, line) in lines.iter().enumerate() {
        let lower = line.to_ascii_lowercase();
        if lower.contains("subtotal") || lower.contains("sub total") || lower.contains("excl") {
            continue;
        }
        // The reader breaks words ("Amount pai d"), so a label is also
        // sought with every space removed.
        let tight: String = lower.chars().filter(|c| !c.is_whitespace()).collect();
        let Some(rank) = AMOUNT_LABELS
            .iter()
            .filter(|(l, _)| {
                lower.contains(l) || tight.contains(&l.replace(' ', "").trim().to_owned())
            })
            .map(|(_, r)| *r)
            .max()
        else {
            continue;
        };
        // The figure is on the label's line, or the reader put it on the
        // next one or two.
        let window = &lines[i..lines.len().min(i + 3)];
        let Some((minor, currency)) = window.iter().find_map(|l| best_money(l)) else {
            continue;
        };
        if best.as_ref().is_none_or(|(r, _, _)| rank >= *r) {
            best = Some((rank, minor, currency));
        }
    }
    if let Some((_, minor, currency)) = best {
        return Some((minor, currency, By::Label));
    }
    // No label anywhere: the last figure with a currency mark.
    lines
        .iter()
        .rev()
        .find_map(|l| best_money(l))
        .map(|(m, c)| (m, c, By::First))
}

/// The largest money figure on a line, as minor units and ISO currency. A
/// summary line lists net, tax and gross side by side, and the gross is the
/// largest; a single "Amount due" is its own largest. A line that names a
/// currency once ("Total (€) / 14,500.82") lends it to its bare figures.
fn best_money(line: &str) -> Option<(i64, String)> {
    let marked = AMOUNT
        .captures_iter(line)
        .filter_map(|c| {
            let pre = c.name("pre").map(|m| m.as_str());
            // A mark with a figure after it belongs to that figure.
            let post = c
                .name("post")
                .filter(|m| {
                    !line[m.end()..]
                        .trim_start()
                        .starts_with(|ch: char| ch.is_ascii_digit())
                })
                .map(|m| m.as_str());
            // An explicit code beats a symbol: "$50.00 USD" is dollars,
            // and "€ 12,00 EUR" agrees with itself.
            let currency = [post, pre]
                .into_iter()
                .flatten()
                .max_by_key(|m| m.len())
                .map(iso)?;
            let minor = minor_units(&c["num"])?;
            Some((minor, currency))
        })
        .max_by_key(|(m, _)| *m);
    if marked.is_some() {
        return marked;
    }
    let mark = MARK.find(line)?;
    let currency = iso(mark.as_str());
    BARE.find_iter(line)
        .filter_map(|m| minor_units(m.as_str()))
        .max()
        .map(|m| (m, currency))
}

/// A currency mark on its own, for a line whose figures carry none.
static MARK: LazyLock<Regex> =
    LazyLock::new(|| re(r"€|\$|£|\bEUR\b|\bUSD\b|\bGBP\b|\bCHF\b|\bHRK\b"));

/// A figure with decimals or thousands grouping: money, not a count or a year.
static BARE: LazyLock<Regex> =
    LazyLock::new(|| re(r"\b\d{1,3}(?:[.,]\d{3})+(?:[.,]\d{1,2})?\b|\b\d+[.,]\d{2}\b"));

fn iso(mark: &str) -> String {
    match mark {
        "€" => "EUR",
        "$" => "USD",
        "£" => "GBP",
        "kn" => "HRK",
        code => code,
    }
    .to_owned()
}

/// "13,750.00" → `1_375_000`; "25.021,67" → `2_502_167`; "546,67" → `54_667`;
/// "1,000" → `100_000` (a grouped thousand, not one with three decimals).
fn minor_units(num: &str) -> Option<i64> {
    let cleaned: String = num.chars().filter(|c| *c != ' ').collect();
    let last_dot = cleaned.rfind('.');
    let last_comma = cleaned.rfind(',');
    let decimal_at = match (last_dot, last_comma) {
        (Some(d), Some(c)) => Some(d.max(c)),
        (Some(p), None) | (None, Some(p)) => {
            // One kind of separator: decimal if it is the only one and is
            // followed by one or two digits; grouping otherwise.
            let tail = cleaned.len() - p - 1;
            let count = cleaned.matches(cleaned.as_bytes()[p] as char).count();
            if count == 1 && tail == 3 && cleaned.starts_with('0') {
                // "0.015": a unit price with three decimals, not a grouped
                // thousand. Not money.
                return None;
            }
            (count == 1 && (1..=2).contains(&tail)).then_some(p)
        }
        (None, None) => None,
    };
    let (whole, frac) = match decimal_at {
        Some(p) => (&cleaned[..p], &cleaned[p + 1..]),
        None => (cleaned.as_str(), ""),
    };
    let whole: String = whole.chars().filter(char::is_ascii_digit).collect();
    if whole.is_empty() && frac.is_empty() {
        return None;
    }
    let whole: i64 = if whole.is_empty() {
        0
    } else {
        whole.parse().ok()?
    };
    let frac: i64 = match frac.len() {
        0 => 0,
        1 => frac.parse::<i64>().ok()? * 10,
        2 => frac.parse().ok()?,
        _ => return None,
    };
    whole.checked_mul(100)?.checked_add(frac)
}

// ------------------------------------------------------------------ date

static DATE_ISO: LazyLock<Regex> = LazyLock::new(|| re(r"\b(\d{4})-(\d{2})-(\d{2})\b"));
static DATE_EU: LazyLock<Regex> = LazyLock::new(|| re(r"\b(\d{1,2})\.\s?(\d{1,2})\.\s?(\d{4})\b"));
static DATE_US: LazyLock<Regex> = LazyLock::new(|| re(r"\b(\d{1,2})/(\d{1,2})/(\d{2}|\d{4})\b"));
static DATE_MDY: LazyLock<Regex> = LazyLock::new(|| {
    re(
        r"(?i)\b(jan|feb|mar|apr|may|jun|jul|aug|sep|sept|oct|nov|dec)[a-z]*\.?\s+(\d{1,2})(?:st|nd|rd|th)?,?\s+(\d{4})\b",
    )
});
static DATE_DMY: LazyLock<Regex> = LazyLock::new(|| {
    re(
        r"(?i)\b(\d{1,2})\.?\s+(jan|feb|mar|apr|may|jun|jul|aug|sep|sept|oct|nov|dec)[a-z]*\.?,?\s+(\d{4})\b",
    )
});

/// Labels for the document's own date. "Due" lines are skipped: a due date
/// is not the invoice date, and Stripe puts both a line apart.
const DATE_LABELS: &[&str] = &[
    "date of issue",
    "invoice date",
    "issue date",
    "issued",
    "payment date",
    "receipt date",
    "datum i vrijeme",
    "rechnungsdatum",
    "datum",
    "date:",
    "date ",
];

fn date(lines: &[&str], anchor: Option<NaiveDate>) -> Option<(NaiveDate, By)> {
    for (i, line) in lines.iter().enumerate() {
        let lower = line.to_ascii_lowercase();
        if lower.contains("due") || lower.contains("dospije") || lower.contains("fällig") {
            continue;
        }
        if !DATE_LABELS.iter().any(|l| lower.contains(l)) {
            continue;
        }
        let window = &lines[i..lines.len().min(i + 3)];
        if let Some(d) = window.iter().find_map(|l| first_date_near(l, anchor)) {
            return Some((d, By::Label));
        }
    }
    lines
        .iter()
        .find_map(|l| first_date_near(l, anchor))
        .map(|d| (d, By::First))
}

/// The first date on the line, a slash date read the way that lands
/// nearest `anchor`; without one, day/month for a four-digit year (the
/// European billers) and month/day for a two-digit one (the American).
fn first_date_near(line: &str, anchor: Option<NaiveDate>) -> Option<NaiveDate> {
    first_date_with(line, |a, b, y| {
        let (dm, md) = (
            NaiveDate::from_ymd_opt(y, b, a),
            NaiveDate::from_ymd_opt(y, a, b),
        );
        match (dm, md, anchor) {
            (Some(dm), Some(md), Some(at)) => {
                if (dm - at).num_days().abs() <= (md - at).num_days().abs() {
                    Some(dm)
                } else {
                    Some(md)
                }
            }
            (Some(dm), Some(md), None) => Some(if y >= 2000 && line.contains(&format!("/{y}")) {
                dm
            } else {
                md
            }),
            (Some(d), None, _) | (None, Some(d), _) => Some(d),
            (None, None, _) => None,
        }
    })
}

fn first_date(line: &str) -> Option<NaiveDate> {
    first_date_near(line, None)
}

fn month(name: &str) -> Option<u32> {
    let m = name.to_ascii_lowercase();
    let n = match &m[..3] {
        "jan" => 1,
        "feb" => 2,
        "mar" => 3,
        "apr" => 4,
        "may" => 5,
        "jun" => 6,
        "jul" => 7,
        "aug" => 8,
        "sep" => 9,
        "oct" => 10,
        "nov" => 11,
        "dec" => 12,
        _ => return None,
    };
    Some(n)
}

fn num(s: &str) -> Option<u32> {
    s.parse().ok()
}

/// Every date form on the line; `slash` decides a `a/b/y` form, given the
/// two numbers and the year.
fn first_date_with(
    line: &str,
    slash: impl Fn(u32, u32, i32) -> Option<NaiveDate>,
) -> Option<NaiveDate> {
    let mut found: Vec<(usize, NaiveDate)> = Vec::new();
    for c in DATE_ISO.captures_iter(line) {
        if let Some(d) = NaiveDate::from_ymd_opt(c[1].parse().ok()?, num(&c[2])?, num(&c[3])?) {
            found.push((c.get(0)?.start(), d));
        }
    }
    for c in DATE_EU.captures_iter(line) {
        if let Some(d) = NaiveDate::from_ymd_opt(c[3].parse().ok()?, num(&c[2])?, num(&c[1])?) {
            found.push((c.get(0)?.start(), d));
        }
    }
    for c in DATE_US.captures_iter(line) {
        // The slash form is written both ways: "06/10/26" by an American
        // biller, "11/08/2026" by a German one. The caller decides.
        let year: i32 = c[3].parse().ok()?;
        let year = if c[3].len() == 2 { 2000 + year } else { year };
        if let Some(d) = slash(num(&c[1])?, num(&c[2])?, year) {
            found.push((c.get(0)?.start(), d));
        }
    }
    for c in DATE_MDY.captures_iter(line) {
        if let Some(d) = NaiveDate::from_ymd_opt(c[3].parse().ok()?, month(&c[1])?, num(&c[2])?) {
            found.push((c.get(0)?.start(), d));
        }
    }
    for c in DATE_DMY.captures_iter(line) {
        if let Some(d) = NaiveDate::from_ymd_opt(c[3].parse().ok()?, month(&c[2])?, num(&c[1])?) {
            found.push((c.get(0)?.start(), d));
        }
    }
    found.into_iter().min_by_key(|(at, _)| *at).map(|(_, d)| d)
}

// ---------------------------------------------------------------- vendor

/// Names we know, keyed by what their invoices and addresses say. Lowercase
/// needle, display name. Longer needles first where one contains another.
const KNOWN: &[(&str, &str)] = &[
    ("google cloud", "Google Cloud"),
    ("google workspace", "Google Workspace"),
    ("google", "Google"),
    ("anthropic", "Anthropic"),
    ("openai", "OpenAI"),
    ("cloudflare", "Cloudflare"),
    ("hetzner", "Hetzner"),
    ("a medium corporation", "Medium"),
    ("medium.com", "Medium"),
    ("excalidraw", "Excalidraw"),
    ("github", "GitHub"),
    ("digitalocean", "DigitalOcean"),
    ("amazon web services", "Amazon Web Services"),
    ("aws emea", "Amazon Web Services"),
    ("vercel", "Vercel"),
    ("jetbrains", "JetBrains"),
    ("microsoft", "Microsoft"),
    ("atlassian", "Atlassian"),
    ("slack", "Slack"),
    ("notion", "Notion"),
    ("figma", "Figma"),
    ("1password", "1Password"),
    ("namecheap", "Namecheap"),
    ("apple", "Apple"),
    ("hrvatski telekom", "Hrvatski Telekom"),
    ("a1 hrvatska", "A1"),
    ("tenderly", "Tenderly"),
];

/// Mailers whose domain names the pipe, not the supplier.
const RELAYS: &[&str] = &[
    "stripe.com",
    "sendgrid.net",
    "mailgun.org",
    "amazonses.com",
    "mandrillapp.com",
    "mailchimp.com",
    "sparkpostmail.com",
    "postmarkapp.com",
    "intercom-mail.com",
    "hubspot.com",
];

/// Mailbox prefixes that say nothing about who.
const GENERIC_NAMES: &[&str] = &[
    "noreply",
    "no-reply",
    "no_reply",
    "donotreply",
    "billing",
    "invoice",
    "invoices",
    "receipts",
    "receipt",
    "notifications",
    "notification",
    "payments",
    "support",
    "team",
    "hello",
    "info",
    "mail",
    "mailer",
    "statements",
    "accounts",
    "accounting",
    "ar",
];

fn vendor(lines: &[&str], sender: &str) -> Option<(String, By)> {
    // 1. The sender's display name, unless it is a bare address or a
    //    generic word: "Anthropic <invoice+statements@mail.anthropic.com>".
    let (display, addr) = split_sender(sender);
    let domain = addr.rsplit('@').next().unwrap_or("").to_ascii_lowercase();
    let relayed = RELAYS
        .iter()
        .any(|r| domain == *r || domain.ends_with(&format!(".{r}")));
    if let Some(name) = display.filter(|d| plausible_name(d)) {
        let known = KNOWN
            .iter()
            .find(|(needle, _)| name.to_ascii_lowercase().contains(needle))
            .map(|(_, v)| (*v).to_owned());
        return Some((known.unwrap_or_else(|| name.to_owned()), By::Sender));
    }
    // 2. A name we know, anywhere in the text (top of the page first).
    let head = lines
        .iter()
        .take(40)
        .map(|l| l.to_ascii_lowercase())
        .collect::<Vec<_>>();
    if let Some((_, v)) = KNOWN
        .iter()
        .find(|(needle, _)| head.iter().any(|l| l.contains(needle)))
    {
        return Some(((*v).to_owned(), By::First));
    }
    // 3. The sender's domain, when it is theirs and not a relay.
    if !domain.is_empty()
        && !relayed
        && let Some(name) = name_from_domain(&domain)
    {
        return Some((name, By::Sender));
    }
    // 4. The first line that looks like a company, not a heading.
    lines
        .iter()
        .take(12)
        .map(|l| {
            l.split([' ', '\u{2022}', '|'])
                .take(6)
                .collect::<Vec<_>>()
                .join(" ")
        })
        .map(|l| l.split(" • ").next().unwrap_or("").trim().to_owned())
        .find(|l| {
            let lower = l.to_ascii_lowercase();
            l.chars().any(char::is_alphabetic)
                && l.len() >= 3
                && !lower.starts_with("invoice")
                && !lower.starts_with("receipt")
                && !lower.starts_with("tax invoice")
                && !lower.starts_with("račun")
                && !lower.contains("page ")
                && first_date(l).is_none()
        })
        .map(|l| (l, By::First))
}

fn split_sender(sender: &str) -> (Option<&str>, &str) {
    let sender = sender.trim();
    if let Some((name, rest)) = sender.split_once('<') {
        let addr = rest.trim_end_matches('>').trim();
        let name = name.trim().trim_matches('"').trim();
        return ((!name.is_empty()).then_some(name), addr);
    }
    (None, sender)
}

fn plausible_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    !name.contains('@')
        && name.chars().any(char::is_alphabetic)
        && !GENERIC_NAMES
            .iter()
            .any(|g| lower == *g || lower.starts_with(&format!("{g} ")))
        && !lower.contains("do not reply")
        && !lower.contains("noreply")
}

/// `billing.hetzner.com` → `Hetzner`; `mail.anthropic.com` → `Anthropic`;
/// `hrvatskitelekom.hr` → `Hrvatskitelekom` (a correction away from right).
fn name_from_domain(domain: &str) -> Option<String> {
    let labels: Vec<&str> = domain.split('.').collect();
    if labels.len() < 2 {
        return None;
    }
    // The registrable label: second from the end, or third under a
    // two-part public suffix such as co.uk / com.hr.
    let tld = labels[labels.len() - 1];
    let second = labels[labels.len() - 2];
    let base = if matches!(second, "co" | "com" | "org" | "net" | "ac" | "gov") && tld.len() == 2 {
        labels.get(labels.len().checked_sub(3)?)?
    } else {
        second
    };
    let base = base.trim();
    if base.is_empty() || GENERIC_NAMES.contains(&base) {
        return None;
    }
    if let Some((_, v)) = KNOWN.iter().find(|(needle, _)| *needle == base) {
        return Some((*v).to_owned());
    }
    let mut chars = base.chars();
    let first = chars.next()?.to_uppercase().collect::<String>();
    Some(first + chars.as_str())
}

// ------------------------------------------------------------ invoice no.

/// The keyword, then any run of label words ("Number / Broj računa",
/// "no.:", "#", an em dash), then the value. Croatian invoices say "Broj
/// računa", "Broj ovog računa", "Račun br." and "Račun-otpremnica br.";
/// an order confirmation says "narudžbe #".
static INVOICE_NO: LazyLock<Regex> = LazyLock::new(|| {
    re(r"(?ix)
        \b(?:broj\s*(?:ovog\s*)?ra[čc]una|invoice|receipt|rechnung|ra[čc]un(?:-otpremnica)?|racun|faktura|narud[žz]b[ae]|order)
        (?:\s*(?:number|num|no\.?|nr\.?|br\.?|\#|id|/|broj|ra[čc]una|invoice|receipt|:|—|–|-))*
        \s*
        (?P<no>[A-Za-z0-9][A-Za-z0-9/-]{3,})")
});

/// The same keywords with the value on their left: Adobe's columns read
/// "3364411459Invoice Number".
static INVOICE_NO_BEFORE: LazyLock<Regex> =
    LazyLock::new(|| re(r"(?i)(?P<no>\b\d{6,})\s*(?:invoice|receipt)\s*(?:number|no\.?|nr\.?|\#)"));

/// The reader breaks words across glyph runs: "Inv oi ce number", "Recei p t
/// number", "561 1703644". With every space removed the keyword is whole and
/// the value is the upper-case run that follows, stopped where prose starts.
static INVOICE_NO_TIGHT: LazyLock<Regex> = LazyLock::new(|| {
    re(r"(?x)
        (?i:broj(?:ovog)?ra[čc]una|invoice|receipt|rechnung|ra[čc]un|racun|faktura|narud[žz]b[ae])
        (?i:number|num|no\.?|nr\.?|br\.?|\#|id|/|broj|ra[čc]una|invoice|receipt|:|—|–|-)*
        (?P<no>[A-Z0-9][A-Z0-9/-]{3,})")
});

/// A value that is really a date or a year, which the label "Invoice date"
/// or a column layout puts where the number would be.
static DATE_LIKE: LazyLock<Regex> =
    LazyLock::new(|| re(r"(?i)^\d{1,2}-[a-z]{3}-\d{2,4}$|^(?:19|20)\d{2}$"));

fn plausible_no(no: &str) -> bool {
    let lower = no.to_ascii_lowercase();
    no.chars().any(|c| c.is_ascii_digit())
        && !matches!(
            lower.as_str(),
            "number" | "date" | "amount" | "total" | "period" | "details" | "summary"
        )
        && !DATE_LIKE.is_match(no)
        && first_date(no).is_none()
}

/// Trim the punctuation a sentence leaves, and the prose the reader glued
/// on: `G9DMV7ZW0009Paymentmethod` ends where a capitalised word begins.
/// A lower-case hex number (`930d0222020d`) has no such word and stays.
fn clean_no(no: &str) -> String {
    let chars: Vec<char> = no.chars().collect();
    let end = (1..chars.len())
        .find(|&i| {
            chars[i].is_ascii_uppercase() && chars.get(i + 1).is_some_and(char::is_ascii_lowercase)
        })
        .unwrap_or(chars.len());
    chars[..end]
        .iter()
        .collect::<String>()
        .trim_end_matches(['.', ',', ':', '-', '/'])
        .to_owned()
}

fn invoice_no(lines: &[&str]) -> Option<(String, By)> {
    for line in lines.iter().take(80) {
        for c in INVOICE_NO.captures_iter(line) {
            let no = &c["no"];
            if plausible_no(no) {
                return Some((clean_no(no), By::Label));
            }
        }
        if let Some(c) = INVOICE_NO_BEFORE.captures(line) {
            return Some((clean_no(&c["no"]), By::Label));
        }
        let tight: String = line.chars().filter(|c| !c.is_whitespace()).collect();
        for c in INVOICE_NO_TIGHT.captures_iter(&tight) {
            let no = clean_no(&c["no"]);
            if no.len() >= 4 && plausible_no(&no) {
                return Some((no, By::Label));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn a_year_before_a_dollar_figure_is_not_money() {
        assert_eq!(best_money("29.04.2026 EUROPA 92 MUSTANG"), None);
        assert_eq!(
            best_money("Amount paid on Dec 24, 2025€ 1,328.02"),
            Some((132_802, "EUR".into()))
        );
        assert_eq!(
            best_money("Visa - 9355 August 29, 2026 $75.00 2278 6704"),
            Some((7_500, "USD".into()))
        );
        assert_eq!(
            best_money("$75.00 pai d on August 29, 2026"),
            Some((7_500, "USD".into()))
        );
        assert_eq!(
            best_money("Total 1.234,50 EUR"),
            Some((123_450, "EUR".into()))
        );
        assert_eq!(best_money("USD 12.00"), Some((1_200, "USD".into())));
        let lines = [
            "Receipt",
            "Amount pai d $75.00",
            "Visa - 9355 August 29, 2026 $75.00 2278 6704",
        ];
        assert_eq!(amount(&lines), Some((7_500, "USD".into(), By::Label)));
    }

    #[test]
    fn minor_units_read_every_grouping() {
        assert_eq!(minor_units("13,750.00"), Some(1_375_000));
        assert_eq!(minor_units("25.021,67"), Some(2_502_167));
        assert_eq!(minor_units("546,67"), Some(54_667));
        assert_eq!(minor_units("32.40"), Some(3_240));
        assert_eq!(minor_units("7"), Some(700));
        assert_eq!(minor_units("1,000"), Some(100_000), "a grouped thousand");
        assert_eq!(minor_units("1,000,000"), Some(100_000_000));
        assert_eq!(minor_units("1 234,50"), Some(123_450));
        assert_eq!(
            minor_units("0.015"),
            None,
            "three decimals is a unit price, not money"
        );
    }

    #[test]
    fn stripe_layout_amount_due_wins_over_line_items() {
        let text = "Invoice\nInvoice number G9DMV7ZW-0015\nDate of issue June 25, 2026\nDate due June 25, 2026\n\
                    Anthropic, PBC Bill to\n548 Market Street Example d.o.o.\nsupport@anthropic.com\n\
                    €180.00 due June 25, 2026\nPay online\n\
                    Description Qty Unit price Amount\nMax plan - 20x 1 €180.00 €180.00\nJun 25–Jul 25, 2026\n\
                    Subtotal €180.00\nTotal €180.00\nAmount due €180.00\nPage 1 of 1";
        let f = read(
            text,
            "Anthropic <invoice+statements@mail.anthropic.com>",
            None,
            "",
        );
        assert_eq!(f.vendor, Some(("Anthropic".into(), By::Sender)));
        assert_eq!(f.date, Some((d(2026, 6, 25), By::Label)));
        assert_eq!(f.amount, Some((18_000, "EUR".into(), By::Label)));
        assert_eq!(f.invoice_no, Some(("G9DMV7ZW-0015".into(), By::Label)));
    }

    #[test]
    fn explicit_code_beats_symbol_and_relay_domain_is_ignored() {
        let text = "Invoice\nInvoice number B3A33354-0023\nDate of issue June 29, 2026\nDate due June 29, 2026\n\
                    OpenAI OpCo, LLC\n$50.00 USD due June 29, 2026\nChatGPT Business Subscription (per seat) 2 $25.00 0% $50.00\n\
                    Subtotal $50.00\nTotal $50.00\nAmount due $50.00 USD";
        let f = read(text, "receipts+acct_1@stripe.com", None, "");
        assert_eq!(
            f.vendor,
            Some(("OpenAI".into(), By::First)),
            "from the text, not stripe.com"
        );
        assert_eq!(f.amount, Some((5_000, "USD".into(), By::Label)));
        assert_eq!(f.invoice_no.unwrap().0, "B3A33354-0023");
    }

    #[test]
    fn google_total_in_eur_and_mdy_date() {
        let text = "Google Cloud EMEA Limited\nVelasco\nClanwilliam Place\nInvoice Dublin 2\nIreland\n\
                    Invoice number: 5611703644\nVAT number: IE3668997OH\nBill to\nExample\nDetails Google Workspace\n\
                    Invoice number\n5611703644\nInvoice date\nJun 30, 2026 Total in EUR €32.40\nBilling ID\n\
                    Summary for Jun 1, 2026 - Jun 30, 2026\nSubtotal in EUR €32.40\nVAT (0%) €0.00\nTotal in EUR €32.40";
        let f = read(text, "payments-noreply@google.com", None, "");
        assert_eq!(f.vendor.unwrap(), ("Google Cloud".into(), By::First));
        assert_eq!(f.date, Some((d(2026, 6, 30), By::Label)));
        assert_eq!(f.amount, Some((3_240, "EUR".into(), By::Label)));
        assert_eq!(f.invoice_no.unwrap().0, "5611703644");
    }

    #[test]
    fn hetzner_total_excl_vat_is_skipped_for_the_total() {
        let text = "Hetzner Online GmbH • Industriestr. 25 • 91710 Gunzenhausen • Germany\nExample d.o.o.\n\
                    Invoice 080000969768\nOverview\nService Period Total (excl. VAT) Total\n\
                    Project \"Ameba\" 05/2026 € 62.99 € 62.99\nStorage 05/2026 € 6.49 € 6.49\nTotal € 69.48 € 69.48\n\
                    Tax code Tax rate Total (excl. VAT) Tax\nA7 0% € 69.48 € 0.00\nTotal € 69.48 € 0.00\n\
                    Invoice date: 11.06.2026\nDue upon receipt.";
        let f = read(text, "Hetzner Online GmbH <billing@hetzner.com>", None, "");
        assert_eq!(f.vendor, Some(("Hetzner".into(), By::Sender)));
        assert_eq!(f.amount, Some((6_948, "EUR".into(), By::Label)));
        assert_eq!(f.date, Some((d(2026, 6, 11), By::Label)));
        assert_eq!(f.invoice_no.unwrap().0, "080000969768");
    }

    #[test]
    fn medium_us_slash_date_and_total_paid() {
        let text = "7/30/26, 11:02 AM Medium\nInvoice 930d0222020d\nPayment date: 06/10/26 · Status: Paid in full\n\
                    From To\nA Medium Corporation Example d.o.o.\nDescription Price\n\
                    Medium Monthly Membership (06/10/26 - 07/10/26) $5.00\nTotal $5.00 USD\nTotal paid $5.00 USD";
        let f = read(text, "Medium <noreply@medium.com>", None, "");
        assert_eq!(f.vendor, Some(("Medium".into(), By::Sender)));
        assert_eq!(
            f.date,
            Some((d(2026, 6, 10), By::Label)),
            "the payment date, not the print header"
        );
        assert_eq!(f.amount, Some((500, "USD".into(), By::Label)));
        assert_eq!(f.invoice_no.unwrap().0, "930d0222020d");
    }

    #[test]
    fn croatian_invoice_reads_broj_racuna_and_za_platiti() {
        let text = "Invoice Number / Broj računa 9-1-1-2026\nDate and time / Datum i vrijeme: 31.08.2026. 13:55\n\
                    Due date / Rok dospijeća: 15.09.2026\nExample d.o.o.\n\
                    Sub Total / Ukupno 14,500.82\nVAT (PDV) 0,00\nTotal Due (€) / 14,500.82\nUkupno za platiti (€)";
        let f = read(text, "", Some(d(2026, 9, 1)), "");
        assert_eq!(f.invoice_no.unwrap().0, "9-1-1-2026");
        assert_eq!(f.date, Some((d(2026, 8, 31), By::Label)));
        // "Total Due (€) / 14,500.82": the mark comes before the figure.
        assert_eq!(f.amount, Some((1_450_082, "EUR".into(), By::Label)));
    }

    #[test]
    fn a_bilingual_heading_yields_the_number_after_both_words() {
        let f = read(
            "Račun / Invoice 9-1-1-2026\nTotal due / Za platiti 14,500.82 EUR",
            "",
            None,
            "",
        );
        assert_eq!(f.invoice_no.unwrap().0, "9-1-1-2026");
        assert_eq!(f.amount, Some((1_450_082, "EUR".into(), By::Label)));
    }

    #[test]
    fn nothing_labelled_falls_back_honestly() {
        let f = read(
            "Thanks for your order\nPaid €12.50 by card",
            "shop@example.hr",
            Some(d(2026, 1, 2)),
            "",
        );
        assert_eq!(f.amount, Some((1_250, "EUR".into(), By::First)));
        assert_eq!(f.date, Some((d(2026, 1, 2), By::Received)));
        assert_eq!(f.vendor, Some(("Example".into(), By::Sender)));
        assert_eq!(f.invoice_no, None);
        assert_eq!(read("", "", None, ""), Fields::default());
    }

    #[test]
    fn a_slash_date_is_read_the_way_the_file_or_the_mail_says() {
        // Hetzner writes day/month; the file name carries the ISO date.
        let text =
            "Hetzner Online GmbH\nInvoice 086001061910\nInvoice date: 11/08/2026\nTotal € 69.48";
        let f = read(
            text,
            "billing@hetzner.com",
            None,
            "Hetzner_2026-08-11_086001061910.pdf",
        );
        assert_eq!(f.date, Some((d(2026, 8, 11), By::Label)));
        // Without a file date, the mail's receipt time decides.
        let f = read(
            text,
            "billing@hetzner.com",
            Some(d(2026, 8, 12)),
            "invoice.pdf",
        );
        assert_eq!(f.date, Some((d(2026, 8, 11), By::Label)));
        // Without either: a four-digit year reads day/month, a two-digit one month/day.
        assert_eq!(
            read(text, "", None, "").date,
            Some((d(2026, 8, 11), By::Label))
        );
        let us = "Payment date: 06/10/26 · Status: Paid";
        assert_eq!(
            read(us, "", None, "").date,
            Some((d(2026, 6, 10), By::Label))
        );
        // An unambiguous one is itself whatever the anchor says.
        let f = read("Invoice date: 25/08/2026", "", Some(d(2026, 1, 1)), "");
        assert_eq!(f.date, Some((d(2026, 8, 25), By::Label)));
    }

    #[test]
    fn a_number_is_read_through_the_reader_s_broken_words() {
        let no = |text: &str| invoice_no(&text.lines().collect::<Vec<_>>()).map(|n| n.0);
        assert_eq!(
            no("Inv oi ce number G9DMV7ZW0009\nPayment method"),
            Some("G9DMV7ZW0009".into())
        );
        assert_eq!(
            no("Receipt\nInv oice number SBIE11477587\nRecei pt number 239297698626"),
            Some("SBIE11477587".into())
        );
        assert_eq!(no("Invoice number: 561 1703644"), Some("5611703644".into()));
        assert_eq!(no("INVOICE\nINVOICE TS1 3 5 3 6"), Some("TS13536".into()));
        assert_eq!(
            no("InvoicenumberG9DMV7ZW0009Paymentmethod"),
            Some("G9DMV7ZW0009".into())
        );
    }

    #[test]
    fn croatian_and_column_layouts_yield_the_number() {
        let no = |text: &str| invoice_no(&text.lines().collect::<Vec<_>>()).map(|n| n.0);
        assert_eq!(
            no("Broj računa: 260007839477-A-1"),
            Some("260007839477-A-1".into())
        );
        assert_eq!(
            no("Broj ovog računa: 5030539125-315-6"),
            Some("5030539125-315-6".into())
        );
        assert_eq!(no("RAČUN br. 1002-02-261"), Some("1002-02-261".into()));
        assert_eq!(
            no("Račun-otpremnica br. 1288/19/200"),
            Some("1288/19/200".into())
        );
        assert_eq!(
            no("Potvrda narudžbe #1762795891284"),
            Some("1762795891284".into())
        );
        assert_eq!(
            no("Invoice #— 186948\nInvoice Date— Jun 03, 2026"),
            Some("186948".into())
        );
        assert_eq!(
            no("3364411459Invoice Number 12-FEB-2026Invoice Date Credit CardPayment Terms"),
            Some("3364411459".into()),
            "the value on the left, not the date on the right"
        );
        assert_eq!(no("Invoice date 2026-09-01\nInvoice for August"), None);
        assert_eq!(no("Račun 2026"), None, "a year is not a number");
    }

    #[test]
    fn domains_give_names() {
        assert_eq!(
            name_from_domain("mail.anthropic.com").as_deref(),
            Some("Anthropic")
        );
        assert_eq!(
            name_from_domain("billing.example.co.uk").as_deref(),
            Some("Example")
        );
        assert_eq!(name_from_domain("noreply.com"), None);
        assert_eq!(name_from_domain("localhost"), None);
    }
}
