//! Filings: the company's ePorezna forms, taken in as the XML ePorezna hands
//! back and read into figures.
//!
//! A form uploaded through `UploadDocument` with an XML content type becomes
//! a document of kind `filing`; [`parse`] recognises the form from the root
//! element (`ObrazacPD`, `ObrazacPDV`, `ObrazacPDVS`, `ObrazacZP`,
//! `ObrazacJOPPD`, `ObrazacPDIPO`, `ObrazacTZ`), reads its header (period,
//! OIB, who prepared it and when) and flattens its body into
//! `path -> decimal string` values plus row objects ([`xml`]). The upload
//! refuses a form for another OIB than the party's; [`read`] is the one
//! path that writes `finance.filings`, used by the upload, `ExtractDocument`
//! and the start-up backfill. Amounts stay the strings the form carries;
//! [`minor`] turns one into minor units exactly when a number is needed.

pub mod form;
pub mod header;
pub mod joppd;
pub mod store;
pub mod xml;

use std::collections::BTreeMap;

use chrono::{DateTime, NaiveDate, Utc};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::connectors::store::StoreError;

/// Stamped on every row this reader writes; a change to what it reads is a
/// new version, and a re-read refreshes the row.
pub const PARSER_VERSION: &str = "filings/1";

/// Every ePorezna form namespace starts with this; what follows is
/// `Obrazac<FORM>/v<major>-<minor>`.
const NAMESPACE_PREFIX: &str = "http://e-porezna.porezna-uprava.hr/sheme/zahtjevi/";

/// The forms the reader knows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Form {
    /// Prijava poreza na dobit, the yearly corporate tax return.
    Pd,
    /// Prijava PDV-a, the VAT return.
    Pdv,
    /// PDV-S, services received from the EU.
    PdvS,
    /// Zbirna prijava, supplies to EU businesses.
    Zp,
    /// JOPPD, the pay and contributions report.
    Joppd,
    /// PD-IPO, transactions with related persons.
    PdIpo,
    /// TZ, the tourist board membership fee.
    Tz,
}

impl Form {
    /// Every form, in the order a page lists them.
    pub const ALL: [Self; 7] = [
        Self::Pd,
        Self::Pdv,
        Self::PdvS,
        Self::Zp,
        Self::Joppd,
        Self::PdIpo,
        Self::Tz,
    ];

    /// The word rows and the wire carry.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pd => "pd",
            Self::Pdv => "pdv",
            Self::PdvS => "pdv_s",
            Self::Zp => "zp",
            Self::Joppd => "joppd",
            Self::PdIpo => "pd_ipo",
            Self::Tz => "tz",
        }
    }

    /// The form's own name.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Pd => "PD",
            Self::Pdv => "PDV",
            Self::PdvS => "PDV-S",
            Self::Zp => "ZP",
            Self::Joppd => "JOPPD",
            Self::PdIpo => "PD-IPO",
            Self::Tz => "TZ",
        }
    }

    /// From the word on the wire; empty is none.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|f| f.as_str() == s.trim())
    }

    /// From the root element's local name.
    #[must_use]
    pub fn from_root(local: &str) -> Option<Self> {
        match local {
            "ObrazacPD" => Some(Self::Pd),
            "ObrazacPDV" => Some(Self::Pdv),
            "ObrazacPDVS" => Some(Self::PdvS),
            "ObrazacZP" => Some(Self::Zp),
            "ObrazacJOPPD" => Some(Self::Joppd),
            "ObrazacPDIPO" => Some(Self::PdIpo),
            "ObrazacTZ" => Some(Self::Tz),
            _ => None,
        }
    }
}

impl std::fmt::Display for Form {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

/// What the form says about itself.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Header {
    /// `Metapodaci/Uskladjenost`, e.g. `ObrazacPD-v9-0`.
    pub schema: String,
    /// The period's first day; a JOPPD's report date.
    pub period_from: Option<NaiveDate>,
    /// The period's last day; a JOPPD's report date.
    pub period_to: Option<NaiveDate>,
    /// The taxpayer's OIB.
    pub oib: String,
    /// The taxpayer's name as written.
    pub obveznik: String,
    /// When the software prepared the form.
    pub prepared_at: Option<DateTime<Utc>>,
    /// `Metapodaci/Autor`, or who compiled the report.
    pub author: String,
    /// `Metapodaci/Identifikator`.
    pub identifier: String,
    /// JOPPD's `OznakaIzvjesca`; empty otherwise.
    pub report_mark: String,
}

/// A form read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parsed {
    /// Which form.
    pub form: Form,
    /// Its header.
    pub header: Header,
    /// Body values by path.
    pub values: BTreeMap<String, String>,
    /// Repeated blocks, each tagged `_kind`.
    pub rows: Vec<Value>,
    /// Set when the body could not be read in full.
    pub error: Option<String>,
}

/// Why bytes are not a form. Anything here refuses an upload; a body the
/// reader cannot finish is stored with [`Parsed::error`] instead.
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    /// Not well-formed XML, or not UTF-8.
    #[error("not xml: {0}")]
    Xml(String),
    /// Well-formed, but the root is not an ePorezna form.
    #[error("not an ePorezna form (root element {0})")]
    UnknownForm(String),
    /// The header names no taxpayer.
    #[error("{0}: no OIB in the header")]
    NoOib(Form),
}

/// Recognise and read a form.
///
/// # Errors
/// The bytes are not a form at all ([`ParseError`]).
pub fn parse(bytes: &[u8]) -> Result<Parsed, ParseError> {
    let bytes = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(bytes);
    let text = std::str::from_utf8(bytes).map_err(|e| ParseError::Xml(e.to_string()))?;
    let doc = roxmltree::Document::parse(text).map_err(|e| ParseError::Xml(e.to_string()))?;
    let root = doc.root_element();
    let local = root.tag_name().name();
    let form = Form::from_root(local).ok_or_else(|| ParseError::UnknownForm(local.to_owned()))?;
    let version = root
        .tag_name()
        .namespace()
        .and_then(|ns| ns.strip_prefix(NAMESPACE_PREFIX))
        .and_then(|rest| rest.split('/').nth(1))
        .unwrap_or("v0-0");
    let fallback_schema = format!("{local}-{version}");
    let header = match form {
        Form::Joppd => joppd::header(root, &fallback_schema)?,
        _ => header::zaglavlje(root, form, &fallback_schema)?,
    };
    let mut values = BTreeMap::new();
    let mut rows = Vec::new();
    let mut error = None;
    match form {
        Form::Joppd => {
            match xml::child(root, "StranaA") {
                Some(a) => xml::flatten(a, "A", "", &[], &mut values, &mut rows),
                None => error = Some("no StranaA".to_owned()),
            }
            match xml::child(root, "StranaB") {
                Some(b) => xml::flatten(b, "B", "", form::row_paths(form), &mut values, &mut rows),
                None => error = Some("no StranaB".to_owned()),
            }
        }
        _ => match xml::child(root, "Tijelo") {
            Some(body) => xml::flatten(body, "", "", form::row_paths(form), &mut values, &mut rows),
            None => error = Some("no Tijelo".to_owned()),
        },
    }
    Ok(Parsed {
        form,
        header,
        values,
        rows,
        error,
    })
}

/// The few figures a listing shows for a form, in display order, only those
/// the form carries.
#[must_use]
pub fn headline(form: Form, values: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    form::headline_keys(form)
        .iter()
        .filter_map(|k| values.get(*k).map(|v| ((*k).to_owned(), v.clone())))
        .collect()
}

/// A decimal string that is not an amount.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AmountError {
    /// Not `-?digits(.digits)?`.
    #[error("not a decimal: {0:?}")]
    Malformed(String),
    /// More decimals than the scale allows.
    #[error("too precise for {places} places: {value:?}")]
    TooPrecise {
        /// The value.
        value: String,
        /// The scale asked for.
        places: u32,
    },
    /// Beyond i64.
    #[error("out of range: {0:?}")]
    Overflow(String),
}

/// `"12140.65"` to `1214065`, exactly: no floats, no rounding. Up to two
/// decimals; fewer are padded. A sign is allowed; a comma is not (the forms
/// write a point).
///
/// # Errors
/// Malformed, more than two decimals, or beyond i64.
pub fn minor(s: &str) -> Result<i64, AmountError> {
    scaled(s, 2)
}

/// [`minor`] for a value with `places` decimals: a rate with four, a count
/// with none.
///
/// # Errors
/// Malformed, too precise for `places`, or beyond i64.
pub fn scaled(s: &str, places: u32) -> Result<i64, AmountError> {
    let raw = s.trim();
    let (negative, body) = match raw.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, raw.strip_prefix('+').unwrap_or(raw)),
    };
    let (whole, frac) = body.split_once('.').unwrap_or((body, ""));
    if whole.is_empty() && frac.is_empty()
        || !whole.bytes().all(|b| b.is_ascii_digit())
        || !frac.bytes().all(|b| b.is_ascii_digit())
    {
        return Err(AmountError::Malformed(raw.to_owned()));
    }
    let places_usize =
        usize::try_from(places).map_err(|_| AmountError::Overflow(raw.to_owned()))?;
    if frac.len() > places_usize {
        return Err(AmountError::TooPrecise {
            value: raw.to_owned(),
            places,
        });
    }
    let mut digits = String::with_capacity(whole.len() + places_usize);
    digits.push_str(if whole.is_empty() { "0" } else { whole });
    digits.push_str(frac);
    for _ in frac.len()..places_usize {
        digits.push('0');
    }
    let magnitude: i64 = digits
        .parse()
        .map_err(|_| AmountError::Overflow(raw.to_owned()))?;
    Ok(if negative { -magnitude } else { magnitude })
}

/// Read one filing document: its bytes to a form, the form to
/// `finance.filings`, and the document's own read record. A document whose
/// bytes no longer parse as a form (the reader moved on, or the file was
/// never one) gets the reason written on the document and no filing row.
///
/// # Errors
/// The database.
pub async fn read(pool: &PgPool, id: Uuid) -> Result<(), StoreError> {
    let Some(party_id) = store::party_of(pool, id).await? else {
        return Ok(());
    };
    let bytes = crate::documents::store::bytes(pool, id).await?;
    match parse(&bytes) {
        Ok(parsed) => {
            tracing::debug!(
                document = %id,
                form = parsed.form.as_str(),
                schema = %parsed.header.schema,
                period = ?parsed.header.period_from,
                values = parsed.values.len(),
                rows = parsed.rows.len(),
                error = ?parsed.error,
                "filing read"
            );
            store::write(pool, id, party_id, &parsed).await
        }
        Err(e) => {
            tracing::warn!(document = %id, error = %e, "filing not read");
            store::write_unread(pool, id, &e.to_string()).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PD: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<ObrazacPD xmlns="http://e-porezna.porezna-uprava.hr/sheme/zahtjevi/ObrazacPD/v9-0" verzijaSheme="9.0">
  <Metapodaci xmlns="http://e-porezna.porezna-uprava.hr/sheme/Metapodaci/v2-0">
    <Naslov>Prijava poreza na dobit</Naslov>
    <Autor>Test d.o.o.</Autor>
    <Datum>2026-04-16T13:31:17</Datum>
    <Identifikator>2a0e165c-8194-4fab-befa-50e6ac7c9729</Identifikator>
    <Uskladjenost>ObrazacPD-v9-0</Uskladjenost>
  </Metapodaci>
  <Zaglavlje>
    <Razdoblje><DatumOd>2025-01-01</DatumOd><DatumDo>2025-12-31</DatumDo></Razdoblje>
    <Obveznik><Naziv>Test d.o.o.</Naziv><OIB>00000000001</OIB><SifraDjelatnosti>6209</SifraDjelatnosti>
      <Adresa><Mjesto>Viškovo</Mjesto><Ulica>Benčani</Ulica><Broj>15A</Broj></Adresa></Obveznik>
    <BrojZaposlenih>1</BrojZaposlenih>
    <Racuni><Racun><BrojRacuna>HR9224020061100925189</BrojRacuna><NazivInstitucije>Erste</NazivInstitucije></Racun></Racuni>
  </Zaglavlje>
  <Tijelo>
    <Podatak1>177733.47</Podatak1>
    <Podatak2>56332.75</Podatak2>
    <Podatak3>121400.72</Podatak3>
    <Podatak4>0.00</Podatak4>
    <Godina00>2025</Godina00>
    <PodatakPG00>0.00</PodatakPG00>
    <PodatakDO00>121406.54</PodatakDO00>
    <Podatak110>500.00</Podatak110>
    <Primatelji><Primatelj><NazivPrimatelja>Udruga</NazivPrimatelja><OibPrimatelja>00000000002</OibPrimatelja><IznosDarovanja>500.00</IznosDarovanja></Primatelj></Primatelji>
    <Podatak150>500.00</Podatak150>
  </Tijelo>
</ObrazacPD>"#;

    #[test]
    fn minor_is_exact() {
        assert_eq!(minor("12140.65"), Ok(1_214_065));
        assert_eq!(minor("177733.47"), Ok(17_773_347));
        assert_eq!(minor("-50.00"), Ok(-5000));
        assert_eq!(minor("0.43"), Ok(43));
        assert_eq!(minor("10.5"), Ok(1050));
        assert_eq!(minor("7"), Ok(700));
        assert_eq!(minor(" 0.00 "), Ok(0));
        assert!(matches!(
            minor("0.2000"),
            Err(AmountError::TooPrecise { .. })
        ));
        assert_eq!(scaled("0.2000", 4), Ok(2000));
        assert_eq!(scaled("3", 0), Ok(3));
        assert!(matches!(minor("1,50"), Err(AmountError::Malformed(_))));
        assert!(matches!(minor(""), Err(AmountError::Malformed(_))));
        assert!(matches!(
            minor("99999999999999999999"),
            Err(AmountError::Overflow(_))
        ));
    }

    #[test]
    fn pd_v9_flattens_to_row_keys() {
        let p = parse(PD.as_bytes()).expect("a PD");
        assert_eq!(p.form, Form::Pd);
        assert_eq!(p.header.schema, "ObrazacPD-v9-0");
        assert_eq!(p.header.oib, "00000000001");
        assert_eq!(p.header.obveznik, "Test d.o.o.");
        assert_eq!(p.header.author, "Test d.o.o.");
        assert_eq!(p.header.period_from, NaiveDate::from_ymd_opt(2025, 1, 1));
        assert_eq!(p.header.period_to, NaiveDate::from_ymd_opt(2025, 12, 31));
        assert!(p.header.prepared_at.is_some());
        assert_eq!(p.values["1"], "177733.47");
        assert_eq!(p.values["3"], "121400.72");
        assert_eq!(p.values["Godina00"], "2025");
        assert_eq!(p.values["DO00"], "121406.54");
        assert_eq!(p.values["150"], "500.00");
        assert_eq!(p.rows.len(), 1);
        assert_eq!(p.rows[0]["_kind"], "Primatelji");
        assert_eq!(p.rows[0]["IznosDarovanja"], "500.00");
        assert!(p.error.is_none());
        let h = headline(Form::Pd, &p.values);
        assert_eq!(h["1"], "177733.47");
        assert!(!h.contains_key("44"), "a key the form lacks is not shown");
    }

    #[test]
    fn pdv_pairs_become_value_and_tax_keys() {
        let xml = r#"<ObrazacPDV xmlns="http://e-porezna.porezna-uprava.hr/sheme/zahtjevi/ObrazacPDV/v11-0">
<Zaglavlje><Razdoblje><DatumOd>2026-01-01</DatumOd><DatumDo>2026-01-31</DatumDo></Razdoblje>
<Obveznik><Naziv>T</Naziv><OIB>00000000001</OIB></Obveznik></Zaglavlje>
<Tijelo><Podatak000>0.00</Podatak000><Podatak104>1000.00</Podatak104>
<Podatak200><Vrijednost>100.00</Vrijednost><Porez>25.00</Porez></Podatak200>
<Podatak300><Vrijednost>100.00</Vrijednost><Porez>25.00</Porez></Podatak300>
<Podatak400>0.00</Podatak400><Podatak500>0.00</Podatak500>
<Preknjizenja><Stavke><Stavka><Vrsta>1</Vrsta><Iznos>5.00</Iznos></Stavka></Stavke></Preknjizenja></Tijelo></ObrazacPDV>"#;
        let p = parse(xml.as_bytes()).expect("a PDV");
        assert_eq!(p.form, Form::Pdv);
        assert_eq!(
            p.header.schema, "ObrazacPDV-v11-0",
            "built from the namespace"
        );
        assert_eq!(p.values["200.Vrijednost"], "100.00");
        assert_eq!(p.values["300.Porez"], "25.00");
        assert_eq!(p.values["500"], "0.00");
        assert_eq!(p.rows[0]["_kind"], "Preknjizenja.Stavke");
        assert_eq!(p.rows[0]["Iznos"], "5.00");
        let h = headline(Form::Pdv, &p.values);
        assert_eq!(h["104"], "1000.00");
    }

    #[test]
    fn zp_and_pdvs_rows() {
        let xml = r#"<ObrazacZP xmlns="http://e-porezna.porezna-uprava.hr/sheme/zahtjevi/ObrazacZP/v1-0">
<Zaglavlje><Razdoblje><DatumOd>2025-03-01</DatumOd><DatumDo>2025-03-31</DatumDo></Razdoblje>
<Obveznik><Naziv>T</Naziv><OIB>00000000001</OIB></Obveznik></Zaglavlje>
<Tijelo><Isporuke><Isporuka><RedBr>1</RedBr><KodDrzave>DE</KodDrzave><PDVID>123</PDVID><I1>0.00</I1><I2>9167.00</I2><I3>0.00</I3><I4>0.00</I4></Isporuka></Isporuke>
<IsporukeUkupno><I1>0.00</I1><I2>9167.00</I2><I3>0.00</I3><I4>0.00</I4></IsporukeUkupno></Tijelo></ObrazacZP>"#;
        let p = parse(xml.as_bytes()).expect("a ZP");
        assert_eq!(p.form, Form::Zp);
        assert_eq!(p.rows.len(), 1, "one supply is still a row");
        assert_eq!(p.rows[0]["KodDrzave"], "DE");
        assert_eq!(p.values["IsporukeUkupno.I2"], "9167.00");
        let xml = xml
            .replace("ObrazacZP", "ObrazacPDVS")
            .replace("<I3>0.00</I3><I4>0.00</I4>", "");
        let p = parse(xml.as_bytes()).expect("a PDV-S");
        assert_eq!(p.form, Form::PdvS);
        assert_eq!(
            headline(Form::PdvS, &p.values)["IsporukeUkupno.I2"],
            "9167.00"
        );
    }

    #[test]
    fn pdipo_sections_and_nested_rows() {
        let xml = r#"<ObrazacPDIPO xmlns="http://e-porezna.porezna-uprava.hr/sheme/zahtjevi/ObrazacPDIPO/v1-0">
<Metapodaci><Datum>2016-11-17T09:30:47.0Z</Datum><Uskladjenost>ObrazacPDIPO-v1-0</Uskladjenost></Metapodaci>
<Zaglavlje><Razdoblje><DatumOd>2025-01-01</DatumOd><DatumDo>2025-12-31</DatumDo></Razdoblje>
<Obveznik><Naziv>T</Naziv><OIB>00000000001</OIB></Obveznik></Zaglavlje>
<Tijelo><Podaci><Podaci2><Osobe><Osoba><O1>1</O1><O2>Member</O2><O4>00000000002</O4>
<Potrazivanja><Potrazivanje><Z1>72515.51</Z1><Z2>97387.68</Z2></Potrazivanje></Potrazivanja></Osoba></Osobe>
<Sveukupno><S1>97387.68</S1></Sveukupno></Podaci2></Podaci></Tijelo></ObrazacPDIPO>"#;
        let p = parse(xml.as_bytes()).expect("a PD-IPO");
        assert_eq!(p.form, Form::PdIpo);
        assert_eq!(
            p.header.prepared_at.map(|t| t.to_rfc3339()).as_deref(),
            Some("2016-11-17T09:30:47+00:00")
        );
        assert_eq!(p.rows.len(), 1);
        assert_eq!(p.rows[0]["_kind"], "Podaci.Podaci2.Osobe");
        assert_eq!(p.rows[0]["Potrazivanja"]["Potrazivanje"]["Z2"], "97387.68");
        assert_eq!(p.values["Podaci.Podaci2.Sveukupno.S1"], "97387.68");
        assert_eq!(headline(Form::PdIpo, &p.values).len(), 1);
    }

    #[test]
    fn tz_seven_values() {
        let xml = r#"<ObrazacTZ xmlns="http://e-porezna.porezna-uprava.hr/sheme/zahtjevi/ObrazacTZ/v1-1">
<Zaglavlje><Razdoblje><DatumOd>2025-01-01</DatumOd><DatumDo>2025-12-31</DatumDo></Razdoblje>
<Obveznik><Naziv>T</Naziv><OIB>00000000001</OIB><SifraDjelatnosti>6209</SifraDjelatnosti><SifraOpcine>495</SifraOpcine></Obveznik></Zaglavlje>
<Tijelo><Podatak01>177733.47</Podatak01><Podatak02>0.2000</Podatak02><Podatak03>0.00</Podatak03><Podatak04>0.00</Podatak04>
<Podatak05>0.00</Podatak05><Podatak06>0.00</Podatak06><Podatak07>0.00</Podatak07></Tijelo></ObrazacTZ>"#;
        let p = parse(xml.as_bytes()).expect("a TZ");
        assert_eq!(p.form, Form::Tz);
        assert_eq!(headline(Form::Tz, &p.values).len(), 7);
        assert_eq!(scaled(&p.values["02"], 4), Ok(2000));
    }

    #[test]
    fn joppd_header_comes_from_page_a() {
        let xml = r#"<ObrazacJOPPD xmlns="http://e-porezna.porezna-uprava.hr/sheme/zahtjevi/ObrazacJOPPD/v1-1">
<Metapodaci><Uskladjenost>ObrazacJOPPD-v1-1</Uskladjenost></Metapodaci>
<StranaA><DatumIzvjesca>2025-02-24</DatumIzvjesca><OznakaIzvjesca>25055</OznakaIzvjesca><VrstaIzvjesca>1</VrstaIzvjesca>
<PodnositeljIzvjesca><Naziv>T</Naziv><OIB>00000000001</OIB><Oznaka>1</Oznaka></PodnositeljIzvjesca>
<BrojOsoba>1</BrojOsoba><BrojRedaka>1</BrojRedaka>
<PredujamPoreza><P1>1493.13</P1><P11>0.00</P11><P12>0.00</P12><P2>118.90</P2><P3>0.00</P3><P4>0.00</P4><P5>0.00</P5></PredujamPoreza>
<Doprinosi><GeneracijskaSolidarnost><P1>223.97</P1></GeneracijskaSolidarnost><KapitaliziranaStednja><P1>74.66</P1></KapitaliziranaStednja>
<ZdravstvenoOsiguranje><P1>246.37</P1></ZdravstvenoOsiguranje></Doprinosi>
<IzvjesceSastavio><Ime>Bruno</Ime><Prezime>T</Prezime></IzvjesceSastavio></StranaA>
<StranaB><Primatelji><P><P1>1</P1><P2>00495</P2><P3>00495</P3><P4>00000000002</P4><P5>Nevio V</P5><P8>1</P8><P11>1493.13</P11><P12>298.63</P12><P162>1075.61</P162></P></Primatelji></StranaB>
</ObrazacJOPPD>"#;
        let p = parse(xml.as_bytes()).expect("a JOPPD");
        assert_eq!(p.form, Form::Joppd);
        assert_eq!(p.header.oib, "00000000001");
        assert_eq!(p.header.report_mark, "25055");
        assert_eq!(p.header.period_from, NaiveDate::from_ymd_opt(2025, 2, 24));
        assert_eq!(
            p.header.author, "Bruno T",
            "the compiler when Metapodaci has no author"
        );
        assert_eq!(p.values["A.PredujamPoreza.P1"], "1493.13");
        assert_eq!(p.values["A.Doprinosi.ZdravstvenoOsiguranje.P1"], "246.37");
        assert_eq!(p.rows.len(), 1);
        assert_eq!(p.rows[0]["_kind"], "B.Primatelji");
        assert_eq!(p.rows[0]["P4"], "00000000002");
        assert_eq!(p.rows[0]["P162"], "1075.61");
        let h = headline(Form::Joppd, &p.values);
        assert_eq!(h["A.BrojOsoba"], "1");
    }

    #[test]
    fn what_is_not_a_form_is_refused() {
        assert!(matches!(parse(b"%PDF-1.4"), Err(ParseError::Xml(_))));
        assert!(matches!(
            parse(b"<html><body/></html>"),
            Err(ParseError::UnknownForm(r)) if r == "html"
        ));
        let no_oib = PD.replace("<OIB>00000000001</OIB>", "");
        assert!(matches!(
            parse(no_oib.as_bytes()),
            Err(ParseError::NoOib(Form::Pd))
        ));
        let bom = [b"\xEF\xBB\xBF".as_slice(), PD.as_bytes()].concat();
        assert!(parse(&bom).is_ok(), "a byte-order mark is skipped");
    }

    #[test]
    fn a_form_without_a_body_is_kept_with_an_error() {
        let headless = PD.split("<Tijelo>").next().expect("has a body").to_owned() + "</ObrazacPD>";
        let p = parse(headless.as_bytes()).expect("still a PD");
        assert_eq!(p.error.as_deref(), Some("no Tijelo"));
        assert!(p.values.is_empty());
    }

    #[test]
    fn pd_2025_fixture_matches_expected_csv() {
        let xml = include_str!("../../tests/fixtures/filings/pd-2025.xml");
        let csv = include_str!("../../../../docs/accountant/expected/pd.csv");
        let p = parse(xml.as_bytes()).expect("the 2025 PD");
        assert_eq!(p.header.oib, "38846238650");
        assert_eq!(p.header.schema, "ObrazacPD-v9-0");
        assert_eq!(p.header.period_from, NaiveDate::from_ymd_opt(2025, 1, 1));
        assert_eq!(p.header.period_to, NaiveDate::from_ymd_opt(2025, 12, 31));
        assert!(p.error.is_none());
        let mut compared = 0;
        for line in csv.lines().skip(1) {
            let (row, amount) = line.split_once(',').expect("row,amount");
            let key = form::pd_csv_key(row).expect("a known row");
            let filed = p
                .values
                .get(key)
                .unwrap_or_else(|| panic!("row {row} (key {key}) missing"));
            assert_eq!(
                minor(filed).expect("an amount"),
                minor(amount).expect("an amount"),
                "row {row}"
            );
            compared += 1;
        }
        assert!(compared >= 20, "the csv has the rows: {compared}");
        assert_eq!(minor(&p.values["44"]), Ok(1_214_065));
    }
}
