//! The read and write RPCs behind the finance UI, driven over gRPC.
//!
//! Every test here is about the grant. The accountant may read the company
//! and nothing else; a row, category, account or connection outside that is
//! *not found*, never forbidden, on every RPC -- because "forbidden" would
//! confirm the thing exists.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use base64::Engine as _;
use serde_json::json;
use sqlx::PgPool;
use tbd_db::{Capability, PartyId, UserId, create_org, ensure_user, grant};
use tbd_finance::banking::Mock;
use tbd_proto::finance::v1::{
    CompleteConnectionRequest, DeclareCategoryRequest, GetTransactionRequest,
    ListCategoriesRequest, ListPartiesRequest, ListTransactionsRequest, MonthlySummaryRequest,
    RefreshAccountRequest, StartConnectionRequest, UpsertCategoryRequest, UpsertRuleRequest,
};
use tonic::{Code, Request, metadata::MetadataValue};
use uuid::Uuid;

use crate::support::{Server, start_with_bank, start_with_store};

const OWNER: &str = "rpc-owner";
const READER: &str = "rpc-reader";

fn as_caller<T>(subject: &str, message: T) -> Request<T> {
    let claims = json!({ "sub": subject, "scp": ["tbd.finance"] });
    let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(claims.to_string());
    let mut request = Request::new(message);
    request.metadata_mut().insert(
        "x-jwt-payload",
        MetadataValue::try_from(encoded.as_str()).unwrap(),
    );
    request
}

struct World {
    personal: Uuid,
    company: Uuid,
    personal_txn: Uuid,
    company_txn: Uuid,
    personal_account: Uuid,
    company_account: Uuid,
    personal_category: Uuid,
    company_category: Uuid,
}

async fn category(pool: &PgPool, party: Uuid, slug: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query("insert into finance.categories (id, party_id, slug, name, kind) values ($1,$2,$3,$3,'expense')")
        .bind(id)
        .bind(party)
        .bind(slug)
        .execute(pool)
        .await
        .unwrap();
    id
}

async fn seed(pool: &PgPool) -> World {
    let owner: UserId = ensure_user(pool, OWNER, None, "Owner").await.unwrap();
    let reader = ensure_user(pool, READER, None, "Reader").await.unwrap();
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
        reader,
        PartyId(company.0),
        Capability::Read,
        Some(owner),
        None,
    )
    .await
    .unwrap();

    let mut accounts = Vec::new();
    let mut txns = Vec::new();
    for (party, label, uid) in [
        (owner.0, "personal", "uid-p"),
        (company.0, "company", "uid-c"),
    ] {
        let connection = Uuid::new_v4();
        sqlx::query(
            "insert into finance.connections
                (id, party_id, provider, psu_type, aspsp_name, aspsp_country, session_id, status, state, authorized_at)
             values ($1, $2, 'mock', 'business', 'Mock', 'HR', 'sess', 'authorized', $3, now())",
        )
        .bind(connection)
        .bind(party)
        .bind(Uuid::new_v4().to_string())
        .execute(pool)
        .await
        .unwrap();
        let account = Uuid::new_v4();
        sqlx::query(
            "insert into finance.accounts (id, party_id, provider, provider_uid, currency, name, connection_id)
             values ($1, $2, 'mock', $3, 'EUR', $4, $5)",
        )
        .bind(account)
        .bind(party)
        .bind(uid)
        .bind(label)
        .bind(connection)
        .execute(pool)
        .await
        .unwrap();
        let txn = Uuid::new_v4();
        sqlx::query(
            "insert into finance.bank_transactions
                (id, party_id, account_id, status, dedup_key, amount_minor, currency,
                 credit_debit, booking_date, counterparty_name, remittance)
             values ($1, $2, $3, 'booked', $4, -1000, 'EUR', 'DBIT', date '2026-09-10', 'ANTHROPIC', $5)",
        )
        .bind(txn)
        .bind(party)
        .bind(account)
        .bind(format!("dedup-{label}").into_bytes())
        .bind(format!("a {label} payment"))
        .execute(pool)
        .await
        .unwrap();
        accounts.push(account);
        txns.push(txn);
    }
    let personal_category = category(pool, owner.0, "software").await;
    let company_category = category(pool, company.0, "software").await;
    World {
        personal: owner.0,
        company: company.0,
        personal_txn: txns[0],
        company_txn: txns[1],
        personal_account: accounts[0],
        company_account: accounts[1],
        personal_category,
        company_category,
    }
}

#[tokio::test]
async fn the_summary_never_contains_a_row_the_reader_was_not_granted() {
    let (server, pool) = start_with_store().await;
    let w = seed(&pool).await;
    let mut client = server.client().await;

    let mine = client
        .monthly_summary(as_caller(OWNER, MonthlySummaryRequest::default()))
        .await
        .unwrap()
        .into_inner();
    let parties: std::collections::BTreeSet<_> =
        mine.rows.iter().map(|r| r.party_id.clone()).collect();
    assert_eq!(parties.len(), 2, "the owner sees both parties: {mine:?}");

    let theirs = client
        .monthly_summary(as_caller(READER, MonthlySummaryRequest::default()))
        .await
        .unwrap()
        .into_inner();
    // Every user owns their own person party, so the reader's view is the
    // company plus themselves -- and never the owner's personal party.
    assert!(!theirs.rows.is_empty());
    assert!(
        theirs
            .rows
            .iter()
            .all(|r| r.party_id != w.personal.to_string()),
        "{theirs:?}"
    );
    assert!(theirs.party_ids.contains(&w.company.to_string()));
    assert!(!theirs.party_ids.contains(&w.personal.to_string()));

    // Asking for the personal party by id does not widen the view; it is
    // dropped, and the answer is the same as not asking.
    let asked = client
        .monthly_summary(as_caller(
            READER,
            MonthlySummaryRequest {
                party_ids: vec![w.personal.to_string(), w.company.to_string()],
                ..MonthlySummaryRequest::default()
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(asked.rows, theirs.rows);
}

#[tokio::test]
async fn parties_are_exactly_the_grant() {
    let (server, pool) = start_with_store().await;
    let w = seed(&pool).await;
    let mut client = server.client().await;
    let theirs = client
        .list_parties(as_caller(READER, ListPartiesRequest {}))
        .await
        .unwrap()
        .into_inner();
    let ids: Vec<_> = theirs.parties.iter().map(|p| p.id.as_str()).collect();
    assert!(ids.contains(&w.company.to_string().as_str()));
    assert!(
        !ids.contains(&w.personal.to_string().as_str()),
        "the owner's personal party leaked"
    );
    let company = theirs
        .parties
        .iter()
        .find(|p| p.id == w.company.to_string())
        .unwrap();
    assert_eq!(company.capability, "read");
    assert_eq!(company.kind, "org");
}

#[tokio::test]
async fn declaring_on_a_row_outside_the_grant_is_not_found_and_changes_nothing() {
    let (server, pool) = start_with_store().await;
    let w = seed(&pool).await;
    let mut client = server.client().await;

    let e = client
        .declare_category(as_caller(
            READER,
            DeclareCategoryRequest {
                transaction_id: w.personal_txn.to_string(),
                category_id: w.personal_category.to_string(),
            },
        ))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::NotFound, "{e}");

    // A category from another party is not-found too, even on a row the
    // reader may see: a category is the party's, like the row.
    let e = client
        .declare_category(as_caller(
            READER,
            DeclareCategoryRequest {
                transaction_id: w.company_txn.to_string(),
                category_id: w.personal_category.to_string(),
            },
        ))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::NotFound, "{e}");

    let (touched,): (i64,) = sqlx::query_as(
        "select count(*) from finance.bank_transactions where category_id is not null",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(touched, 0);

    // The legitimate case, with the answer carrying the declared row.
    let done = client
        .declare_category(as_caller(
            OWNER,
            DeclareCategoryRequest {
                transaction_id: w.company_txn.to_string(),
                category_id: w.company_category.to_string(),
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .transaction
        .unwrap();
    assert_eq!(done.category_source, "declared");
    assert_eq!(done.category, "software");
}

#[tokio::test]
async fn a_rule_for_a_party_not_granted_is_not_found_and_a_granted_one_applies_at_once() {
    let (server, pool) = start_with_store().await;
    let w = seed(&pool).await;
    let mut client = server.client().await;
    let rule = |party: Uuid, category: Uuid| UpsertRuleRequest {
        party_id: party.to_string(),
        priority: 10,
        name: "anthropic".into(),
        category_id: category.to_string(),
        match_counterparty_like: "anthropic".into(),
        enabled: true,
        ..UpsertRuleRequest::default()
    };

    let e = client
        .upsert_rule(as_caller(READER, rule(w.personal, w.personal_category)))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::NotFound, "{e}");

    let done = client
        .upsert_rule(as_caller(OWNER, rule(w.company, w.company_category)))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(
        done.categorised, 1,
        "the pass ran and claimed the company row"
    );
    let r = done.rule.unwrap();
    assert_eq!(r.match_counterparty_like, "ANTHROPIC", "stored normalised");
    assert_eq!(r.hits, 1);

    // And the row now lists with its category, for the company only.
    let rows = client
        .list_transactions(as_caller(
            READER,
            ListTransactionsRequest {
                category_id: r.category_id.clone(),
                ..ListTransactionsRequest::default()
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .transactions;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].category_source, "inferred");
    let none = client
        .list_transactions(as_caller(
            OWNER,
            ListTransactionsRequest {
                category_id: "none".into(),
                ..ListTransactionsRequest::default()
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .transactions;
    assert_eq!(
        none.len(),
        1,
        "only the personal row is still uncategorised"
    );
    assert_eq!(none[0].party_id, w.personal.to_string());
}

#[tokio::test]
async fn refreshing_an_account_outside_the_grant_is_not_found_and_costs_no_request() {
    let bank = Mock::new().with_pages("uid-c", vec![json!({"transactions": []})]);
    let provider: Arc<dyn tbd_finance::banking::Provider> = bank.clone();
    let (server, pool): (Server, PgPool) = start_with_bank(provider).await;
    let w = seed(&pool).await;
    let mut client = server.client().await;

    let e = client
        .refresh_account(as_caller(
            READER,
            RefreshAccountRequest {
                account_id: w.personal_account.to_string(),
            },
        ))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::NotFound, "{e}");
    assert_eq!(bank.calls(), 0);

    let done = client
        .refresh_account(as_caller(
            OWNER,
            RefreshAccountRequest {
                account_id: w.company_account.to_string(),
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(done.outcome, "ok");
    assert!(bank.calls() > 0);
}

#[tokio::test]
async fn completing_a_connection_with_a_foreign_state_is_not_found_and_exchanges_nothing() {
    let session = json!({
        "session_id": "s", "accounts": [
            {"uid": "uid-new", "currency": "EUR", "account_id": {"iban": "HR77"}, "name": "New"}
        ], "access": {"valid_until": "2027-03-15T10:41:56Z"}
    });
    let bank = Mock::new().with_session(&session);
    let provider: Arc<dyn tbd_finance::banking::Provider> = bank.clone();
    let (server, pool) = start_with_bank(provider).await;
    let w = seed(&pool).await;
    let mut client = server.client().await;

    // The reader cannot even start one for the personal party.
    let e = client
        .start_connection(as_caller(
            READER,
            StartConnectionRequest {
                party_id: w.personal.to_string(),
                psu_type: "personal".into(),
                ..StartConnectionRequest::default()
            },
        ))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::NotFound, "{e}");

    let started = client
        .start_connection(as_caller(
            OWNER,
            StartConnectionRequest {
                party_id: w.personal.to_string(),
                psu_type: "personal".into(),
                ..StartConnectionRequest::default()
            },
        ))
        .await
        .unwrap()
        .into_inner();
    let state = url::Url::parse(&started.url)
        .unwrap()
        .query_pairs()
        .find(|(k, _)| k == "state")
        .map(|(_, v)| v.into_owned())
        .unwrap();

    let e = client
        .complete_connection(as_caller(
            READER,
            CompleteConnectionRequest {
                state: state.clone(),
                code: "stolen".into(),
            },
        ))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::NotFound, "{e}");
    assert_eq!(bank.seen().iter().filter(|p| *p == "/sessions").count(), 0);

    let done = client
        .complete_connection(as_caller(
            OWNER,
            CompleteConnectionRequest {
                state,
                code: "the-code".into(),
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(done.account_ids.len(), 1);

    let e = client
        .complete_connection(as_caller(
            OWNER,
            CompleteConnectionRequest {
                state: "never-issued".into(),
                code: "x".into(),
            },
        ))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::NotFound);
}

#[tokio::test]
async fn one_transaction_outside_the_grant_is_not_found_and_inside_carries_the_bank_record() {
    let (server, pool) = start_with_store().await;
    let w = seed(&pool).await;
    let mut client = server.client().await;

    let e = client
        .get_transaction(as_caller(
            READER,
            GetTransactionRequest {
                id: w.personal_txn.to_string(),
            },
        ))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::NotFound, "{e}");

    let t = client
        .get_transaction(as_caller(
            READER,
            GetTransactionRequest {
                id: w.company_txn.to_string(),
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .transaction
        .unwrap();
    assert_eq!(t.account_name, "company");
    assert_eq!(t.raw, "{}", "the record the bank sent, verbatim");

    // The list does not carry records: a page of a hundred is not a hundred
    // bank records.
    let listed = client
        .list_transactions(as_caller(READER, ListTransactionsRequest::default()))
        .await
        .unwrap()
        .into_inner()
        .transactions;
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].raw, "");
    assert_eq!(listed[0].account_name, "company");
}

#[tokio::test]
async fn a_category_is_created_slugged_and_archiving_it_disables_its_rules() {
    let (server, pool) = start_with_store().await;
    let w = seed(&pool).await;
    let mut client = server.client().await;
    let input = |party: Uuid| UpsertCategoryRequest {
        party_id: party.to_string(),
        name: "Kiosks & newsstands".into(),
        kind: "expense".into(),
        deductible: false,
        ..UpsertCategoryRequest::default()
    };

    let e = client
        .upsert_category(as_caller(READER, input(w.personal)))
        .await
        .unwrap_err();
    assert_eq!(
        e.code(),
        Code::NotFound,
        "a party not granted does not exist"
    );
    let e = client
        .upsert_category(as_caller(
            READER,
            UpsertCategoryRequest {
                kind: "fun".into(),
                ..input(w.company)
            },
        ))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::InvalidArgument, "{e}");

    let made = client
        .upsert_category(as_caller(OWNER, input(w.company)))
        .await
        .unwrap()
        .into_inner()
        .category
        .unwrap();
    assert_eq!(made.slug, "kiosks_newsstands");
    assert_eq!(made.name, "Kiosks & newsstands");
    let e = client
        .upsert_category(as_caller(OWNER, input(w.company)))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::InvalidArgument, "same slug twice: {e}");

    // A rule points at it and claims the company row.
    let done = client
        .upsert_rule(as_caller(
            OWNER,
            UpsertRuleRequest {
                party_id: w.company.to_string(),
                priority: 10,
                name: "anthropic".into(),
                category_id: made.id.clone(),
                match_counterparty_like: "anthropic".into(),
                enabled: true,
                ..UpsertRuleRequest::default()
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(done.rule.unwrap().hits, 1);

    // Archiving disables that rule and the pass lets the row go.
    let archived = client
        .upsert_category(as_caller(
            OWNER,
            UpsertCategoryRequest {
                id: made.id.clone(),
                archived: true,
                ..input(w.company)
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert!(archived.category.unwrap().archived);
    assert_eq!(
        archived.unmatched, 1,
        "the company row is uncategorised again"
    );
    let rows = client
        .list_transactions(as_caller(
            OWNER,
            ListTransactionsRequest {
                category_id: "none".into(),
                ..ListTransactionsRequest::default()
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .transactions;
    assert_eq!(rows.len(), 2);
    let cats = client
        .list_categories(as_caller(OWNER, ListCategoriesRequest::default()))
        .await
        .unwrap()
        .into_inner()
        .categories;
    assert!(cats.iter().any(|c| c.id == made.id && c.archived));
}

#[tokio::test]
async fn a_rule_made_over_the_api_keeps_the_punctuation_the_bank_writes() {
    let (server, pool) = start_with_store().await;
    let w = seed(&pool).await;
    let mut client = server.client().await;
    sqlx::query(
        "insert into finance.bank_transactions
            (id, party_id, account_id, status, dedup_key, amount_minor, currency,
             credit_debit, booking_date, counterparty_name, remittance)
         values ($1, $2, $3, 'booked', $4, -1674, 'EUR', 'DBIT', date '2026-09-11', 'NAME-CHEAP.COM* 3MVGXB', 'domain')",
    )
    .bind(Uuid::new_v4())
    .bind(w.company)
    .bind(w.company_account)
    .bind(b"dedup-namecheap".to_vec())
    .execute(&pool)
    .await
    .unwrap();

    let done = client
        .upsert_rule(as_caller(
            OWNER,
            UpsertRuleRequest {
                party_id: w.company.to_string(),
                priority: 10,
                name: "namecheap".into(),
                category_id: w.company_category.to_string(),
                match_counterparty_like: " name-čheap ".into(),
                enabled: true,
                ..UpsertRuleRequest::default()
            },
        ))
        .await
        .unwrap()
        .into_inner();
    let r = done.rule.unwrap();
    assert_eq!(
        r.match_counterparty_like, "NAME-CHEAP",
        "trimmed, upper, diacritics gone, hyphen kept"
    );
    assert_eq!(
        r.hits, 1,
        "and it claims the row the bank wrote with the hyphen"
    );
}
