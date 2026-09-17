//! Access control over the real gRPC surface.
//!
//! The database tests in `tbd-db` prove the store and the row-level security
//! policies. These prove the thing an attacker actually touches: the RPC, with
//! a real server, a real Postgres, and the identity arriving the way Envoy
//! delivers it — base64url JSON in `x-jwt-payload`.
//!
//! The scenario is the one that will happen: an owner with a personal party and
//! a company, and an accountant granted the company only. Every assertion about
//! the accountant is about **absence**, because that is the failure that is
//! silent.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use base64::Engine as _;
use sqlx::PgPool;
use tbd_db::{Capability, PartyId, UserId, create_org, ensure_user, grant};
use tbd_proto::finance::v1::ListTransactionsRequest;
use tonic::{Code, Request, metadata::MetadataValue};
use uuid::Uuid;

use crate::support::{self, Server};

/// The header Envoy forwards after it has verified the token. Envoy strips
/// whatever a client sends in it, so in production its presence means
/// "verified"; here the test plays Envoy.
fn as_caller<T>(subject: &str, message: T) -> Request<T> {
    let claims = serde_json::json!({ "sub": subject, "scp": ["tbd.finance"] });
    let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(claims.to_string());
    let mut request = Request::new(message);
    request.metadata_mut().insert(
        "x-jwt-payload",
        MetadataValue::try_from(encoded.as_str()).unwrap(),
    );
    request
}

struct World {
    owner_party: Uuid,
    company: Uuid,
}

/// An owner with a personal party and a company, an accountant who may read the
/// company only, and one transaction in each.
async fn seed(pool: &PgPool) -> World {
    let owner: UserId = ensure_user(pool, "owner-subject", None, "Owner")
        .await
        .unwrap();
    let accountant = ensure_user(pool, "accountant-subject", None, "Accountant")
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

    for (party, label) in [(owner.0, "personal"), (company.0, "company")] {
        let account = Uuid::new_v4();
        sqlx::query(
            "insert into finance.accounts (id, party_id, currency, name) values ($1, $2, 'EUR', $3)",
        )
        .bind(account)
        .bind(party)
        .bind(label)
        .execute(pool)
        .await
        .unwrap();
        sqlx::query(
            "insert into finance.bank_transactions
                (id, party_id, account_id, status, dedup_key, amount_minor, currency,
                 credit_debit, booking_date, remittance)
             values ($1, $2, $3, 'booked', $4, -1000, 'EUR', 'DBIT', current_date, $5)",
        )
        .bind(Uuid::new_v4())
        .bind(party)
        .bind(account)
        .bind(format!("dedup-{label}").into_bytes())
        .bind(format!("a {label} payment"))
        .execute(pool)
        .await
        .unwrap();
    }

    World {
        owner_party: owner.0,
        company: company.0,
    }
}

async fn stack() -> (Server, PgPool, World) {
    let (server, pool) = support::start_with_store().await;
    let world = seed(&pool).await;
    (server, pool, world)
}

#[tokio::test]
async fn the_owner_sees_both_parties() {
    let (server, _pool, w) = stack().await;
    let mut client = server.client().await;

    let resp = client
        .list_transactions(as_caller(
            "owner-subject",
            ListTransactionsRequest::default(),
        ))
        .await
        .unwrap()
        .into_inner();

    let parties: Vec<String> = resp
        .transactions
        .iter()
        .map(|t| t.party_id.clone())
        .collect();
    assert!(parties.contains(&w.owner_party.to_string()));
    assert!(parties.contains(&w.company.to_string()));
    assert_eq!(resp.transactions.len(), 2);
}

#[tokio::test]
async fn the_accountant_never_receives_the_personal_transaction() {
    let (server, _pool, w) = stack().await;
    let mut client = server.client().await;

    let resp = client
        .list_transactions(as_caller(
            "accountant-subject",
            ListTransactionsRequest::default(),
        ))
        .await
        .unwrap()
        .into_inner();

    let parties: Vec<String> = resp
        .transactions
        .iter()
        .map(|t| t.party_id.clone())
        .collect();
    assert!(
        parties.contains(&w.company.to_string()),
        "the accountant must see the company"
    );
    assert!(
        !parties.contains(&w.owner_party.to_string()),
        "LEAK: the accountant received the owner's personal transaction"
    );
    assert_eq!(resp.transactions.len(), 1, "{:?}", resp.transactions);
    for t in &resp.transactions {
        assert!(
            !t.remittance.contains("personal"),
            "LEAK: a personal payment reached the accountant: {t:?}"
        );
    }
}

#[tokio::test]
async fn asking_for_a_party_you_were_not_granted_returns_nothing_and_reveals_nothing() {
    let (server, _pool, w) = stack().await;
    let mut client = server.client().await;

    // The accountant names the owner's personal party explicitly.
    let resp = client
        .list_transactions(as_caller(
            "accountant-subject",
            ListTransactionsRequest {
                party_ids: vec![w.owner_party.to_string()],
                ..Default::default()
            },
        ))
        .await
        .expect("a party outside the grant must not be an error: that would confirm it exists")
        .into_inner();

    assert!(
        resp.transactions.is_empty(),
        "LEAK: naming a party directly bypassed the grant"
    );
    assert!(
        resp.party_ids.is_empty(),
        "the answer disclosed which parties were considered: {:?}",
        resp.party_ids
    );
}

#[tokio::test]
async fn a_party_that_does_not_exist_is_indistinguishable_from_one_you_cannot_see() {
    let (server, _pool, w) = stack().await;
    let mut client = server.client().await;

    let invented = client
        .list_transactions(as_caller(
            "accountant-subject",
            ListTransactionsRequest {
                party_ids: vec![Uuid::new_v4().to_string()],
                ..Default::default()
            },
        ))
        .await
        .unwrap()
        .into_inner();
    let real_but_forbidden = client
        .list_transactions(as_caller(
            "accountant-subject",
            ListTransactionsRequest {
                party_ids: vec![w.owner_party.to_string()],
                ..Default::default()
            },
        ))
        .await
        .unwrap()
        .into_inner();

    assert_eq!(
        (invented.transactions.len(), invented.party_ids.len()),
        (
            real_but_forbidden.transactions.len(),
            real_but_forbidden.party_ids.len()
        ),
        "an invented party and a real-but-forbidden one gave different answers, \
         which is enough to enumerate what exists"
    );
}

#[tokio::test]
async fn narrowing_to_a_granted_party_works_and_is_the_personal_business_toggle() {
    let (server, _pool, w) = stack().await;
    let mut client = server.client().await;

    let resp = client
        .list_transactions(as_caller(
            "owner-subject",
            ListTransactionsRequest {
                party_ids: vec![w.company.to_string()],
                ..Default::default()
            },
        ))
        .await
        .unwrap()
        .into_inner();

    assert_eq!(resp.transactions.len(), 1);
    assert_eq!(resp.transactions[0].party_id, w.company.to_string());
}

#[tokio::test]
async fn an_unauthenticated_call_is_refused_rather_than_answered_empty() {
    let (server, _pool, _w) = stack().await;
    let mut client = server.client().await;

    // No x-jwt-payload at all: Envoy would never let this through, but the
    // service must not treat "no caller" as "no rows" -- that would hide a
    // misconfiguration behind a plausible empty page.
    let status = client
        .list_transactions(ListTransactionsRequest::default())
        .await
        .unwrap_err();
    assert_eq!(status.code(), Code::Unauthenticated, "{status:?}");
}

#[tokio::test]
async fn a_malformed_party_id_is_a_bad_request_not_an_empty_page() {
    let (server, _pool, _w) = stack().await;
    let mut client = server.client().await;

    let status = client
        .list_transactions(as_caller(
            "owner-subject",
            ListTransactionsRequest {
                party_ids: vec!["not-a-uuid".into()],
                ..Default::default()
            },
        ))
        .await
        .unwrap_err();
    assert_eq!(status.code(), Code::InvalidArgument, "{status:?}");
}

#[tokio::test]
async fn revoking_takes_effect_on_the_very_next_call() {
    let (server, pool, w) = stack().await;
    let mut client = server.client().await;

    let before = client
        .list_transactions(as_caller(
            "accountant-subject",
            ListTransactionsRequest::default(),
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(before.transactions.len(), 1);

    let accountant = ensure_user(&pool, "accountant-subject", None, "")
        .await
        .unwrap();
    tbd_db::revoke(&pool, accountant, PartyId(w.company))
        .await
        .unwrap();

    let after = client
        .list_transactions(as_caller(
            "accountant-subject",
            ListTransactionsRequest::default(),
        ))
        .await
        .unwrap()
        .into_inner();
    assert!(
        after.transactions.is_empty(),
        "LEAK: access survived a revoke for the length of a connection"
    );
}

#[tokio::test]
async fn a_caller_with_no_grants_gets_an_empty_page_not_everything() {
    let (server, _pool, _w) = stack().await;
    let mut client = server.client().await;

    // A brand-new identity: provisioned on first sight, owns only itself, and
    // there are no transactions under its own party.
    let resp = client
        .list_transactions(as_caller("a-stranger", ListTransactionsRequest::default()))
        .await
        .unwrap()
        .into_inner();
    assert!(
        resp.transactions.is_empty(),
        "LEAK: an unrelated caller saw {} transactions",
        resp.transactions.len()
    );
}
