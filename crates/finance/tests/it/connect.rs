//! The consent flow, both legs, and the ways it must refuse.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use serde_json::json;
use sqlx::PgPool;
use tbd_db::ensure_user;
use tbd_finance::banking::{
    Mock,
    connect::{ConnectError, complete, start},
    mock::Mode,
};
use uuid::Uuid;

use crate::support::start_with_store;

fn session_with_accounts() -> serde_json::Value {
    json!({
        "session_id": "ignored-by-mock",
        "accounts": [
            {"uid": "u-usd", "currency": "USD", "account_id": {"iban": "HR1"}, "name": "Company"},
            {"uid": "u-eur", "currency": "EUR", "account_id": {"iban": "HR1"}, "name": "Company"}
        ],
        "access": {"valid_until": "2027-03-15T10:41:56Z"}
    })
}

async fn owner(pool: &PgPool, subject: &str) -> Uuid {
    ensure_user(pool, subject, None, "O").await.unwrap().0
}

#[tokio::test]
async fn a_consent_goes_from_pending_to_authorized_and_creates_the_accounts() {
    let (_s, pool) = start_with_store().await;
    let party = owner(&pool, "c-owner").await;
    let bank = Mock::new().with_session(&session_with_accounts());

    let started = start(
        &pool,
        &*bank,
        party,
        "business",
        "Mock Bank",
        "HR",
        "https://x/cb",
    )
    .await
    .unwrap();
    assert!(
        started.url.contains(&started.state),
        "the bank sees our state"
    );
    let (status,): (String,) =
        sqlx::query_as("select status from finance.connections where id = $1")
            .bind(started.connection_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(status, "pending");

    let done = complete(&pool, &*bank, party, &started.state, "the-code")
        .await
        .unwrap();
    assert_eq!(done.connection_id, started.connection_id);
    assert_eq!(done.accounts.len(), 2, "one per currency, same IBAN");

    let (status, session): (String, Option<String>) =
        sqlx::query_as("select status, session_id from finance.connections where id = $1")
            .bind(started.connection_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(status, "authorized");
    assert_eq!(session.as_deref(), Some("mock-session-the-code"));

    let rows: Vec<(String, String, Option<Uuid>)> = sqlx::query_as(
        "select provider_uid, currency, connection_id from finance.accounts
          where party_id = $1 order by currency",
    )
    .bind(party)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        rows[0],
        ("u-eur".into(), "EUR".into(), Some(started.connection_id))
    );
    assert_eq!(
        rows[1],
        ("u-usd".into(), "USD".into(), Some(started.connection_id))
    );
}

#[tokio::test]
async fn a_state_belonging_to_another_party_is_not_found_not_forbidden() {
    let (_s, pool) = start_with_store().await;
    let alice = owner(&pool, "c-alice").await;
    let mallory = owner(&pool, "c-mallory").await;
    let bank = Mock::new().with_session(&session_with_accounts());
    let started = start(
        &pool,
        &*bank,
        alice,
        "personal",
        "Mock Bank",
        "HR",
        "https://x/cb",
    )
    .await
    .unwrap();

    let e = complete(&pool, &*bank, mallory, &started.state, "stolen-code")
        .await
        .unwrap_err();
    assert!(matches!(e, ConnectError::NotFound), "{e}");
    // The code was never sent to the bank on Mallory's behalf.
    assert_eq!(bank.seen().iter().filter(|p| *p == "/sessions").count(), 0);
    // And Alice's connection is untouched.
    let (status,): (String,) =
        sqlx::query_as("select status from finance.connections where id = $1")
            .bind(started.connection_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(status, "pending");
}

#[tokio::test]
async fn a_replayed_callback_is_told_it_was_already_completed_and_exchanges_nothing() {
    let (_s, pool) = start_with_store().await;
    let party = owner(&pool, "c-replay").await;
    let bank = Mock::new().with_session(&session_with_accounts());
    let started = start(
        &pool,
        &*bank,
        party,
        "business",
        "Mock Bank",
        "HR",
        "https://x/cb",
    )
    .await
    .unwrap();
    complete(&pool, &*bank, party, &started.state, "code-1")
        .await
        .unwrap();
    let exchanges = bank.seen().iter().filter(|p| *p == "/sessions").count();

    let e = complete(&pool, &*bank, party, &started.state, "code-1")
        .await
        .unwrap_err();
    assert!(matches!(e, ConnectError::AlreadyCompleted), "{e}");
    assert_eq!(
        bank.seen().iter().filter(|p| *p == "/sessions").count(),
        exchanges,
        "no second exchange"
    );
}

#[tokio::test]
async fn an_unknown_state_is_not_found() {
    let (_s, pool) = start_with_store().await;
    let party = owner(&pool, "c-unknown").await;
    let bank = Mock::new();
    let e = complete(&pool, &*bank, party, "never-issued", "code")
        .await
        .unwrap_err();
    assert!(matches!(e, ConnectError::NotFound));
    assert_eq!(bank.calls(), 0);
}

#[tokio::test]
async fn a_refused_exchange_marks_the_connection_failed_and_does_not_leak_the_code() {
    let (_s, pool) = start_with_store().await;
    let party = owner(&pool, "c-refused").await;
    let bank = Mock::new().with_session(&session_with_accounts());
    let started = start(
        &pool,
        &*bank,
        party,
        "business",
        "Mock Bank",
        "HR",
        "https://x/cb",
    )
    .await
    .unwrap();
    bank.set_mode(Mode::Unauthorized);

    let e = complete(&pool, &*bank, party, &started.state, "secret-code-value")
        .await
        .unwrap_err();
    assert!(matches!(e, ConnectError::Provider(_)), "{e}");
    let (status, failure): (String, Option<String>) =
        sqlx::query_as("select status, failure from finance.connections where id = $1")
            .bind(started.connection_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(status, "failed");
    assert!(!failure.unwrap().contains("secret-code-value"));
    // A failed state cannot be tried again either.
    let e = complete(&pool, &*bank, party, &started.state, "secret-code-value")
        .await
        .unwrap_err();
    assert!(matches!(e, ConnectError::NotFound));
}

#[tokio::test]
async fn relinking_after_expiry_reuses_the_accounts_and_their_history() {
    let (_s, pool) = start_with_store().await;
    let party = owner(&pool, "c-relink").await;
    let bank = Mock::new().with_session(&session_with_accounts());
    let first = start(
        &pool,
        &*bank,
        party,
        "business",
        "Mock Bank",
        "HR",
        "https://x/cb",
    )
    .await
    .unwrap();
    let done = complete(&pool, &*bank, party, &first.state, "c1")
        .await
        .unwrap();
    sqlx::query("update finance.connections set status = 'expired' where id = $1")
        .bind(first.connection_id)
        .execute(&pool)
        .await
        .unwrap();

    let second = start(
        &pool,
        &*bank,
        party,
        "business",
        "Mock Bank",
        "HR",
        "https://x/cb",
    )
    .await
    .unwrap();
    let again = complete(&pool, &*bank, party, &second.state, "c2")
        .await
        .unwrap();
    assert_eq!(
        again.accounts, done.accounts,
        "same rows, now on the new connection"
    );
    let (n,): (i64,) = sqlx::query_as("select count(*) from finance.accounts where party_id = $1")
        .bind(party)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(n, 2);
}
