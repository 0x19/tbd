//! The sync worker against the mock bank and a real Postgres.
//!
//! Every test here is about a call that must *not* be made. The bank allows
//! four fetches per account per day; a worker that makes a fifth loses the
//! rest of the day, and one that makes the fourth on a timer leaves nothing
//! for the person pressing Refresh. So the assertions count the mock's calls,
//! and the interesting number is usually zero.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use chrono::{DateTime, Duration, TimeZone, Utc};
use serde_json::{Value, json};
use sqlx::PgPool;
use tbd_db::ensure_user;
use tbd_finance::{
    banking::{Mock, mock::Mode},
    config::Sync as SyncConfig,
    sync::{Outcome, Skipped, Syncer},
};
use uuid::Uuid;

use crate::support::start_with_store;

const IBAN: &str = "HR9224020061100925189";

/// A transaction in the shape Erste sends, minimal but complete enough for
/// the audited parser to accept it.
fn txn(reference: &str, booking: &str, amount: &str, who: &str) -> Value {
    json!({
        "entry_reference": reference,
        "transaction_amount": { "amount": amount, "currency": "EUR" },
        "credit_debit_indicator": "DBIT",
        "status": "BOOK",
        "booking_date": booking,
        "value_date": booking,
        "creditor": { "name": who },
        "creditor_account": { "iban": "HR0000000000000000000" },
        "debtor": { "name": "INORBIT D.O.O." },
        "debtor_account": { "iban": IBAN },
        "remittance_information": ["HR99", "test"],
    })
}

fn page(rows: &[Value], next: Option<&str>) -> Value {
    json!({ "transactions": rows, "continuation_key": next })
}

fn balances() -> Value {
    json!({ "balances": [
        { "balance_type": "CLBD", "balance_amount": { "amount": "1000.00", "currency": "EUR" } }
    ]})
}

fn config() -> SyncConfig {
    SyncConfig {
        // Zero so a test can tick twice without moving the clock, and see
        // the budget -- not the interval -- be what stops the second call.
        min_interval_secs: 0,
        budget_per_day: 4,
        scheduled_budget: 3,
        ..SyncConfig::default()
    }
}

fn at(hour: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 9, 17, hour, 0, 0).unwrap()
}

struct World {
    pool: PgPool,
    party: Uuid,
    connection: Uuid,
    account: Uuid,
    mock: Arc<Mock>,
}

async fn world() -> (crate::support::Server, World) {
    let (server, pool) = start_with_store().await;
    let party = ensure_user(&pool, "sync-owner", None, "O").await.unwrap().0;
    let connection = Uuid::new_v4();
    sqlx::query(
        "insert into finance.connections
            (id, party_id, provider, psu_type, aspsp_name, aspsp_country, session_id,
             status, state, valid_until, authorized_at)
         values ($1, $2, 'mock', 'business', 'Mock Bank', 'HR', 'sess-1', 'authorized',
                 $3, now() + interval '100 days', now())",
    )
    .bind(connection)
    .bind(party)
    .bind(Uuid::new_v4().to_string())
    .execute(&pool)
    .await
    .unwrap();
    let account = Uuid::new_v4();
    sqlx::query(
        "insert into finance.accounts
            (id, party_id, provider, provider_uid, iban, currency, name, connection_id)
         values ($1, $2, 'mock', 'uid-eur', $3, 'EUR', 'Main', $4)",
    )
    .bind(account)
    .bind(party)
    .bind(IBAN)
    .bind(connection)
    .execute(&pool)
    .await
    .unwrap();

    let mock = Mock::new()
        .with_pages(
            "uid-eur",
            vec![
                page(&[txn("r1", "2026-09-10", "10.00", "ANTHROPIC")], Some("k")),
                page(&[txn("r2", "2026-09-15", "20.50", "STUDENAC")], None),
            ],
        )
        .with_balances("uid-eur", balances());
    (
        server,
        World {
            pool,
            party,
            connection,
            account,
            mock,
        },
    )
}

async fn count(pool: &PgPool, sql: &'static str, id: Uuid) -> i64 {
    let (n,): (i64,) = sqlx::query_as(sql).bind(id).fetch_one(pool).await.unwrap();
    n
}

#[tokio::test]
async fn the_first_tick_fetches_every_page_and_records_a_run() {
    let (_s, w) = world().await;
    let syncer = Syncer::new(w.pool.clone(), Arc::clone(&w.mock), config());

    let tick = syncer.tick(at(8)).await.unwrap();
    assert_eq!(
        tick.synced,
        vec![(
            w.account,
            Outcome::Ok {
                pages: 2,
                inserted: 2,
                duplicates: 0,
                booked: 0,
            }
        )]
    );
    // One balances call and one transactions call (the mock returns every
    // page in one go, as the real client does).
    assert_eq!(w.mock.calls(), 2);
    assert_eq!(
        count(
            &w.pool,
            "select count(*) from finance.bank_transactions where account_id = $1",
            w.account
        )
        .await,
        2
    );
    assert_eq!(
        count(
            &w.pool,
            "select count(*) from finance.balances where account_id = $1",
            w.account
        )
        .await,
        1
    );
    let (trigger, outcome, inserted, through): (String, String, i32, Option<chrono::NaiveDate>) =
        sqlx::query_as(
            "select r.trigger, r.outcome, r.inserted, a.last_booked_through
               from finance.sync_runs r join finance.accounts a on a.id = r.account_id
              where r.account_id = $1",
        )
        .bind(w.account)
        .fetch_one(&w.pool)
        .await
        .unwrap();
    assert_eq!(trigger, "initial");
    assert_eq!(outcome, "ok");
    assert_eq!(inserted, 2);
    assert_eq!(
        through.unwrap().to_string(),
        "2026-09-15",
        "the watermark is the latest booking date"
    );
}

#[tokio::test]
async fn a_pending_row_becomes_booked_when_the_bank_books_it_and_never_the_reverse() {
    let (_s, w) = world().await;
    let mut pending = txn("p1", "2026-09-16", "19.79", "PLODINE");
    pending["status"] = json!("PDNG");
    w.mock
        .with_pages("uid-eur", vec![page(&[pending.clone()], None)]);
    let syncer = Syncer::new(w.pool.clone(), Arc::clone(&w.mock), config());
    syncer.tick(at(8)).await.unwrap();
    let (status,): (String,) = sqlx::query_as(
        "select status from finance.bank_transactions where account_id = $1 and entry_reference = 'p1'",
    )
    .bind(w.account)
    .fetch_one(&w.pool)
    .await
    .unwrap();
    assert_eq!(status, "pending");

    // The next day the bank books it: same reference, status BOOK.
    let mut booked = pending.clone();
    booked["status"] = json!("BOOK");
    booked["booking_date"] = json!("2026-09-17");
    w.mock.with_pages("uid-eur", vec![page(&[booked], None)]);
    let tick = syncer.tick(at(9)).await.unwrap();
    assert!(matches!(
        tick.synced[0].1,
        Outcome::Ok {
            inserted: 0,
            booked: 1,
            ..
        }
    ));
    let (booked_on_run,): (i32,) = sqlx::query_as(
        "select booked from finance.sync_runs where account_id = $1 order by started_at desc limit 1",
    )
    .bind(w.account)
    .fetch_one(&w.pool)
    .await
    .unwrap();
    assert_eq!(booked_on_run, 1, "the run records the promotion");
    let (status, booking, n): (String, chrono::NaiveDate, i64) = sqlx::query_as(
        "select status, booking_date, (select count(*) from finance.bank_transactions where account_id = $1)
           from finance.bank_transactions where account_id = $1 and entry_reference = 'p1'",
    )
    .bind(w.account)
    .fetch_one(&w.pool)
    .await
    .unwrap();
    assert_eq!(
        status, "booked",
        "the pending row was promoted, not duplicated"
    );
    assert_eq!(booking.to_string(), "2026-09-17");
    assert_eq!(n, 1);

    // A booked row is immutable: a later page claiming it is pending again,
    // or carries a different amount, changes nothing.
    let mut again = pending;
    again["transaction_amount"]["amount"] = json!("99.99");
    w.mock.with_pages("uid-eur", vec![page(&[again], None)]);
    syncer.tick(at(10)).await.unwrap();
    let (status, amount): (String, i64) = sqlx::query_as(
        "select status, amount_minor from finance.bank_transactions where account_id = $1 and entry_reference = 'p1'",
    )
    .bind(w.account)
    .fetch_one(&w.pool)
    .await
    .unwrap();
    assert_eq!((status.as_str(), amount), ("booked", -1979));
}

#[tokio::test]
async fn a_second_tick_finds_nothing_new_and_the_rows_stay_two() {
    let (_s, w) = world().await;
    let syncer = Syncer::new(w.pool.clone(), Arc::clone(&w.mock), config());
    syncer.tick(at(8)).await.unwrap();
    let tick = syncer.tick(at(9)).await.unwrap();
    assert!(matches!(
        tick.synced[0].1,
        Outcome::Ok {
            inserted: 0,
            duplicates: 2,
            ..
        }
    ));
    assert_eq!(
        count(
            &w.pool,
            "select count(*) from finance.bank_transactions where account_id = $1",
            w.account
        )
        .await,
        2,
        "re-fetching an overlapping window must not duplicate"
    );
}

#[tokio::test]
async fn the_scheduler_stops_at_its_share_and_the_reserve_is_left_for_a_person() {
    let (_s, w) = world().await;
    let syncer = Syncer::new(w.pool.clone(), Arc::clone(&w.mock), config());

    for hour in 8..11 {
        let tick = syncer.tick(at(hour)).await.unwrap();
        assert!(
            matches!(tick.synced[0].1, Outcome::Ok { .. }),
            "tick {hour}"
        );
    }
    let calls_after_three = w.mock.calls();

    // The fourth scheduled tick must not reach the bank at all.
    let tick = syncer.tick(at(11)).await.unwrap();
    assert_eq!(tick.synced, vec![]);
    assert_eq!(tick.skipped, vec![(w.account, Skipped::BudgetSpent)]);
    assert_eq!(
        w.mock.calls(),
        calls_after_three,
        "a skipped account costs no request"
    );

    // A person may still spend the fourth.
    let manual = syncer.refresh(w.account, at(12)).await.unwrap();
    assert!(matches!(manual, Ok(Outcome::Ok { .. })));
    assert!(w.mock.calls() > calls_after_three);
    let calls_after_four = w.mock.calls();

    // And not a fifth, from anyone.
    let manual = syncer.refresh(w.account, at(13)).await.unwrap();
    assert_eq!(manual, Err(Skipped::BudgetSpent));
    assert_eq!(w.mock.calls(), calls_after_four);
}

#[tokio::test]
async fn the_budget_resets_on_the_next_day() {
    let (_s, w) = world().await;
    let syncer = Syncer::new(w.pool.clone(), Arc::clone(&w.mock), config());
    for hour in 8..11 {
        syncer.tick(at(hour)).await.unwrap();
    }
    assert_eq!(syncer.tick(at(23)).await.unwrap().skipped.len(), 1);

    let tomorrow = at(23) + Duration::hours(2);
    let tick = syncer.tick(tomorrow).await.unwrap();
    assert!(
        matches!(tick.synced[0].1, Outcome::Ok { .. }),
        "a new day, a new allowance"
    );
}

#[tokio::test]
async fn a_429_sets_the_backoff_the_bank_asked_for_and_nothing_is_called_until_then() {
    let (_s, w) = world().await;
    let syncer = Syncer::new(w.pool.clone(), Arc::clone(&w.mock), config());
    w.mock
        .set_mode(Mode::RateLimited(Some(std::time::Duration::from_hours(6))));

    let tick = syncer.tick(at(8)).await.unwrap();
    assert_eq!(tick.synced, vec![(w.account, Outcome::RateLimited)]);
    let calls = w.mock.calls();
    let (until,): (Option<DateTime<Utc>>,) =
        sqlx::query_as("select sync_backoff_until from finance.accounts where id = $1")
            .bind(w.account)
            .fetch_one(&w.pool)
            .await
            .unwrap();
    assert_eq!(until.unwrap(), at(14), "backoff is now + Retry-After");

    // Inside the backoff: silence.
    w.mock.set_mode(Mode::Ok);
    let tick = syncer.tick(at(13)).await.unwrap();
    assert_eq!(tick.skipped, vec![(w.account, Skipped::BackingOff)]);
    assert_eq!(w.mock.calls(), calls);

    // After it: business as usual, and the backoff is cleared by success.
    let tick = syncer.tick(at(15)).await.unwrap();
    assert!(matches!(tick.synced[0].1, Outcome::Ok { .. }));
    let (until,): (Option<DateTime<Utc>>,) =
        sqlx::query_as("select sync_backoff_until from finance.accounts where id = $1")
            .bind(w.account)
            .fetch_one(&w.pool)
            .await
            .unwrap();
    assert_eq!(until, None);
}

#[tokio::test]
async fn a_429_without_retry_after_uses_the_documented_six_hours() {
    let (_s, w) = world().await;
    let syncer = Syncer::new(w.pool.clone(), Arc::clone(&w.mock), config());
    w.mock.set_mode(Mode::RateLimited(None));
    syncer.tick(at(8)).await.unwrap();
    let (until,): (Option<DateTime<Utc>>,) =
        sqlx::query_as("select sync_backoff_until from finance.accounts where id = $1")
            .bind(w.account)
            .fetch_one(&w.pool)
            .await
            .unwrap();
    assert_eq!(until.unwrap(), at(14));
}

#[tokio::test]
async fn a_lost_consent_marks_the_connection_and_stops_every_account_on_it() {
    let (_s, w) = world().await;
    // A second account on the same connection.
    let other = Uuid::new_v4();
    sqlx::query(
        "insert into finance.accounts
            (id, party_id, provider, provider_uid, iban, currency, name, connection_id)
         values ($1, $2, 'mock', 'uid-usd', $3, 'USD', 'Dollars', $4)",
    )
    .bind(other)
    .bind(w.party)
    .bind(IBAN)
    .bind(w.connection)
    .execute(&w.pool)
    .await
    .unwrap();
    let syncer = Syncer::new(w.pool.clone(), Arc::clone(&w.mock), config());
    w.mock.expire();

    let tick = syncer.tick(at(8)).await.unwrap();
    // The first account learns the consent is gone; the second is skipped
    // without a call, because the connection is already marked.
    assert_eq!(tick.synced.len(), 1);
    assert_eq!(tick.synced[0].1, Outcome::ConsentInvalid);
    assert_eq!(tick.skipped, vec![(other, Skipped::NoConsent)]);
    let (status,): (String,) =
        sqlx::query_as("select status from finance.connections where id = $1")
            .bind(w.connection)
            .fetch_one(&w.pool)
            .await
            .unwrap();
    assert_eq!(status, "expired");

    // And from then on, nothing is called for either.
    let calls = w.mock.calls();
    let tick = syncer.tick(at(9)).await.unwrap();
    assert!(tick.synced.is_empty());
    assert_eq!(w.mock.calls(), calls);
}

#[tokio::test]
async fn a_failed_call_still_spends_the_budget_because_the_bank_saw_it() {
    // The claim commits before the request. A transport failure after that
    // may or may not have reached the bank; assuming it did not is how a
    // retry loop spends the day.
    let (_s, w) = world().await;
    let syncer = Syncer::new(w.pool.clone(), Arc::clone(&w.mock), config());
    w.mock.set_mode(Mode::Transport);
    let tick = syncer.tick(at(8)).await.unwrap();
    assert_eq!(tick.synced, vec![(w.account, Outcome::Transport)]);
    let (used,): (i32,) =
        sqlx::query_as("select sync_budget_used from finance.accounts where id = $1")
            .bind(w.account)
            .fetch_one(&w.pool)
            .await
            .unwrap();
    assert_eq!(used, 1);
    let (outcome, error): (String, String) =
        sqlx::query_as("select outcome, error from finance.sync_runs where account_id = $1")
            .bind(w.account)
            .fetch_one(&w.pool)
            .await
            .unwrap();
    assert_eq!(outcome, "transport");
    assert!(error.contains("connection reset"), "{error}");
}

#[tokio::test]
async fn an_account_without_a_connection_is_never_fetched() {
    let (_s, w) = world().await;
    sqlx::query("update finance.accounts set connection_id = null where id = $1")
        .bind(w.account)
        .execute(&w.pool)
        .await
        .unwrap();
    let syncer = Syncer::new(w.pool.clone(), Arc::clone(&w.mock), config());
    let tick = syncer.tick(at(8)).await.unwrap();
    assert_eq!(tick.skipped, vec![(w.account, Skipped::NoConsent)]);
    assert_eq!(w.mock.calls(), 0);
}

#[tokio::test]
async fn the_routine_window_starts_before_the_watermark_not_at_now() {
    let (_s, w) = world().await;
    let syncer = Syncer::new(w.pool.clone(), Arc::clone(&w.mock), config());
    syncer.tick(at(8)).await.unwrap();
    syncer.tick(at(9)).await.unwrap();
    let rows: Vec<(String, chrono::NaiveDate)> = sqlx::query_as(
        "select trigger, date_from from finance.sync_runs where account_id = $1 order by started_at",
    )
    .bind(w.account)
    .fetch_all(&w.pool)
    .await
    .unwrap();
    assert_eq!(rows[0].0, "initial");
    assert_eq!(
        rows[0].1.to_string(),
        "2026-01-01",
        "the start of the year on first sight"
    );
    assert_eq!(rows[1].0, "scheduled");
    assert_eq!(
        rows[1].1.to_string(),
        "2026-09-08",
        "watermark 09-15 minus 7 days of overlap"
    );
}

#[tokio::test]
async fn new_rows_are_categorised_by_the_sync_itself() {
    let (_s, w) = world().await;
    let software = Uuid::new_v4();
    sqlx::query(
        "insert into finance.categories (id, party_id, slug, name, kind)
         values ($1, $2, 'software', 'Software', 'expense')",
    )
    .bind(software)
    .bind(w.party)
    .execute(&w.pool)
    .await
    .unwrap();
    sqlx::query(
        "insert into finance.rules (id, party_id, priority, name, match_counterparty_like, category_id)
         values ($1, $2, 10, 'anthropic', 'ANTHROPIC', $3)",
    )
    .bind(Uuid::new_v4())
    .bind(w.party)
    .bind(software)
    .execute(&w.pool)
    .await
    .unwrap();

    let syncer = Syncer::new(w.pool.clone(), Arc::clone(&w.mock), config());
    syncer.tick(at(8)).await.unwrap();

    let (category,): (Option<Uuid>,) = sqlx::query_as(
        "select category_id from finance.bank_transactions
          where account_id = $1 and counterparty_name = 'ANTHROPIC'",
    )
    .bind(w.account)
    .fetch_one(&w.pool)
    .await
    .unwrap();
    assert_eq!(
        category,
        Some(software),
        "nobody ran `finance categorise`; the sync did"
    );
}

#[tokio::test]
async fn two_workers_cannot_both_claim_the_same_call() {
    // Two syncers on one database, as two replicas would be. Their ticks
    // race; the budget must end at exactly what one tick spends.
    let (_s, w) = world().await;
    let a = Arc::new(Syncer::new(w.pool.clone(), Arc::clone(&w.mock), config()));
    let b = Arc::new(Syncer::new(w.pool.clone(), Arc::clone(&w.mock), config()));
    let mut set = tokio::task::JoinSet::new();
    for s in [a, b] {
        for hour in 8..12 {
            let s = Arc::clone(&s);
            set.spawn(async move { s.tick(at(hour)).await.unwrap() });
        }
    }
    let mut fetched = 0;
    let mut busy = 0;
    while let Some(t) = set.join_next().await {
        let t = t.unwrap();
        fetched += t.synced.len();
        busy += t
            .skipped
            .iter()
            .filter(|(_, s)| *s == Skipped::Busy)
            .count();
    }
    // A tick that finds the row held by the other worker skips it -- it is
    // not lost, just deferred to the next interval -- so the number fetched
    // is *at most* the budget, and every fetch is one that was paid for.
    assert!((1..=3).contains(&fetched), "fetched {fetched}");
    assert!(busy <= 8 - fetched, "busy {busy}");
    let (used,): (i32,) =
        sqlx::query_as("select sync_budget_used from finance.accounts where id = $1")
            .bind(w.account)
            .fetch_one(&w.pool)
            .await
            .unwrap();
    assert_eq!(
        usize::try_from(used).unwrap(),
        fetched,
        "every fetch was claimed, no claim was unfetched"
    );
    assert_eq!(
        w.mock.calls(),
        fetched * 2,
        "two requests per fetch, none for a skip"
    );
}
