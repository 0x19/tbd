//! Line and invoice totals, in integers.
//!
//! A quantity is thousandths and a price is minor units, so a line amount is
//! `quantity * price / 1000`, rounded half up. VAT is `subtotal * rate /
//! 10000`, rounded the same way. There is no float anywhere: `0.1 + 0.2` is
//! not a thing money does, and the schema's `total = subtotal + vat` check
//! turns any disagreement into a refused write.

use super::{Line, VatTreatment};

/// `numerator / denominator`, rounded half away from zero.
#[must_use]
pub fn round_half_up(numerator: i128, denominator: i128) -> i64 {
    debug_assert!(denominator > 0);
    let sign = if numerator < 0 { -1 } else { 1 };
    let n = numerator.abs();
    let q = (n + denominator / 2) / denominator;
    i64::try_from(sign * q).unwrap_or(if sign < 0 { i64::MIN } else { i64::MAX })
}

/// A line's amount from its quantity and unit price.
#[must_use]
pub fn line_amount(quantity_milli: i64, unit_price_minor: i64) -> i64 {
    round_half_up(
        i128::from(quantity_milli) * i128::from(unit_price_minor),
        1000,
    )
}

/// Subtotal, VAT and total for a set of lines.
#[must_use]
pub fn totals(lines: &[Line], treatment: VatTreatment) -> (i64, i64, i64) {
    let subtotal: i64 = lines.iter().map(|l| l.amount_minor).sum();
    let vat = round_half_up(
        i128::from(subtotal) * i128::from(treatment.rate_bp()),
        10_000,
    );
    (subtotal, vat, subtotal + vat)
}

/// Fill every line's amount from its quantity and price, in place.
pub fn settle(lines: &mut [Line]) {
    for line in lines {
        line.amount_minor = line_amount(line.quantity_milli, line.unit_price_minor);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(q: i64, p: i64) -> Line {
        Line {
            position: 1,
            description: String::new(),
            quantity_milli: q,
            unit_price_minor: p,
            amount_minor: line_amount(q, p),
        }
    }

    #[test]
    fn the_august_invoice_adds_up() {
        let lines = [line(1000, 1_375_000), line(1000, 75_082)];
        assert_eq!(
            totals(&lines, VatTreatment::OutsideScopeNonEu),
            (1_450_082, 0, 1_450_082)
        );
    }

    #[test]
    fn half_a_cent_rounds_up_not_to_even() {
        // 0.005 * 1 = 0.005 -> 0.01
        assert_eq!(line_amount(1000, 0), 0);
        assert_eq!(round_half_up(5, 10), 1);
        assert_eq!(round_half_up(4, 10), 0);
        assert_eq!(round_half_up(-5, 10), -1);
        // 33.333 hours at 45.00 -> 1,499.985 -> 1,499.99
        assert_eq!(line_amount(33_333, 4_500), 149_999);
    }

    #[test]
    fn croatian_vat_is_a_quarter_rounded_on_the_subtotal() {
        let lines = [line(3000, 1_001)];
        // 30.03 * 25% = 7.5075 -> 7.51
        assert_eq!(
            totals(&lines, VatTreatment::StandardHr),
            (3_003, 751, 3_754)
        );
    }

    #[test]
    fn a_negative_line_is_a_credit_and_rounds_away_from_zero() {
        assert_eq!(line_amount(1000, -1_001), -1_001);
        assert_eq!(line_amount(500, -1_001), -501);
    }
}
