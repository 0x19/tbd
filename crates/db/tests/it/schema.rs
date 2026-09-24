//! The migration set itself: it applies, it is idempotent, and a binary refuses
//! to run against a database that is behind it.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use tbd_db::{applied_migrations, expected_migrations, migrate, require_current_schema};

use crate::support::db;

#[tokio::test]
async fn a_migrated_database_reports_every_migration() {
    let db = db().await;
    assert_eq!(
        applied_migrations(&db).await.unwrap(),
        expected_migrations(),
        "the database and the binary disagree on how many migrations exist"
    );
    require_current_schema(&db).await.unwrap();
}

#[tokio::test]
async fn migrating_twice_is_a_no_op() {
    let db = db().await;
    let before = applied_migrations(&db).await.unwrap();
    migrate(&db).await.unwrap();
    assert_eq!(applied_migrations(&db).await.unwrap(), before);
}

#[tokio::test]
async fn an_unmigrated_database_is_zero_rather_than_an_error() {
    // A fresh database has no tracking table. That has to read as "nothing
    // applied", so the operator is told to run `tbd migrate` instead of being
    // shown a driver error about a missing relation.
    let db = db().await;
    sqlx::query("drop table _sqlx_migrations")
        .execute(&*db)
        .await
        .unwrap();
    assert_eq!(applied_migrations(&db).await.unwrap(), 0);

    let e = require_current_schema(&db).await.unwrap_err();
    assert!(
        e.to_string().contains("tbd migrate"),
        "the refusal must name the fix: {e}"
    );
}

#[tokio::test]
async fn a_database_behind_the_binary_is_refused_with_both_numbers() {
    let db = db().await;
    sqlx::query(
        "delete from _sqlx_migrations where version = (select max(version) from _sqlx_migrations)",
    )
    .execute(&*db)
    .await
    .unwrap();
    let e = require_current_schema(&db).await.unwrap_err();
    let message = e.to_string();
    assert!(message.contains("this binary expects"), "{message}");
    assert!(
        message.contains(&expected_migrations().to_string()),
        "the refusal must say what was expected: {message}"
    );
}

#[tokio::test]
async fn identity_is_in_the_public_schema_where_other_services_can_reference_it() {
    let db = db().await;
    for table in [
        "parties",
        "users",
        "orgs",
        "memberships",
        "party_access",
        "access_log",
    ] {
        let (exists,): (bool,) = sqlx::query_as("select to_regclass('public.' || $1) is not null")
            .bind(table)
            .fetch_one(&*db)
            .await
            .unwrap();
        assert!(exists, "public.{table} is missing");
    }
}

#[tokio::test]
async fn a_user_cannot_exist_without_the_party_it_is() {
    let db = db().await;
    let e = sqlx::query("insert into users (id, subject) values ($1, 'orphan')")
        .bind(uuid::Uuid::new_v4())
        .execute(&*db)
        .await
        .unwrap_err();
    assert!(
        tbd_db::map_err(e).to_string().contains("invalid"),
        "a user without a party was accepted"
    );
}

#[tokio::test]
async fn two_users_cannot_share_a_subject() {
    let db = db().await;
    let first = tbd_db::ensure_user(&db, "kratos-one", None, "One")
        .await
        .unwrap();
    let other = uuid::Uuid::new_v4();
    sqlx::query("insert into parties (id, kind, display_name) values ($1, 'person', 'Impostor')")
        .bind(other)
        .execute(&*db)
        .await
        .unwrap();
    let e = sqlx::query("insert into users (id, subject) values ($1, 'kratos-one')")
        .bind(other)
        .execute(&*db)
        .await
        .unwrap_err();
    assert!(matches!(
        tbd_db::map_err(e),
        tbd_db::DbError::Conflict { .. }
    ));
    assert_eq!(
        tbd_db::ensure_user(&db, "kratos-one", None, "One")
            .await
            .unwrap(),
        first
    );
}

#[tokio::test]
async fn migrating_as_a_role_whose_name_matches_a_schema_does_not_duplicate_everything() {
    // The bug this reproduces: Postgres' default search_path is
    // `"$user", public`. On a database whose role is named `ledger` -- which is
    // exactly the cluster's -- and which also has a `ledger` schema, an
    // unqualified `create table` lands in `ledger`, not `public`. sqlx's own
    // `_sqlx_migrations` went there too, the applied-count check looked in
    // `public`, found nothing, and re-ran every migration into the wrong
    // schema. The identity tables ended up existing twice, and the foreign keys
    // pointed at the empty copy.
    let db = db().await;

    // Recreate the conditions: a role named after an existing schema.
    sqlx::query("create schema if not exists probe")
        .execute(&*db)
        .await
        .unwrap();
    sqlx::query("do $$ begin if not exists (select 1 from pg_roles where rolname='probe') then create role probe nologin; end if; end $$")
        .execute(&*db)
        .await
        .unwrap();

    // Migrate again under that identity. It must be a no-op, not a second copy.
    let before = applied_migrations(&db).await.unwrap();
    let mut tx = db.begin().await.unwrap();
    sqlx::Executor::execute(&mut *tx, "set local role probe")
        .await
        .unwrap();
    sqlx::Executor::execute(&mut *tx, "set local search_path to probe, public")
        .await
        .unwrap();
    let (found,): (Option<String>,) =
        sqlx::query_as("select to_regclass('_sqlx_migrations')::text")
            .fetch_one(&mut *tx)
            .await
            .unwrap();
    tx.commit().await.unwrap();

    // Unqualified, the bookkeeping table still has to resolve to the real one.
    assert_eq!(
        found.as_deref(),
        Some("_sqlx_migrations"),
        "an unqualified lookup found {found:?}; migrations would re-run"
    );

    migrate(&db).await.unwrap();
    assert_eq!(
        applied_migrations(&db).await.unwrap(),
        before,
        "migrating twice applied something a second time"
    );

    // And exactly one of each identity table, in public.
    for table in ["parties", "users", "orgs", "party_access"] {
        let (copies,): (i64,) =
            sqlx::query_as("select count(*) from information_schema.tables where table_name = $1")
                .bind(table)
                .fetch_one(&*db)
                .await
                .unwrap();
        assert_eq!(copies, 1, "{table} exists {copies} times, not once");
    }
}
