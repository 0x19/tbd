//! The radar's own behaviour: reading sources into the store, the admin gate,
//! and a digest that is refused rather than stored half.

use std::fmt::Write as _;

use chrono::{Duration, Utc};
use tbd_proto::radar::v1::{
    ListDigestsRequest, ListItemsRequest, RefreshRequest, RunDigestRequest,
};
use tbd_radar::config::SourceKind;
use tonic::Code;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

use crate::support::{self, as_caller, source};

fn atom(entries: &[(&str, &str)]) -> String {
    let when = (Utc::now() - Duration::days(1)).to_rfc3339();
    let mut s = format!(
        r#"<?xml version="1.0" encoding="utf-8"?><feed xmlns="http://www.w3.org/2005/Atom"><title>t</title><id>f</id><updated>{when}</updated>"#
    );
    for (id, title) in entries {
        let _ = write!(
            s,
            r#"<entry><title>{title}</title><id>{id}</id><link href="https://example.org/{id}"/><updated>{when}</updated><summary>About {title}.</summary></entry>"#
        );
    }
    s.push_str("</feed>");
    s
}

fn rss(items: &[(&str, &str)]) -> String {
    let when = (Utc::now() - Duration::days(2)).to_rfc2822();
    let mut s = String::from(
        r#"<?xml version="1.0"?><rss version="2.0"><channel><title>t</title><link>https://example.org</link><description>d</description>"#,
    );
    for (id, title) in items {
        let _ = write!(
            s,
            "<item><title>{title}</title><guid>{id}</guid><link>https://example.org/{id}</link><pubDate>{when}</pubDate></item>"
        );
    }
    s.push_str("</channel></rss>");
    s
}

fn github(titles: &[&str]) -> serde_json::Value {
    let when = (Utc::now() - Duration::days(3)).to_rfc3339();
    serde_json::json!({
        "total_count": titles.len(),
        "items": titles.iter().enumerate().map(|(n, t)| serde_json::json!({
            "html_url": format!("https://github.com/x/y/issues/{n}"),
            "title": t, "body": "Accepted.", "updated_at": when, "closed_at": null,
        })).collect::<Vec<_>>(),
    })
}

async fn sources() -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/go.atom"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(atom(&[("a1", "Go 1.27"), ("a2", "Iterators")])),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/twir.rss"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(rss(&[("r1", "This Week in Rust 600")])),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/search/issues"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(github(&["proposal: slices", "proposal: maps"])),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/broken"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;
    server
}

#[tokio::test]
async fn reads_without_a_store_are_unavailable() {
    let server = support::start().await;
    let mut client = server.client().await;
    let err = client
        .list_digests(ListDigestsRequest::default())
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::Unavailable);
}

#[tokio::test]
async fn refresh_reads_every_kind_and_is_idempotent() {
    let feeds = sources().await;
    let base = feeds.uri();
    let (server, _pool) = support::start_with_store(|c| {
        c.sources = vec![
            source(
                "go-blog",
                SourceKind::Atom,
                "go",
                &format!("{base}/go.atom"),
            ),
            source("twir", SourceKind::Rss, "rust", &format!("{base}/twir.rss")),
            source(
                "go-proposals",
                SourceKind::Github,
                "go",
                &format!("{base}/search/issues"),
            ),
        ];
    })
    .await;
    let mut client = server.client().await;

    let first = client
        .refresh(as_caller("owner", Some("admin"), RefreshRequest {}))
        .await
        .unwrap()
        .into_inner();
    assert_eq!((first.sources, first.failed, first.new_items), (3, 0, 5));
    let again = client
        .refresh(as_caller("owner", Some("admin"), RefreshRequest {}))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(again.new_items, 0, "an item read twice is stored once");

    let go = client
        .list_items(ListItemsRequest {
            language: "go".into(),
            limit: 0,
        })
        .await
        .unwrap()
        .into_inner()
        .items;
    assert_eq!(go.len(), 4);
    assert!(go.iter().all(|i| i.language == "go"));
    let rust = client
        .list_items(ListItemsRequest {
            language: "rust".into(),
            limit: 0,
        })
        .await
        .unwrap()
        .into_inner()
        .items;
    assert_eq!(rust.len(), 1);
    assert_eq!(rust[0].title, "This Week in Rust 600");
}

#[tokio::test]
async fn a_failing_source_is_counted_and_the_rest_are_read() {
    let feeds = sources().await;
    let base = feeds.uri();
    let (server, _pool) = support::start_with_store(|c| {
        c.sources = vec![
            source("broken", SourceKind::Atom, "go", &format!("{base}/broken")),
            source(
                "go-blog",
                SourceKind::Atom,
                "go",
                &format!("{base}/go.atom"),
            ),
        ];
    })
    .await;
    let done = server
        .client()
        .await
        .refresh(as_caller("owner", Some("admin"), RefreshRequest {}))
        .await
        .unwrap()
        .into_inner();
    assert_eq!((done.sources, done.failed, done.new_items), (2, 1, 2));
}

#[tokio::test]
async fn refresh_and_run_digest_are_for_admins_only() {
    let (server, _pool) = support::start_with_store(|_| {}).await;
    let mut client = server.client().await;

    let nobody = client.refresh(RefreshRequest {}).await.unwrap_err();
    assert_eq!(nobody.code(), Code::Unauthenticated);
    let viewer = client
        .refresh(as_caller("someone", Some("viewer"), RefreshRequest {}))
        .await
        .unwrap_err();
    assert_eq!(viewer.code(), Code::PermissionDenied);
    let digest = client
        .run_digest(as_caller(
            "someone",
            None,
            RunDigestRequest {
                force: false,
                wait: true,
            },
        ))
        .await
        .unwrap_err();
    assert_eq!(digest.code(), Code::PermissionDenied);
}

#[tokio::test]
async fn run_digest_without_an_llm_service_says_so() {
    let (server, _pool) = support::start_with_store(|_| {}).await;
    let err = server
        .client()
        .await
        .run_digest(as_caller(
            "owner",
            Some("admin"),
            RunDigestRequest {
                force: false,
                wait: true,
            },
        ))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::FailedPrecondition);
}

#[tokio::test]
async fn an_answer_without_the_headings_is_not_stored() {
    // The llm service's stub engine echoes the prompt: no headings, so every
    // digest must be refused, and nothing half-written may reach the list.
    let (llm_url, _llm) = support::llm_stub().await;
    let feeds = sources().await;
    let base = feeds.uri();
    let (server, _pool) = support::start_with_store(|c| {
        c.llm.url = llm_url;
        c.llm.timeout_secs = 30;
        c.sources = vec![source(
            "go-blog",
            SourceKind::Atom,
            "go",
            &format!("{base}/go.atom"),
        )];
    })
    .await;
    let mut client = server.client().await;
    client
        .refresh(as_caller("owner", Some("admin"), RefreshRequest {}))
        .await
        .unwrap();

    let run = client
        .run_digest(as_caller(
            "owner",
            Some("admin"),
            RunDigestRequest {
                force: true,
                wait: true,
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert!(run.week.contains("-W"), "a week label: {}", run.week);
    assert!(
        run.digests.is_empty(),
        "no digest from an answer without headings"
    );
    let listed = client
        .list_digests(ListDigestsRequest::default())
        .await
        .unwrap()
        .into_inner()
        .digests;
    assert!(listed.is_empty());
}

#[tokio::test]
async fn filters_are_checked() {
    let (server, _pool) = support::start_with_store(|_| {}).await;
    let err = server
        .client()
        .await
        .list_digests(ListDigestsRequest {
            language: "python".into(),
            lang: String::new(),
            limit: 0,
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::InvalidArgument);
}

#[tokio::test]
async fn run_digest_starts_in_the_background_by_default() {
    let (llm_url, _llm) = support::llm_stub().await;
    let (server, _pool) = support::start_with_store(|c| {
        c.llm.url = llm_url;
    })
    .await;
    let run = server
        .client()
        .await
        .run_digest(as_caller(
            "owner",
            Some("admin"),
            RunDigestRequest {
                force: false,
                wait: false,
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert!(run.started, "a background run was started");
    assert!(!run.already_running);
    assert!(
        run.digests.is_empty(),
        "a background run returns before it writes"
    );
    assert!(run.week.contains("-W"));
}
