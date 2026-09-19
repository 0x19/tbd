//! What the schema layout and the grants actually permit.
//!
//! A shared database is only safe if "shared" does not mean "shared
//! credentials". These prove the boundary the grants are supposed to draw:
//! a service reaches its own schema and identity, and not the other service's
//! data.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use crate::support::db;

#[tokio::test]
async fn both_service_schemas_exist() {
    let db = db().await;
    for schema in ["public", "ledger", "finance"] {
        let (exists,): (bool,) = sqlx::query_as(
            "select exists (select 1 from information_schema.schemata where schema_name = $1)",
        )
        .bind(schema)
        .fetch_one(&*db)
        .await
        .unwrap();
        assert!(exists, "schema {schema} is missing");
    }
}

#[tokio::test]
async fn the_group_roles_exist_and_cannot_log_in() {
    let db = db().await;
    for role in ["tbd_ledger", "tbd_finance"] {
        let row: Option<(bool,)> =
            sqlx::query_as("select rolcanlogin from pg_roles where rolname = $1")
                .bind(role)
                .fetch_optional(&*db)
                .await
                .unwrap();
        let Some((can_login,)) = row else {
            panic!("role {role} was not created; the migration's exception guard swallowed it");
        };
        assert!(
            !can_login,
            "{role} can log in — a group role must hold no credential"
        );
    }
}

#[tokio::test]
async fn a_service_role_cannot_read_the_other_services_schema() {
    let db = db().await;
    // Give each schema a table to reach for.
    sqlx::query("create table ledger.secret_thing (id int)")
        .execute(&*db)
        .await
        .unwrap();
    sqlx::query("create table finance.secret_thing (id int)")
        .execute(&*db)
        .await
        .unwrap();
    // Re-apply the grants the migration makes, now that the tables exist.
    sqlx::query("grant all on all tables in schema ledger to tbd_ledger")
        .execute(&*db)
        .await
        .unwrap();
    sqlx::query("grant all on all tables in schema finance to tbd_finance")
        .execute(&*db)
        .await
        .unwrap();

    let can = |role: &'static str, table: &'static str| {
        let pool = db.pool.clone();
        async move {
            let (allowed,): (Option<bool>,) =
                sqlx::query_as("select has_table_privilege($1, $2, 'select')")
                    .bind(role)
                    .bind(table)
                    .fetch_one(&pool)
                    .await
                    .unwrap();
            allowed.unwrap_or(false)
        }
    };

    assert!(can("tbd_ledger", "ledger.secret_thing").await);
    assert!(can("tbd_finance", "finance.secret_thing").await);
    assert!(
        !can("tbd_finance", "ledger.secret_thing").await,
        "LEAK: the finance role can read the ledger's tables"
    );
    assert!(
        !can("tbd_ledger", "finance.secret_thing").await,
        "LEAK: the ledger role can read finance's tables"
    );
}

#[tokio::test]
async fn services_may_read_identity_but_not_rewrite_someone_elses_access() {
    let db = db().await;
    let can = |role: &'static str, table: &'static str, privilege: &'static str| {
        let pool = db.pool.clone();
        async move {
            let (allowed,): (Option<bool>,) =
                sqlx::query_as("select has_table_privilege($1, $2, $3)")
                    .bind(role)
                    .bind(table)
                    .bind(privilege)
                    .fetch_one(&pool)
                    .await
                    .unwrap();
            allowed.unwrap_or(false)
        }
    };

    for role in ["tbd_ledger", "tbd_finance"] {
        assert!(
            can(role, "public.party_access", "select").await,
            "{role} cannot read grants"
        );
        assert!(can(role, "public.users", "select").await);
        assert!(
            can(role, "public.access_log", "insert").await,
            "{role} cannot audit"
        );

        // The boundary: a service provisions users and audits reads. It does
        // not hand out access, and it does not erase the audit trail.
        assert!(
            !can(role, "public.party_access", "insert").await,
            "LEAK: {role} can grant itself access"
        );
        assert!(
            !can(role, "public.party_access", "update").await,
            "LEAK: {role} can widen an existing grant"
        );
        assert!(
            !can(role, "public.party_access", "delete").await,
            "LEAK: {role} can revoke access"
        );
        assert!(
            !can(role, "public.access_log", "delete").await,
            "LEAK: {role} can delete its own audit trail"
        );
    }
}

#[tokio::test]
async fn a_table_added_later_is_covered_without_editing_the_grants() {
    let db = db().await;
    sqlx::query("create table finance.added_later (id int)")
        .execute(&*db)
        .await
        .unwrap();
    let (allowed,): (Option<bool>,) = sqlx::query_as(
        "select has_table_privilege('tbd_finance', 'finance.added_later', 'select')",
    )
    .fetch_one(&*db)
    .await
    .unwrap();
    assert_eq!(
        allowed,
        Some(true),
        "default privileges did not cover a new table; every migration would have to re-grant"
    );
}
