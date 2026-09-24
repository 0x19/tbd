//! The reusable [`Access`] primitive, tested as the thing every future service
//! will rely on rather than as a finance detail.
//!
//! The property that matters: a caller-supplied filter can narrow the visible
//! set and can never widen it. If that ever regresses, a UI toggle becomes a
//! privilege escalation.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use tbd_db::{Access, Capability, PartyId, create_org, ensure_user, grant};

use crate::support::db;

#[tokio::test]
async fn resolve_provisions_on_first_sight_and_sees_only_itself() {
    let db = db().await;
    let access = Access::resolve(&db, "kratos-a", Some("a@example.com"), "A")
        .await
        .unwrap();
    assert_eq!(access.party_ids().len(), 1);
    assert!(access.allows(PartyId(access.user().0)));
    assert!(!access.is_empty());
}

#[tokio::test]
async fn narrow_cannot_widen_the_set() {
    let db = db().await;
    let owner = ensure_user(&db, "kratos-owner", None, "Owner")
        .await
        .unwrap();
    let other = ensure_user(&db, "kratos-other", None, "Other")
        .await
        .unwrap();
    let company = create_org(&db, "Inorbit d.o.o.", Some("38846238650"), Some("HR"), true)
        .await
        .unwrap();
    grant(&db, owner, PartyId(company.0), Capability::Own, None, None)
        .await
        .unwrap();

    let access = Access::for_user(&db, other).await.unwrap();
    assert_eq!(access.party_ids().len(), 1, "other owns only themselves");

    // The attack: ask for parties you were never granted.
    let widened = access.narrow(&[company.0, owner.0, other.0]);
    assert_eq!(
        widened.party_ids(),
        &[other.0],
        "narrow widened the visible set — this is a privilege escalation"
    );
    assert!(!widened.allows(PartyId(company.0)));
    assert!(!widened.allows(PartyId(owner.0)));
}

#[tokio::test]
async fn narrow_to_nothing_reads_as_empty_not_as_everything() {
    let db = db().await;
    let user = ensure_user(&db, "kratos-empty", None, "Empty")
        .await
        .unwrap();
    let access = Access::for_user(&db, user).await.unwrap();
    let none = access.narrow(&[uuid::Uuid::new_v4()]);
    assert!(
        none.is_empty(),
        "an unmatched filter must yield nothing, never an unfiltered set"
    );
    assert!(none.party_ids().is_empty());
}

#[tokio::test]
async fn require_answers_not_found_rather_than_forbidden() {
    let db = db().await;
    let user = ensure_user(&db, "kratos-req", None, "Req").await.unwrap();
    let company = create_org(&db, "Someone Else d.o.o.", None, Some("HR"), false)
        .await
        .unwrap();
    let access = Access::for_user(&db, user).await.unwrap();

    let e = access.require(PartyId(company.0), "account").unwrap_err();
    assert!(
        matches!(e, tbd_db::DbError::NotFound { .. }),
        "an ungranted party gave {e:?}; a forbidden would confirm it exists"
    );
    assert!(
        !e.to_string().to_lowercase().contains("forbidden"),
        "the message must not hint that the party exists: {e}"
    );

    access.require(PartyId(user.0), "account").unwrap();
}

#[tokio::test]
async fn a_revoked_grant_is_gone_on_the_next_resolve() {
    let db = db().await;
    let owner = ensure_user(&db, "kratos-rev", None, "Owner").await.unwrap();
    let company = create_org(&db, "Inorbit d.o.o.", Some("38846238650"), Some("HR"), true)
        .await
        .unwrap();
    grant(&db, owner, PartyId(company.0), Capability::Own, None, None)
        .await
        .unwrap();
    assert!(
        Access::for_user(&db, owner)
            .await
            .unwrap()
            .allows(PartyId(company.0))
    );

    tbd_db::revoke(&db, owner, PartyId(company.0))
        .await
        .unwrap();

    // An Access already in flight is a snapshot; the next one must not be.
    assert!(
        !Access::for_user(&db, owner)
            .await
            .unwrap()
            .allows(PartyId(company.0)),
        "a revoked grant survived into a freshly resolved Access"
    );
}

#[tokio::test]
async fn reading_your_own_party_is_not_logged_but_a_delegated_read_is() {
    let db = db().await;
    let owner = ensure_user(&db, "kratos-log-owner", None, "Owner")
        .await
        .unwrap();
    let accountant = ensure_user(&db, "kratos-log-acct", None, "Accountant")
        .await
        .unwrap();
    let company = create_org(&db, "Inorbit d.o.o.", Some("38846238650"), Some("HR"), true)
        .await
        .unwrap();
    grant(
        &db,
        accountant,
        PartyId(company.0),
        Capability::Read,
        Some(owner),
        None,
    )
    .await
    .unwrap();

    let access = Access::for_user(&db, accountant).await.unwrap();
    access
        .log_read(
            &db,
            PartyId(accountant.0),
            "ListTransactions",
            Some(3),
            "t-1",
        )
        .await
        .unwrap();
    let (own,): (i64,) = sqlx::query_as("select count(*) from access_log")
        .fetch_one(&*db)
        .await
        .unwrap();
    assert_eq!(
        own, 0,
        "reading your own data should not fill the audit log"
    );

    access
        .log_read(&db, PartyId(company.0), "ListTransactions", Some(42), "t-2")
        .await
        .unwrap();
    let (route, rows, trace): (String, Option<i32>, String) =
        sqlx::query_as("select route, rows_seen, trace_id from access_log")
            .fetch_one(&*db)
            .await
            .unwrap();
    assert_eq!(route, "ListTransactions");
    assert_eq!(rows, Some(42));
    assert_eq!(trace, "t-2");
}

#[tokio::test]
async fn bind_rls_user_is_transaction_scoped_and_does_not_leak_to_the_next_borrower() {
    let db = db().await;
    let user = ensure_user(&db, "kratos-rls", None, "Rls").await.unwrap();

    let mut tx = db.begin().await.unwrap();
    tbd_db::bind_rls_user(&mut tx, user).await.unwrap();
    let (inside,): (String,) = sqlx::query_as("select current_setting('app.user_id', true)")
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    assert_eq!(inside, user.0.to_string());
    tx.commit().await.unwrap();

    // A later transaction on a pooled connection must not inherit it. `set
    // local` is what guarantees this; a plain `set` would not.
    let (after,): (Option<String>,) = sqlx::query_as("select current_setting('app.user_id', true)")
        .fetch_one(&*db)
        .await
        .unwrap();
    assert!(
        after.is_none() || after.as_deref() == Some(""),
        "app.user_id leaked out of its transaction as {after:?}"
    );
}
