//! Whose a document is: decided from the account that paid it, the text,
//! or the mailbox, and settled for good by a person.

use sqlx::PgPool;
use tbd_db::{Capability, PartyId, UserId, create_org, ensure_user, grant};
use tbd_finance::documents::party::{self, By};
use tbd_proto::finance::v1::{ExtractDocumentRequest, UpdateDocumentRequest};
use tonic::{Code, Request, metadata::MetadataValue};
use uuid::Uuid;

use crate::support::start_with_store;

const OWNER: &str = "doc-owner";
const READER: &str = "doc-reader";

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
    person_account: Uuid,
    company_account: Uuid,
}

/// One owner of a person and a company; a reader granted the company only.
async fn seed(pool: &PgPool) -> World {
    let owner: UserId = ensure_user(pool, OWNER, None, "Nevio Vesic").await.unwrap();
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
    for (user, party) in [(owner, PartyId(owner.0)), (owner, PartyId(company.0))] {
        grant(pool, user, party, Capability::Own, Some(owner), None)
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
    let mut accounts = Vec::new();
    for (party, uid) in [(owner.0, "uid-p"), (company.0, "uid-c")] {
        let account = Uuid::new_v4();
        sqlx::query(
            "insert into finance.accounts (id, party_id, provider, provider_uid, currency, name)
             values ($1, $2, 'mock', $3, 'EUR', 'acct')",
        )
        .bind(account)
        .bind(party)
        .bind(uid)
        .execute(pool)
        .await
        .unwrap();
        accounts.push(account);
    }
    World {
        person: owner.0,
        company: company.0,
        person_account: accounts[0],
        company_account: accounts[1],
    }
}

async fn debit(pool: &PgPool, party: Uuid, account: Uuid, date: &str, minor: i64) {
    sqlx::query(
        "insert into finance.bank_transactions
            (id, party_id, account_id, status, dedup_key, amount_minor, currency, credit_debit, booking_date, counterparty_name)
         values ($1, $2, $3, 'booked', $4, $5, 'EUR', 'DBIT', $6::date, 'SHOP')",
    )
    .bind(Uuid::new_v4())
    .bind(party)
    .bind(account)
    .bind(Uuid::new_v4().as_bytes().to_vec())
    .bind(-minor.abs())
    .bind(date)
    .execute(pool)
    .await
    .unwrap();
}

/// A document already read: fields and text as the reader would have left them.
async fn document(
    pool: &PgPool,
    party: Uuid,
    date: Option<&str>,
    minor: Option<i64>,
    text: &str,
) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "insert into finance.documents (id, party_id, kind, sha256, content_type, size_bytes, filename,
                                        doc_date, total_minor, currency, text, extracted, extracted_at)
         values ($1, $2, 'receipt', $3, 'application/pdf', 10, 'r.pdf', $4::date, $5, 'EUR', $6, '{}'::jsonb, now())",
    )
    .bind(id)
    .bind(party)
    .bind(Uuid::new_v4().simple().to_string())
    .bind(date)
    .bind(minor)
    .bind(text)
    .execute(pool)
    .await
    .unwrap();
    sqlx::query("insert into finance.document_blobs (document_id, bytes) values ($1, $2)")
        .bind(id)
        .bind(b"not a pdf".to_vec())
        .execute(pool)
        .await
        .unwrap();
    id
}

async fn party_of(pool: &PgPool, id: Uuid) -> (Uuid, String) {
    sqlx::query_as(
        "select party_id, coalesce(extracted->>'party', '') from finance.documents where id = $1",
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .unwrap()
}

#[tokio::test]
async fn the_account_that_paid_decides_then_the_text_then_the_mailbox() {
    let (_s, pool) = start_with_store().await;
    let w = seed(&pool).await;

    // Arrived in the company's mailbox; paid from the personal account.
    let parts = document(
        &pool,
        w.company,
        Some("2026-08-25"),
        Some(1_314_413),
        "Receipt V/2026/01166",
    )
    .await;
    debit(&pool, w.person, w.person_account, "2026-08-27", 1_314_413).await;
    let moved = party::assign(&pool, parts).await.unwrap().unwrap();
    assert_eq!((moved.party_id, moved.by), (w.person, By::Payment));
    assert_eq!(party_of(&pool, parts).await, (w.person, "payment".into()));

    // Arrived in the personal mailbox; the company is billed, and both
    // accounts show the amount, so the text decides.
    let cloud = document(
        &pool,
        w.person,
        Some("2026-06-30"),
        Some(3_240),
        "Google Cloud EMEA\nBill to\nINORBIT d. o. o.\nNevio Vesić\nTotal in EUR €32.40",
    )
    .await;
    debit(&pool, w.person, w.person_account, "2026-07-01", 3_240).await;
    debit(&pool, w.company, w.company_account, "2026-07-01", 3_240).await;
    let moved = party::assign(&pool, cloud).await.unwrap().unwrap();
    assert_eq!((moved.party_id, moved.by), (w.company, By::Text));

    // A person's own name, and no payment anywhere: the person.
    let hotel = document(&pool, w.company, None, None, "Guest: Vesić Nevio\nRoom 12").await;
    let moved = party::assign(&pool, hotel).await.unwrap().unwrap();
    assert_eq!((moved.party_id, moved.by), (w.person, By::Text));

    // Nothing to go on: the mailbox's party stands, and says so.
    let blank = document(&pool, w.company, None, None, "Thanks for your order").await;
    let kept = party::assign(&pool, blank).await.unwrap().unwrap();
    assert_eq!((kept.party_id, kept.by), (w.company, By::Mailbox));
    assert_eq!(party_of(&pool, blank).await.1, "mailbox");
}

#[tokio::test]
async fn a_declared_party_is_final_and_only_within_the_grant() {
    let (server, pool) = start_with_store().await;
    let w = seed(&pool).await;
    let mut client = server.client().await;
    let doc = document(&pool, w.company, Some("2026-08-25"), Some(500), "Receipt").await;
    debit(&pool, w.person, w.person_account, "2026-08-25", 500).await;

    // The reader may see the company, not the person: moving it there is
    // not found, like everything else outside the grant.
    let e = client
        .update_document(as_caller(
            READER,
            UpdateDocumentRequest {
                id: doc.to_string(),
                party_id: w.person.to_string(),
                ..UpdateDocumentRequest::default()
            },
        ))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::NotFound, "{e}");

    // The owner says it is the company's, whatever the payment says.
    let fixed = client
        .update_document(as_caller(
            OWNER,
            UpdateDocumentRequest {
                id: doc.to_string(),
                party_id: w.company.to_string(),
                vendor: "Shop".into(),
                ..UpdateDocumentRequest::default()
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .document
        .unwrap();
    assert_eq!(fixed.party_id, w.company.to_string());
    assert_eq!(
        fixed.found_by.get("party").map(String::as_str),
        Some("declared")
    );
    assert!(
        party::assign(&pool, doc).await.unwrap().is_none(),
        "declared: nothing to decide"
    );
    let _ = client
        .extract_document(as_caller(
            OWNER,
            ExtractDocumentRequest {
                id: doc.to_string(),
            },
        ))
        .await
        .unwrap();
    assert_eq!(party_of(&pool, doc).await, (w.company, "declared".into()));
}
