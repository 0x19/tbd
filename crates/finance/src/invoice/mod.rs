//! Invoices: drafted, previewed, approved, rendered.
//!
//! The load-bearing decisions, each in its own file:
//!
//! - [`totals`]: money is integer minor units, rounded half up, and
//!   `total = subtotal + vat` is a database constraint, so a rounding bug is
//!   a refused write and never a wrong invoice.
//! - [`numbering`]: a number is allocated at approval, under the counter's
//!   row lock, inside the approving transaction. Never a sequence.
//! - [`render`]: Typst as a library, fonts and the mark embedded, the PDF's
//!   id and date pinned to the invoice, so a render is a pure function of
//!   the document.
//!
//! What ties them together is the **content hash**: the approver sees a
//! preview of a canonical document and approves *that hash*. A draft changed
//! since is refused. That is what makes "nothing goes out without my say-so"
//! a test rather than a hope.

pub mod import;
pub mod numbering;
pub mod render;
pub mod store;
pub mod totals;

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// The VAT treatments an invoice can carry, a closed set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VatTreatment {
    /// Croatian VAT at the standard rate.
    StandardHr,
    /// EU business customer: reverse charge.
    ReverseChargeEu,
    /// Outside the EU: not subject to Croatian VAT. Tenderly.
    OutsideScopeNonEu,
    /// The issuer is not in the VAT system.
    ExemptIssuer,
}

impl VatTreatment {
    /// The database's spelling.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::StandardHr => "standard_hr",
            Self::ReverseChargeEu => "reverse_charge_eu",
            Self::OutsideScopeNonEu => "outside_scope_non_eu",
            Self::ExemptIssuer => "exempt_issuer",
        }
    }

    /// From the database's spelling.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "standard_hr" => Self::StandardHr,
            "reverse_charge_eu" => Self::ReverseChargeEu,
            "outside_scope_non_eu" => Self::OutsideScopeNonEu,
            "exempt_issuer" => Self::ExemptIssuer,
            _ => return None,
        })
    }

    /// VAT rate in basis points.
    #[must_use]
    pub fn rate_bp(self) -> i64 {
        match self {
            Self::StandardHr => 2500,
            _ => 0,
        }
    }

    /// The note the invoice must carry. Copied onto the invoice at approval,
    /// never looked up at render time: a law change next year must not
    /// rewrite an issued invoice.
    #[must_use]
    pub fn note(self) -> &'static str {
        match self {
            Self::StandardHr => "",
            Self::ReverseChargeEu | Self::OutsideScopeNonEu => {
                "The service is subject to the reverse charge mechanism. Therefore, VAT is not \
                 charged pursuant to Article 17, Paragraph 1 of the Value Added Tax Act, as \
                 amended. / Usluga je predmet prijenosa porezne obveze te PDV nije obračunat na \
                 temelju članka 17. stavka 1. Zakona o porezu na dodanu vrijednost, s izmjenama \
                 i dopunama."
            }
            Self::ExemptIssuer => {
                "VAT is not charged: the issuer is not registered for VAT pursuant to Article \
                 90 of the Value Added Tax Act. / PDV nije obračunat: izdavatelj nije u sustavu \
                 PDV-a temeljem članka 90. Zakona o porezu na dodanu vrijednost."
            }
        }
    }

    /// The label of the VAT line.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::StandardHr => "VAT 25% / PDV 25%",
            _ => "VAT / PDV",
        }
    }
}

/// The issuing side, as the invoice prints it.
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Issuer {
    pub legal_name: String,
    pub address_lines: Vec<String>,
    pub oib: String,
    pub vat_id: String,
    pub iban: String,
    pub swift: String,
    pub bank_name: String,
    pub court: String,
    pub registration_no: String,
    pub share_capital: String,
    pub board_member: String,
    pub issued_by: String,
    pub operator_id: String,
}

/// Who is billed, as the invoice prints it.
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Client {
    pub name: String,
    pub address_lines: Vec<String>,
    pub country: String,
    pub tax_id: String,
}

/// One line.
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Line {
    pub position: i32,
    pub description: String,
    /// Thousandths.
    pub quantity_milli: i64,
    pub unit_price_minor: i64,
    pub amount_minor: i64,
}

/// The whole document, in its canonical form.
///
/// Everything the PDF shows, and nothing else: the hash of this is what an
/// approval names. Field order is fixed by serde; `serde_json` sorts map
/// keys, so the bytes are the same on every machine.
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvoiceDoc {
    /// `9-1-1-2026`, or the number the next approval would take.
    pub number: String,
    /// True until approved: the number is a preview, not an allocation.
    pub number_preview: bool,
    pub issued_at: DateTime<Utc>,
    pub delivery_date: NaiveDate,
    pub due_date: NaiveDate,
    pub place_of_issue: String,
    pub currency: String,
    pub issuer: Issuer,
    pub client: Client,
    pub lines: Vec<Line>,
    pub subtotal_minor: i64,
    pub vat_minor: i64,
    pub total_minor: i64,
    pub vat_treatment: VatTreatment,
    pub vat_note: String,
    pub note: String,
}

impl InvoiceDoc {
    /// The canonical bytes: sorted keys, no whitespace.
    ///
    /// # Errors
    /// Serialisation, which for this type means never.
    pub fn canonical(&self) -> Result<Vec<u8>, serde_json::Error> {
        // `serde_json::Value` sorts keys; going through it makes the order
        // independent of the struct's field order.
        let value = serde_json::to_value(self)?;
        serde_json::to_vec(&value)
    }

    /// The content hash an approval names: SHA-256 of the canonical bytes,
    /// hex. The number preview is part of it, so an approval that would take
    /// a different number than the one shown is refused too.
    ///
    /// # Errors
    /// Serialisation, which for this type means never.
    pub fn content_hash(&self) -> Result<String, serde_json::Error> {
        let mut hasher = Sha256::new();
        hasher.update(self.canonical()?);
        Ok(format!("{:x}", hasher.finalize()))
    }
}

/// Minor units as `13,750.00`, the way the existing invoices print them.
#[must_use]
pub fn money(minor: i64) -> String {
    let negative = minor < 0;
    let abs = minor.unsigned_abs();
    let whole = abs / 100;
    let frac = abs % 100;
    let mut digits = whole.to_string();
    let mut grouped = String::with_capacity(digits.len() + digits.len() / 3);
    while digits.len() > 3 {
        let rest = digits.split_off(digits.len() - 3);
        grouped.insert_str(0, &format!(",{rest}"));
    }
    grouped.insert_str(0, &digits);
    format!("{}{grouped}.{frac:02}", if negative { "-" } else { "" })
}

/// Thousandths as `1`, `1.5`, `0.25`.
#[must_use]
pub fn quantity(milli: i64) -> String {
    let whole = milli / 1000;
    let frac = (milli % 1000).abs();
    if frac == 0 {
        whole.to_string()
    } else {
        format!("{whole}.{}", format!("{frac:03}").trim_end_matches('0'))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn money_prints_like_the_existing_invoices() {
        assert_eq!(money(1_450_082), "14,500.82");
        assert_eq!(money(75_082), "750.82");
        assert_eq!(money(0), "0.00");
        assert_eq!(money(123_456_789), "1,234,567.89");
        assert_eq!(money(-500), "-5.00");
    }

    #[test]
    fn quantities_drop_the_zeros_they_do_not_need() {
        assert_eq!(quantity(1000), "1");
        assert_eq!(quantity(1500), "1.5");
        assert_eq!(quantity(250), "0.25");
        assert_eq!(quantity(1_125), "1.125");
    }

    #[test]
    fn the_hash_is_stable_and_sensitive() {
        let doc = render::sample();
        let a = doc.content_hash().unwrap();
        let b = doc.clone().content_hash().unwrap();
        assert_eq!(a, b);
        let mut changed = doc;
        changed.lines[0].unit_price_minor += 1;
        assert_ne!(a, changed.content_hash().unwrap());
    }
}
