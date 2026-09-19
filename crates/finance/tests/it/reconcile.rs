//! A month for the accountant over gRPC: every transaction sorted into what
//! is needed, receipts matched to card charges, a person's policy and link
//! overriding the rules, and a stranger's party not found.

use base64::Engine as _;
use serde_json::json;
use sqlx::PgPool;
use tbd_db::{Capability, PartyId, create_org, ensure_user, grant};
use tbd_proto::finance::v1::{
    LinkDocumentRequest, MonthlyReconciliationRequest, MonthlyReconciliationResponse,
    SetCounterpartyPolicyRequest, SetTransactionNoteRequest, UnlinkDocumentRequest,
    finance_service_client::FinanceServiceClient,
};
use tonic::{Code, Request, metadata::MetadataValue, transport::Channel};
use uuid::Uuid;

use crate::support::start_with_store;

const OWNER: &str = "recon-owner";
const STRANGER: &str = "recon-stranger";

fn as_caller<T>(subject: &str, message: T) -> Request<T> {
    let claims = json!({ "sub": subject, "scp": ["tbd.finance"] });
    let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(claims.to_string());
    let mut request = Request::new(message);
    request
        .metadata_mut()
        .insert("x-jwt-payload", MetadataValue::try_from(encoded).unwrap());
    request
}

async fn seed(pool: &PgPool) -> (Uuid, Uuid) {
    let owner = ensure_user(pool, OWNER, None, "Owner").await.unwrap();
    ensure_user(pool, STRANGER, None, "Stranger").await.unwrap();
    let company = create_org(pool, "Inorbit d.o.o.", None, Some("HR"), true)
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
    let account = Uuid::new_v4();
    sqlx::query(
        "insert into finance.accounts (id, party_id, iban, currency, name) values ($1,$2,'HR9224020061100925189','EUR','biz')",
    )
    .bind(account)
    .bind(company.0)
    .execute(pool)
    .await
    .unwrap();
    (company.0, account)
}

#[allow(clippy::too_many_arguments)]
async fn txn(
    pool: &PgPool,
    party: Uuid,
    account: Uuid,
    date: &str,
    minor: i64,
    who: &str,
    iban: Option<&str>,
    remittance: &str,
    reference: &str,
) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "insert into finance.bank_transactions
            (id, party_id, account_id, status, dedup_key, amount_minor, currency, credit_debit,
             booking_date, counterparty_name, counterparty_iban, remittance, reference_number)
         values ($1,$2,$3,'booked',$4,$5,'EUR',$6,$7::date,$8,$9,$10,$11)",
    )
    .bind(id)
    .bind(party)
    .bind(account)
    .bind(Uuid::new_v4().as_bytes().to_vec())
    .bind(minor)
    .bind(if minor < 0 { "DBIT" } else { "CRDT" })
    .bind(date)
    .bind(who)
    .bind(iban)
    .bind(remittance)
    .bind(reference)
    .execute(pool)
    .await
    .unwrap();
    id
}

async fn receipt(
    pool: &PgPool,
    party: Uuid,
    vendor: &str,
    date: &str,
    minor: i64,
    currency: &str,
) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "insert into finance.documents (id, party_id, kind, sha256, content_type, size_bytes, filename,
                                        vendor, doc_date, total_minor, currency, extracted_at)
         values ($1,$2,'receipt',$3,'application/pdf',10,$4,$5,$6::date,$7,$8,now())",
    )
    .bind(id)
    .bind(party)
    .bind(Uuid::new_v4().simple().to_string())
    .bind(format!("{vendor}.pdf"))
    .bind(vendor)
    .bind(date)
    .bind(minor)
    .bind(currency)
    .execute(pool)
    .await
    .unwrap();
    id
}

/// August as the statement had it: a foreign card charge, a domestic
/// supplier, a tax payment, a cash withdrawal, our invoice paid, and a games
/// console. Returns the card charge and the console.
async fn august(pool: &PgPool, company: Uuid, account: Uuid) -> (Uuid, Uuid) {
    let card = "HR99 | 424472XXXXXX9355, OPENAI *CHATGPT SUBSCR 75,00 USD,  30.08.2026";
    let openai = txn(
        pool,
        company,
        account,
        "2026-08-30",
        -6636,
        "OPENAI *CHATGPT SUBSCR",
        None,
        card,
        "HR99",
    )
    .await;
    txn(
        pool,
        company,
        account,
        "2026-08-14",
        -12981,
        "HT D.D. - UPLATNI RAČUN",
        Some("HR6023400091110000000"),
        "HR01 1183539863 | RAČUN ZA HT USLUGE ZA 07/26",
        "HR01 1183539863",
    )
    .await;
    txn(
        pool,
        company,
        account,
        "2026-08-03",
        -22397,
        "DRZAVNI PRORACUN REPUBLIKE HRV",
        Some("HR1210010051863000160"),
        "HR68 8168-38846238650-26215 | Placa za 7 / 2026",
        "HR68 8168-38846238650-26215",
    )
    .await;
    txn(
        pool,
        company,
        account,
        "2026-08-16",
        -100_000,
        "ERSTE ATM  ZG-N. ZAGREB",
        None,
        "HR99 | 424472XXXXXX9355, ERSTE ATM  ZG-N. ZAGREB",
        "HR99",
    )
    .await;
    txn(
        pool,
        company,
        account,
        "2026-08-06",
        2_502_167,
        "TENDERLY D.O.O.",
        Some("RS35265100000012345678"),
        "HR99 | BROJ RACUNA 8-1-1-2026",
        "HR99",
    )
    .await;
    let playstation = txn(
        pool,
        company,
        account,
        "2026-08-23",
        -3999,
        "PLAYSTATION",
        None,
        "HR99 | 424472XXXXXX9355, PLAYSTATION Hilversum",
        "HR99",
    )
    .await;
    (openai, playstation)
}

async fn month(server: &crate::support::Server, party: Uuid) -> MonthlyReconciliationResponse {
    server
        .client()
        .await
        .monthly_reconciliation(as_caller(
            OWNER,
            MonthlyReconciliationRequest {
                party_id: party.to_string(),
                month: "2026-08".into(),
            },
        ))
        .await
        .unwrap()
        .into_inner()
}

#[tokio::test]
async fn a_month_is_sorted_and_card_charges_find_their_receipts() {
    let (server, pool) = start_with_store().await;
    let (company, account) = seed(&pool).await;
    let (openai, playstation) = august(&pool, company, account).await;
    let receipt_openai = receipt(&pool, company, "OpenAI", "2026-08-29", 7500, "USD").await;
    receipt(&pool, company, "Hetzner", "2026-08-13", 6948, "EUR").await;

    let r = month(&server, company).await;
    let s = r.summary.clone().unwrap();
    assert_eq!(
        (
            s.transactions,
            s.eracun,
            s.none,
            s.income,
            s.receipt_covered,
            s.receipt_missing,
            s.personal
        ),
        (6, 1, 2, 1, 1, 1, 0),
        "{s:?}"
    );
    assert_eq!(s.missing_minor.get("EUR").map(String::as_str), Some("3999"));
    let row = |r: &MonthlyReconciliationResponse, id: Uuid| {
        r.rows
            .iter()
            .find(|x| x.transaction.as_ref().unwrap().id == id.to_string())
            .cloned()
            .unwrap()
    };
    let o = row(&r, openai);
    assert_eq!((o.need.as_str(), o.status.as_str()), ("receipt", "covered"));
    assert_eq!(o.need_reason, "card, charged 75,00 USD");
    assert_eq!(
        (
            o.original_amount_minor.as_str(),
            o.original_currency.as_str()
        ),
        ("7500", "USD")
    );
    assert_eq!(o.documents.len(), 1);
    assert_eq!(o.documents[0].source, "inferred");
    assert!(
        o.documents[0].reason.contains("amount 75,00 USD"),
        "{}",
        o.documents[0].reason
    );
    let p = row(&r, playstation);
    assert_eq!((p.need.as_str(), p.status.as_str()), ("receipt", "missing"));
    assert!(
        p.suggestions.is_empty(),
        "nothing fits a games console: {:?}",
        p.suggestions
    );

    // A second look makes no second link, and the receipt stays where it is.
    let again = month(&server, company).await;
    assert_eq!(row(&again, openai).documents.len(), 1);
    assert_eq!(
        row(&again, openai).documents[0].document_id,
        receipt_openai.to_string()
    );
}

#[tokio::test]
async fn a_policy_and_a_hand_made_link_override_the_rules() {
    let (server, pool) = start_with_store().await;
    let (company, account) = seed(&pool).await;
    let mut c = server.client().await;
    let (openai, playstation) = august(&pool, company, account).await;
    let doc = receipt(&pool, company, "OpenAI", "2026-08-29", 7500, "USD").await;
    let other = receipt(&pool, company, "Something", "2026-08-20", 100, "EUR").await;

    let policy = c
        .set_counterparty_policy(as_caller(
            OWNER,
            SetCounterpartyPolicyRequest {
                party_id: company.to_string(),
                r#match: "playstation".into(),
                exact: false,
                policy: "personal".into(),
                note: "not a business cost".into(),
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .policy
        .unwrap();
    assert_eq!(policy.r#match, "PLAYSTATION", "stored normalised");
    let r = month(&server, company).await;
    let p = r
        .rows
        .iter()
        .find(|x| x.transaction.as_ref().unwrap().id == playstation.to_string())
        .unwrap();
    assert_eq!(
        (p.need.as_str(), p.policy_id.as_str()),
        ("personal", policy.id.as_str())
    );
    assert_eq!(r.summary.unwrap().personal, 1);
    assert_eq!(r.policies.len(), 1);

    // The matcher linked OpenAI; a person undoes it, and it stays undone.
    let undone = c
        .unlink_document(as_caller(
            OWNER,
            UnlinkDocumentRequest {
                transaction_id: openai.to_string(),
                document_id: doc.to_string(),
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .row
        .unwrap();
    assert_eq!(undone.status, "missing");
    let r = month(&server, company).await;
    let o = r
        .rows
        .iter()
        .find(|x| x.transaction.as_ref().unwrap().id == openai.to_string())
        .unwrap();
    assert_eq!(o.status, "missing", "an undone match is not remade");

    note_round_trip(&mut c, &server, company, openai).await;

    // Linked by hand to another receipt whose amount is not this charge's:
    // refused, saying what was read, until the person says anyway.
    let refused = c
        .link_document(as_caller(
            OWNER,
            LinkDocumentRequest {
                transaction_id: openai.to_string(),
                document_id: other.to_string(),
                force: false,
            },
        ))
        .await
        .unwrap_err();
    assert_eq!(refused.code(), Code::FailedPrecondition, "{refused}");
    assert!(
        refused.message().contains("1,00 EUR") && refused.message().contains("75,00 USD"),
        "{}",
        refused.message()
    );
    let linked = c
        .link_document(as_caller(
            OWNER,
            LinkDocumentRequest {
                transaction_id: openai.to_string(),
                document_id: other.to_string(),
                force: true,
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .row
        .unwrap();
    assert_eq!(linked.status, "covered");
    assert_eq!(linked.documents[0].source, "declared");
    assert_eq!(linked.documents[0].confidence, 100);
    assert_eq!(
        linked.documents[0].reason,
        "linked by hand · attached against what was read"
    );
}

#[tokio::test]
async fn a_stranger_finds_nothing_and_a_bad_month_is_refused() {
    let (server, pool) = start_with_store().await;
    let (company, account) = seed(&pool).await;
    let mut c = server.client().await;
    let (openai, _) = august(&pool, company, account).await;
    let doc = receipt(&pool, company, "OpenAI", "2026-08-29", 7500, "USD").await;
    // A stranger's party is not found, and so is a stranger's link attempt.
    let refused = c
        .monthly_reconciliation(as_caller(
            STRANGER,
            MonthlyReconciliationRequest {
                party_id: company.to_string(),
                month: "2026-08".into(),
            },
        ))
        .await
        .unwrap_err();
    assert_eq!(refused.code(), Code::NotFound);
    let refused = c
        .link_document(as_caller(
            STRANGER,
            LinkDocumentRequest {
                transaction_id: openai.to_string(),
                document_id: doc.to_string(),
                force: false,
            },
        ))
        .await
        .unwrap_err();
    assert_eq!(refused.code(), Code::NotFound);
    let bad = c
        .monthly_reconciliation(as_caller(
            OWNER,
            MonthlyReconciliationRequest {
                party_id: company.to_string(),
                month: "August".into(),
            },
        ))
        .await
        .unwrap_err();
    assert_eq!(bad.code(), Code::FailedPrecondition);
}

#[tokio::test]
async fn a_hand_link_is_checked_against_the_charge_and_says_so() {
    let (server, pool) = start_with_store().await;
    let (company, account) = seed(&pool).await;
    let mut client = server.client().await;
    let (openai, _) = august(&pool, company, account).await;
    let doc = receipt(&pool, company, "OpenAI", "2026-08-29", 7500, "USD").await;
    let by = |row: &tbd_proto::finance::v1::ReconciliationRow, id: Uuid| {
        row.documents
            .iter()
            .find(|d| d.document_id == id.to_string())
            .map(|d| d.reason.clone())
            .unwrap_or_default()
    };
    // The right receipt links without a word, and says it was checked; one
    // nothing was read from is linked and says so.
    let checked = client
        .link_document(as_caller(
            OWNER,
            LinkDocumentRequest {
                transaction_id: openai.to_string(),
                document_id: doc.to_string(),
                force: false,
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .row
        .unwrap();
    assert_eq!(
        by(&checked, doc),
        "linked by hand · amount checked against the charge"
    );
    let photo = Uuid::new_v4();
    sqlx::query(
        "insert into finance.documents (id, party_id, kind, sha256, content_type, size_bytes, filename, extracted_at)
         values ($1, $2, 'receipt', $3, 'image/jpeg', 10, 'till.jpg', now())",
    )
    .bind(photo)
    .bind(company)
    .bind(Uuid::new_v4().simple().to_string())
    .execute(&pool)
    .await
    .unwrap();
    let unread = client
        .link_document(as_caller(
            OWNER,
            LinkDocumentRequest {
                transaction_id: openai.to_string(),
                document_id: photo.to_string(),
                force: false,
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .row
        .unwrap();
    assert_eq!(by(&unread, photo), "linked by hand · nothing read to check");
}

/// A note for the accountant on the row that is missing its receipt: it
/// comes back with the row, with the month, and empty removes it.
async fn note_round_trip(
    client: &mut FinanceServiceClient<Channel>,
    server: &crate::support::Server,
    company: Uuid,
    openai: Uuid,
) {
    let note_of = |text: String| SetTransactionNoteRequest {
        transaction_id: openai.to_string(),
        note: text,
    };
    let noted = client
        .set_transaction_note(as_caller(
            OWNER,
            note_of("  OpenAI ne šalje račun; tražen preko portala.  ".into()),
        ))
        .await
        .unwrap()
        .into_inner()
        .row
        .unwrap();
    assert_eq!(noted.note, "OpenAI ne šalje račun; tražen preko portala.");
    assert_eq!(noted.status, "missing", "a note changes nothing else");
    let listed = month(server, company).await;
    let row = listed
        .rows
        .iter()
        .find(|x| x.transaction.as_ref().unwrap().id == openai.to_string())
        .unwrap();
    assert_eq!(row.note, "OpenAI ne šalje račun; tražen preko portala.");
    let e = client
        .set_transaction_note(as_caller(OWNER, note_of("x".repeat(2001))))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::FailedPrecondition, "{e}");
    let cleared = client
        .set_transaction_note(as_caller(OWNER, note_of("   ".into())))
        .await
        .unwrap()
        .into_inner()
        .row
        .unwrap();
    assert_eq!(cleared.note, "");
}
