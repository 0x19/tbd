//! The books RPCs: the chart, the periods, the opening import and the trial
//! balance. Reads need the party in the grant; writes need `own` on it.

use chrono::{Datelike, NaiveDate, SecondsFormat, Utc};
use tbd_db::{PartyId, UserId};
use tbd_proto::finance::v1::{
    ClosePeriodRequest, ClosePeriodResponse, ImportOpeningBalancesRequest,
    ImportOpeningBalancesResponse, LedgerAccount, ListLedgerAccountsRequest,
    ListLedgerAccountsResponse, ListPeriodsRequest, ListPeriodsResponse, LockPeriodRequest,
    LockPeriodResponse, Period, TrialBalanceClass, TrialBalanceRequest, TrialBalanceResponse,
    TrialBalanceRow, UpsertLedgerAccountRequest, UpsertLedgerAccountResponse,
};
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::{
    books::{
        BooksError, OpeningSource, PeriodStatus,
        store::{self, AccountRow, Opening, PeriodRow, TrialBalance},
    },
    service::Finance,
};

/// The books' currency. One for now; a foreign document is valued into it
/// on the line (`original_minor`).
const CURRENCY: &str = "EUR";

fn uuid(s: &str, field: &str) -> Result<Uuid, Status> {
    Uuid::parse_str(s).map_err(|_| Status::invalid_argument(format!("{field}: not a uuid")))
}

fn year_or_now(y: u32) -> Result<i32, Status> {
    if y == 0 {
        Ok(Utc::now().year())
    } else {
        i32::try_from(y)
            .ok()
            .filter(|y| (2000..=2100).contains(y))
            .ok_or_else(|| Status::invalid_argument("fiscal_year: 2000 to 2100"))
    }
}

fn account_proto(a: AccountRow) -> LedgerAccount {
    LedgerAccount {
        code: a.code,
        name: a.name,
        parent_code: a.parent_code.unwrap_or_default(),
        class: u32::try_from(a.class).unwrap_or(0),
        kind: a.kind,
        synthetic: a.synthetic,
        archived: a.archived_at.is_some(),
    }
}

fn period_proto(p: &PeriodRow) -> Period {
    Period {
        fiscal_year: u32::try_from(p.fiscal_year).unwrap_or(0),
        month: u32::try_from(p.month).unwrap_or(0),
        status: p.status.as_str().to_owned(),
        changed_at: p
            .changed_at
            .map(|t| t.to_rfc3339_opts(SecondsFormat::Secs, true))
            .unwrap_or_default(),
    }
}

/// The trial balance on the wire: sums and balances are computed here, in
/// integers, once.
pub(crate) fn trial_balance_proto(party: Uuid, tb: TrialBalance) -> TrialBalanceResponse {
    let mut rows = Vec::with_capacity(tb.rows.len());
    let mut classes: Vec<TrialBalanceClass> = Vec::new();
    let (mut debit, mut credit) = (0i64, 0i64);
    for r in tb.rows {
        let total_debit = r.opening_debit_minor + r.period_debit_minor;
        let total_credit = r.opening_credit_minor + r.period_credit_minor;
        debit += total_debit;
        credit += total_credit;
        let class = u32::try_from(r.class).unwrap_or(0);
        let at = if let Some(i) = classes.iter().position(|c| c.class == class) {
            i
        } else {
            classes.push(TrialBalanceClass {
                class,
                ..TrialBalanceClass::default()
            });
            classes.len() - 1
        };
        let c = &mut classes[at];
        c.opening_debit_minor += r.opening_debit_minor;
        c.opening_credit_minor += r.opening_credit_minor;
        c.period_debit_minor += r.period_debit_minor;
        c.period_credit_minor += r.period_credit_minor;
        c.total_debit_minor += total_debit;
        c.total_credit_minor += total_credit;
        c.balance_minor += total_debit - total_credit;
        rows.push(TrialBalanceRow {
            account_code: r.account_code,
            name: r.name,
            class,
            kind: r.kind,
            opening_debit_minor: r.opening_debit_minor,
            opening_credit_minor: r.opening_credit_minor,
            period_debit_minor: r.period_debit_minor,
            period_credit_minor: r.period_credit_minor,
            total_debit_minor: total_debit,
            total_credit_minor: total_credit,
            balance_minor: total_debit - total_credit,
        });
    }
    TrialBalanceResponse {
        party_id: party.to_string(),
        fiscal_year: u32::try_from(tb.fiscal_year).unwrap_or(0),
        through_month: tb.through_month,
        currency: CURRENCY.to_owned(),
        opening_as_of: tb.opening_as_of.map(|d| d.to_string()).unwrap_or_default(),
        rows,
        classes,
        total_debit_minor: debit,
        total_credit_minor: credit,
        balanced: debit == credit,
        periods: tb.periods.iter().map(period_proto).collect(),
        years: tb
            .years
            .into_iter()
            .filter_map(|y| u32::try_from(y).ok())
            .collect(),
    }
}

impl Finance {
    fn done_b<T>(
        &self,
        timer: &mut tbd_common::metrics::RequestTimer,
        r: Result<T, BooksError>,
    ) -> Result<Response<T>, Status> {
        match r {
            Ok(v) => Ok(Response::new(v)),
            Err(e) => {
                tracing::warn!(error = %e, "books rpc refused");
                Err(self.reject(timer, e.status()))
            }
        }
    }

    /// The caller must hold `own` on the party to write its books. Answers
    /// not found for a party outside the grant, like every read.
    async fn owner_of(
        &self,
        pool: &sqlx::PgPool,
        access: &tbd_db::Access,
        party: Uuid,
        what: &'static str,
    ) -> Result<UserId, BooksError> {
        access.require(PartyId(party), "party")?;
        if store::owns(pool, access.user(), PartyId(party)).await? {
            Ok(access.user())
        } else {
            Err(BooksError::NotOwner(what))
        }
    }

    pub(crate) async fn rpc_list_ledger_accounts(
        &self,
        request: Request<ListLedgerAccountsRequest>,
    ) -> Result<Response<ListLedgerAccountsResponse>, Status> {
        let party = uuid(&request.get_ref().party_id, "party_id")?;
        let (mut timer, pool, access, _) = self
            .invoice_context("FinanceService/ListLedgerAccounts", &request, &[])
            .await?;
        let r =
            store::accounts(pool, &access, party)
                .await
                .map(|rows| ListLedgerAccountsResponse {
                    accounts: rows.into_iter().map(account_proto).collect(),
                });
        self.done_b(&mut timer, r)
    }

    pub(crate) async fn rpc_upsert_ledger_account(
        &self,
        request: Request<UpsertLedgerAccountRequest>,
    ) -> Result<Response<UpsertLedgerAccountResponse>, Status> {
        let req = request.get_ref().clone();
        let party = uuid(&req.party_id, "party_id")?;
        let (mut timer, pool, access, _) = self
            .invoice_context("FinanceService/UpsertLedgerAccount", &request, &[])
            .await?;
        let r = async {
            self.owner_of(pool, &access, party, "change the chart")
                .await?;
            store::upsert_account(pool, party, &req.code, &req.name).await
        }
        .await
        .map(|a| UpsertLedgerAccountResponse {
            account: Some(account_proto(a)),
        });
        self.done_b(&mut timer, r)
    }

    pub(crate) async fn rpc_list_periods(
        &self,
        request: Request<ListPeriodsRequest>,
    ) -> Result<Response<ListPeriodsResponse>, Status> {
        let req = request.get_ref().clone();
        let party = uuid(&req.party_id, "party_id")?;
        let year = year_or_now(req.fiscal_year)?;
        let (mut timer, pool, access, _) = self
            .invoice_context("FinanceService/ListPeriods", &request, &[])
            .await?;
        let r = store::periods(pool, &access, party, year)
            .await
            .map(|rows| ListPeriodsResponse {
                fiscal_year: u32::try_from(year).unwrap_or(0),
                periods: rows.iter().map(period_proto).collect(),
            });
        self.done_b(&mut timer, r)
    }

    pub(crate) async fn rpc_close_period(
        &self,
        request: Request<ClosePeriodRequest>,
    ) -> Result<Response<ClosePeriodResponse>, Status> {
        let req = request.get_ref().clone();
        let party = uuid(&req.party_id, "party_id")?;
        let year = year_or_now(req.fiscal_year)?;
        let (mut timer, pool, access, _) = self
            .invoice_context("FinanceService/ClosePeriod", &request, &[])
            .await?;
        let status = if req.reopen {
            PeriodStatus::Open
        } else {
            PeriodStatus::Closed
        };
        let r = async {
            let by = self
                .owner_of(pool, &access, party, "close a period")
                .await?;
            store::set_period(pool, party, year, req.month, status, Some(by)).await
        }
        .await
        .map(|p| ClosePeriodResponse {
            period: Some(period_proto(&p)),
        });
        self.done_b(&mut timer, r)
    }

    pub(crate) async fn rpc_lock_period(
        &self,
        request: Request<LockPeriodRequest>,
    ) -> Result<Response<LockPeriodResponse>, Status> {
        let req = request.get_ref().clone();
        let party = uuid(&req.party_id, "party_id")?;
        let year = year_or_now(req.fiscal_year)?;
        let (mut timer, pool, access, _) = self
            .invoice_context("FinanceService/LockPeriod", &request, &[])
            .await?;
        let r = async {
            let by = self.owner_of(pool, &access, party, "lock a period").await?;
            store::set_period(pool, party, year, req.month, PeriodStatus::Locked, Some(by)).await
        }
        .await
        .map(|p| LockPeriodResponse {
            period: Some(period_proto(&p)),
        });
        self.done_b(&mut timer, r)
    }

    pub(crate) async fn rpc_import_opening_balances(
        &self,
        request: Request<ImportOpeningBalancesRequest>,
    ) -> Result<Response<ImportOpeningBalancesResponse>, Status> {
        let req = request.get_ref().clone();
        let party = uuid(&req.party_id, "party_id")?;
        let year = year_or_now(req.fiscal_year)?;
        let as_of = NaiveDate::parse_from_str(req.as_of.trim(), "%Y-%m-%d")
            .map_err(|_| Status::invalid_argument("as_of: want YYYY-MM-DD"))?;
        let source = OpeningSource::parse(&req.source)
            .ok_or_else(|| Status::invalid_argument("source: filed, imported or derived"))?;
        let rows: Vec<Opening> = req
            .rows
            .into_iter()
            .map(|r| Opening {
                account_code: r.account_code.trim().to_owned(),
                debit_minor: r.debit_minor,
                credit_minor: r.credit_minor,
            })
            .collect();
        let (mut timer, pool, access, _) = self
            .invoice_context("FinanceService/ImportOpeningBalances", &request, &[])
            .await?;
        let r = async {
            let by = self
                .owner_of(pool, &access, party, "import an opening")
                .await?;
            store::import_opening(pool, party, year, as_of, source, &rows, Some(by)).await
        }
        .await
        .map(|done| ImportOpeningBalancesResponse {
            accounts: u32::try_from(done.accounts).unwrap_or(u32::MAX),
            total_debit_minor: done.total_debit_minor,
            total_credit_minor: done.total_credit_minor,
        });
        self.done_b(&mut timer, r)
    }

    pub(crate) async fn rpc_trial_balance(
        &self,
        request: Request<TrialBalanceRequest>,
    ) -> Result<Response<TrialBalanceResponse>, Status> {
        let req = request.get_ref().clone();
        let party = uuid(&req.party_id, "party_id")?;
        let year = year_or_now(req.fiscal_year)?;
        let through = if req.through_month == 0 {
            12
        } else {
            req.through_month
        };
        if !(1..=12).contains(&through) {
            return Err(Status::invalid_argument("through_month: 1 to 12"));
        }
        let (mut timer, pool, access, _) = self
            .invoice_context("FinanceService/TrialBalance", &request, &[])
            .await?;
        let r = store::trial_balance(pool, &access, party, year, through)
            .await
            .map(|tb| trial_balance_proto(party, tb));
        self.done_b(&mut timer, r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::books::store::TrialBalanceRow as Row;

    fn row(code: &str, od: i64, oc: i64, pd: i64, pc: i64) -> Row {
        Row {
            account_code: code.into(),
            name: code.into(),
            class: i16::from(code.as_bytes()[0] - b'0'),
            kind: String::new(),
            opening_debit_minor: od,
            opening_credit_minor: oc,
            period_debit_minor: pd,
            period_credit_minor: pc,
        }
    }

    #[test]
    fn totals_balances_and_classes_are_integer_sums() {
        // The accountant's 2211: debit 1.089,31 opening, a negative credit of
        // -96,27, then 1.809,48 / 2.995,06 in the period: totals equal, balance 0.
        let tb = TrialBalance {
            fiscal_year: 2025,
            through_month: 12,
            opening_as_of: None,
            rows: vec![
                row("2211", 108_931, -9_627, 180_948, 299_506),
                row("2200", 398_871, 39_954, 0, 0),
                row("1000", 0, 0, 100, 0),
            ],
            periods: Vec::new(),
            years: vec![2025],
        };
        let out = trial_balance_proto(Uuid::nil(), tb);
        assert_eq!(out.rows[0].total_debit_minor, 289_879);
        assert_eq!(out.rows[0].total_credit_minor, 289_879);
        assert_eq!(out.rows[0].balance_minor, 0);
        assert_eq!(out.rows[1].balance_minor, 358_917);
        let c2 = out.classes.iter().find(|c| c.class == 2).unwrap();
        assert_eq!(c2.opening_credit_minor, 30_327);
        assert_eq!(c2.balance_minor, 358_917);
        assert_eq!(out.total_debit_minor, 289_879 + 398_871 + 100);
        assert!(!out.balanced);
        assert_eq!(out.currency, "EUR");
    }
}
