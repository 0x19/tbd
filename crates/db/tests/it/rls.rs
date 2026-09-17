//! Row-level security on the finance tables: the second layer.
//!
//! The service filters every query on the caller's granted parties. These tests
//! delete that filter on purpose and check the database refuses anyway. If they
//! ever pass for the wrong reason, defence-in-depth is decorative and a single
//! forgotten `where` clause in a handler becomes a leak.
//!
//! Every query here runs under `set role tbd_finance`, which is how the service
//! actually connects. Postgres exempts a table's owner from its policies, and
//! the test connects as the owner — so a test that skipped `set role` would see
//! everything and pass while proving nothing.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use sqlx::{Executor, PgPool, Postgres, Transaction};
use tbd_db::{Capability, PartyId, UserId, create_org, ensure_user, grant};
use uuid::Uuid;

use crate::support::db;

struct World {
    owner: UserId,
    accountant: UserId,
    personal: PartyId,
    company: PartyId,
}

async fn world(pool: &PgPool) -> World {
    let owner = ensure_user(pool, "rls-owner", None, "Owner").await.unwrap();
    let accountant = ensure_user(pool, "rls-accountant", None, "Accountant")
        .await
        .unwrap();
    let company = create_org(
        pool,
        "Inorbit d.o.o.",
        Some("38846238650"),
        Some("HR"),
        true,
    )
    .await
    .unwrap();
    grant(
        pool,
        owner,
        PartyId(company.0),
        Capability::Own,
        Some(owner),
        None,
    )
    .await
    .unwrap();
    grant(
        pool,
        accountant,
        PartyId(company.0),
        Capability::Read,
        Some(owner),
        None,
    )
    .await
    .unwrap();

    // One personal account and one company account, each with a transaction.
    for (party, iban, who) in [
        (PartyId(owner.0), "HR3924020063202676456", "personal"),
        (PartyId(company.0), "HR9224020061100925189", "company"),
    ] {
        let account = Uuid::new_v4();
        sqlx::query(
            "insert into finance.accounts (id, party_id, iban, currency, name)
             values ($1, $2, $3, 'EUR', $4)",
        )
        .bind(account)
        .bind(party.0)
        .bind(iban)
        .bind(who)
        .execute(pool)
        .await
        .unwrap();
        sqlx::query(
            "insert into finance.bank_transactions
                (id, party_id, account_id, status, entry_reference, dedup_key,
                 amount_minor, currency, credit_debit, booking_date, remittance)
             values ($1, $2, $3, 'booked', $4, $5, 1000, 'EUR', 'DBIT', current_date, $6)",
        )
        .bind(Uuid::new_v4())
        .bind(party.0)
        .bind(account)
        .bind(format!("ref-{who}"))
        .bind(format!("dedup-{who}").into_bytes())
        .bind(format!("a {who} payment"))
        .execute(pool)
        .await
        .unwrap();
    }

    World {
        owner,
        accountant,
        personal: PartyId(owner.0),
        company: PartyId(company.0),
    }
}

/// Query as the service role, with the caller bound, and **no filter at all**.
async fn unfiltered_as(pool: &PgPool, user: Option<UserId>) -> Vec<(Uuid, String)> {
    let mut tx: Transaction<'_, Postgres> = pool.begin().await.unwrap();
    tx.execute("set local role tbd_finance").await.unwrap();
    if let Some(user) = user {
        tbd_db::bind_rls_user(&mut tx, user).await.unwrap();
    }
    let rows: Vec<(Uuid, String)> =
        sqlx::query_as("select party_id, remittance from finance.bank_transactions")
            .fetch_all(&mut *tx)
            .await
            .unwrap();
    tx.commit().await.unwrap();
    rows
}

#[tokio::test]
async fn an_unfiltered_query_still_only_returns_the_callers_rows() {
    let db = db().await;
    let w = world(&db).await;

    let owner_rows = unfiltered_as(&db, Some(w.owner)).await;
    let parties: Vec<Uuid> = owner_rows.iter().map(|(p, _)| *p).collect();
    assert!(
        parties.contains(&w.personal.0),
        "the owner lost their own row"
    );
    assert!(
        parties.contains(&w.company.0),
        "the owner lost the company row"
    );
    assert_eq!(owner_rows.len(), 2);
}

#[tokio::test]
async fn the_accountant_cannot_reach_the_personal_row_even_without_a_filter() {
    let db = db().await;
    let w = world(&db).await;

    let rows = unfiltered_as(&db, Some(w.accountant)).await;
    let parties: Vec<Uuid> = rows.iter().map(|(p, _)| *p).collect();
    assert!(
        parties.contains(&w.company.0),
        "the accountant must still see the company"
    );
    assert!(
        !parties.contains(&w.personal.0),
        "LEAK: RLS let the accountant read the owner's personal transaction \
         through a query with no filter — the second layer is not real"
    );
    assert_eq!(
        rows.len(),
        1,
        "expected exactly the company row, got {rows:?}"
    );
}

#[tokio::test]
async fn a_revoked_grant_stops_the_rows_at_the_database() {
    let db = db().await;
    let w = world(&db).await;
    assert_eq!(unfiltered_as(&db, Some(w.accountant)).await.len(), 1);

    tbd_db::revoke(&db, w.accountant, w.company).await.unwrap();

    assert!(
        unfiltered_as(&db, Some(w.accountant)).await.is_empty(),
        "LEAK: rows survived a revoke at the database layer"
    );
}

#[tokio::test]
async fn an_expired_grant_stops_the_rows_at_the_database() {
    let db = db().await;
    let w = world(&db).await;
    grant(
        &db,
        w.accountant,
        w.company,
        Capability::Read,
        Some(w.owner),
        Some(chrono::Utc::now() - chrono::Duration::seconds(1)),
    )
    .await
    .unwrap();
    assert!(
        unfiltered_as(&db, Some(w.accountant)).await.is_empty(),
        "LEAK: an expired grant still returned rows"
    );
}

#[tokio::test]
async fn a_forged_user_id_reaches_nothing() {
    let db = db().await;
    world(&db).await;

    // Binding an id that holds no grants must yield nothing, rather than
    // failing open. A caller who could influence `app.user_id` gains nothing by
    // inventing one.
    let mut tx = db.begin().await.unwrap();
    tx.execute("set local role tbd_finance").await.unwrap();
    sqlx::query("select set_config('app.user_id', $1::text, true)")
        .bind(Uuid::new_v4())
        .execute(&mut *tx)
        .await
        .unwrap();
    let rows: Vec<(Uuid,)> = sqlx::query_as("select party_id from finance.bank_transactions")
        .fetch_all(&mut *tx)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    assert!(
        rows.is_empty(),
        "LEAK: an unknown user id saw {} rows",
        rows.len()
    );
}

#[tokio::test]
async fn accounts_are_filtered_the_same_way_as_transactions() {
    let db = db().await;
    let w = world(&db).await;

    let mut tx = db.begin().await.unwrap();
    tx.execute("set local role tbd_finance").await.unwrap();
    tbd_db::bind_rls_user(&mut tx, w.accountant).await.unwrap();
    let rows: Vec<(Uuid, String)> = sqlx::query_as("select party_id, name from finance.accounts")
        .fetch_all(&mut *tx)
        .await
        .unwrap();
    tx.commit().await.unwrap();

    assert_eq!(rows.len(), 1, "expected only the company account: {rows:?}");
    assert_eq!(rows[0].0, w.company.0);
    assert_eq!(rows[0].1, "company");
}

#[tokio::test]
async fn a_transaction_cannot_claim_a_party_its_account_does_not_belong_to() {
    let db = db().await;
    let w = world(&db).await;

    // The company's account, but labelled with the owner's personal party.
    // Were `party_id` merely a copy kept in sync by convention, this would
    // succeed and the row would be invisible to the accountant while sitting in
    // a company account -- or worse, the mirror image.
    let (company_account,): (Uuid,) =
        sqlx::query_as("select id from finance.accounts where party_id = $1")
            .bind(w.company.0)
            .fetch_one(&*db)
            .await
            .unwrap();

    let e = sqlx::query(
        "insert into finance.bank_transactions
            (id, party_id, account_id, status, dedup_key, amount_minor, currency, credit_debit)
         values ($1, $2, $3, 'booked', $4, 1, 'EUR', 'DBIT')",
    )
    .bind(Uuid::new_v4())
    .bind(w.personal.0)
    .bind(company_account)
    .bind(b"mismatched".to_vec())
    .execute(&*db)
    .await
    .unwrap_err();

    assert!(
        matches!(tbd_db::map_err(e), tbd_db::DbError::Invalid { .. }),
        "the database accepted a transaction whose party disagrees with its account"
    );
}

#[tokio::test]
async fn moving_an_account_to_another_party_moves_its_transactions_with_it() {
    let db = db().await;
    let w = world(&db).await;

    let (personal_account,): (Uuid,) =
        sqlx::query_as("select id from finance.accounts where party_id = $1")
            .bind(w.personal.0)
            .fetch_one(&*db)
            .await
            .unwrap();

    // A correction: the account was really the company's all along.
    sqlx::query("update finance.accounts set party_id = $1 where id = $2")
        .bind(w.company.0)
        .bind(personal_account)
        .execute(&*db)
        .await
        .unwrap();

    let rows: Vec<(Uuid,)> =
        sqlx::query_as("select party_id from finance.bank_transactions where account_id = $1")
            .bind(personal_account)
            .fetch_all(&*db)
            .await
            .unwrap();
    assert!(!rows.is_empty());
    for (party,) in rows {
        assert_eq!(
            party, w.company.0,
            "a transaction kept its old party after the account moved: the copy went stale"
        );
    }

    // And the accountant, who may read the company, now sees it.
    assert_eq!(
        unfiltered_as(&db, Some(w.accountant)).await.len(),
        2,
        "the moved transaction did not become visible to the company's reader"
    );
}
