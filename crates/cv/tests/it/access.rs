//! The whole flow, on a real database: ask, be decided on, download, be
//! revoked -- and every refusal along the way.

use tbd_proto::cv::v1::{
    DecideRequestRequest, DownloadCvRequest, GetAccessRequest, ListRequestsRequest,
    RequestAccessRequest,
};
use tonic::Code;

use crate::support::{self, NAMELESS, OWNER, VISITOR, as_caller};

#[tokio::test]
async fn nobody_is_anybody_without_a_verified_caller() {
    let (server, _pool) = support::start_with_store(|_| {}).await;
    let mut c = server.client().await;
    let err = c.get_access(GetAccessRequest {}).await.unwrap_err();
    assert_eq!(err.code(), Code::Unauthenticated);
    let err = c
        .list_requests(as_caller(&VISITOR, ListRequestsRequest::default()))
        .await
        .unwrap_err();
    assert_eq!(
        err.code(),
        Code::PermissionDenied,
        "a viewer is not the owner"
    );
}

#[tokio::test]
async fn without_a_store_every_rpc_but_ping_is_unavailable() {
    let server = support::start().await;
    let mut c = server.client().await;
    let err = c
        .get_access(as_caller(&VISITOR, GetAccessRequest {}))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::Unavailable);
}

#[tokio::test]
async fn a_request_needs_an_email_in_the_token() {
    let (server, _pool) = support::start_with_store(|_| {}).await;
    let mut c = server.client().await;
    let err = c
        .request_access(as_caller(&NAMELESS, RequestAccessRequest::default()))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::InvalidArgument);
}

#[allow(clippy::too_many_lines)]
#[tokio::test]
async fn the_owner_decides_and_every_download_is_recorded() {
    let (server, pool) = support::start_with_store(|c| c.finance.url = String::new()).await;
    let mut c = server.client().await;

    // Nothing yet.
    let state = c
        .get_access(as_caller(&VISITOR, GetAccessRequest {}))
        .await
        .unwrap()
        .into_inner()
        .state
        .unwrap();
    assert_eq!(state.status, "none");
    assert!(!state.notifications, "no finance url: nobody is told");

    // Asking stores the request; with no finance url nobody was told.
    let state = c
        .request_access(as_caller(
            &VISITOR,
            RequestAccessRequest {
                note: "  I am hiring for a Rust role.  ".into(),
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .state
        .unwrap();
    assert_eq!(state.status, "requested");
    assert_eq!(state.note, "I am hiring for a Rust role.");
    assert_eq!(state.email, "rita@example.org");
    assert_eq!(state.name, "Rita Reader");
    assert!(!state.requested_at.is_empty());
    assert!(state.notified_at.is_empty());

    // Not yet: the download is refused, and says so without leaking.
    let err = c
        .download_cv(as_caller(&VISITOR, DownloadCvRequest {}))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::PermissionDenied);

    // The owner sees it; a visitor does not.
    let listed = c
        .list_requests(as_caller(&OWNER, ListRequestsRequest::default()))
        .await
        .unwrap()
        .into_inner()
        .requests;
    assert_eq!(listed.len(), 1);
    let id = listed[0].id.clone();
    assert_eq!(listed[0].subject, "visitor-1");
    assert_eq!(listed[0].downloads, 0);

    // A wrong word, a wrong id, a wrong transition.
    let err = c
        .decide_request(as_caller(
            &OWNER,
            DecideRequestRequest {
                id: id.clone(),
                decision: "maybe".into(),
            },
        ))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::FailedPrecondition);
    let err = c
        .decide_request(as_caller(
            &OWNER,
            DecideRequestRequest {
                id: "nope".into(),
                decision: "approve".into(),
            },
        ))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::InvalidArgument);
    let err = c
        .decide_request(as_caller(
            &OWNER,
            DecideRequestRequest {
                id: id.clone(),
                decision: "revoke".into(),
            },
        ))
        .await
        .unwrap_err();
    assert_eq!(
        err.code(),
        Code::FailedPrecondition,
        "nothing to revoke yet"
    );
    let err = c
        .decide_request(as_caller(
            &VISITOR,
            DecideRequestRequest {
                id: id.clone(),
                decision: "approve".into(),
            },
        ))
        .await
        .unwrap_err();
    assert_eq!(
        err.code(),
        Code::PermissionDenied,
        "a visitor cannot approve themselves"
    );

    // Approved: the download is a PDF for this reader, and it is recorded.
    let decided = c
        .decide_request(as_caller(
            &OWNER,
            DecideRequestRequest {
                id: id.clone(),
                decision: "approve".into(),
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .request
        .unwrap();
    assert_eq!(decided.status, "approved");
    assert_eq!(decided.decided_by, "nevio@inorbit.hr");
    let doc = c
        .download_cv(as_caller(&VISITOR, DownloadCvRequest {}))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(doc.content_type, "application/pdf");
    assert!(doc.pdf.starts_with(b"%PDF-"));
    assert!(doc.pdf.len() < 1_000_000, "{} bytes", doc.pdf.len());
    assert_eq!(doc.filename, "nevio-vesic-cv.pdf");
    let text = pdf_extract::extract_text_from_mem(&doc.pdf).unwrap();
    assert!(
        text.contains("Rita Reader"),
        "the reader is named on the page"
    );
    let (n, ua, ip): (i64, String, String) = sqlx::query_as(
        "select count(*), max(user_agent), max(ip) from cv.downloads d join cv.requests r on r.id = d.request_id where r.subject = 'visitor-1'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    // tonic appends its own token to the user agent; the page's part comes first.
    assert_eq!((n, ip.as_str()), (1, "203.0.113.7"));
    assert!(ua.starts_with("test-agent/1"), "{ua}");
    let listed = c
        .list_requests(as_caller(
            &OWNER,
            ListRequestsRequest {
                status: "approved".into(),
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .requests;
    assert_eq!(listed[0].downloads, 1);
    assert!(!listed[0].last_download_at.is_empty());

    // Asking again while approved changes nothing but the note.
    let state = c
        .request_access(as_caller(
            &VISITOR,
            RequestAccessRequest {
                note: "still hiring".into(),
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .state
        .unwrap();
    assert_eq!(state.status, "approved");
    assert_eq!(state.note, "still hiring");

    // Revoked: no more downloads; asking again is a fresh request.
    c.decide_request(as_caller(
        &OWNER,
        DecideRequestRequest {
            id: id.clone(),
            decision: "revoke".into(),
        },
    ))
    .await
    .unwrap();
    let err = c
        .download_cv(as_caller(&VISITOR, DownloadCvRequest {}))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::PermissionDenied);
    let state = c
        .request_access(as_caller(
            &VISITOR,
            RequestAccessRequest {
                note: "please".into(),
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .state
        .unwrap();
    assert_eq!(state.status, "requested");
    assert!(state.decided_at.is_empty());
    let refused = c
        .decide_request(as_caller(
            &OWNER,
            DecideRequestRequest {
                id,
                decision: "refuse".into(),
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .request
        .unwrap();
    assert_eq!(refused.status, "refused");
    assert_eq!(refused.downloads, 1, "the history of downloads stays");
}
