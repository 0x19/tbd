//! Filings: an ePorezna form uploaded as XML is read, listed with its figures,
//! and refused when it is not the party's.

use sqlx::PgPool;
use tbd_db::{Capability, PartyId, UserId, create_org, ensure_user, grant};
use tbd_proto::finance::v1::{
    ExtractDocumentRequest, GetFilingRequest, ListDocumentsRequest, ListFilingsRequest,
    UpdateDocumentRequest, UploadDocumentRequest,
};
use tonic::{Code, Request, metadata::MetadataValue};
use uuid::Uuid;

use crate::support::start_with_store;

const OWNER: &str = "filing-owner";
const READER: &str = "filing-reader";
const PD_2025: &str = include_str!("../fixtures/filings/pd-2025.xml");

fn as_caller<T>(subject: &str, message: T) -> Request<T> {
    let claims = serde_json::json!({ "sub": subject, "scp": ["tbd.finance"] });
    let encoded = base64::Engine::encode(
        &base64::engine::general_purpose::URL_SAFE_NO_PAD,
        claims.to_string(),
    );
    let mut request = Request::new(message);
    request.metadata_mut().insert(
        "x-jwt-payload",
        MetadataValue::try_from(encoded.as_str()).unwrap(),
    );
    request
}

struct World {
    person: Uuid,
    company: Uuid,
}

/// An owner of a person and of the company whose OIB the fixture carries; a
/// reader granted the company only.
async fn seed(pool: &PgPool) -> World {
    let owner: UserId = ensure_user(pool, OWNER, None, "Owner").await.unwrap();
    let reader = ensure_user(pool, READER, None, "Reader").await.unwrap();
    let company = create_org(
        pool,
        "INORBIT d.o.o.",
        Some("38846238650"),
        Some("HR"),
        true,
    )
    .await
    .unwrap();
    for party in [PartyId(owner.0), PartyId(company.0)] {
        grant(pool, owner, party, Capability::Own, Some(owner), None)
            .await
            .unwrap();
    }
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
    World {
        person: owner.0,
        company: company.0,
    }
}

fn upload(party: Uuid, filename: &str, content_type: &str, bytes: &[u8]) -> UploadDocumentRequest {
    UploadDocumentRequest {
        party_id: party.to_string(),
        filename: filename.into(),
        content_type: content_type.into(),
        bytes: bytes.to_vec(),
    }
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn an_uploaded_pd_is_a_filing_of_the_oibs_company() {
    let (server, pool) = start_with_store().await;
    let w = seed(&pool).await;
    let mut client = server.client().await;

    let doc = client
        .upload_document(as_caller(
            OWNER,
            upload(w.company, "PD 2025.xml", "text/xml", PD_2025.as_bytes()),
        ))
        .await
        .unwrap()
        .into_inner()
        .document
        .unwrap();
    assert_eq!(doc.kind, "filing");
    assert_eq!(doc.filename, "PD 2025.xml");
    assert!(!doc.extracted_at.is_empty(), "the reader ran on upload");
    assert_eq!(
        doc.found_by.get("engine").map(String::as_str),
        Some("filings/2")
    );
    assert!(
        doc.vendor.is_empty() && doc.total_minor.is_empty(),
        "a form has no receipt fields"
    );

    // Listed under its form and year, with the headline figures.
    let listing = client
        .list_filings(as_caller(
            OWNER,
            ListFilingsRequest {
                party_ids: vec![],
                form: "pd".into(),
                year: 2025,
                limit: 0,
                offset: 0,
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(listing.total, 1);
    assert_eq!(listing.years, vec![2025]);
    let f = &listing.filings[0];
    assert_eq!(f.id, doc.id);
    assert_eq!(f.form, "pd");
    assert_eq!(f.schema, "ObrazacPD-v9-0");
    assert_eq!(f.period_from, "2025-01-01");
    assert_eq!(f.period_to, "2025-12-31");
    assert_eq!(f.oib, "38846238650");
    assert_eq!(f.obveznik, "INORBIT d.o.o.");
    assert_eq!(f.headline.get("44").map(String::as_str), Some("12140.65"));
    assert_eq!(f.headline.get("57").map(String::as_str), Some("4689.04"));
    assert!(
        f.values.is_empty() && f.rows_json.is_empty(),
        "the listing is light"
    );
    assert!(f.error.is_empty());
    assert!(!f.prepared_at.is_empty());

    // The whole form on request, matching the figures the accountant filed.
    let full = client
        .get_filing(as_caller(OWNER, GetFilingRequest { id: doc.id.clone() }))
        .await
        .unwrap()
        .into_inner()
        .filing
        .unwrap();
    assert_eq!(full.values.get("1").map(String::as_str), Some("177733.47"));
    assert_eq!(full.values.get("59").map(String::as_str), Some("1011.72"));
    assert_eq!(
        full.values.get("DO00").map(String::as_str),
        Some("121406.54")
    );
    let rows: serde_json::Value = serde_json::from_str(&full.rows_json).unwrap();
    assert_eq!(
        rows,
        serde_json::json!([]),
        "the fixture names no donation recipient"
    );
    let csv = include_str!("../../../../docs/accountant/expected/pd.csv");
    for line in csv.lines().skip(1) {
        let (row, amount) = line.split_once(',').unwrap();
        let key = tbd_finance::filings::form::pd_csv_key(row).unwrap();
        assert_eq!(
            tbd_finance::filings::minor(&full.values[key]).unwrap(),
            tbd_finance::filings::minor(amount).unwrap(),
            "row {row}"
        );
    }

    // Not a receipt: the receipts listing does not show it, the documents
    // listing with no kind does, and search reaches its summary.
    let receipts = client
        .list_documents(as_caller(
            OWNER,
            ListDocumentsRequest {
                kind: "receipt".into(),
                ..Default::default()
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(receipts.total, 0);
    let any = client
        .list_documents(as_caller(
            OWNER,
            ListDocumentsRequest {
                q: "ObrazacPD-v9-0".into(),
                ..Default::default()
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(any.total, 1);

    // Re-read: the same row, refreshed, not a second one.
    client
        .extract_document(as_caller(
            OWNER,
            ExtractDocumentRequest { id: doc.id.clone() },
        ))
        .await
        .unwrap();
    let (count,): (i64,) = sqlx::query_as("select count(*) from finance.filings")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 1);

    // The same bytes again are the same document.
    let again = client
        .upload_document(as_caller(
            OWNER,
            upload(w.company, "copy.xml", "application/xml", PD_2025.as_bytes()),
        ))
        .await
        .unwrap()
        .into_inner()
        .document
        .unwrap();
    assert_eq!(again.id, doc.id);

    // The reader sees it too, through the grant.
    let seen = client
        .list_filings(as_caller(READER, ListFilingsRequest::default()))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(seen.total, 1);
    let mine = client
        .get_filing(as_caller(READER, GetFilingRequest { id: doc.id.clone() }))
        .await
        .unwrap()
        .into_inner()
        .filing
        .unwrap();
    assert_eq!(mine.id, doc.id);
}

#[tokio::test]
async fn a_filing_for_another_oib_or_a_person_is_refused() {
    let (server, pool) = start_with_store().await;
    let w = seed(&pool).await;
    let mut client = server.client().await;

    let foreign = PD_2025.replace("<OIB>38846238650</OIB>", "<OIB>00000000001</OIB>");
    let e = client
        .upload_document(as_caller(
            OWNER,
            upload(w.company, "other.xml", "text/xml", foreign.as_bytes()),
        ))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::InvalidArgument);
    assert!(
        e.message().contains("00000000001") && e.message().contains("38846238650"),
        "names both OIBs: {}",
        e.message()
    );

    let e = client
        .upload_document(as_caller(
            OWNER,
            upload(w.person, "mine.xml", "text/xml", PD_2025.as_bytes()),
        ))
        .await
        .unwrap_err();
    assert_eq!(
        e.code(),
        Code::InvalidArgument,
        "a person has no OIB to match"
    );

    for (name, content_type, bytes) in [
        ("page.html", "text/html", PD_2025.as_bytes()),
        ("not.xml", "text/xml", b"%PDF-1.4 not xml".as_slice()),
        ("html.xml", "text/xml", b"<html><body/></html>".as_slice()),
        ("empty.xml", "application/xml", b"".as_slice()),
    ] {
        let e = client
            .upload_document(as_caller(
                OWNER,
                upload(w.company, name, content_type, bytes),
            ))
            .await
            .unwrap_err();
        assert_eq!(e.code(), Code::InvalidArgument, "{name}: {}", e.message());
    }
    let (count,): (i64,) = sqlx::query_as("select count(*) from finance.documents")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0, "nothing refused was stored");

    // A stored filing cannot be moved to another party.
    let doc = client
        .upload_document(as_caller(
            OWNER,
            upload(w.company, "PD.xml", "text/xml", PD_2025.as_bytes()),
        ))
        .await
        .unwrap()
        .into_inner()
        .document
        .unwrap();
    let e = client
        .update_document(as_caller(
            OWNER,
            UpdateDocumentRequest {
                id: doc.id.clone(),
                party_id: w.person.to_string(),
                ..Default::default()
            },
        ))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::InvalidArgument);

    // A reader may upload for the company too, as with receipts: the same
    // bytes are the same document, whoever hands them in.
    let same = client
        .upload_document(as_caller(
            READER,
            upload(w.company, "PD.xml", "text/xml", PD_2025.as_bytes()),
        ))
        .await
        .unwrap()
        .into_inner()
        .document
        .unwrap();
    assert_eq!(same.id, doc.id);
    let e = client
        .get_filing(as_caller("nobody", GetFilingRequest { id: doc.id }))
        .await
        .unwrap_err();
    assert_eq!(
        e.code(),
        Code::NotFound,
        "outside the grant, it does not exist"
    );
}
