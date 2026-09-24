//! Books: the accountant's 2025 opening column goes in through the RPC and
//! comes back as a trial balance equal to it, to the cent; what is refused is
//! refused with the figure or the code in the message; a locked month stays
//! locked; an unbalanced entry never lands, not even past the code.

use sqlx::PgPool;
use tbd_db::{Capability, PartyId, UserId, create_org, ensure_user, grant};
use tbd_finance::books::{
    SourceKind,
    store::{self, NewEntry, NewLine},
};
use tbd_proto::finance::v1::{
    ClosePeriodRequest, ImportOpeningBalancesRequest, ListLedgerAccountsRequest,
    ListPeriodsRequest, LockPeriodRequest, OpeningBalance, TrialBalanceRequest,
    UpsertLedgerAccountRequest,
};
use tonic::{Code, Request, metadata::MetadataValue};
use uuid::Uuid;

use crate::support::start_with_store;

const OWNER: &str = "books-owner";
const READER: &str = "books-reader";

/// The accountant's opening column at the 1 July 2025 hand-over
/// (docs/accountant/gfi-2025.md §1), signed.
const OPENING: &str = include_str!("../../../../docs/accountant/expected/opening-2025.csv");
/// The accountant's year-end trial balance, for the names and the row count.
const TRIAL_BALANCE: &str = include_str!("../../../../docs/accountant/expected/trial-balance.csv");

fn as_caller<T>(subject: &str, message: T) -> Request<T> {
    let claims = serde_json::json!({ "sub": subject, "scp": ["tbd.finance"] });
    let encoded = base64::Engine::encode(
        &base64::engine::general_purpose::URL_SAFE_NO_PAD,
        claims.to_string(),
    );
    let mut request = Request::new(message);
    request.metadata_mut().insert(
        "x-jwt-payload",
        MetadataValue::try_from(encoded.as_str()).unwrap(),
    );
    request
}

struct World {
    person: Uuid,
    company: Uuid,
}

async fn seed(pool: &PgPool) -> World {
    let owner: UserId = ensure_user(pool, OWNER, None, "Owner").await.unwrap();
    let reader = ensure_user(pool, READER, None, "Reader").await.unwrap();
    let company = create_org(
        pool,
        "INORBIT d.o.o.",
        Some("38846238650"),
        Some("HR"),
        true,
    )
    .await
    .unwrap();
    for party in [PartyId(owner.0), PartyId(company.0)] {
        grant(pool, owner, party, Capability::Own, Some(owner), None)
            .await
            .unwrap();
    }
    grant(
        pool,
        reader,
        PartyId(company.0),
        Capability::Read,
        Some(owner),
        None,
    )
    .await
    .unwrap();
    World {
        person: owner.0,
        company: company.0,
    }
}

/// `1724.22` -> 172422, `-96.27` -> -9627. Exact; the CSV always has two decimals.
fn minor(s: &str) -> i64 {
    let (neg, s) = match s.trim().strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, s.trim()),
    };
    let (whole, frac) = s.split_once('.').unwrap_or((s, "00"));
    assert_eq!(frac.len(), 2, "{s}: two decimals");
    let v = whole.parse::<i64>().unwrap() * 100 + frac.parse::<i64>().unwrap();
    if neg { -v } else { v }
}

fn opening_rows() -> Vec<OpeningBalance> {
    OPENING
        .lines()
        .skip(1)
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let mut f = l.split(',');
            OpeningBalance {
                account_code: f.next().unwrap().to_owned(),
                debit_minor: minor(f.next().unwrap()),
                credit_minor: minor(f.next().unwrap()),
            }
        })
        .collect()
}

fn import(party: Uuid, rows: Vec<OpeningBalance>) -> ImportOpeningBalancesRequest {
    ImportOpeningBalancesRequest {
        party_id: party.to_string(),
        fiscal_year: 2025,
        as_of: "2025-07-01".into(),
        source: "filed".into(),
        rows,
    }
}

fn tb(party: Uuid) -> TrialBalanceRequest {
    TrialBalanceRequest {
        party_id: party.to_string(),
        fiscal_year: 2025,
        through_month: 0,
    }
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn the_2025_opening_column_comes_back_as_the_trial_balance_to_the_cent() {
    let (server, pool) = start_with_store().await;
    let w = seed(&pool).await;
    let mut client = server.client().await;

    let rows = opening_rows();
    assert_eq!(rows.len(), 77);
    let done = client
        .import_opening_balances(as_caller(OWNER, import(w.company, rows.clone())))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(done.accounts, 77);
    assert_eq!(
        done.total_debit_minor, 18_498_801,
        "184.988,01 on the debit side"
    );
    assert_eq!(done.total_credit_minor, 18_498_801);

    let out = client
        .trial_balance(as_caller(READER, tb(w.company)))
        .await
        .unwrap()
        .into_inner();
    assert!(out.balanced);
    assert_eq!(out.total_debit_minor, 18_498_801);
    assert_eq!(out.total_credit_minor, 18_498_801);
    assert_eq!(out.currency, "EUR");
    assert_eq!(out.opening_as_of, "2025-07-01");
    assert_eq!(out.fiscal_year, 2025);
    assert_eq!(out.through_month, 12);
    assert_eq!(out.years, vec![2025]);
    assert_eq!(out.rows.len(), 77);
    // Every account, in the accountant's order, with its opening and, with no
    // postings yet, the same totals and the balance.
    for (r, want) in out.rows.iter().zip(rows.iter()) {
        assert_eq!(r.account_code, want.account_code);
        assert_eq!(
            r.opening_debit_minor, want.debit_minor,
            "{}",
            r.account_code
        );
        assert_eq!(
            r.opening_credit_minor, want.credit_minor,
            "{}",
            r.account_code
        );
        assert_eq!(r.period_debit_minor, 0);
        assert_eq!(r.total_debit_minor, want.debit_minor);
        assert_eq!(r.total_credit_minor, want.credit_minor);
        assert_eq!(r.balance_minor, want.debit_minor - want.credit_minor);
        assert_eq!(r.class, u32::from(r.account_code.as_bytes()[0] - b'0'));
    }
    // Names come from the chart, which carries the accountant's.
    let names: Vec<(&str, &str)> = TRIAL_BALANCE
        .lines()
        .skip(1)
        .filter_map(|l| l.split_once(','))
        .map(|(code, _)| (code, code))
        .collect();
    assert_eq!(names.len(), 77);
    let bank = out.rows.iter().find(|r| r.account_code == "1000").unwrap();
    assert_eq!(bank.name, "Transakcijski račun u banci, Erste");
    assert_eq!(bank.kind, "asset");
    assert_eq!(bank.opening_debit_minor, 726_168);
    // The signed cases of the hand-over column survive as they were filed.
    let third = out.rows.iter().find(|r| r.account_code == "2211").unwrap();
    assert_eq!(third.opening_credit_minor, -9_627);
    let domestic = out.rows.iter().find(|r| r.account_code == "2200").unwrap();
    assert_eq!(
        (domestic.opening_debit_minor, domestic.opening_credit_minor),
        (398_871, 39_954)
    );
    let salaries = out.rows.iter().find(|r| r.account_code == "4200").unwrap();
    assert_eq!(
        salaries.opening_debit_minor, 645_360,
        "a P&L balance at mid-year"
    );
    // The class totals of gfi-2025.md §1, opening columns.
    let class = |c: u32| out.classes.iter().find(|k| k.class == c).unwrap();
    assert_eq!(
        (class(0).opening_debit_minor, class(0).opening_credit_minor),
        (2_901_633, 2_576_707)
    );
    assert_eq!(
        (class(1).opening_debit_minor, class(1).opening_credit_minor),
        (10_087_323, 0)
    );
    assert_eq!(
        (class(2).opening_debit_minor, class(2).opening_credit_minor),
        (537_381, 206_417)
    );
    assert_eq!(
        (class(3).opening_debit_minor, class(3).opening_credit_minor),
        (1_647_270, 1_647_270)
    );
    assert_eq!(
        (class(4).opening_debit_minor, class(4).opening_credit_minor),
        (3_325_194, 0)
    );
    assert_eq!(
        (class(7).opening_debit_minor, class(7).opening_credit_minor),
        (0, 7_594_747)
    );
    assert_eq!(
        (class(9).opening_debit_minor, class(9).opening_credit_minor),
        (0, 6_473_660)
    );
    assert_eq!(class(1).balance_minor, 10_087_323);
    assert_eq!(out.classes.len(), 7, "no class 5, 6 or 8 in these books");
    // Twelve open months came with it.
    assert_eq!(out.periods.len(), 12);
    assert!(out.periods.iter().all(|p| p.status == "open"));

    // A second import of the same year replaces, never duplicates.
    let again = client
        .import_opening_balances(as_caller(OWNER, import(w.company, rows.clone())))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(again.accounts, 77);
    let out = client
        .trial_balance(as_caller(OWNER, tb(w.company)))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(out.rows.len(), 77);
    assert_eq!(out.total_debit_minor, 18_498_801);

    // The chart the company got: the 77 plus the groups above them.
    let chart = client
        .list_ledger_accounts(as_caller(
            READER,
            ListLedgerAccountsRequest {
                party_id: w.company.to_string(),
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert!(chart.accounts.len() > 77);
    let group = chart.accounts.iter().find(|a| a.code == "039").unwrap();
    assert!(group.synthetic && group.parent_code.is_empty());
    let leaf = chart.accounts.iter().find(|a| a.code == "03910").unwrap();
    assert_eq!(leaf.parent_code, "0391");
    assert_eq!(leaf.class, 0);
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn what_is_refused_says_why() {
    let (server, pool) = start_with_store().await;
    let w = seed(&pool).await;
    let mut client = server.client().await;

    // A reader may look but not import.
    let e = client
        .import_opening_balances(as_caller(READER, import(w.company, opening_rows())))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::PermissionDenied, "{e:?}");
    // Books belong to a company.
    let e = client
        .import_opening_balances(as_caller(OWNER, import(w.person, opening_rows())))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::InvalidArgument);
    assert!(e.message().contains("company"), "{e:?}");
    // A stranger's party reads as not found.
    let e = client
        .trial_balance(as_caller("nobody", tb(w.company)))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::NotFound);
    // An unbalanced set names both sums.
    let mut rows = opening_rows();
    rows[0].debit_minor += 1;
    let e = client
        .import_opening_balances(as_caller(OWNER, import(w.company, rows)))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::InvalidArgument);
    assert!(
        e.message().contains("18498802") && e.message().contains("18498801"),
        "{e:?}"
    );
    // An unknown code is named, and nothing of the set lands.
    let mut rows = opening_rows();
    rows[0].account_code = "4711".into();
    let e = client
        .import_opening_balances(as_caller(OWNER, import(w.company, rows.clone())))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::InvalidArgument);
    assert!(e.message().contains("4711"), "{e:?}");
    let out = client
        .trial_balance(as_caller(OWNER, tb(w.company)))
        .await
        .unwrap()
        .into_inner();
    assert!(out.rows.is_empty());
    assert!(out.balanced, "nothing balances");
    // A synthetic account takes no opening.
    let mut rows = opening_rows();
    rows[0].account_code = "031".into();
    let e = client
        .import_opening_balances(as_caller(OWNER, import(w.company, rows)))
        .await
        .unwrap_err();
    assert!(e.message().contains("synthetic"), "{e:?}");
    // Adding the analytic account first makes the same import go through.
    let e = client
        .upsert_ledger_account(as_caller(
            OWNER,
            UpsertLedgerAccountRequest {
                party_id: w.company.to_string(),
                code: "8711".into(),
                name: "Nowhere".into(),
            },
        ))
        .await
        .unwrap_err();
    assert_eq!(
        e.code(),
        Code::InvalidArgument,
        "no group above 8711: {e:?}"
    );
    let added = client
        .upsert_ledger_account(as_caller(
            OWNER,
            UpsertLedgerAccountRequest {
                party_id: w.company.to_string(),
                code: "4742".into(),
                name: "Troškovi testiranja".into(),
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .account
        .unwrap();
    assert_eq!(added.parent_code, "474");
    assert_eq!(added.kind, "expense");
    let mut rows = opening_rows();
    rows[0].account_code = "4742".into();
    client
        .import_opening_balances(as_caller(OWNER, import(w.company, rows)))
        .await
        .unwrap();
    // The reader cannot change the chart.
    let e = client
        .upsert_ledger_account(as_caller(
            READER,
            UpsertLedgerAccountRequest {
                party_id: w.company.to_string(),
                code: "4712".into(),
                name: "x".into(),
            },
        ))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::PermissionDenied);
    // Malformed dates and codes are invalid arguments, not store errors.
    let mut bad = import(w.company, opening_rows());
    bad.as_of = "01.07.2025".into();
    let e = client
        .import_opening_balances(as_caller(OWNER, bad))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::InvalidArgument);
    let mut bad = import(w.company, opening_rows());
    bad.as_of = "2024-07-01".into();
    let e = client
        .import_opening_balances(as_caller(OWNER, bad))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::InvalidArgument);
    assert!(e.message().contains("2024-07-01"), "{e:?}");
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn a_locked_month_refuses_the_import_and_stays_locked() {
    let (server, pool) = start_with_store().await;
    let w = seed(&pool).await;
    let mut client = server.client().await;

    let months = client
        .list_periods(as_caller(
            READER,
            ListPeriodsRequest {
                party_id: w.company.to_string(),
                fiscal_year: 2025,
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(months.periods.len(), 12);
    assert!(
        months
            .periods
            .iter()
            .all(|p| p.status == "open" && p.changed_at.is_empty())
    );

    // Close July, import refused; reopen, import goes through.
    let p = client
        .close_period(as_caller(
            OWNER,
            ClosePeriodRequest {
                party_id: w.company.to_string(),
                fiscal_year: 2025,
                month: 7,
                reopen: false,
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .period
        .unwrap();
    assert_eq!(p.status, "closed");
    assert!(!p.changed_at.is_empty());
    let e = client
        .import_opening_balances(as_caller(OWNER, import(w.company, opening_rows())))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::FailedPrecondition);
    assert!(
        e.message().contains("2025-07") && e.message().contains("closed"),
        "{e:?}"
    );
    client
        .close_period(as_caller(
            OWNER,
            ClosePeriodRequest {
                party_id: w.company.to_string(),
                fiscal_year: 2025,
                month: 7,
                reopen: true,
            },
        ))
        .await
        .unwrap();
    client
        .import_opening_balances(as_caller(OWNER, import(w.company, opening_rows())))
        .await
        .unwrap();

    // Lock July: final. Reopening is refused, a re-import is refused, and a
    // reader cannot lock at all.
    let e = client
        .lock_period(as_caller(
            READER,
            LockPeriodRequest {
                party_id: w.company.to_string(),
                fiscal_year: 2025,
                month: 7,
            },
        ))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::PermissionDenied);
    let p = client
        .lock_period(as_caller(
            OWNER,
            LockPeriodRequest {
                party_id: w.company.to_string(),
                fiscal_year: 2025,
                month: 7,
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .period
        .unwrap();
    assert_eq!(p.status, "locked");
    let e = client
        .close_period(as_caller(
            OWNER,
            ClosePeriodRequest {
                party_id: w.company.to_string(),
                fiscal_year: 2025,
                month: 7,
                reopen: true,
            },
        ))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::FailedPrecondition);
    assert!(e.message().contains("final"), "{e:?}");
    let e = client
        .import_opening_balances(as_caller(OWNER, import(w.company, opening_rows())))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::FailedPrecondition);
    assert!(e.message().contains("locked"), "{e:?}");
    // The trial balance still shows the import that went through, and the lock.
    let out = client
        .trial_balance(as_caller(OWNER, tb(w.company)))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(out.rows.len(), 77);
    assert_eq!(out.periods[6].status, "locked");
    // A locked month refuses a posting too.
    let e = store::post(&pool, &entry(w.company, "2025-07-15", 100, 100))
        .await
        .unwrap_err();
    assert!(e.to_string().contains("locked"), "{e}");
}

fn entry(party: Uuid, date: &str, debit: i64, credit: i64) -> NewEntry {
    NewEntry {
        party_id: party,
        entry_date: date.parse().unwrap(),
        source: SourceKind::Chaos,
        source_id: format!("{date}/{debit}/{credit}"),
        rule_id: "test".into(),
        rule_version: "1".into(),
        memo: String::new(),
        lines: vec![
            NewLine {
                account_code: "4164".into(),
                debit_minor: debit,
                ..NewLine::default()
            },
            NewLine {
                account_code: "2200".into(),
                credit_minor: credit,
                ..NewLine::default()
            },
        ],
    }
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn an_entry_balances_or_does_not_land() {
    let (server, pool) = start_with_store().await;
    let w = seed(&pool).await;
    let mut client = server.client().await;
    client
        .import_opening_balances(as_caller(OWNER, import(w.company, opening_rows())))
        .await
        .unwrap();

    // A balanced entry lands and moves the trial balance on both accounts.
    let id = store::post(&pool, &entry(w.company, "2025-08-05", 151_000, 151_000))
        .await
        .unwrap();
    let out = client
        .trial_balance(as_caller(OWNER, tb(w.company)))
        .await
        .unwrap()
        .into_inner();
    assert!(out.balanced);
    assert_eq!(out.total_debit_minor, 18_498_801 + 151_000);
    let fee = out.rows.iter().find(|r| r.account_code == "4164").unwrap();
    assert_eq!(
        (
            fee.opening_debit_minor,
            fee.period_debit_minor,
            fee.total_debit_minor
        ),
        (110_000, 151_000, 261_000)
    );
    let supplier = out.rows.iter().find(|r| r.account_code == "2200").unwrap();
    assert_eq!(supplier.period_credit_minor, 151_000);
    assert_eq!(supplier.balance_minor, 398_871 - 39_954 - 151_000);
    // Through July the posting is not there yet.
    let july = client
        .trial_balance(as_caller(
            OWNER,
            TrialBalanceRequest {
                through_month: 7,
                ..tb(w.company)
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(july.total_debit_minor, 18_498_801);

    // The same source key again is a conflict, not a second entry.
    let e = store::post(&pool, &entry(w.company, "2025-08-05", 151_000, 151_000))
        .await
        .unwrap_err();
    assert_eq!(e.status().code(), Code::Aborted, "{e}");

    // Unbalanced, both sides on a line, one line, a synthetic account: refused
    // by the code with the figure or the code in the message.
    let e = store::post(&pool, &entry(w.company, "2025-08-06", 100, 99))
        .await
        .unwrap_err();
    assert!(
        e.to_string().contains("100") && e.to_string().contains("99"),
        "{e}"
    );
    let mut both = entry(w.company, "2025-08-06", 100, 100);
    both.lines[0].credit_minor = 5;
    let e = store::post(&pool, &both).await.unwrap_err();
    assert!(e.to_string().starts_with("line 1"), "{e}");
    let mut one = entry(w.company, "2025-08-06", 100, 100);
    one.lines.pop();
    assert!(store::post(&pool, &one).await.is_err());
    let mut synthetic = entry(w.company, "2025-08-06", 100, 100);
    synthetic.lines[0].account_code = "416".into();
    let e = store::post(&pool, &synthetic).await.unwrap_err();
    assert!(e.to_string().contains("synthetic"), "{e}");
    let e = store::post(&pool, &entry(w.person, "2025-08-06", 100, 100))
        .await
        .unwrap_err();
    assert!(e.to_string().contains("company"), "{e}");

    // Past the code: an unbalanced set of lines written straight into the
    // table fails at commit, on the deferred constraint trigger.
    let mut tx = pool.begin().await.unwrap();
    let rogue = Uuid::now_v7();
    sqlx::query(
        "insert into finance.journal_entries (id, party_id, entry_date, source_kind, source_id, rule_id, rule_version)
         values ($1, $2, '2025-08-07', 'chaos', 'rogue', 'sql', '1')",
    )
    .bind(rogue)
    .bind(w.company)
    .execute(&mut *tx)
    .await
    .unwrap();
    sqlx::query(
        "insert into finance.journal_lines (entry_id, party_id, position, account_code, debit_minor)
         values ($1, $2, 1, '4164', 700)",
    )
    .bind(rogue)
    .bind(w.company)
    .execute(&mut *tx)
    .await
    .unwrap();
    sqlx::query(
        "insert into finance.journal_lines (entry_id, party_id, position, account_code, credit_minor)
         values ($1, $2, 2, '2200', 600)",
    )
    .bind(rogue)
    .bind(w.company)
    .execute(&mut *tx)
    .await
    .unwrap();
    let e = tx.commit().await.unwrap_err();
    assert!(
        e.to_string().contains("unbalanced") && e.to_string().contains("100"),
        "{e}"
    );
    let out = client
        .trial_balance(as_caller(OWNER, tb(w.company)))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(
        out.total_debit_minor,
        18_498_801 + 151_000,
        "the rogue entry left no trace"
    );
    assert_eq!(out.years, vec![2025]);
    let _ = id;
}
