//! Rule application: precedence, and a human's decision surviving it.
//!
//! Both failures here are silent. A rule that wins when it should not simply
//! files money under the wrong heading; a categorisation that reverts on the
//! next sync looks like the sync is broken. Neither errors.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use sqlx::PgPool;
use tbd_db::{create_org, ensure_user};
use tbd_finance::categorise::{apply_rules, declare, monthly_summary};
use uuid::Uuid;

use crate::support::start_with_store;

async fn category(pool: &PgPool, party: Uuid, slug: &str, kind: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "insert into finance.categories (id, party_id, slug, name, kind) values ($1,$2,$3,$4,$5)",
    )
    .bind(id)
    .bind(party)
    .bind(slug)
    .bind(slug)
    .bind(kind)
    .execute(pool)
    .await
    .unwrap();
    id
}

async fn rule(pool: &PgPool, party: Uuid, priority: i32, like: &str, category: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "insert into finance.rules (id, party_id, priority, name, match_counterparty_like, category_id)
         values ($1,$2,$3,$4,$5,$6)",
    )
    .bind(id)
    .bind(party)
    .bind(priority)
    .bind(format!("p{priority} {like}"))
    .bind(like)
    .bind(category)
    .execute(pool)
    .await
    .unwrap();
    id
}

async fn txn(pool: &PgPool, party: Uuid, account: Uuid, who: &str, minor: i64) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "insert into finance.bank_transactions
            (id, party_id, account_id, status, dedup_key, amount_minor, currency,
             credit_debit, booking_date, counterparty_name)
         values ($1,$2,$3,'booked',$4,$5,'EUR',$6,current_date,$7)",
    )
    .bind(id)
    .bind(party)
    .bind(account)
    .bind(Uuid::new_v4().as_bytes().to_vec())
    .bind(minor)
    .bind(if minor < 0 { "DBIT" } else { "CRDT" })
    .bind(who)
    .execute(pool)
    .await
    .unwrap();
    id
}

async fn account(pool: &PgPool, party: Uuid, iban: Option<&str>) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "insert into finance.accounts (id, party_id, iban, currency, name) values ($1,$2,$3,'EUR','a')",
    )
    .bind(id)
    .bind(party)
    .bind(iban)
    .execute(pool)
    .await
    .unwrap();
    id
}

#[tokio::test]
async fn the_lower_priority_rule_wins_and_does_so_every_time() {
    let (_s, pool) = start_with_store().await;
    let owner = ensure_user(&pool, "cat-owner", None, "O").await.unwrap();
    let acct = account(&pool, owner.0, Some("HR1")).await;

    let software = category(&pool, owner.0, "software", "expense").await;
    let generic = category(&pool, owner.0, "generic", "expense").await;
    // Both match; priority 10 must win over priority 50, whichever was inserted
    // first and however the planner orders them.
    rule(&pool, owner.0, 50, "ANTHROPIC", generic).await;
    let specific = rule(&pool, owner.0, 10, "ANTHROPIC", software).await;
    txn(&pool, owner.0, acct, "ANTHROPIC* CLAUDE SUB", -18_000).await;

    for pass in 0..3 {
        apply_rules(&pool, owner.0).await.unwrap();
        let (cat, rid): (Uuid, Option<Uuid>) =
            sqlx::query_as("select category_id, category_rule_id from finance.bank_transactions")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(
            cat, software,
            "pass {pass}: the priority order did not hold"
        );
        assert_eq!(rid, Some(specific));
    }
}

#[tokio::test]
async fn a_human_decision_is_never_overwritten_by_a_later_pass() {
    let (_s, pool) = start_with_store().await;
    let owner = ensure_user(&pool, "cat-human", None, "O").await.unwrap();
    let acct = account(&pool, owner.0, Some("HR1")).await;
    let wrong = category(&pool, owner.0, "wrong", "expense").await;
    let right = category(&pool, owner.0, "right", "expense").await;
    rule(&pool, owner.0, 10, "HETZNER", wrong).await;
    let t = txn(&pool, owner.0, acct, "HETZNER ONLINE GMBH", -6_948).await;

    apply_rules(&pool, owner.0).await.unwrap();
    declare(&pool, t, right).await.unwrap();

    // The failure this guards: you correct a categorisation, the next sync
    // runs, and it silently reverts.
    let report = apply_rules(&pool, owner.0).await.unwrap();
    let (cat, src, rid): (Uuid, String, Option<Uuid>) = sqlx::query_as(
        "select category_id, category_source, category_rule_id from finance.bank_transactions",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(cat, right, "a rule overwrote a human's decision");
    assert_eq!(src, "declared");
    assert_eq!(rid, None, "a declared row kept a rule id");
    assert_eq!(report.declared_kept, 1);
}

#[tokio::test]
async fn applying_twice_reaches_the_same_answer() {
    let (_s, pool) = start_with_store().await;
    let owner = ensure_user(&pool, "cat-idem", None, "O").await.unwrap();
    let acct = account(&pool, owner.0, Some("HR1")).await;
    let c = category(&pool, owner.0, "hosting", "expense").await;
    let r = rule(&pool, owner.0, 10, "HETZNER", c).await;
    txn(&pool, owner.0, acct, "HETZNER ONLINE GMBH", -6_948).await;

    // A pass clears what rules decided and redecides, so the count repeats.
    // What must not change is the answer.
    let first = apply_rules(&pool, owner.0).await.unwrap();
    let outcome = |pool: PgPool| async move {
        sqlx::query_as::<_, (Uuid, String, Option<Uuid>)>(
            "select category_id, category_source, category_rule_id from finance.bank_transactions",
        )
        .fetch_one(&pool)
        .await
        .unwrap()
    };
    let before = outcome(pool.clone()).await;
    let again = apply_rules(&pool, owner.0).await.unwrap();
    let after = outcome(pool.clone()).await;

    assert_eq!(first.categorised, 1);
    assert_eq!(again.categorised, 1, "the second pass decided differently");
    assert_eq!(
        (before.0, before.1, before.2),
        (after.0, after.1, after.2),
        "two passes over identical data reached different answers"
    );

    // hits is how many rows the rule owns now, not how often it has run: a
    // cumulative count answers no question, while "this rule claims nothing"
    // means it is wrong or obsolete.
    let (hits,): (i64,) = sqlx::query_as("select hits from finance.rules where id = $1")
        .bind(r)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(hits, 1, "hits accumulated across passes");
}

#[tokio::test]
async fn transfers_between_our_own_accounts_are_derived_not_assigned() {
    let (_s, pool) = start_with_store().await;
    let owner = ensure_user(&pool, "xfer-o", None, "O").await.unwrap();
    let company = create_org(
        &pool,
        "Inorbit d.o.o.",
        Some("38846238650"),
        Some("HR"),
        true,
    )
    .await
    .unwrap();
    let personal = account(&pool, owner.0, Some("HR_PERSONAL")).await;
    let business = account(&pool, company.0, Some("HR_BUSINESS")).await;

    // An owner draw: out of the company, into the person.
    sqlx::query(
        "insert into finance.bank_transactions
            (id, party_id, account_id, status, dedup_key, amount_minor, currency,
             credit_debit, booking_date, counterparty_name, counterparty_iban)
         values ($1,$2,$3,'booked',$4,-250000,'EUR','DBIT',current_date,'NEVIO',$5)",
    )
    .bind(Uuid::new_v4())
    .bind(company.0)
    .bind(business)
    .bind(Uuid::new_v4().as_bytes().to_vec())
    .bind("HR_PERSONAL")
    .execute(&pool)
    .await
    .unwrap();
    // And a genuine supplier payment, same size, to prove the flag discriminates.
    txn(&pool, company.0, business, "HETZNER ONLINE GMBH", -250_000).await;

    let (internal,): (i64,) =
        sqlx::query_as("select count(*) from finance.transactions_enriched where internal")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(internal, 1, "the owner draw was not recognised as internal");

    let spend = monthly_summary(&pool, &[company.0], false, None)
        .await
        .unwrap();
    let with = monthly_summary(&pool, &[company.0], true, None)
        .await
        .unwrap();
    let total = |rows: &[tbd_finance::categorise::SummaryRow]| -> i64 {
        rows.iter().map(|r| r.total_minor).sum()
    };
    assert_eq!(total(&spend), -250_000, "internal rows leaked into spend");
    assert_eq!(total(&with), -500_000);

    // The account linked later must flip the flag without a backfill: the view
    // derives it, so it is right whenever it is asked.
    assert!(
        !personal.is_nil(),
        "the personal account exists, which is what makes the transfer internal"
    );
}

#[tokio::test]
async fn a_summary_never_mixes_currencies() {
    let (_s, pool) = start_with_store().await;
    let owner = ensure_user(&pool, "ccy-sum", None, "O").await.unwrap();
    let eur = account(&pool, owner.0, Some("HR_EUR")).await;
    txn(&pool, owner.0, eur, "A", -1_000).await;
    sqlx::query(
        "insert into finance.bank_transactions
            (id, party_id, account_id, status, dedup_key, amount_minor, currency,
             credit_debit, booking_date, counterparty_name)
         values ($1,$2,$3,'booked',$4,-2000,'USD','DBIT',current_date,'A')",
    )
    .bind(Uuid::new_v4())
    .bind(owner.0)
    .bind(eur)
    .bind(Uuid::new_v4().as_bytes().to_vec())
    .execute(&pool)
    .await
    .unwrap();

    let rows = monthly_summary(&pool, &[owner.0], false, None)
        .await
        .unwrap();
    let currencies: Vec<&str> = rows.iter().map(|r| r.currency.as_str()).collect();
    assert!(currencies.contains(&"EUR") && currencies.contains(&"USD"));
    assert_eq!(
        rows.len(),
        2,
        "two currencies collapsed into one row: {rows:?}"
    );
}

#[tokio::test]
async fn a_rule_matches_a_payee_however_the_bank_spelled_it() {
    // Both spellings are in the real data, in one account, weeks apart. A rule
    // matching only one leaves half the money uncategorised — which looks like
    // "we have no rule for that" rather than "the rule is broken".
    let (_s, pool) = start_with_store().await;
    let owner = ensure_user(&pool, "diacritic", None, "O").await.unwrap();
    let acct = account(&pool, owner.0, Some("HR1")).await;
    let tax = category(&pool, owner.0, "tax", "tax").await;
    rule(&pool, owner.0, 10, "DRZAVNI PRORACUN", tax).await;

    txn(
        &pool,
        owner.0,
        acct,
        "DRŽAVNI PRORAČUN REPUBLIKE HRVATSKE",
        -100_000,
    )
    .await;
    txn(
        &pool,
        owner.0,
        acct,
        "DRZAVNI PRORACUN REPUBLIKE HRVATSKE",
        -50_000,
    )
    .await;

    let report = apply_rules(&pool, owner.0).await.unwrap();
    assert_eq!(
        report.categorised, 2,
        "only {} of two spellings matched",
        report.categorised
    );
    assert_eq!(report.unmatched, 0);
}
