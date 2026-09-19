//! The shipped rule seed, checked against a real database.
//!
//! The seed joins each rule to a category by slug. A join drops what it cannot
//! match, so a mistyped slug does not fail -- the rule simply never exists, the
//! money it would have claimed stays uncategorised, and nothing anywhere says
//! why. That is the failure this file exists to catch.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use sqlx::PgPool;
use tbd_db::ensure_user;
use uuid::Uuid;

use crate::support::start_with_store;

const SEED: &str = include_str!("../../seeds/rules.sql");

/// The seed as sqlx can run it: psql's `\set` is a client directive, and
/// `:party` is client-side interpolation, neither of which reach the server.
fn runnable(party: Uuid) -> String {
    SEED.lines()
        .filter(|l| !l.trim_start().starts_with("\\set"))
        .collect::<Vec<_>>()
        .join("\n")
        .replace(":party", &format!("'{party}'::uuid"))
}

async fn apply(pool: &PgPool, party: Uuid) {
    let sql = runnable(party);
    for statement in sql.split(";\n") {
        if statement.trim().is_empty() {
            continue;
        }
        let statement = statement.to_owned();
        sqlx::raw_sql(sqlx::AssertSqlSafe(statement.clone()))
            .execute(pool)
            .await
            .unwrap_or_else(|e| panic!("seed statement failed: {e}\n---\n{statement}"));
    }
}

async fn party(pool: &PgPool) -> Uuid {
    ensure_user(pool, "seed-owner", Some("seed@example.test"), "Seed Owner")
        .await
        .unwrap()
        .0
}

#[tokio::test]
async fn every_rule_in_the_seed_finds_its_category() {
    let (_server, pool) = start_with_store().await;
    let pool = &pool;
    let party = party(pool).await;
    apply(pool, party).await;

    // The join is silent, so count what the file asks for and compare. A rule
    // line is `(priority, 'name', 'slug', ...)`; the file's own rule count is
    // the only honest expectation.
    let declared = SEED
        .lines()
        .filter(|l| {
            let t = l.trim_start();
            t.starts_with('(') && t.matches('\'').count() >= 2
        })
        .count();
    let (categories,): (i64,) =
        sqlx::query_as("select count(*) from finance.categories where party_id = $1")
            .bind(party)
            .fetch_one(pool)
            .await
            .unwrap();
    let (rules,): (i64,) = sqlx::query_as("select count(*) from finance.rules where party_id = $1")
        .bind(party)
        .fetch_one(pool)
        .await
        .unwrap();

    let got = usize::try_from(categories + rules).unwrap();
    assert_eq!(
        got, declared,
        "the seed declares {declared} rows but produced {got}; a slug in the rules list \
         names no category and the join dropped it"
    );
}

#[tokio::test]
async fn applying_the_seed_twice_corrects_rather_than_duplicates() {
    let (_server, pool) = start_with_store().await;
    let pool = &pool;
    let party = party(pool).await;
    apply(pool, party).await;
    let (first,): (i64,) = sqlx::query_as("select count(*) from finance.rules where party_id = $1")
        .bind(party)
        .fetch_one(pool)
        .await
        .unwrap();

    // Move a rule out of place, the way an edit would, and re-seed.
    sqlx::query(
        "update finance.rules set priority = 999 where party_id = $1 and name = 'atm_remittance'",
    )
    .bind(party)
    .execute(pool)
    .await
    .unwrap();
    apply(pool, party).await;

    let (second,): (i64,) =
        sqlx::query_as("select count(*) from finance.rules where party_id = $1")
            .bind(party)
            .fetch_one(pool)
            .await
            .unwrap();
    assert_eq!(first, second, "re-seeding added rules instead of updating");

    let (priority,): (i32,) =
        sqlx::query_as("select priority from finance.rules where party_id = $1 and name = $2")
            .bind(party)
            .bind("atm_remittance")
            .fetch_one(pool)
            .await
            .unwrap();
    assert_eq!(
        priority, 25,
        "re-seeding did not restore the rule's priority"
    );
}

#[tokio::test]
async fn cash_is_matched_on_the_remittance_because_the_bank_names_no_counterparty() {
    // The regression this is here for: the first rule set matched `ERSTE ATM`
    // against `counterparty_name`, which is null on every withdrawal. It scored
    // zero against the largest single bucket of spending in the real data.
    let (_server, pool) = start_with_store().await;
    let pool = &pool;
    let party = party(pool).await;
    apply(pool, party).await;

    let (cp, rem): (Option<String>, Option<String>) = sqlx::query_as(
        "select match_counterparty_like, match_remittance_like
           from finance.rules where party_id = $1 and name = 'atm_remittance'",
    )
    .bind(party)
    .fetch_one(pool)
    .await
    .unwrap();
    assert_eq!(cp, None, "the ATM rule must not match on the counterparty");
    assert_eq!(rem.as_deref(), Some("ERSTE ATM"));
}

#[tokio::test]
async fn the_broad_google_rule_stays_disabled() {
    // `GOOGLE` matched Workspace and 313 Play Store charges alike, filing
    // personal purchases as business hosting. The seed must keep it off.
    let (_server, pool) = start_with_store().await;
    let pool = &pool;
    let party = party(pool).await;
    sqlx::query(
        "insert into finance.categories (id, party_id, slug, name, kind)
         values (gen_random_uuid(), $1, 'software', 'Software', 'expense')",
    )
    .bind(party)
    .execute(pool)
    .await
    .unwrap();
    sqlx::query(
        "insert into finance.rules (id, party_id, priority, name, match_counterparty_like, category_id)
         select gen_random_uuid(), $1, 10, 'google', 'GOOGLE', id
           from finance.categories where party_id = $1 and slug = 'software'",
    )
    .bind(party)
    .execute(pool)
    .await
    .unwrap();

    apply(pool, party).await;

    let (enabled,): (bool,) =
        sqlx::query_as("select enabled from finance.rules where party_id = $1 and name = 'google'")
            .bind(party)
            .fetch_one(pool)
            .await
            .unwrap();
    assert!(!enabled, "the broad GOOGLE rule was re-enabled by the seed");
}

#[tokio::test]
async fn a_rule_name_is_unique_within_a_party() {
    let (_server, pool) = start_with_store().await;
    let pool = &pool;
    let party = party(pool).await;
    apply(pool, party).await;

    let clash = sqlx::query(
        "insert into finance.rules (id, party_id, priority, name, match_counterparty_like, category_id)
         select gen_random_uuid(), $1, 10, 'atm', 'X', id
           from finance.categories where party_id = $1 and slug = 'cash'",
    )
    .bind(party)
    .execute(pool)
    .await;
    assert!(
        clash.is_err(),
        "a second rule named `atm` was accepted; the seed can no longer correct in place"
    );
}
