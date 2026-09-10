//! Integration tests: start the real server on an ephemeral port and drive it
//! with the generated client.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod support;

use futures::StreamExt;
use tbd_proto::engine::v1::{
    EvaluateRequest, SessionFrame, SubscribeRequest, event, session_frame,
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
            Some(event::Kind::Heartbeat(hb)) => seqs.push(hb.seq),
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

    tx.send(SessionFrame {
        session_id: "sess-1".into(),
        seq: 1,
        body: Some(session_frame::Body::Data(b"hello".to_vec())),
    })
    .await
    .unwrap();

    // Skip heartbeats until the echo arrives.
    let echoed = loop {
        let frame = outbound.next().await.unwrap().unwrap();
        if let Some(session_frame::Body::Data(data)) = frame.body {
            break data;
        }
    };
    assert_eq!(echoed, b"hello");

    tx.send(SessionFrame {
        session_id: "sess-1".into(),
        seq: 2,
        body: Some(session_frame::Body::Close(tbd_proto::engine::v1::Close {
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
