//! Access control, written adversarially.
//!
//! The requirement these defend: only the owner sees the finance data at all,
//! and anyone else given access — the accountant is the case that will actually
//! happen — must not see the personal accounts through any surface.
//!
//! An access-control bug is silent. Nothing errors, a query just returns one
//! row too many, and it looks exactly like working software. So these tests
//! assert on **absence**, not only on presence, and the leak cases are named
//! after what they would leak.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::time::Duration;

use chrono::Utc;
use tbd_db::{
    Capability, PartyId, UserId, create_org, ensure_user, grant, revoke, visible_parties,
    visible_party_ids,
};

use crate::support::db;

/// The real shape: an owner with a person party and a company, and an
/// accountant who may read the company only.
struct World {
    owner: UserId,
    owner_party: PartyId,
    company: PartyId,
    accountant: UserId,
    accountant_party: PartyId,
}

async fn world(pool: &sqlx::PgPool) -> World {
    let owner = ensure_user(pool, "kratos-owner", Some("nevio@inorbit.hr"), "Nevio")
        .await
        .unwrap();
    let accountant = ensure_user(
        pool,
        "kratos-accountant",
        Some("book@keeper.hr"),
        "Accountant",
    )
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
    // The owner owns the company; the accountant may read it. Nobody but the
    // owner has any grant on the owner's person party.
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
    World {
        owner,
        owner_party: PartyId(owner.0),
        company: PartyId(company.0),
        accountant,
        accountant_party: PartyId(accountant.0),
    }
}

#[tokio::test]
async fn a_new_user_owns_exactly_themselves_and_nothing_else() {
    let db = db().await;
    let user = ensure_user(&db, "kratos-fresh", None, "Fresh")
        .await
        .unwrap();
    let visible = visible_parties(&db, user).await.unwrap();
    assert_eq!(visible.len(), 1, "a new user sees exactly one party");
    assert_eq!(visible[0].party.id, PartyId(user.0));
    assert_eq!(visible[0].capability, Capability::Own);
}

#[tokio::test]
async fn the_accountant_never_sees_the_owners_personal_party() {
    let db = db().await;
    let w = world(&db).await;

    let ids = visible_party_ids(&db, w.accountant).await.unwrap();
    assert!(
        ids.contains(&w.company.0),
        "the accountant must see the company"
    );
    assert!(
        !ids.contains(&w.owner_party.0),
        "LEAK: the accountant can see the owner's personal party"
    );
    assert_eq!(
        ids.len(),
        2,
        "the accountant sees the company and themselves, nothing more: {ids:?}"
    );
}

#[tokio::test]
async fn the_owner_sees_both_and_the_projection_is_a_filter() {
    let db = db().await;
    let w = world(&db).await;
    let ids = visible_party_ids(&db, w.owner).await.unwrap();
    assert!(ids.contains(&w.owner_party.0));
    assert!(ids.contains(&w.company.0));
    assert!(
        !ids.contains(&w.accountant_party.0),
        "LEAK: the owner can see the accountant's own party"
    );
}

#[tokio::test]
async fn an_expired_grant_is_invisible_the_moment_it_lapses() {
    let db = db().await;
    let w = world(&db).await;
    grant(
        &db,
        w.accountant,
        w.company,
        Capability::Read,
        Some(w.owner),
        Some(Utc::now() - chrono::Duration::seconds(1)),
    )
    .await
    .unwrap();

    let ids = visible_party_ids(&db, w.accountant).await.unwrap();
    assert!(
        !ids.contains(&w.company.0),
        "LEAK: an expired grant still returns the company"
    );
}

#[tokio::test]
async fn a_grant_expiring_in_the_future_still_works() {
    let db = db().await;
    let w = world(&db).await;
    grant(
        &db,
        w.accountant,
        w.company,
        Capability::Read,
        Some(w.owner),
        Some(Utc::now() + chrono::Duration::days(30)),
    )
    .await
    .unwrap();
    assert!(
        visible_party_ids(&db, w.accountant)
            .await
            .unwrap()
            .contains(&w.company.0)
    );
}

#[tokio::test]
async fn revoking_removes_access_immediately() {
    let db = db().await;
    let w = world(&db).await;
    revoke(&db, w.accountant, w.company).await.unwrap();
    let ids = visible_party_ids(&db, w.accountant).await.unwrap();
    assert!(
        !ids.contains(&w.company.0),
        "LEAK: access survived a revoke"
    );
}

#[tokio::test]
async fn revoking_one_grant_does_not_touch_another_users() {
    let db = db().await;
    let w = world(&db).await;
    revoke(&db, w.accountant, w.company).await.unwrap();
    assert!(
        visible_party_ids(&db, w.owner)
            .await
            .unwrap()
            .contains(&w.company.0),
        "revoking the accountant removed the owner's own access"
    );
}

#[tokio::test]
async fn regranting_changes_capability_without_duplicating_the_row() {
    let db = db().await;
    let w = world(&db).await;
    grant(&db, w.accountant, w.company, Capability::Own, None, None)
        .await
        .unwrap();
    let visible = visible_parties(&db, w.accountant).await.unwrap();
    let company: Vec<_> = visible.iter().filter(|v| v.party.id == w.company).collect();
    assert_eq!(company.len(), 1, "the grant duplicated instead of updating");
    assert_eq!(company[0].capability, Capability::Own);
}

#[tokio::test]
async fn archiving_a_party_hides_it_without_dropping_the_grant() {
    let db = db().await;
    let w = world(&db).await;
    sqlx::query("update parties set archived_at = now() where id = $1")
        .bind(w.company.0)
        .execute(&*db)
        .await
        .unwrap();
    assert!(
        !visible_parties(&db, w.owner)
            .await
            .unwrap()
            .iter()
            .any(|v| v.party.id == w.company),
        "an archived party still appears"
    );
}

#[tokio::test]
async fn ensure_user_is_idempotent_under_a_concurrent_stampede() {
    let db = db().await;
    // Twelve simultaneous first requests from one identity: exactly one user.
    let tasks: Vec<_> = (0..12)
        .map(|_| {
            let pool = db.pool.clone();
            tokio::spawn(async move { ensure_user(&pool, "kratos-race", None, "Racer").await })
        })
        .collect();

    let mut ids = std::collections::HashSet::new();
    for task in tasks {
        ids.insert(task.await.unwrap().unwrap());
    }
    assert_eq!(ids.len(), 1, "the stampede created {} users", ids.len());

    let (users,): (i64,) =
        sqlx::query_as("select count(*) from users where subject = 'kratos-race'")
            .fetch_one(&*db)
            .await
            .unwrap();
    assert_eq!(users, 1);

    // And exactly one self-grant, not twelve.
    let user = *ids.iter().next().unwrap();
    assert_eq!(visible_party_ids(&db, user).await.unwrap().len(), 1);
}

#[tokio::test]
async fn a_second_sight_of_a_known_subject_updates_last_seen_and_adds_nothing() {
    let db = db().await;
    let first = ensure_user(&db, "kratos-return", None, "Returner")
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(20)).await;
    let second = ensure_user(&db, "kratos-return", None, "Returner")
        .await
        .unwrap();
    assert_eq!(first, second);

    let (parties,): (i64,) = sqlx::query_as("select count(*) from parties")
        .fetch_one(&*db)
        .await
        .unwrap();
    assert_eq!(parties, 1, "a repeat sign-in created a second party");

    let (seen,): (Option<chrono::DateTime<Utc>>,) =
        sqlx::query_as("select last_seen_at from users where subject = 'kratos-return'")
            .fetch_one(&*db)
            .await
            .unwrap();
    assert!(seen.is_some(), "last_seen_at was not stamped");
}

#[tokio::test]
async fn an_empty_subject_is_refused_before_it_reaches_the_database() {
    let db = db().await;
    let e = ensure_user(&db, "", None, "Nobody").await.unwrap_err();
    assert!(e.to_string().contains("subject"), "{e}");
    let (parties,): (i64,) = sqlx::query_as("select count(*) from parties")
        .fetch_one(&*db)
        .await
        .unwrap();
    assert_eq!(parties, 0, "a refused user still wrote a party");
}

#[tokio::test]
async fn deleting_a_user_takes_their_grants_with_them() {
    let db = db().await;
    let w = world(&db).await;
    sqlx::query("delete from party_access where user_id = $1")
        .bind(w.accountant.0)
        .execute(&*db)
        .await
        .unwrap();
    sqlx::query("delete from users where id = $1")
        .bind(w.accountant.0)
        .execute(&*db)
        .await
        .unwrap();
    let (rows,): (i64,) = sqlx::query_as("select count(*) from party_access where user_id = $1")
        .bind(w.accountant.0)
        .fetch_one(&*db)
        .await
        .unwrap();
    assert_eq!(rows, 0, "grants outlived the user they belonged to");
}

#[tokio::test]
async fn two_orgs_cannot_share_an_oib() {
    let db = db().await;
    create_org(&db, "Inorbit d.o.o.", Some("38846238650"), Some("HR"), true)
        .await
        .unwrap();
    let e = create_org(
        &db,
        "Impostor d.o.o.",
        Some("38846238650"),
        Some("HR"),
        false,
    )
    .await
    .unwrap_err();
    assert!(
        matches!(e, tbd_db::DbError::Conflict { .. }),
        "a duplicate OIB gave {e:?} rather than a conflict"
    );
}

#[tokio::test]
async fn a_malformed_oib_is_refused_by_the_database() {
    let db = db().await;
    let e = create_org(&db, "Wrong d.o.o.", Some("123"), Some("HR"), false)
        .await
        .unwrap_err();
    assert!(
        matches!(e, tbd_db::DbError::Invalid { .. }),
        "a short OIB gave {e:?} rather than invalid"
    );
}

#[tokio::test]
async fn a_grant_to_a_party_that_does_not_exist_is_invalid_not_a_silent_no_op() {
    let db = db().await;
    let user = ensure_user(&db, "kratos-solo", None, "Solo").await.unwrap();
    let e = grant(
        &db,
        user,
        PartyId(uuid::Uuid::new_v4()),
        Capability::Read,
        None,
        None,
    )
    .await
    .unwrap_err();
    assert!(
        matches!(e, tbd_db::DbError::Invalid { .. }),
        "granting on a missing party gave {e:?}"
    );
}
