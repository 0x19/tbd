//! The books: a general ledger per company party.
//!
//! Postings are derived, never typed: every journal entry names its source
//! and the rule that produced it, and re-running a rule is a no-op. Money is
//! integer minor units. The trial balance is the opening balances the year
//! starts from plus the journal's movement; the opening is kept exactly as
//! the accountant handed it over, both sides per account and signed, because
//! a mid-year hand-over carries year-to-date P&L balances and reversals
//! booked as negative postings (`docs/accountant/findings.md` §1). Journal
//! lines are strict: one positive side each, and an entry balances or does
//! not land (`finance.assert_entry_balanced`, deferred to commit).
//!
//! [`chart`] is the `RRiF` chart of accounts the books are seeded with, data
//! in `configs/finance/hr/`. [`store`] is the one writer of the five tables.

pub mod chart;
pub mod store;

use tbd_db::DbError;
use tonic::Status;

/// What class of account a code is, from its first digit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountKind {
    /// Classes 0, 1 and 3.
    Asset,
    /// Class 2.
    Liability,
    /// Class 9.
    Equity,
    /// Class 7.
    Revenue,
    /// Class 4.
    Expense,
    /// Class 8.
    Result,
    /// Classes 5 and 6, not used by a service company.
    Other,
}

impl AccountKind {
    /// The kind of an account code, from its first digit; `Other` for a code
    /// that does not start with a digit.
    #[must_use]
    pub fn of(code: &str) -> Self {
        match code.as_bytes().first() {
            Some(b'0' | b'1' | b'3') => Self::Asset,
            Some(b'2') => Self::Liability,
            Some(b'4') => Self::Expense,
            Some(b'7') => Self::Revenue,
            Some(b'8') => Self::Result,
            Some(b'9') => Self::Equity,
            _ => Self::Other,
        }
    }

    /// The word on the wire and in the database.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Asset => "asset",
            Self::Liability => "liability",
            Self::Equity => "equity",
            Self::Revenue => "revenue",
            Self::Expense => "expense",
            Self::Result => "result",
            Self::Other => "other",
        }
    }
}

/// The class of an account code: its first digit.
#[must_use]
pub fn class_of(code: &str) -> u8 {
    code.as_bytes()
        .first()
        .filter(|b| b.is_ascii_digit())
        .map_or(0, |b| b - b'0')
}

/// What a month is in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeriodStatus {
    /// Postings land.
    Open,
    /// A person closed the month; it reopens.
    Closed,
    /// Filed; final.
    Locked,
}

impl PeriodStatus {
    /// The word on the wire and in the database.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Closed => "closed",
            Self::Locked => "locked",
        }
    }

    /// From the database word.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "open" => Some(Self::Open),
            "closed" => Some(Self::Closed),
            "locked" => Some(Self::Locked),
            _ => None,
        }
    }
}

/// Where an opening came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpeningSource {
    /// The accountant's column as handed over.
    Filed,
    /// Another system's export.
    Imported,
    /// Computed by this service from a prior year.
    Derived,
}

impl OpeningSource {
    /// The word on the wire and in the database.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Filed => "filed",
            Self::Imported => "imported",
            Self::Derived => "derived",
        }
    }

    /// From the wire word; empty means `filed`.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim() {
            "" | "filed" => Some(Self::Filed),
            "imported" => Some(Self::Imported),
            "derived" => Some(Self::Derived),
            _ => None,
        }
    }
}

/// What produced a journal entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum SourceKind {
    Opening,
    Bank,
    Invoice,
    Document,
    Payroll,
    Judgement,
    Closing,
    /// The chaos tool's own postings.
    Chaos,
}

impl SourceKind {
    /// The word in the database.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Opening => "opening",
            Self::Bank => "bank",
            Self::Invoice => "invoice",
            Self::Document => "document",
            Self::Payroll => "payroll",
            Self::Judgement => "judgement",
            Self::Closing => "closing",
            Self::Chaos => "chaos",
        }
    }
}

/// Why a books call was refused.
#[derive(Debug, thiserror::Error)]
pub enum BooksError {
    /// The database, or a row outside the grant.
    #[error(transparent)]
    Db(#[from] DbError),
    /// The set of lines or openings does not balance.
    #[error("unbalanced: debit {debit} against credit {credit} minor units")]
    Unbalanced {
        /// Sum of the debit side.
        debit: i64,
        /// Sum of the credit side.
        credit: i64,
    },
    /// A code the party's chart does not have.
    #[error("unknown account {0}: add it to the chart first")]
    UnknownAccount(String),
    /// A posting or opening on a synthetic account.
    #[error("account {0} is synthetic: post to an analytic account under it")]
    Synthetic(String),
    /// Books belong to companies.
    #[error("books belong to a company; this party is a person")]
    NotCompany,
    /// The month is not open.
    #[error("{year}-{month:02} is {status}")]
    PeriodNotOpen {
        /// Fiscal year.
        year: i32,
        /// Month, 1 to 12.
        month: u32,
        /// The status it has.
        status: &'static str,
    },
    /// A locked period stays locked.
    #[error("{year}-{month:02} is locked; a lock is final")]
    Locked {
        /// Fiscal year.
        year: i32,
        /// Month, 1 to 12.
        month: u32,
    },
    /// A line with both sides, or neither.
    #[error("line {position}: exactly one of debit and credit, and not zero")]
    OneSide {
        /// 1-based position in the entry.
        position: usize,
    },
    /// Nothing to write.
    #[error("{0}")]
    Empty(&'static str),
    /// A malformed value.
    #[error("{0}")]
    Invalid(String),
    /// A caller that may read but not write here.
    #[error("only an owner of the party may {0}")]
    NotOwner(&'static str),
}

impl BooksError {
    /// The gRPC status a refusal becomes.
    #[must_use]
    pub fn status(&self) -> Status {
        match self {
            Self::Db(DbError::NotFound { what }) => Status::not_found(*what),
            Self::Db(DbError::Invalid { field, reason }) => {
                Status::invalid_argument(format!("invalid {field}: {reason}"))
            }
            Self::Db(DbError::Conflict { reason }) => Status::aborted(reason.clone()),
            Self::Db(DbError::Unavailable { .. }) => Status::unavailable("store unavailable"),
            Self::Db(DbError::Internal(_)) => Status::internal("store error"),
            Self::Unbalanced { .. }
            | Self::UnknownAccount(_)
            | Self::Synthetic(_)
            | Self::NotCompany
            | Self::OneSide { .. }
            | Self::Empty(_)
            | Self::Invalid(_) => Status::invalid_argument(self.to_string()),
            Self::PeriodNotOpen { .. } | Self::Locked { .. } => {
                Status::failed_precondition(self.to_string())
            }
            Self::NotOwner(_) => Status::permission_denied(self.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_and_class_come_from_the_first_digit() {
        assert_eq!(AccountKind::of("03910"), AccountKind::Asset);
        assert_eq!(AccountKind::of("1000"), AccountKind::Asset);
        assert_eq!(AccountKind::of("3630"), AccountKind::Asset);
        assert_eq!(AccountKind::of("2200"), AccountKind::Liability);
        assert_eq!(AccountKind::of("4200"), AccountKind::Expense);
        assert_eq!(AccountKind::of("75402"), AccountKind::Revenue);
        assert_eq!(AccountKind::of("800"), AccountKind::Result);
        assert_eq!(AccountKind::of("94010"), AccountKind::Equity);
        assert_eq!(AccountKind::of("5000"), AccountKind::Other);
        assert_eq!(AccountKind::of(""), AccountKind::Other);
        assert_eq!(class_of("75402"), 7);
        assert_eq!(class_of("x"), 0);
    }

    #[test]
    fn refusals_map_to_the_status_a_caller_can_act_on() {
        let s = BooksError::Unbalanced {
            debit: 100,
            credit: 90,
        }
        .status();
        assert_eq!(s.code(), tonic::Code::InvalidArgument);
        assert!(s.message().contains("100") && s.message().contains("90"));
        assert_eq!(
            BooksError::UnknownAccount("4711".into()).status().code(),
            tonic::Code::InvalidArgument
        );
        assert_eq!(
            BooksError::PeriodNotOpen {
                year: 2025,
                month: 7,
                status: "locked"
            }
            .status()
            .code(),
            tonic::Code::FailedPrecondition
        );
        assert_eq!(
            BooksError::Locked {
                year: 2025,
                month: 7
            }
            .status()
            .code(),
            tonic::Code::FailedPrecondition
        );
        assert_eq!(
            BooksError::NotOwner("import").status().code(),
            tonic::Code::PermissionDenied
        );
        assert_eq!(
            BooksError::Db(DbError::Conflict {
                reason: "twice".into()
            })
            .status()
            .code(),
            tonic::Code::Aborted
        );
        assert_eq!(PeriodStatus::parse("locked"), Some(PeriodStatus::Locked));
        assert_eq!(OpeningSource::parse(""), Some(OpeningSource::Filed));
        assert_eq!(OpeningSource::parse("x"), None);
    }
}
