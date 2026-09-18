//! What the accountant needs for each outgoing transaction, and which
//! receipt covers it.
//!
//! Two questions per transaction. *Need*: does the accountant need a
//! document from us at all? A domestic supplier's e-invoice reaches them on
//! its own (mandatory B2B eRačun since 2026), tax and salary payments carry
//! their own paperwork, a cash withdrawal has none; a card charge to a
//! foreign `SaaS` is the case where we owe a receipt. *Cover*: for those, is
//! there a pulled receipt that matches, by the original amount the card was
//! charged in, the vendor, and the date? Both are rules a person can read,
//! and both yield to a person's word: a counterparty policy for the first,
//! a declared link for the second.

pub mod store;

use std::sync::LazyLock;

use chrono::NaiveDate;
use regex::Regex;
use uuid::Uuid;

/// What the accountant needs from us for a transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Need {
    /// The supplier's e-invoice reaches the accountant; nothing to send.
    Eracun,
    /// We must supply the document.
    Receipt,
    /// Nothing exists or is needed: tax, salary, bank fee, cash.
    None,
    /// Not a business cost.
    Personal,
    /// Money in: our own invoice, or a refund.
    Income,
    /// A transfer between our own accounts.
    Internal,
}

impl Need {
    /// The wire word.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Eracun => "eracun",
            Self::Receipt => "receipt",
            Self::None => "none",
            Self::Personal => "personal",
            Self::Income => "income",
            Self::Internal => "internal",
        }
    }

    /// A policy word from the wire, for the four a person may set.
    #[must_use]
    pub fn from_policy(s: &str) -> Option<Self> {
        match s {
            "eracun" => Some(Self::Eracun),
            "receipt" => Some(Self::Receipt),
            "none" => Some(Self::None),
            "personal" => Some(Self::Personal),
            _ => None,
        }
    }
}

/// The facts about a transaction the rules read.
#[derive(Debug, Clone)]
pub struct TxFacts {
    /// Money in.
    pub credit: bool,
    /// Signed minor units.
    pub amount_minor: i64,
    /// ISO code.
    pub currency: String,
    /// As the bank gave it.
    pub counterparty_name: String,
    /// Empty for a card payment.
    pub counterparty_iban: String,
    /// The bank's free text, joined.
    pub remittance: String,
    /// The structured reference, "HR68 ..." and the like.
    pub reference_number: String,
    /// Booking date.
    pub booking_date: NaiveDate,
    /// The counterparty is one of our own accounts.
    pub internal: bool,
}

/// A person's word on a counterparty.
#[derive(Debug, Clone)]
pub struct Policy {
    /// Row id.
    pub id: Uuid,
    /// Normalised name or fragment.
    pub match_normalised: String,
    /// Whole name, not a fragment.
    pub exact: bool,
    /// What the accountant needs.
    pub need: Need,
}

/// The decision, with why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decision {
    /// What is needed.
    pub need: Need,
    /// One line the page shows.
    pub reason: String,
    /// The policy that decided, if one did.
    pub policy_id: Option<Uuid>,
}

/// Upper-case ASCII, diacritics folded, the way `finance.normalise` does
/// in SQL, so a policy typed against the page matches what SQL would.
#[must_use]
pub fn normalise(value: &str) -> String {
    value
        .chars()
        .map(|c| match c {
            'Č' | 'č' | 'Ć' | 'ć' => 'C',
            'Đ' | 'đ' => 'D',
            'Š' | 'š' => 'S',
            'Ž' | 'ž' => 'Z',
            'À' | 'Á' | 'Â' | 'Ã' | 'Ä' | 'Å' | 'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' => {
                'A'
            }
            'È' | 'É' | 'Ê' | 'Ë' | 'è' | 'é' | 'ê' | 'ë' => 'E',
            'Ì' | 'Í' | 'Î' | 'Ï' | 'ì' | 'í' | 'î' | 'ï' => 'I',
            'Ò' | 'Ó' | 'Ô' | 'Õ' | 'Ö' | 'ò' | 'ó' | 'ô' | 'õ' | 'ö' => 'O',
            'Ù' | 'Ú' | 'Û' | 'Ü' | 'ù' | 'ú' | 'û' | 'ü' => 'U',
            c => c.to_ascii_uppercase(),
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// "424472XXXXXX9355, OPENAI *CHATGPT SUBSCR 75,00 USD,  30.08.2026" →
/// the amount the card was charged in, when the bank wrote it out.
static ORIGINAL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(\d{1,3}(?:\.\d{3})*,\d{2}|\d+,\d{2}) ([A-Z]{3})\b")
        .unwrap_or_else(|e| panic!("original amount regex: {e}"))
});

/// The original amount and currency of a card charge, from the remittance.
#[must_use]
pub fn original_amount(remittance: &str) -> Option<(i64, String)> {
    let c = ORIGINAL.captures(remittance)?;
    let num = c[1].replace('.', "");
    let (whole, frac) = num.split_once(',')?;
    let minor = whole.parse::<i64>().ok()?.checked_mul(100)? + frac.parse::<i64>().ok()?;
    Some((minor, c[2].to_owned()))
}

const STATE: &[&str] = &[
    "DRZAVNI PRORACUN",
    "POREZ",
    "HRVATSKI ZAVOD",
    "ZDRAVSTVENO",
    "MIROVINSKO",
    "MIROV.OSIG",
    "DOPRINOS",
    "DOPR.ZA",
];

/// What the accountant needs, from the facts and the person's policies.
#[must_use]
pub fn classify(tx: &TxFacts, policies: &[Policy]) -> Decision {
    let decide = |need: Need, reason: &str| Decision {
        need,
        reason: reason.to_owned(),
        policy_id: None,
    };
    // Tax and payouts first: a salary or dividend goes to the owner's own
    // account, which is in the grant and would otherwise read as a mere
    // transfer between accounts. To the accountant it is payroll.
    let name = normalise(&tx.counterparty_name);
    let reference = tx.reference_number.trim().to_ascii_uppercase();
    if !tx.credit && (reference.starts_with("HR68") || STATE.iter().any(|s| name.contains(s))) {
        return decide(Need::None, "state budget: tax or contribution");
    }
    if !tx.credit && reference.starts_with("HR69 40002") {
        return decide(
            Need::None,
            "payout to a person: salary, dividend, allowance",
        );
    }
    if tx.internal {
        return decide(Need::Internal, "transfer between own accounts");
    }
    if tx.credit {
        return decide(Need::Income, "money in");
    }
    if let Some(p) = policies.iter().find(|p| {
        if p.exact {
            name == p.match_normalised
        } else {
            name.contains(&p.match_normalised)
        }
    }) {
        return Decision {
            need: p.need,
            reason: "policy".into(),
            policy_id: Some(p.id),
        };
    }
    let text = normalise(&tx.remittance);
    if tx.counterparty_iban.is_empty() && (name.contains(" ATM") || name.starts_with("ATM")) {
        return decide(Need::None, "cash withdrawal");
    }
    if name.contains("BANK") && text.contains("NAKNAD") {
        return decide(Need::None, "bank fee: the statement is the document");
    }
    if tx.counterparty_iban.to_ascii_uppercase().starts_with("HR") {
        return decide(
            Need::Eracun,
            "domestic supplier: e-invoice reaches the accountant",
        );
    }
    if tx.counterparty_iban.is_empty() {
        if let Some((minor, cur)) = original_amount(&tx.remittance) {
            return decide(
                Need::Receipt,
                &format!("card, charged {} {cur}", money(minor)),
            );
        }
        return decide(Need::Receipt, "card payment");
    }
    decide(Need::Receipt, "foreign transfer")
}

fn money(minor: i64) -> String {
    format!("{},{:02}", minor / 100, (minor % 100).abs())
}

/// A receipt as the matcher sees it.
#[derive(Debug, Clone)]
pub struct DocFacts {
    /// Row id.
    pub id: Uuid,
    /// As read or declared.
    pub vendor: String,
    /// The document's date.
    pub doc_date: Option<NaiveDate>,
    /// Minor units.
    pub total_minor: Option<i64>,
    /// ISO code.
    pub currency: String,
}

/// A link is made at this score; below it, down to `SUGGEST`, the page
/// offers the receipt as a suggestion.
pub const LINK: u8 = 70;
/// Below this a receipt is not worth showing.
pub const SUGGEST: u8 = 35;

/// How well a receipt fits a transaction, with what fitted.
#[must_use]
pub fn score(tx: &TxFacts, doc: &DocFacts) -> Option<(u8, String)> {
    let mut points = 0u8;
    let mut why: Vec<String> = Vec::new();
    let paid = tx.amount_minor.abs();
    match (doc.total_minor, original_amount(&tx.remittance)) {
        (Some(total), Some((orig, cur))) if doc.currency == cur && total == orig => {
            points += 60;
            why.push(format!("amount {} {cur}", money(total)));
        }
        (Some(total), _) if doc.currency == tx.currency && total == paid => {
            points += 60;
            why.push(format!("amount {} {}", money(total), tx.currency));
        }
        (Some(total), _) if doc.currency != tx.currency && within(total, paid, 12) => {
            points += 20;
            why.push(format!("≈ {} {} at FX", money(total), doc.currency));
        }
        _ => {}
    }
    let party = normalise(&tx.counterparty_name)
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .collect::<String>();
    let vendor_hit = normalise(&doc.vendor)
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|t| t.len() >= 4)
        .any(|t| party.contains(t));
    if vendor_hit {
        points += 30;
        why.push("vendor".into());
    }
    if let Some(d) = doc.doc_date {
        let days = (tx.booking_date - d).num_days().abs();
        if days <= 3 {
            points += 10;
            why.push("same days".into());
        } else if days <= 10 {
            points += 5;
            why.push(format!("{days} days apart"));
        } else {
            // Last month's invoice for this month's charge of the same
            // amount: a subscription, and the wrong receipt. Offered, not
            // linked.
            points = points.saturating_sub(25);
            why.push(format!("{days} days apart"));
        }
    }
    (points >= SUGGEST).then(|| (points, why.join(" · ")))
}

fn within(a: i64, b: i64, percent: i64) -> bool {
    if a == 0 || b == 0 {
        return false;
    }
    let diff = (a - b).abs() * 100;
    diff <= b.abs() * percent
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    fn tx(name: &str, iban: &str, remittance: &str, reference: &str, amount: i64) -> TxFacts {
        TxFacts {
            credit: amount > 0,
            amount_minor: amount,
            currency: "EUR".into(),
            counterparty_name: name.into(),
            counterparty_iban: iban.into(),
            remittance: remittance.into(),
            reference_number: reference.into(),
            booking_date: d("2026-08-30"),
            internal: false,
        }
    }

    #[test]
    fn the_august_statement_is_sorted_into_needs() {
        let cases = [
            (
                tx(
                    "DRZAVNI PRORACUN REPUBLIKE HRV",
                    "HR1210010051863000160",
                    "HR68 8168-38846238650-26215 | Placa za 7 / 2026",
                    "HR68 8168-38846238650-26215",
                    -22397,
                ),
                Need::None,
            ),
            (
                tx(
                    "Vesic Nevio",
                    "HR3924020061100000000",
                    "HR69 40002-38846238650-100 | Placa za 7 / 2026",
                    "HR69 40002-38846238650-100",
                    -107_560,
                ),
                Need::None,
            ),
            (
                tx(
                    "TENDERLY D.O.O.",
                    "RS35265100000012345678",
                    "HR99 | BROJ RACUNA 8-1-1-2026",
                    "HR99",
                    2_502_167,
                ),
                Need::Income,
            ),
            (
                tx(
                    "HT D.D. - UPLATNI RAČUN",
                    "HR6023400091110000000",
                    "HR01 1183539863 | RAČUN ZA HT USLUGE",
                    "HR01 1183539863",
                    -12981,
                ),
                Need::Eracun,
            ),
            (
                tx(
                    "ERSTE&STEIERMÄRKISCHE BANK d.d",
                    "HR9524020061000000000",
                    "HR99 | Naplata naknade platnog prometa",
                    "HR99",
                    -3568,
                ),
                Need::None,
            ),
            (
                tx(
                    "ERSTE ATM  ZG-N. ZAGREB",
                    "",
                    "HR99 | 424472XXXXXX9355, ERSTE ATM  ZG-N. ZAGREB",
                    "HR99",
                    -100_000,
                ),
                Need::None,
            ),
            (
                tx(
                    "OPENAI *CHATGPT SUBSCR",
                    "",
                    "HR99 | 424472XXXXXX9355, OPENAI *CHATGPT SUBSCR 75,00 USD,  30.08.2026",
                    "HR99",
                    -6636,
                ),
                Need::Receipt,
            ),
            (
                tx(
                    "ELIPSO P-18 Rijeka",
                    "",
                    "HR99 | 424472XXXXXX9355, ELIPSO P-18 Rijeka ,  13.08.2026",
                    "HR99",
                    -15599,
                ),
                Need::Receipt,
            ),
            (
                tx(
                    "SOME GMBH",
                    "DE89370400440532013000",
                    "invoice 42",
                    "",
                    -50_000,
                ),
                Need::Receipt,
            ),
        ];
        for (t, want) in cases {
            let got = classify(&t, &[]);
            assert_eq!(got.need, want, "{}: {}", t.counterparty_name, got.reason);
        }
    }

    #[test]
    fn a_card_charge_says_what_it_was_charged_in_and_own_accounts_are_internal() {
        let card = tx(
            "OPENAI *CHATGPT SUBSCR",
            "",
            "HR99 | 424472XXXXXX9355, OPENAI *CHATGPT SUBSCR 75,00 USD",
            "HR99",
            -6636,
        );
        assert_eq!(classify(&card, &[]).reason, "card, charged 75,00 USD");
        let mut own = tx("Vesic Nevio", "HR3924020061100000000", "x", "", -100);
        own.internal = true;
        assert_eq!(classify(&own, &[]).need, Need::Internal);
        // Salary to the owner's own account: payroll to the accountant, not
        // a transfer, even though the account is in the grant.
        let mut salary = tx(
            "Vesic Nevio",
            "HR3924020061100000000",
            "HR69 40002-38846238650-100 | Placa za 7 / 2026",
            "HR69 40002-38846238650-100",
            -107_560,
        );
        salary.internal = true;
        let got = classify(&salary, &[]);
        assert_eq!(got.need, Need::None, "{}", got.reason);
    }

    #[test]
    fn a_policy_overrides_the_rules() {
        let id = Uuid::new_v4();
        let policies = [Policy {
            id,
            match_normalised: "PLAYSTATION".into(),
            exact: false,
            need: Need::Personal,
        }];
        let t = tx(
            "PLAYSTATION",
            "",
            "HR99 | 424472XXXXXX9355, PLAYSTATION Hilversum",
            "HR99",
            -3999,
        );
        let got = classify(&t, &policies);
        assert_eq!((got.need, got.policy_id), (Need::Personal, Some(id)));
        let exact = [Policy {
            id,
            match_normalised: "ELIPSO".into(),
            exact: true,
            need: Need::None,
        }];
        assert_eq!(
            classify(&tx("ELIPSO P-18 Rijeka", "", "", "", -1), &exact).need,
            Need::Receipt,
            "an exact policy does not match a longer name"
        );
    }

    #[test]
    fn original_amounts_are_read_from_the_remittance() {
        assert_eq!(
            original_amount("424472XXXXXX9355, OPENAI *CHATGPT SUBSCR 75,00 USD,  30.08.2026"),
            Some((7500, "USD".into()))
        );
        assert_eq!(
            original_amount("MEDIUM MONTHLY 5,00 USD,  10.08.2026"),
            Some((500, "USD".into()))
        );
        assert_eq!(
            original_amount("NAME-CHEAP.COM* TTOP1L 1.075,98 USD"),
            Some((107_598, "USD".into()))
        );
        assert_eq!(
            original_amount("ELIPSO P-18 Rijeka ,  13.08.2026 11:02"),
            None
        );
    }

    #[test]
    fn receipts_score_by_original_amount_vendor_and_date() {
        let t = tx(
            "OPENAI *CHATGPT SUBSCR",
            "",
            "HR99 | 424472XXXXXX9355, OPENAI *CHATGPT SUBSCR 75,00 USD,  30.08.2026",
            "HR99",
            -6636,
        );
        let exact = DocFacts {
            id: Uuid::new_v4(),
            vendor: "OpenAI".into(),
            doc_date: Some(d("2026-08-29")),
            total_minor: Some(7500),
            currency: "USD".into(),
        };
        let (points, why) = score(&t, &exact).unwrap();
        assert!(points >= LINK, "{points}: {why}");
        assert!(why.contains("amount 75,00 USD") && why.contains("vendor"));
        // Same vendor, another month's charge: a suggestion, not a link.
        let other = DocFacts {
            total_minor: Some(2068),
            doc_date: Some(d("2026-08-20")),
            ..exact.clone()
        };
        let (points, _) = score(&t, &other).unwrap();
        assert!((SUGGEST..LINK).contains(&points), "{points}");
        // Same vendor, same amount, last month's invoice: a subscription's
        // previous receipt. Offered, never linked.
        let last_month = DocFacts {
            doc_date: Some(d("2026-07-30")),
            ..exact.clone()
        };
        let (points, why) = score(&t, &last_month).unwrap();
        assert!((SUGGEST..LINK).contains(&points), "{points}: {why}");
        // Nothing in common: not even a suggestion.
        let stranger = DocFacts {
            id: Uuid::new_v4(),
            vendor: "Hetzner".into(),
            doc_date: Some(d("2026-08-13")),
            total_minor: Some(6948),
            currency: "EUR".into(),
        };
        assert_eq!(score(&t, &stranger), None);
        // An EUR receipt for an EUR charge, exact, same days: linked without a vendor hit.
        let eur = tx(
            "HETZNER ONLINE GMBH",
            "",
            "HR99 | 424472XXXXXX9355, HETZNER ONLINE GMBH GUNZENHAUSEN",
            "HR99",
            -6948,
        );
        let (points, _) = score(
            &eur,
            &DocFacts {
                doc_date: Some(d("2026-08-28")),
                ..stranger.clone()
            },
        )
        .unwrap();
        assert!(points >= LINK, "{points}");
    }

    #[test]
    fn normalise_folds_like_sql() {
        assert_eq!(
            normalise("Računovodstvo  Trinajstić"),
            "RACUNOVODSTVO TRINAJSTIC"
        );
    }
}
