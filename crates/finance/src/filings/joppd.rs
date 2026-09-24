//! JOPPD: the monthly report of pay, tax and contributions. It has no
//! `Zaglavlje`; page A carries the date, the report mark, the submitter and
//! the totals, page B one row per recipient.

use chrono::{Datelike, NaiveDate};
use roxmltree::Node;

use super::{Form, Header, ParseError, header, xml};

/// `OznakaIzvjesca` is `yyDDD`: two digits of the year and the day of the
/// year, so `25055` is 24 February 2025. The fallback for the period when
/// `DatumIzvjesca` is missing.
#[must_use]
pub fn mark_date(mark: &str) -> Option<NaiveDate> {
    let mark = mark.trim();
    if mark.len() != 5 || !mark.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let year = 2000 + mark[..2].parse::<i32>().ok()?;
    let day = mark[2..].parse::<u32>().ok()?;
    NaiveDate::from_yo_opt(year, day)
}

/// The header from page A.
///
/// # Errors
/// No OIB in `StranaA/PodnositeljIzvjesca`.
pub fn header(root: Node<'_, '_>, fallback_schema: &str) -> Result<Header, ParseError> {
    let mut h = header::metadata(root, fallback_schema);
    let a = xml::child(root, "StranaA");
    let get = |local: &str| a.and_then(|a| xml::text(a, local));
    h.report_mark = get("OznakaIzvjesca").unwrap_or_default();
    let day = header::date(get("DatumIzvjesca")).or_else(|| mark_date(&h.report_mark));
    h.period_from = day;
    h.period_to = day;
    let submitter = a.and_then(|a| xml::child(a, "PodnositeljIzvjesca"));
    h.oib = submitter
        .and_then(|s| xml::text(s, "OIB"))
        .ok_or(ParseError::NoOib(Form::Joppd))?;
    h.obveznik = header::name(submitter);
    if h.author.is_empty() {
        h.author = header::name(a.and_then(|a| xml::child(a, "IzvjesceSastavio")));
    }
    Ok(h)
}

/// The year a mark names, for a listing filter when the date is absent.
#[must_use]
pub fn mark_year(mark: &str) -> Option<i32> {
    mark_date(mark).map(|d| d.year())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_mark_is_a_day_of_the_year() {
        assert_eq!(mark_date("25055"), NaiveDate::from_ymd_opt(2025, 2, 24));
        assert_eq!(mark_date("15001"), NaiveDate::from_ymd_opt(2015, 1, 1));
        assert!(mark_date("25400").is_none());
        assert!(mark_date("abc").is_none());
        assert_eq!(mark_year("25055"), Some(2025));
    }
}
