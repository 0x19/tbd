//! The Enable Banking client against a fixture server.
//!
//! Every case here is something the real bank did during the prototype, replayed
//! so the client is proven against it without spending a call.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::time::Duration;

use chrono::NaiveDate;
use serde_json::json;
use tbd_finance::banking::{EnableBanking, Provider, ProviderError, auth::Signer};
use wiremock::{
    Mock, MockServer, Request, ResponseTemplate,
    matchers::{header_exists, method, path, query_param},
};

const KEY: &str = include_str!("../fixtures/test-signing-key.pkcs8");

fn client(server: &MockServer) -> EnableBanking {
    let signer = Signer::from_pem("app-test", KEY).unwrap();
    EnableBanking::new(&server.uri(), signer, Duration::from_secs(5)).unwrap()
}

fn window() -> (NaiveDate, NaiveDate) {
    (
        NaiveDate::from_ymd_opt(2026, 8, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 9, 17).unwrap(),
    )
}

#[tokio::test]
async fn every_request_carries_a_signed_bearer_token_with_the_application_as_kid() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/sessions/s1"))
        .and(header_exists("authorization"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": "AUTHORIZED", "accounts": ["u1"], "access": {"valid_until": "2027-03-15T10:41:56Z"}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let status = client(&server).session("s1").await.unwrap();
    assert_eq!(status.status, "AUTHORIZED");
    assert_eq!(status.account_uids, vec!["u1"]);
    assert!(status.valid_until.is_some());

    let received: Vec<Request> = server.received_requests().await.unwrap();
    let auth = received[0]
        .headers
        .get("authorization")
        .unwrap()
        .to_str()
        .unwrap();
    let token = auth.strip_prefix("Bearer ").expect("bearer scheme");
    let parts: Vec<&str> = token.split('.').collect();
    assert_eq!(parts.len(), 3, "a JWT");
    let header: serde_json::Value = serde_json::from_slice(
        &base64::Engine::decode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, parts[0])
            .unwrap(),
    )
    .unwrap();
    assert_eq!(header["kid"], "app-test");
    assert_eq!(header["alg"], "RS256");
}

#[tokio::test]
async fn a_continuation_repeats_the_date_window_because_erste_rejects_it_alone() {
    // The real 422: "dateFrom in request is not the same as in continuationKey.
    // Continuation key is only valid for the same getAccountTransactions
    // parameters". The second page must carry the original window *and* the key.
    let server = MockServer::start().await;
    let (from, to) = window();
    Mock::given(method("GET"))
        .and(path("/accounts/u1/transactions"))
        .and(query_param("date_from", from.to_string()))
        .and(query_param("date_to", to.to_string()))
        .and(query_param("continuation_key", "k1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "transactions": [{"entry_reference": "b"}], "continuation_key": null
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/accounts/u1/transactions"))
        .and(query_param("date_from", from.to_string()))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "transactions": [{"entry_reference": "a"}], "continuation_key": "k1"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let pages = client(&server)
        .transactions("u1", from, to, None)
        .await
        .unwrap();
    assert_eq!(pages.len(), 2);
    assert_eq!(pages[0]["transactions"][0]["entry_reference"], "a");
    assert_eq!(pages[1]["transactions"][0]["entry_reference"], "b");
}

#[tokio::test]
async fn a_429_is_rate_limited_with_the_banks_retry_after_and_is_not_retried() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/accounts/u1/balances"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("retry-after", "21600")
                .set_body_json(
                    json!({"code": 429, "error": "ASPSP_RATE_LIMIT_EXCEEDED", "message": "x"}),
                ),
        )
        .expect(1)
        .mount(&server)
        .await;

    let e = client(&server).balances("u1", None).await.unwrap_err();
    assert!(
        matches!(e, ProviderError::RateLimited { retry_after: Some(d) } if d == Duration::from_secs(21600)),
        "{e}"
    );
    assert_eq!(
        server.received_requests().await.unwrap().len(),
        1,
        "exactly one request"
    );
}

#[tokio::test]
async fn an_expired_consent_is_reported_as_needing_a_person() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/accounts/u1/transactions"))
        .respond_with(ResponseTemplate::new(403).set_body_json(json!({
            "code": 403, "error": "FORBIDDEN", "message": "Session has expired",
            "detail": {"error_name": "SessionExpiredException"}
        })))
        .mount(&server)
        .await;
    let (from, to) = window();
    let e = client(&server)
        .transactions("u1", from, to, None)
        .await
        .unwrap_err();
    assert!(e.needs_consent(), "{e}");
    assert!(!e.retryable());
}

#[tokio::test]
async fn a_continuation_that_never_ends_is_refused_rather_than_followed_forever() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/accounts/u1/transactions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "transactions": [], "continuation_key": "again"
        })))
        .mount(&server)
        .await;
    let (from, to) = window();
    let e = client(&server)
        .transactions("u1", from, to, None)
        .await
        .unwrap_err();
    assert!(matches!(e, ProviderError::Malformed(_)), "{e}");
    assert!(server.received_requests().await.unwrap().len() <= 200);
}

#[tokio::test]
async fn a_connection_failure_is_transport_and_may_be_retried() {
    // A bound-then-closed port, so nothing answers. Dropping a MockServer is
    // not enough: another test's server can take the port back at once.
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    let signer = Signer::from_pem("app-test", KEY).unwrap();
    let c = EnableBanking::new(
        &format!("http://127.0.0.1:{port}"),
        signer,
        Duration::from_secs(2),
    )
    .unwrap();
    let e = c.balances("u1", None).await.unwrap_err();
    assert!(matches!(e, ProviderError::Transport(_)), "{e}");
    assert!(e.retryable());
}

#[tokio::test]
async fn creating_a_session_yields_the_accounts_in_the_providers_order() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/sessions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "session_id": "s9",
            "accounts": [
                {"uid": "usd", "currency": "USD", "account_id": {"iban": "HR1"}, "name": "A"},
                {"uid": "eur", "currency": "EUR", "account_id": {"iban": "HR1"}, "name": "A"}
            ],
            "access": {"valid_until": "2027-03-15T10:41:56Z"}
        })))
        .mount(&server)
        .await;
    let s = client(&server).create_session("code").await.unwrap();
    assert_eq!(s.session_id, "s9");
    let uids: Vec<_> = s.accounts.iter().map(|a| a.uid.as_str()).collect();
    assert_eq!(uids, ["usd", "eur"]);
    assert_eq!(s.accounts[0].currency, "USD");
}

#[tokio::test]
async fn an_attended_call_carries_the_persons_address_and_an_unattended_one_does_not() {
    use tbd_finance::banking::Psu;
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/accounts/u1/balances"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"balances": []})))
        .mount(&server)
        .await;
    let c = client(&server);
    c.balances("u1", None).await.unwrap();
    let psu = Psu {
        ip: "203.0.113.7".into(),
        user_agent: Some("Mozilla/5.0 test".into()),
    };
    c.balances("u1", Some(&psu)).await.unwrap();

    let seen: Vec<Request> = server.received_requests().await.unwrap();
    assert_eq!(seen.len(), 2);
    assert!(
        seen[0].headers.get("psu-ip-address").is_none(),
        "unattended: no PSU headers"
    );
    assert_eq!(
        seen[1]
            .headers
            .get("psu-ip-address")
            .unwrap()
            .to_str()
            .unwrap(),
        "203.0.113.7"
    );
    assert_eq!(
        seen[1]
            .headers
            .get("psu-user-agent")
            .unwrap()
            .to_str()
            .unwrap(),
        "Mozilla/5.0 test"
    );
}
