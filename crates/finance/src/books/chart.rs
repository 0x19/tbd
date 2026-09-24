//! The chart of accounts the books are seeded with: `RRiF` numbering, data in
//! `configs/finance/hr/chart-rrif-2025.toml`, compiled in and parsed once.
//! Every party gets the whole chart on its first books call; an account a
//! party adds itself sits beside these with the same rules (a parent that
//! exists and is a prefix of the code).

use std::sync::LazyLock;

use serde::Deserialize;
use sqlx::{PgConnection, PgPool};
use tbd_db::map_err;
use uuid::Uuid;

use super::BooksError;

const SOURCE: &str = include_str!("../../../../configs/finance/hr/chart-rrif-2025.toml");

/// The chart as shipped.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Chart {
    /// Which chart this is.
    pub chart: Meta,
    /// Every account, groups first within a class as the file lists them.
    #[serde(rename = "account")]
    pub accounts: Vec<ChartAccount>,
}

/// The chart's name and edition.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Meta {
    /// E.g. `RRiF XXVIII (2025)`.
    pub name: String,
    /// The year of the edition.
    pub edition: String,
}

/// One account of the shipped chart.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChartAccount {
    /// 3 to 6 digits.
    pub code: String,
    /// The accountant's name for it.
    pub name: String,
    /// A group or synthetic account postings never land on.
    #[serde(default)]
    pub synthetic: bool,
}

impl Chart {
    /// The shipped chart. Parsed once; a malformed file fails the first call
    /// loudly, and the unit tests parse it too, so it never reaches a server.
    #[must_use]
    pub fn shipped() -> &'static Chart {
        static CHART: LazyLock<Chart> = LazyLock::new(|| {
            #[allow(clippy::expect_used)]
            toml::from_str(SOURCE).expect("configs/finance/hr/chart-rrif-2025.toml parses")
        });
        &CHART
    }

    /// The account with this code.
    #[must_use]
    pub fn get(&self, code: &str) -> Option<&ChartAccount> {
        self.accounts.iter().find(|a| a.code == code)
    }

    /// The parent of a code: the longest other code in the chart that is a
    /// proper prefix of it, when there is one.
    #[must_use]
    pub fn parent_of(&self, code: &str) -> Option<&str> {
        self.accounts
            .iter()
            .filter(|a| a.code.len() < code.len() && code.starts_with(&a.code))
            .max_by_key(|a| a.code.len())
            .map(|a| a.code.as_str())
    }
}

/// The parent an account added by a party takes: the longest existing code
/// of that party's chart that is a proper prefix.
///
/// # Errors
/// The database.
pub async fn parent_in_db(
    conn: &mut PgConnection,
    party: Uuid,
    code: &str,
) -> Result<Option<String>, BooksError> {
    let row: Option<(String,)> = sqlx::query_as(
        "select code from finance.ledger_accounts
          where party_id = $1 and length(code) < length($2) and $2 like code || '%'
          order by length(code) desc limit 1",
    )
    .bind(party)
    .bind(code)
    .fetch_optional(conn)
    .await
    .map_err(map_err)?;
    Ok(row.map(|(c,)| c))
}

/// Seed the party's chart with every shipped account it does not have yet.
/// Idempotent; parents are written before children because the file lists
/// groups first and the insert goes in file order.
///
/// # Errors
/// The database.
pub async fn ensure(conn: &mut PgConnection, party: Uuid) -> Result<(), BooksError> {
    let chart = Chart::shipped();
    for a in &chart.accounts {
        let parent = chart.parent_of(&a.code);
        sqlx::query(
            "insert into finance.ledger_accounts (party_id, code, name, parent_code, synthetic)
             values ($1, $2, $3, $4, $5)
             on conflict (party_id, code) do nothing",
        )
        .bind(party)
        .bind(&a.code)
        .bind(&a.name)
        .bind(parent)
        .bind(a.synthetic)
        .execute(&mut *conn)
        .await
        .map_err(map_err)?;
    }
    Ok(())
}

/// Whether the party has any account yet.
///
/// # Errors
/// The database.
pub async fn seeded(pool: &PgPool, party: Uuid) -> Result<bool, BooksError> {
    let (n,): (i64,) =
        sqlx::query_as("select count(*) from finance.ledger_accounts where party_id = $1")
            .bind(party)
            .fetch_one(pool)
            .await
            .map_err(map_err)?;
    Ok(n > 0)
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    const TRIAL_BALANCE: &str =
        include_str!("../../../../docs/accountant/expected/trial-balance.csv");

    #[test]
    fn the_shipped_chart_is_well_formed() {
        let chart = Chart::shipped();
        assert_eq!(chart.chart.edition, "2025");
        let mut seen = HashSet::new();
        for a in &chart.accounts {
            assert!(
                a.code.len() >= 3 && a.code.bytes().all(|b| b.is_ascii_digit()),
                "{}: 3 to 6 digits",
                a.code
            );
            assert!(seen.insert(a.code.as_str()), "{} listed twice", a.code);
            assert!(!a.name.trim().is_empty(), "{} has no name", a.code);
            if a.code.len() > 3 {
                let parent = chart
                    .parent_of(&a.code)
                    .unwrap_or_else(|| panic!("{} has no group above it", a.code));
                assert!(a.code.starts_with(parent));
            } else {
                assert!(a.synthetic, "{}: a group is synthetic", a.code);
                assert!(chart.parent_of(&a.code).is_none());
            }
        }
        // A parent is listed before its children, so the seed can insert in
        // file order under the self-referencing foreign key.
        let mut listed = HashSet::new();
        for a in &chart.accounts {
            if let Some(p) = chart.parent_of(&a.code) {
                assert!(
                    listed.contains(p),
                    "{} is listed before its parent {p}",
                    a.code
                );
            }
            listed.insert(a.code.as_str());
        }
    }

    #[test]
    fn every_account_of_the_2025_books_is_in_the_chart() {
        let chart = Chart::shipped();
        let mut n = 0;
        for line in TRIAL_BALANCE.lines().skip(1) {
            let code = line.split(',').next().unwrap_or_default().trim();
            if code.is_empty() {
                continue;
            }
            n += 1;
            let a = chart
                .get(code)
                .unwrap_or_else(|| panic!("{code} of the 2025 trial balance is not in the chart"));
            assert!(!a.synthetic, "{code} carried postings, so it is analytic");
        }
        assert_eq!(n, 77);
    }

    #[test]
    fn a_parent_is_the_longest_prefix() {
        let chart = Chart::shipped();
        assert_eq!(chart.parent_of("03910"), Some("0391"));
        assert_eq!(chart.parent_of("0391"), Some("039"));
        assert_eq!(chart.parent_of("115091"), Some("115"));
        assert_eq!(chart.parent_of("140012"), Some("140"));
        assert_eq!(chart.parent_of("039"), None);
    }
}
