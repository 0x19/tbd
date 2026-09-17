//! Importing the prototype's real Enable Banking responses.
//!
//! These run against the files in `prototype/bank/data/`, which are real
//! transactions from a real bank. They are skipped when that directory is
//! absent -- it is gitignored, because it holds someone's actual money -- and
//! say so rather than passing quietly.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::{Path, PathBuf};

use tbd_db::{PartyId, create_org, ensure_user};

use crate::support::start_with_store;

fn prototype_dir() -> Option<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../prototype/bank/data");
    dir.join("sessions").is_dir().then_some(dir)
}

#[tokio::test]
async fn importing_real_transactions_is_idempotent() {
    let Some(dir) = prototype_dir() else {
        eprintln!("skipping: prototype/bank/data is absent (it is gitignored)");
        return;
    };
    let (_server, pool) = start_with_store().await;

    let owner = ensure_user(&pool, "import-owner", None, "Owner")
        .await
        .unwrap();
    let company = create_org(
        &pool,
        "Inorbit d.o.o.",
        Some("38846238650"),
        Some("HR"),
        true,
    )
    .await
    .unwrap();

    // The business profile belongs to the company, the personal one to the person.
    let first_business = tbd_finance::import::from_prototype(&pool, &dir, "business", company.0)
        .await
        .unwrap();
    let first_personal = tbd_finance::import::from_prototype(&pool, &dir, "personal", owner.0)
        .await
        .unwrap();

    assert!(
        first_business.inserted > 0,
        "the business profile imported nothing: {first_business:?}"
    );
    assert!(
        first_personal.inserted > 0,
        "the personal profile imported nothing: {first_personal:?}"
    );

    // The property worth having: a second run is free. A sync worker that
    // overlaps its window -- which it must, to catch late corrections -- would
    // otherwise duplicate every row in the overlap.
    let again_business = tbd_finance::import::from_prototype(&pool, &dir, "business", company.0)
        .await
        .unwrap();
    let again_personal = tbd_finance::import::from_prototype(&pool, &dir, "personal", owner.0)
        .await
        .unwrap();

    assert_eq!(
        again_business.inserted, 0,
        "re-importing the business profile inserted {} rows",
        again_business.inserted
    );
    assert_eq!(
        again_personal.inserted, 0,
        "re-importing the personal profile inserted {} rows",
        again_personal.inserted
    );
    // Everything seen the second time is a duplicate, including rows that were
    // already duplicates the first time round. The prototype directory happens
    // to contain one page under two filenames -- an aborted pull that was
    // retried -- so the first run deduplicates against itself, which is exactly
    // the behaviour a sync worker needs when its window overlaps.
    assert_eq!(
        again_business.duplicates,
        first_business.inserted + first_business.duplicates,
        "first: {first_business:?}, again: {again_business:?}"
    );
    assert_eq!(
        again_personal.duplicates,
        first_personal.inserted + first_personal.duplicates
    );
}

#[tokio::test]
async fn the_owner_sees_their_own_money_and_nobody_elses() {
    let Some(dir) = prototype_dir() else {
        eprintln!("skipping: prototype/bank/data is absent (it is gitignored)");
        return;
    };
    let (_server, pool) = start_with_store().await;

    let owner = ensure_user(&pool, "import-owner", None, "Owner")
        .await
        .unwrap();
    let stranger = ensure_user(&pool, "import-stranger", None, "Stranger")
        .await
        .unwrap();
    let company = create_org(
        &pool,
        "Inorbit d.o.o.",
        Some("38846238650"),
        Some("HR"),
        true,
    )
    .await
    .unwrap();
    tbd_db::grant(
        &pool,
        owner,
        PartyId(company.0),
        tbd_db::Capability::Own,
        Some(owner),
        None,
    )
    .await
    .unwrap();

    tbd_finance::import::from_prototype(&pool, &dir, "business", company.0)
        .await
        .unwrap();
    tbd_finance::import::from_prototype(&pool, &dir, "personal", owner.0)
        .await
        .unwrap();

    let access = tbd_db::Access::for_user(&pool, owner).await.unwrap();
    let store = tbd_finance::store::PgStore::new(pool.clone());
    let mine = tbd_finance::store::Store::transactions(
        &store,
        &access,
        &[],
        &tbd_finance::store::TransactionFilter::default(),
        500,
    )
    .await
    .unwrap();
    assert!(
        !mine.is_empty(),
        "the owner sees nothing of their own money"
    );

    let outsider = tbd_db::Access::for_user(&pool, stranger).await.unwrap();
    let theirs = tbd_finance::store::Store::transactions(
        &store,
        &outsider,
        &[],
        &tbd_finance::store::TransactionFilter::default(),
        500,
    )
    .await
    .unwrap();
    assert!(
        theirs.is_empty(),
        "LEAK: an unrelated user saw {} real transactions",
        theirs.len()
    );
}

#[tokio::test]
async fn our_own_iban_is_never_recorded_as_the_counterparty() {
    // Measured on the real data: the obvious rule -- creditor on a debit,
    // debtor on a credit -- names our own account on 120 of 2,912 rows. Those
    // are transfers between accounts we hold; counting them as income or spend
    // is the same euros twice, and it would make every "where does my money go"
    // answer wrong.
    let Some(dir) = prototype_dir() else {
        eprintln!("skipping: prototype/bank/data is absent (it is gitignored)");
        return;
    };
    let (_server, pool) = start_with_store().await;

    let owner = ensure_user(&pool, "iban-owner", None, "Owner")
        .await
        .unwrap();
    let company = create_org(
        &pool,
        "Inorbit d.o.o.",
        Some("38846238650"),
        Some("HR"),
        true,
    )
    .await
    .unwrap();
    tbd_finance::import::from_prototype(&pool, &dir, "business", company.0)
        .await
        .unwrap();
    tbd_finance::import::from_prototype(&pool, &dir, "personal", owner.0)
        .await
        .unwrap();

    let (self_referential,): (i64,) = sqlx::query_as(
        "select count(*)
           from finance.bank_transactions t
           join finance.accounts a on a.id = t.account_id
          where t.counterparty_iban is not null
            and t.counterparty_iban = a.iban",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        self_referential, 0,
        "{self_referential} transactions name their own account as the counterparty"
    );
}

#[tokio::test]
async fn transfers_between_our_own_accounts_are_identifiable() {
    // They are real transactions on both sides and must be kept -- an owner
    // draw is a genuine company expense and a genuine personal receipt. What
    // must not happen is counting them in a combined view, where they net to
    // zero. Being able to find them is what makes that possible.
    let Some(dir) = prototype_dir() else {
        eprintln!("skipping: prototype/bank/data is absent (it is gitignored)");
        return;
    };
    let (_server, pool) = start_with_store().await;

    let owner = ensure_user(&pool, "xfer-owner", None, "Owner")
        .await
        .unwrap();
    let company = create_org(
        &pool,
        "Inorbit d.o.o.",
        Some("38846238650"),
        Some("HR"),
        true,
    )
    .await
    .unwrap();
    tbd_finance::import::from_prototype(&pool, &dir, "business", company.0)
        .await
        .unwrap();
    tbd_finance::import::from_prototype(&pool, &dir, "personal", owner.0)
        .await
        .unwrap();

    let (internal,): (i64,) = sqlx::query_as(
        "select count(*)
           from finance.bank_transactions t
          where exists (select 1 from finance.accounts a where a.iban = t.counterparty_iban)",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(
        internal > 0,
        "no internal transfers found; the real data has them, so the counterparty \
         iban is not being recorded"
    );
}

#[tokio::test]
async fn the_value_date_is_kept_where_it_differs_from_the_booking_date() {
    // 164 of 2,912 real rows settle on a different day than they book. That is
    // what interest and FX reconcile against, so dropping it loses the ability
    // to reconcile at all.
    let Some(dir) = prototype_dir() else {
        eprintln!("skipping: prototype/bank/data is absent (it is gitignored)");
        return;
    };
    let (_server, pool) = start_with_store().await;

    let owner = ensure_user(&pool, "vd-owner", None, "Owner").await.unwrap();
    tbd_finance::import::from_prototype(&pool, &dir, "personal", owner.0)
        .await
        .unwrap();

    let (differing,): (i64,) = sqlx::query_as(
        "select count(*) from finance.bank_transactions
          where value_date is not null and value_date <> booking_date",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(
        differing > 0,
        "no row has a value date differing from its booking date; the column is not populated"
    );

    let (missing,): (i64,) =
        sqlx::query_as("select count(*) from finance.bank_transactions where value_date is null")
            .fetch_one(&pool)
            .await
            .unwrap();
    // The provider omits it on a handful of rows; that is its data, not a bug.
    assert!(missing < 20, "{missing} rows lost their value date");
}

#[tokio::test]
async fn pending_rows_are_recorded_as_pending() {
    let Some(dir) = prototype_dir() else {
        eprintln!("skipping: prototype/bank/data is absent (it is gitignored)");
        return;
    };
    let (_server, pool) = start_with_store().await;
    let owner = ensure_user(&pool, "pdng-owner", None, "Owner")
        .await
        .unwrap();
    tbd_finance::import::from_prototype(&pool, &dir, "personal", owner.0)
        .await
        .unwrap();

    // Erste says PDNG, not "pending"; a mapping that silently defaulted would
    // mark every row pending and nobody would notice until a balance was wrong.
    let (pending,): (i64,) =
        sqlx::query_as("select count(*) from finance.bank_transactions where status = 'pending'")
            .fetch_one(&pool)
            .await
            .unwrap();
    let (booked,): (i64,) =
        sqlx::query_as("select count(*) from finance.bank_transactions where status = 'booked'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(booked > pending * 100, "booked {booked}, pending {pending}");
    assert!(
        pending > 0,
        "the real data has PDNG rows; none were recorded"
    );
}

#[tokio::test]
async fn transactions_land_on_the_account_whose_currency_they_are_in() {
    // The bug this locks out: `POST /sessions` and `GET /sessions/{id}` return
    // the same accounts in different orders. The pull iterated the live order,
    // the importer read the link-time order, and all 255 business EUR rows were
    // filed against the USD account. Nothing errored; the totals were simply
    // attached to the wrong thing.
    let Some(dir) = prototype_dir() else {
        eprintln!("skipping: prototype/bank/data is absent (it is gitignored)");
        return;
    };
    let (_server, pool) = start_with_store().await;

    let owner = ensure_user(&pool, "ccy-owner", None, "Owner")
        .await
        .unwrap();
    let company = create_org(
        &pool,
        "Inorbit d.o.o.",
        Some("38846238650"),
        Some("HR"),
        true,
    )
    .await
    .unwrap();
    tbd_finance::import::from_prototype(&pool, &dir, "business", company.0)
        .await
        .unwrap();
    tbd_finance::import::from_prototype(&pool, &dir, "personal", owner.0)
        .await
        .unwrap();

    let (mismatched,): (i64,) = sqlx::query_as(
        "select count(*)
           from finance.bank_transactions t
           join finance.accounts a on a.id = t.account_id
          where t.currency <> a.currency",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        mismatched, 0,
        "{mismatched} transactions sit on an account of a different currency"
    );

    // And specifically: the company's EUR rows are on its EUR account.
    let (on_eur,): (i64,) = sqlx::query_as(
        "select count(*)
           from finance.bank_transactions t
           join finance.accounts a on a.id = t.account_id
          where a.currency = 'EUR' and a.iban = 'HR9224020061100925189'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(
        on_eur > 200,
        "only {on_eur} rows on the company's EUR account; they went somewhere else"
    );
}
