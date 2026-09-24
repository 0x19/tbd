//! The header every form but JOPPD shares: `Metapodaci` (who prepared it,
//! when, against which schema) and `Zaglavlje` (the period and the taxpayer).

use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use chrono_tz::Europe::Zagreb;
use roxmltree::Node;

use super::{Form, Header, ParseError, xml};

/// A form date, `YYYY-MM-DD`.
#[must_use]
pub fn date(s: Option<String>) -> Option<NaiveDate> {
    s.and_then(|s| NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d").ok())
}

/// `Metapodaci/Datum`: `2026-04-16T13:31:17`, sometimes with fractions or a
/// zone. Without a zone it is the preparer's clock, taken as Zagreb time.
#[must_use]
pub fn stamp(s: Option<String>) -> Option<DateTime<Utc>> {
    let s = s?;
    let s = s.trim();
    if let Ok(t) = DateTime::parse_from_rfc3339(s) {
        return Some(t.with_timezone(&Utc));
    }
    let local = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f")
        .or_else(|_| NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S"))
        .ok()?;
    local
        .and_local_timezone(Zagreb)
        .earliest()
        .map(|t| t.with_timezone(&Utc))
}

/// The `Metapodaci` block into a header with no period or taxpayer yet.
#[must_use]
pub fn metadata(root: Node<'_, '_>, fallback_schema: &str) -> Header {
    let meta = xml::child(root, "Metapodaci");
    let get = |local: &str| meta.and_then(|m| xml::text(m, local));
    Header {
        schema: get("Uskladjenost").unwrap_or_else(|| fallback_schema.to_owned()),
        period_from: None,
        period_to: None,
        oib: String::new(),
        obveznik: String::new(),
        prepared_at: stamp(get("Datum")),
        author: get("Autor").unwrap_or_default(),
        identifier: get("Identifikator").unwrap_or_default(),
        report_mark: String::new(),
    }
}

/// A name as the form writes it: `Naziv`, or `Ime Prezime`.
#[must_use]
pub fn name(node: Option<Node<'_, '_>>) -> String {
    let Some(n) = node else {
        return String::new();
    };
    if let Some(naziv) = xml::text(n, "Naziv") {
        return naziv;
    }
    let ime = xml::text(n, "Ime").unwrap_or_default();
    let prezime = xml::text(n, "Prezime").unwrap_or_default();
    format!("{ime} {prezime}").trim().to_owned()
}

/// The header of a `Zaglavlje` form.
///
/// # Errors
/// No OIB in `Zaglavlje/Obveznik`.
pub fn zaglavlje(
    root: Node<'_, '_>,
    form: Form,
    fallback_schema: &str,
) -> Result<Header, ParseError> {
    let mut h = metadata(root, fallback_schema);
    let z = xml::child(root, "Zaglavlje");
    let razdoblje = z.and_then(|z| xml::child(z, "Razdoblje"));
    h.period_from = date(razdoblje.and_then(|r| xml::text(r, "DatumOd")));
    h.period_to = date(razdoblje.and_then(|r| xml::text(r, "DatumDo")));
    let obveznik = z.and_then(|z| xml::child(z, "Obveznik"));
    h.oib = obveznik
        .and_then(|o| xml::text(o, "OIB"))
        .ok_or(ParseError::NoOib(form))?;
    h.obveznik = name(obveznik);
    Ok(h)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stamps_take_zagreb_time_unless_a_zone_is_given() {
        let t = stamp(Some("2026-04-16T13:31:17".into())).expect("parses");
        // Summer time: UTC+2.
        assert_eq!(t.to_rfc3339(), "2026-04-16T11:31:17+00:00");
        let t = stamp(Some("2016-11-17T09:30:47.0Z".into())).expect("parses");
        assert_eq!(t.to_rfc3339(), "2016-11-17T09:30:47+00:00");
        assert!(stamp(Some("yesterday".into())).is_none());
    }

    #[test]
    fn a_name_is_naziv_or_ime_prezime() {
        let d = roxmltree::Document::parse("<O><Ime>Pero</Ime><Prezime>Perić</Prezime></O>")
            .expect("well-formed");
        assert_eq!(name(Some(d.root_element())), "Pero Perić");
        assert_eq!(name(None), "");
    }
}
