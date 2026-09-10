//! Integration tests: start the real server on an ephemeral port and drive it
//! with the generated client.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod support;

use futures::StreamExt;
use tbd_proto::engine::v1::{
    EvaluateRequest, SessionRequest, SubscribeRequest, session_request, session_response,
    subscribe_response,
};
use tokio_stream::wrappers::ReceiverStream;

#[tokio::test]
async fn evaluate_returns_labelled_stub() {
    let server = support::start().await;
    let mut client = server.client().await;

    let resp = client
        .evaluate(EvaluateRequest {
            subject_id: "s1".into(),
            payload: vec![1, 2, 3],
        })
        .await
        .unwrap()
        .into_inner();

    assert_eq!(resp.subject_id, "s1");
    assert!(
        resp.stub,
        "a stub score must be labelled as such on the wire"
    );
    assert!(resp.model_version.starts_with("stub-"));
}

#[tokio::test]
async fn evaluate_rejects_empty_subject() {
    let server = support::start().await;
    let mut client = server.client().await;

    let err = client
        .evaluate(EvaluateRequest {
            subject_id: String::new(),
            payload: vec![],
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::InvalidArgument);
}

#[tokio::test]
async fn subscribe_streams_monotonic_heartbeats() {
    let server = support::start().await;
    let mut client = server.client().await;

    let mut stream = client
        .subscribe(SubscribeRequest {
            subject_id: "s1".into(),
        })
        .await
        .unwrap()
        .into_inner();

    let started = std::time::Instant::now();
    let mut seqs = Vec::new();
    while seqs.len() < 3 {
        let ev = stream.next().await.unwrap().unwrap();
        assert_eq!(ev.subject_id, "s1");
        match ev.kind {
            Some(subscribe_response::Kind::Heartbeat(hb)) => seqs.push(hb.seq),
            other => panic!("unexpected event kind: {other:?}"),
        }
    }
    assert_eq!(seqs, vec![0, 1, 2]);
    // First tick is immediate, the next two wait the 20 ms heartbeat each.
    assert!(
        started.elapsed() >= std::time::Duration::from_millis(40),
        "heartbeats must be spaced by the configured interval, got {:?}",
        started.elapsed()
    );
}

#[tokio::test]
async fn session_echoes_data_frames() {
    let server = support::start().await;
    let mut client = server.client().await;

    let (tx, rx) = tokio::sync::mpsc::channel(8);
    let mut outbound = client
        .session(ReceiverStream::new(rx))
        .await
        .unwrap()
        .into_inner();

    tx.send(SessionRequest {
        session_id: "sess-1".into(),
        seq: 1,
        body: Some(session_request::Body::Data(b"hello".to_vec())),
    })
    .await
    .unwrap();

    // Skip heartbeats until the echo arrives.
    let echoed = loop {
        let frame = outbound.next().await.unwrap().unwrap();
        if let Some(session_response::Body::Data(data)) = frame.body {
            break data;
        }
    };
    assert_eq!(echoed, b"hello");

    tx.send(SessionRequest {
        session_id: "sess-1".into(),
        seq: 2,
        body: Some(session_request::Body::Close(tbd_proto::engine::v1::Close {
            reason: "done".into(),
        })),
    })
    .await
    .unwrap();

    // After Close the server ends the stream.
    let mut remaining = 0;
    while let Some(frame) = outbound.next().await {
        frame.unwrap();
        remaining += 1;
        assert!(remaining < 10, "stream did not end after Close");
    }
}

#[tokio::test]
async fn injected_error_surfaces_as_grpc_status_and_is_counted() {
    use tbd_common::fault::{Behavior, ErrorKind};
    let server = support::start().await;
    let mut client = server.client().await;

    server.runtime.fault.set(Behavior::Error {
        kind: ErrorKind::Unavailable,
        rate: 1.0,
        message: "chaos".into(),
    });
    let err = client
        .evaluate(EvaluateRequest {
            subject_id: "s1".into(),
            payload: vec![],
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::Unavailable);
    assert_eq!(err.message(), "chaos");

    server.runtime.fault.set(Behavior::Healthy);
    client
        .evaluate(EvaluateRequest {
            subject_id: "s1".into(),
            payload: vec![],
        })
        .await
        .unwrap();

    let snap = server.runtime.stats.snapshot();
    assert_eq!(snap.requests_total, 2);
    assert_eq!(snap.requests_failed, 1);
}

#[tokio::test]
async fn injected_error_terminates_subscribe_stream() {
    use tbd_common::fault::{Behavior, ErrorKind};
    let server = support::start().await;
    let mut client = server.client().await;

    let mut stream = client
        .subscribe(SubscribeRequest {
            subject_id: "s1".into(),
        })
        .await
        .unwrap()
        .into_inner();
    stream.next().await.unwrap().unwrap();

    server.runtime.fault.set(Behavior::Error {
        kind: ErrorKind::Internal,
        rate: 1.0,
        message: "cut".into(),
    });
    let mut saw_error = false;
    while let Some(item) = stream.next().await {
        if let Err(status) = item {
            assert_eq!(status.code(), tonic::Code::Internal);
            saw_error = true;
            break;
        }
    }
    assert!(saw_error, "stream must end with the injected status");
}
