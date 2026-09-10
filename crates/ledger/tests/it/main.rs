//! Integration tests: start the real server on an ephemeral port and drive it
//! with the generated client.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod support;

use tbd_ledger::{Behavior, FaultHandle, Runtime};
use tbd_proto::ledger::v1::PingRequest;
use tonic_health::pb::{HealthCheckRequest, health_check_response::ServingStatus};

#[tokio::test]
async fn ping_returns_labelled_stub() {
    let server = support::start().await;
    let mut client = server.client().await;

    let resp = client
        .ping(PingRequest {
            message: "hello".into(),
        })
        .await
        .unwrap()
        .into_inner();

    assert_eq!(resp.message, "hello");
    assert!(
        resp.stub,
        "a stub answer must be labelled as such on the wire"
    );
    assert_eq!(resp.version, tbd_common::VERSION);
}

#[tokio::test]
async fn ping_rejects_over_long_message() {
    let server = support::start().await;
    let mut client = server.client().await;

    let err = client
        .ping(PingRequest {
            message: "x".repeat(1025),
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::InvalidArgument);
}

#[tokio::test]
async fn health_reports_serving() {
    let server = support::start().await;
    let mut health = server.health().await;

    let resp = health
        .check(HealthCheckRequest {
            service: "tbd.ledger.v1.LedgerService".into(),
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(resp.status(), ServingStatus::Serving);
}

#[tokio::test]
async fn injected_fault_surfaces_and_is_counted() {
    let runtime = Runtime {
        fault: FaultHandle::new(Behavior::Error {
            kind: tbd_common::fault::ErrorKind::Unavailable,
            rate: 1.0,
            message: "injected".into(),
        }),
        ..Default::default()
    };
    let server = support::start_with(runtime).await;
    let mut client = server.client().await;

    let err = client
        .ping(PingRequest {
            message: "x".into(),
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::Unavailable);

    server.runtime.fault.set(Behavior::Healthy);
    client
        .ping(PingRequest {
            message: "x".into(),
        })
        .await
        .unwrap();

    let stats = server.runtime.stats.snapshot();
    assert_eq!(stats.requests_total, 2);
    assert_eq!(stats.requests_failed, 1);
}
