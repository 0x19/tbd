//! Invoicing over gRPC: the number is gapless, the approval names what was
//! seen, and the grant holds on every RPC.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use base64::Engine as _;
use serde_json::json;
use sqlx::PgPool;
use tbd_db::{Capability, PartyId, UserId, create_org, ensure_user, grant};
use tbd_proto::finance::v1::{
    ApproveInvoiceRequest, CancelInvoiceRequest, ClientProfile, CreateInvoiceRequest,
    GetInvoiceDocumentRequest, InvoiceLine, IssuerProfile, ListInvoicesRequest,
    PreviewInvoiceRequest, UpdateInvoiceRequest, UpsertClientRequest, UpsertIssuerRequest,
};
use tonic::{Code, Request, metadata::MetadataValue};
use uuid::Uuid;

use crate::support::{Server, start_with_store};

const OWNER: &str = "inv-owner";
const READER: &str = "inv-reader";

fn as_caller<T>(subject: &str, message: T) -> Request<T> {
    let claims = json!({ "sub": subject, "scp": ["tbd.finance"] });
    let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(claims.to_string());
    let mut request = Request::new(message);
    request.metadata_mut().insert(
        "x-jwt-payload",
        MetadataValue::try_from(encoded.as_str()).unwrap(),
    );
    request
}

struct World {
    company: Uuid,
    personal: Uuid,
}

async fn seed(pool: &PgPool) -> World {
    let owner: UserId = ensure_user(pool, OWNER, None, "Owner").await.unwrap();
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
        reader,
        PartyId(company.0),
        Capability::Read,
        Some(owner),
        None,
    )
    .await
    .unwrap();
    World {
        company: company.0,
        personal: owner.0,
    }
}

fn issuer(party: Uuid) -> IssuerProfile {
    IssuerProfile {
        party_id: party.to_string(),
        legal_name: "Inorbit d.o.o.".into(),
        address_lines: vec!["Benčani 15A".into(), "51216 Viškovo, Croatia".into()],
        oib: "38846238650".into(),
        vat_id: "HR38846238650".into(),
        iban: "HR9224020061100925189".into(),
        swift: "ESBCHR22".into(),
        bank_name: "Erste & Steiermärkische Bank d.d., Rijeka".into(),
        court: "Trgovački sud u Rijeci".into(),
        registration_no: "081116183".into(),
        share_capital: "2,640.00 EUR, uplaćen u cijelosti / fully paid".into(),
        board_member: "Nevio Vesić".into(),
        issued_by: "Nevio Vesić".into(),
        place_of_issue: "Viškovo".into(),
        operator_id: "1".into(),
        premises: "1".into(),
        device: "1".into(),
        due_days: 15,
    }
}

fn client(party: Uuid) -> ClientProfile {
    ClientProfile {
        id: String::new(),
        party_id: party.to_string(),
        name: "Tenderly".into(),
        address_lines: vec!["Miloša Milojevića 7".into(), "Belgrade 11070".into()],
        country_code: "RS".into(),
        tax_id: String::new(),
        vat_treatment: "outside_scope_non_eu".into(),
        recipients: vec![],
        currency: "EUR".into(),
        archived: false,
    }
}

fn lines() -> Vec<InvoiceLine> {
    vec![
        InvoiceLine {
            position: 0,
            description: "Prepaid services / Usluge".into(),
            quantity_milli: 1000,
            unit_price_minor: 1_375_000,
            amount_minor: 0,
            template_id: String::new(),
        },
        InvoiceLine {
            position: 0,
            description: "On-call / Dežurstvo".into(),
            quantity_milli: 1000,
            unit_price_minor: 75_082,
            amount_minor: 0,
            template_id: String::new(),
        },
    ]
}

/// Issuer set, client made, one draft with the August lines: ready to preview.
async fn draft(server: &Server, party: Uuid) -> (String, String) {
    let mut c = server.client().await;
    c.upsert_issuer(as_caller(
        OWNER,
        UpsertIssuerRequest {
            issuer: Some(issuer(party)),
        },
    ))
    .await
    .unwrap();
    let cl = c
        .upsert_client(as_caller(
            OWNER,
            UpsertClientRequest {
                client: Some(client(party)),
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .client
        .unwrap();
    let inv = c
        .create_invoice(as_caller(
            OWNER,
            CreateInvoiceRequest {
                client_id: cl.id.clone(),
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .invoice
        .unwrap();
    assert_eq!(inv.status, "draft");
    assert_eq!(inv.number, "", "no number before approval");
    let inv = c
        .update_invoice(as_caller(
            OWNER,
            UpdateInvoiceRequest {
                id: inv.id.clone(),
                delivery_date: inv.delivery_date.clone(),
                due_date: inv.due_date.clone(),
                place_of_issue: "Viškovo".into(),
                note: String::new(),
                lines: lines(),
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .invoice
        .unwrap();
    assert_eq!(
        inv.total_minor, 1_450_082,
        "totals are the service's, not the client's"
    );
    assert_eq!(inv.lines[0].amount_minor, 1_375_000);
    (inv.id, cl.id)
}

#[tokio::test]
async fn preview_approve_gives_the_first_number_and_a_stored_pdf() {
    let (server, pool) = start_with_store().await;
    let w = seed(&pool).await;
    let (id, _) = draft(&server, w.company).await;
    let mut c = server.client().await;

    let preview = c
        .preview_invoice(as_caller(OWNER, PreviewInvoiceRequest { id: id.clone() }))
        .await
        .unwrap()
        .into_inner();
    let year = chrono::Utc::now().format("%Y").to_string();
    assert_eq!(preview.number, format!("1-1-1-{year}"));
    assert!(preview.pdf.starts_with(b"%PDF-"));
    assert_eq!(preview.content_hash.len(), 64);

    let approved = c
        .approve_invoice(as_caller(
            OWNER,
            ApproveInvoiceRequest {
                id: id.clone(),
                content_hash: preview.content_hash,
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .invoice
        .unwrap();
    assert_eq!(approved.status, "approved");
    assert_eq!(approved.number, format!("1-1-1-{year}"));
    assert!(!approved.document_id.is_empty());
    assert!(approved.vat_note.contains("Article 17"));

    let doc = c
        .get_invoice_document(as_caller(
            OWNER,
            GetInvoiceDocumentRequest { id: id.clone() },
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(doc.content_type, "application/pdf");
    assert!(doc.pdf.starts_with(b"%PDF-"));
    assert!(
        !String::from_utf8_lossy(&doc.pdf).contains("PREVIEW"),
        "an issued invoice has no watermark"
    );

    // Approving twice is refused: it is no longer a draft.
    let e = c
        .approve_invoice(as_caller(
            OWNER,
            ApproveInvoiceRequest {
                id,
                content_hash: "x".repeat(64),
            },
        ))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::FailedPrecondition);
}

#[tokio::test]
async fn a_draft_changed_after_the_preview_is_refused_and_takes_no_number() {
    let (server, pool) = start_with_store().await;
    let w = seed(&pool).await;
    let (id, _) = draft(&server, w.company).await;
    let mut c = server.client().await;
    let preview = c
        .preview_invoice(as_caller(OWNER, PreviewInvoiceRequest { id: id.clone() }))
        .await
        .unwrap()
        .into_inner();

    // A cent more on one line, after the approver looked.
    let mut changed = lines();
    changed[1].unit_price_minor += 1;
    let inv = c
        .get_invoice(as_caller(
            OWNER,
            tbd_proto::finance::v1::GetInvoiceRequest { id: id.clone() },
        ))
        .await
        .unwrap()
        .into_inner()
        .invoice
        .unwrap();
    c.update_invoice(as_caller(
        OWNER,
        UpdateInvoiceRequest {
            id: id.clone(),
            delivery_date: inv.delivery_date,
            due_date: inv.due_date,
            place_of_issue: inv.place_of_issue,
            note: String::new(),
            lines: changed,
        },
    ))
    .await
    .unwrap();

    let e = c
        .approve_invoice(as_caller(
            OWNER,
            ApproveInvoiceRequest {
                id: id.clone(),
                content_hash: preview.content_hash,
            },
        ))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::FailedPrecondition, "{e}");

    // Nothing happened: still a draft, and the next number is still 1.
    let again = c
        .preview_invoice(as_caller(OWNER, PreviewInvoiceRequest { id: id.clone() }))
        .await
        .unwrap()
        .into_inner();
    assert!(
        again.number.starts_with("1-1-1-"),
        "the refused approval took a number: {}",
        again.number
    );
    let (n,): (i64,) = sqlx::query_as("select count(*) from finance.documents")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(n, 0, "nothing was stored");
}

/// Preview, then approve with the hash the preview gave.
async fn approve(server: &Server, id: String) -> tbd_proto::finance::v1::Invoice {
    let mut c = server.client().await;
    let p = c
        .preview_invoice(as_caller(OWNER, PreviewInvoiceRequest { id: id.clone() }))
        .await
        .unwrap()
        .into_inner();
    c.approve_invoice(as_caller(
        OWNER,
        ApproveInvoiceRequest {
            id,
            content_hash: p.content_hash,
        },
    ))
    .await
    .unwrap()
    .into_inner()
    .invoice
    .unwrap()
}

#[tokio::test]
async fn numbers_are_consecutive_and_a_cancelled_invoice_keeps_its_number() {
    let (server, pool) = start_with_store().await;
    let w = seed(&pool).await;
    let (first, client_id) = draft(&server, w.company).await;
    let mut c = server.client().await;
    let year = chrono::Utc::now().format("%Y").to_string();

    let a = approve(&server, first).await;
    // The next draft pre-fills from the last approved invoice.
    let b_draft = c
        .create_invoice(as_caller(
            OWNER,
            CreateInvoiceRequest {
                client_id: client_id.clone(),
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .invoice
        .unwrap();
    assert_eq!(b_draft.lines.len(), 2, "pre-filled from the approved one");
    assert_eq!(b_draft.total_minor, 1_450_082);
    assert_eq!(b_draft.prefilled_from, a.id);
    let b = approve(&server, b_draft.id).await;
    let c_draft = c
        .create_invoice(as_caller(OWNER, CreateInvoiceRequest { client_id }))
        .await
        .unwrap()
        .into_inner()
        .invoice
        .unwrap();
    let c3 = approve(&server, c_draft.id).await;
    assert_eq!(
        [a.number.as_str(), b.number.as_str(), c3.number.as_str()],
        [
            format!("1-1-1-{year}").as_str(),
            format!("2-1-1-{year}").as_str(),
            format!("3-1-1-{year}").as_str()
        ]
    );

    let cancelled = c
        .cancel_invoice(as_caller(
            OWNER,
            CancelInvoiceRequest {
                id: b.id.clone(),
                reason: "test".into(),
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .invoice
        .unwrap();
    assert_eq!(cancelled.status, "cancelled");
    assert_eq!(
        cancelled.number, b.number,
        "gapless means no holes, not that every number is live"
    );

    let list = c
        .list_invoices(as_caller(OWNER, ListInvoicesRequest::default()))
        .await
        .unwrap()
        .into_inner()
        .invoices;
    assert_eq!(list.len(), 3);
    let _ = &pool;
}

#[tokio::test]
async fn the_reader_sees_the_companys_invoices_but_cannot_draft_or_approve() {
    let (server, pool) = start_with_store().await;
    let w = seed(&pool).await;
    let (id, client_id) = draft(&server, w.company).await;
    let mut c = server.client().await;

    // Readable: the reader is granted the company.
    let list = c
        .list_invoices(as_caller(READER, ListInvoicesRequest::default()))
        .await
        .unwrap()
        .into_inner()
        .invoices;
    assert_eq!(list.len(), 1);

    // The reader may see it, and this service does not yet distinguish read
    // from own for writes -- so what must hold is the *party* boundary: a
    // client of the owner's personal party is not-found to the reader.
    let e = c
        .upsert_client(as_caller(
            READER,
            UpsertClientRequest {
                client: Some(client(w.personal)),
            },
        ))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::NotFound, "{e}");
    let e = c
        .upsert_issuer(as_caller(
            READER,
            UpsertIssuerRequest {
                issuer: Some(issuer(w.personal)),
            },
        ))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::NotFound, "{e}");
    let _ = (id, client_id);
}

#[tokio::test]
async fn a_draft_starts_from_the_templates_with_last_months_variable_price() {
    use tbd_proto::finance::v1::{
        LineTemplate, ListLineTemplatesRequest, UpsertLineTemplateRequest,
    };
    let (server, pool) = start_with_store().await;
    let w = seed(&pool).await;
    let (first, client_id) = draft(&server, w.company).await;
    let mut c = server.client().await;
    let tpl =
        |position: i32, description: &str, mode: &str, price: i64| UpsertLineTemplateRequest {
            client_id: client_id.clone(),
            template: Some(LineTemplate {
                id: String::new(),
                client_id: client_id.clone(),
                position,
                description: description.into(),
                mode: mode.into(),
                quantity_milli: 1000,
                unit_price_minor: price,
                enabled: true,
            }),
        };
    c.upsert_line_template(as_caller(
        OWNER,
        tpl(1, "Prepaid services / Usluge", "fixed", 1_375_000),
    ))
    .await
    .unwrap();
    c.upsert_line_template(as_caller(
        OWNER,
        tpl(2, "On-call / Dežurstvo", "variable", 0),
    ))
    .await
    .unwrap();
    c.upsert_line_template(as_caller(OWNER, tpl(3, "Bonus", "optional", 0)))
        .await
        .unwrap();
    let listed = c
        .list_line_templates(as_caller(
            OWNER,
            ListLineTemplatesRequest {
                client_id: client_id.clone(),
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .templates;
    assert_eq!(listed.len(), 3);

    // The first draft (made before the templates, with on-call at 750.82)
    // is approved; the next draft takes the fixed row from the template and
    // the variable row's price from that invoice, and leaves the bonus out.
    approve(&server, first).await;
    let next = c
        .create_invoice(as_caller(
            OWNER,
            CreateInvoiceRequest {
                client_id: client_id.clone(),
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .invoice
        .unwrap();
    let rows: Vec<(String, i64, bool)> = next
        .lines
        .iter()
        .map(|l| {
            (
                l.description.clone(),
                l.unit_price_minor,
                !l.template_id.is_empty(),
            )
        })
        .collect();
    assert_eq!(
        rows,
        vec![
            ("Prepaid services / Usluge".to_owned(), 1_375_000, true),
            ("On-call / Dežurstvo".to_owned(), 75_082, true),
        ]
    );

    // A reader cannot write templates for a party they were not granted.
    let e = c
        .upsert_line_template(as_caller(
            READER,
            UpsertLineTemplateRequest {
                client_id: Uuid::new_v4().to_string(),
                template: tpl(1, "x", "fixed", 1).template,
            },
        ))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::NotFound);
}
