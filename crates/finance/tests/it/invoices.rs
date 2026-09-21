//! Invoicing over gRPC: the number is gapless, the approval names what was
//! seen, and the grant holds on every RPC.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use base64::Engine as _;
use serde_json::json;
use sqlx::PgPool;
use tbd_db::{Capability, PartyId, UserId, create_org, ensure_user, grant};
use tbd_proto::finance::v1::{
    ApproveInvoiceRequest, CancelInvoiceRequest, ClientProfile, CreateInvoiceRequest,
    DeleteInvoiceRequest, GetInvoiceDocumentRequest, GetInvoiceRequest, InvoiceLine, IssuerProfile,
    ListClientsRequest, ListInvoicesRequest, PreviewInvoiceRequest, RecordPaymentRequest,
    SetDefaultClientRequest, UnlinkPaymentRequest, UpdateInvoiceRequest, UpsertClientRequest,
    UpsertIssuerRequest,
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
        address_lines: vec!["Milutina Milankovića 7đ".into(), "Belgrade 11070".into()],
        country_code: "RS".into(),
        tax_id: String::new(),
        vat_treatment: "outside_scope_non_eu".into(),
        recipients: vec![],
        currency: "EUR".into(),
        archived: false,
        is_default: false,
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
                ..Default::default()
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
                ..Default::default()
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
        .get_invoice(as_caller(OWNER, GetInvoiceRequest { id: id.clone() }))
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
            ..Default::default()
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
                ..Default::default()
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
        .create_invoice(as_caller(
            OWNER,
            CreateInvoiceRequest {
                client_id,
                ..Default::default()
            },
        ))
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
async fn the_reader_sees_the_companies_invoices_but_cannot_draft_or_approve() {
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
                ..Default::default()
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

/// A draft is deleted, not cancelled; its header may change while it is a
/// draft; a duplicate starts from any invoice; each series counts on its own.
#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn a_draft_is_deleted_its_header_edited_and_a_series_counts_alone() {
    let (server, pool) = start_with_store().await;
    let w = seed(&pool).await;
    let (first, client_id) = draft(&server, w.company).await;
    let mut c = server.client().await;
    let year = chrono::Utc::now().format("%Y").to_string();

    // A draft cannot be cancelled: it took no number, so it is deleted.
    let e = c
        .cancel_invoice(as_caller(
            OWNER,
            CancelInvoiceRequest {
                id: first.clone(),
                reason: "x".into(),
            },
        ))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::FailedPrecondition, "{e}");

    // The header of a draft: a second client of the same party, the
    // currency, the VAT treatment with its note, the series.
    let other = c
        .upsert_client(as_caller(
            OWNER,
            UpsertClientRequest {
                client: Some(ClientProfile {
                    name: "Bolt d.o.o.".into(),
                    country_code: "HR".into(),
                    vat_treatment: "standard_hr".into(),
                    ..client(w.company)
                }),
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .client
        .unwrap();
    let inv = c
        .get_invoice(as_caller(OWNER, GetInvoiceRequest { id: first.clone() }))
        .await
        .unwrap()
        .into_inner()
        .invoice
        .unwrap();
    let edited = c
        .update_invoice(as_caller(
            OWNER,
            UpdateInvoiceRequest {
                id: first.clone(),
                delivery_date: inv.delivery_date.clone(),
                due_date: inv.due_date.clone(),
                place_of_issue: "Rijeka".into(),
                note: String::new(),
                lines: lines(),
                client_id: other.id.clone(),
                currency: "usd".into(),
                vat_treatment: "standard_hr".into(),
                premises: "2".into(),
                device: "1".into(),
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .invoice
        .unwrap();
    assert_eq!(edited.client_id, other.id);
    assert_eq!(edited.currency, "USD");
    assert_eq!(edited.vat_treatment, "standard_hr");
    assert_eq!(
        edited.vat_minor, 362_521,
        "25 % of 1,450,082 rounded half up"
    );
    assert_eq!(edited.total_minor, 1_450_082 + 362_521);
    assert!(
        edited.vat_note.is_empty(),
        "standard VAT carries no exemption note"
    );

    // A bad treatment or currency is refused and changes nothing.
    let e = c
        .update_invoice(as_caller(
            OWNER,
            UpdateInvoiceRequest {
                id: first.clone(),
                delivery_date: inv.delivery_date.clone(),
                due_date: inv.due_date.clone(),
                lines: lines(),
                vat_treatment: "vat_free".into(),
                ..Default::default()
            },
        ))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::InvalidArgument, "{e}");
    let e = c
        .update_invoice(as_caller(
            OWNER,
            UpdateInvoiceRequest {
                id: first.clone(),
                delivery_date: inv.delivery_date.clone(),
                due_date: inv.due_date.clone(),
                lines: lines(),
                currency: "EURO".into(),
                ..Default::default()
            },
        ))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::InvalidArgument, "{e}");

    // Premises 2 numbers from 1 on its own; premises 1 too.
    let on_two = approve(&server, first.clone()).await;
    assert_eq!(on_two.number, format!("1-2-1-{year}"));
    let fresh = c
        .create_invoice(as_caller(
            OWNER,
            CreateInvoiceRequest {
                client_id: client_id.clone(),
                ..Default::default()
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .invoice
        .unwrap();
    // Tenderly has no issued invoice yet (the first went to Bolt), so the
    // draft starts empty; give it the lines.
    c.update_invoice(as_caller(
        OWNER,
        UpdateInvoiceRequest {
            id: fresh.id.clone(),
            delivery_date: fresh.delivery_date.clone(),
            due_date: fresh.due_date.clone(),
            lines: lines(),
            ..Default::default()
        },
    ))
    .await
    .unwrap();
    let on_one = approve(&server, fresh.id.clone()).await;
    assert_eq!(on_one.number, format!("1-1-1-{year}"), "its own counter");

    // A duplicate of an issued invoice: same header and lines, a draft dated
    // today, no number.
    let dup = c
        .create_invoice(as_caller(
            OWNER,
            CreateInvoiceRequest {
                from_invoice_id: on_two.id.clone(),
                ..Default::default()
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .invoice
        .unwrap();
    assert_eq!(dup.status, "draft");
    assert_eq!(dup.number, "");
    assert_eq!(dup.client_id, other.id);
    assert_eq!(dup.currency, "USD");
    assert_eq!(dup.vat_treatment, "standard_hr");
    assert_eq!(dup.place_of_issue, "Rijeka");
    assert_eq!(dup.lines.len(), 2);
    assert_eq!(dup.total_minor, on_two.total_minor);
    assert_eq!(dup.prefilled_from, on_two.id);

    // Deleting: a draft goes, an issued invoice does not.
    c.delete_invoice(as_caller(
        OWNER,
        DeleteInvoiceRequest { id: dup.id.clone() },
    ))
    .await
    .unwrap();
    let e = c
        .get_invoice(as_caller(OWNER, GetInvoiceRequest { id: dup.id.clone() }))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::NotFound, "{e}");
    let e = c
        .delete_invoice(as_caller(
            OWNER,
            DeleteInvoiceRequest {
                id: on_two.id.clone(),
            },
        ))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::FailedPrecondition, "{e}");
    let list = c
        .list_invoices(as_caller(OWNER, ListInvoicesRequest::default()))
        .await
        .unwrap()
        .into_inner()
        .invoices;
    assert_eq!(list.len(), 2, "two issued, the draft gone");
    let (n,): (i64,) = sqlx::query_as("select count(*) from finance.invoice_lines")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(n, 4, "the deleted draft's lines went with it");
}

/// The importer writes what was issued before this service: approved rows
/// with their printed numbers, the PDF as the document, the client made
/// once, the counter raised past them; a copy of a file is one invoice, two
/// different files claiming one number are both refused, a second run
/// changes nothing.
#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn issued_invoices_are_imported_from_their_pdfs_once() {
    use tbd_db::{Access, UserId};
    use tbd_finance::invoice::{
        VatTreatment,
        import::{Parsed, PrintedLine, apply},
    };
    let (server, pool) = start_with_store().await;
    let w = seed(&pool).await;
    let mut c = server.client().await;
    c.upsert_issuer(as_caller(
        OWNER,
        UpsertIssuerRequest {
            issuer: Some(issuer(w.company)),
        },
    ))
    .await
    .unwrap();
    let day = |y, m, d| chrono::NaiveDate::from_ymd_opt(y, m, d).unwrap();
    let parsed = |ordinal: i32, year: i32, client: &str, bytes: &[u8], source: &str| Parsed {
        source: source.into(),
        sha256: format!("{:x}", <sha2::Sha256 as sha2::Digest>::digest(bytes)),
        bytes: bytes.to_vec(),
        ordinal,
        premises: "1".into(),
        device: "1".into(),
        year,
        kind: "invoice".into(),
        issued_at: day(year, 3, 31).and_hms_opt(16, 2, 0).unwrap(),
        due_date: day(year, 4, 15),
        delivery_date: day(year, 3, 31),
        place_of_issue: "Viškovo".into(),
        client_name: client.into(),
        client_address: vec!["Helsinki".into()],
        client_country: "FI".into(),
        client_tax_id: "FI32746464".into(),
        currency: "EUR".into(),
        vat_treatment: VatTreatment::ReverseChargeEu,
        lines: vec![PrintedLine {
            description: format!("Software development services for {year}"),
            quantity_milli: 1000,
            unit_price_minor: 916_700,
            amount_minor: 916_700,
        }],
        subtotal_minor: 916_700,
        vat_minor: 0,
        total_minor: 916_700,
        note: String::new(),
    };
    let three = parsed(3, 2025, "Eiger Oy", b"%PDF three", "3.pdf");
    let mut three_copy = parsed(
        3,
        2025,
        "Eiger Oy",
        b"%PDF three with a receipt appended",
        "3-copy.pdf",
    );
    three_copy.lines[0].description = "Software development services for 2025".into();
    let four_a = parsed(4, 2025, "Eiger Oy", b"%PDF four a", "4a.pdf");
    let mut four_b = parsed(4, 2025, "Tenderly", b"%PDF four b", "4b.pdf");
    four_b.total_minor = 562_282;
    four_b.subtotal_minor = 562_282;
    four_b.lines[0].unit_price_minor = 562_282;
    four_b.lines[0].amount_minor = 562_282;
    let last_year = parsed(14, 2024, "Eiger Oy", b"%PDF fourteen", "14.pdf");
    let access = Access::for_parties(UserId(Uuid::nil()), vec![w.company]);

    let report = apply(
        &pool,
        &access,
        w.company,
        &[three, three_copy, four_a, four_b, last_year],
    )
    .await
    .unwrap();
    assert_eq!(report.imported, vec!["3-1-1-2025", "14-1-1-2024"]);
    assert!(report.present.is_empty());
    assert_eq!(report.refused.len(), 2, "{:?}", report.refused);
    assert!(
        report
            .refused
            .iter()
            .all(|(n, why)| n == "4-1-1-2025" && why.contains("2 files"))
    );

    let list = c
        .list_invoices(as_caller(OWNER, ListInvoicesRequest::default()))
        .await
        .unwrap()
        .into_inner()
        .invoices;
    assert_eq!(list.len(), 2);
    let three = list.iter().find(|i| i.number == "3-1-1-2025").unwrap();
    assert_eq!(three.status, "approved");
    assert_eq!(three.total_minor, 916_700);
    assert_eq!(three.vat_treatment, "reverse_charge_eu");
    assert!(!three.document_id.is_empty(), "the PDF is the document");
    let doc = c
        .get_invoice_document(as_caller(
            OWNER,
            GetInvoiceDocumentRequest {
                id: three.id.clone(),
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(doc.pdf, b"%PDF three", "the smaller of the two copies");
    let full = c
        .get_invoice(as_caller(
            OWNER,
            GetInvoiceRequest {
                id: three.id.clone(),
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .invoice
        .unwrap();
    assert_eq!(full.lines.len(), 1);
    assert_eq!(
        full.lines[0].description,
        "Software development services for 2025"
    );
    let (clients,): (i64,) =
        sqlx::query_as("select count(*) from finance.clients where party_id = $1")
            .bind(w.company)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(clients, 1, "one client made, reused");
    let (next,): (i32,) = sqlx::query_as(
        "select next_ordinal from finance.invoice_numbers where party_id = $1 and year = 2025",
    )
    .bind(w.company)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(next, 4, "the counter continues after the imported number");

    // A second run: everything already there, nothing written twice.
    let again = apply(
        &pool,
        &access,
        w.company,
        &[parsed(3, 2025, "Eiger Oy", b"%PDF three", "3.pdf")],
    )
    .await
    .unwrap();
    assert_eq!(again.present, vec!["3-1-1-2025"]);
    assert!(again.imported.is_empty());
    let (docs,): (i64,) =
        sqlx::query_as("select count(*) from finance.documents where kind = 'invoice'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(docs, 2);
}

/// Money in: an account of the company and a booked credit on it.
async fn credit(
    pool: &PgPool,
    party: Uuid,
    account: Uuid,
    date: &str,
    minor: i64,
    remittance: &str,
) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "insert into finance.bank_transactions
            (id, party_id, account_id, status, dedup_key, amount_minor, currency, credit_debit,
             booking_date, counterparty_name, remittance, reference_number)
         values ($1,$2,$3,'booked',$4,$5,'EUR','CRDT',$6::date,'TENDERLY D.O.O.',$7,'HR99')",
    )
    .bind(id)
    .bind(party)
    .bind(account)
    .bind(Uuid::new_v4().as_bytes().to_vec())
    .bind(minor)
    .bind(date)
    .bind(remittance)
    .execute(pool)
    .await
    .unwrap();
    id
}

/// A payment that names the invoice in the remittance settles it on the next
/// read; a part pays a part; undoing a match is remembered; a person's own
/// record stands; nothing but an issued invoice takes a payment.
#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn payments_settle_invoices_by_the_number_the_payer_wrote() {
    let (server, pool) = start_with_store().await;
    let world = seed(&pool).await;
    let account = Uuid::new_v4();
    sqlx::query(
        "insert into finance.accounts (id, party_id, iban, currency, name) values ($1,$2,'HR9224020061100925189','EUR','biz')",
    )
    .bind(account)
    .bind(world.company)
    .execute(&pool)
    .await
    .unwrap();
    let (first, client_id) = draft(&server, world.company).await;
    let mut c = server.client().await;
    let year = chrono::Utc::now().format("%Y").to_string();

    // A draft takes no payment.
    let e = c
        .record_payment(as_caller(
            OWNER,
            RecordPaymentRequest {
                invoice_id: first.clone(),
                amount_minor: 100,
                ..Default::default()
            },
        ))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::FailedPrecondition, "{e}");

    let first_invoice = approve(&server, first.clone()).await;
    assert_eq!(first_invoice.total_minor, 1_450_082);
    // The bank shows the settlement, the way Erste writes Tenderly's.
    let paid_tx = credit(
        &pool,
        world.company,
        account,
        "2026-09-04",
        1_450_082,
        &format!("HR99 | BROJ RACUNA 1-1-1-{year}"),
    )
    .await;
    // Noise: a credit naming no invoice, and one naming a number we never issued.
    credit(
        &pool,
        world.company,
        account,
        "2026-09-05",
        5_000,
        "HR99 | Povrat",
    )
    .await;
    credit(
        &pool,
        world.company,
        account,
        "2026-09-05",
        5_000,
        &format!("HR99 | BROJ RACUNA 77-1-1-{year}"),
    )
    .await;

    let listed = c
        .list_invoices(as_caller(OWNER, ListInvoicesRequest::default()))
        .await
        .unwrap()
        .into_inner()
        .invoices;
    let got = listed.iter().find(|i| i.id == first_invoice.id).unwrap();
    assert_eq!(got.status, "paid", "settled on the next read");
    assert_eq!(got.paid_minor, 1_450_082);
    assert_eq!(got.paid_at, "2026-09-04T00:00:00+00:00");
    assert_eq!(got.payments.len(), 1);
    let payment = &got.payments[0];
    assert_eq!(payment.source, "inferred");
    assert_eq!(payment.transaction_id, paid_tx.to_string());
    assert_eq!(payment.reason, format!("reference 1-1-1-{year}"));
    assert_eq!(payment.counterparty, "TENDERLY D.O.O.");
    let (rows,): (i64,) = sqlx::query_as("select count(*) from finance.invoice_payments")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(rows, 1, "the noise made no payment");

    // Undone: the invoice is open again, and the match is not remade.
    let undone = c
        .unlink_payment(as_caller(
            OWNER,
            UnlinkPaymentRequest {
                id: payment.id.clone(),
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .invoice
        .unwrap();
    assert_eq!(undone.status, "approved");
    assert_eq!(undone.paid_minor, 0);
    assert!(undone.payments.is_empty());
    let again = c
        .get_invoice(as_caller(
            OWNER,
            GetInvoiceRequest {
                id: first_invoice.id.clone(),
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .invoice
        .unwrap();
    assert_eq!(again.status, "approved", "a rejected match stays rejected");

    // A person records the same transaction by hand: their word, kept.
    let recorded = c
        .record_payment(as_caller(
            OWNER,
            RecordPaymentRequest {
                invoice_id: first_invoice.id.clone(),
                transaction_id: paid_tx.to_string(),
                note: "checked with the bank".into(),
                ..Default::default()
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .invoice
        .unwrap();
    assert_eq!(recorded.status, "paid");
    assert_eq!(recorded.payments[0].source, "declared");
    assert_eq!(
        recorded.payments[0].amount_minor, 1_450_082,
        "the transaction's amount"
    );
    assert_eq!(
        recorded.payments[0].paid_on, "2026-09-04",
        "the transaction's day"
    );
    // The same transaction cannot settle a second invoice.
    let b_draft = c
        .create_invoice(as_caller(
            OWNER,
            CreateInvoiceRequest {
                client_id,
                ..Default::default()
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .invoice
        .unwrap();
    let second = approve(&server, b_draft.id).await;
    let e = c
        .record_payment(as_caller(
            OWNER,
            RecordPaymentRequest {
                invoice_id: second.id.clone(),
                transaction_id: paid_tx.to_string(),
                ..Default::default()
            },
        ))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::InvalidArgument, "{e}");
    // A part by hand: still open, with the part on it.
    let part = c
        .record_payment(as_caller(
            OWNER,
            RecordPaymentRequest {
                invoice_id: second.id.clone(),
                amount_minor: 1_000_000,
                paid_on: "2026-09-10".into(),
                ..Default::default()
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .invoice
        .unwrap();
    assert_eq!(part.status, "approved");
    assert_eq!(part.paid_minor, 1_000_000);
    assert_eq!(part.paid_at, "");
    // The reader sees the payments; a stranger's payment id is not found.
    let seen = c
        .get_invoice(as_caller(
            READER,
            GetInvoiceRequest {
                id: second.id.clone(),
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .invoice
        .unwrap();
    assert_eq!(seen.payments.len(), 1);
    let e = c
        .unlink_payment(as_caller(
            OWNER,
            UnlinkPaymentRequest {
                id: Uuid::new_v4().to_string(),
            },
        ))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::NotFound, "{e}");
}

/// One client per company is the default; making another one moves it; a
/// client outside the grant is not found.
#[tokio::test]
async fn one_client_is_the_default_and_it_moves() {
    let (server, pool) = start_with_store().await;
    let world = seed(&pool).await;
    let mut c = server.client().await;
    let make = |name: &str| UpsertClientRequest {
        client: Some(ClientProfile {
            name: name.into(),
            ..client(world.company)
        }),
    };
    let eiger = c
        .upsert_client(as_caller(OWNER, make("Eiger Oy")))
        .await
        .unwrap()
        .into_inner()
        .client
        .unwrap();
    let tenderly = c
        .upsert_client(as_caller(OWNER, make("Tenderly")))
        .await
        .unwrap()
        .into_inner()
        .client
        .unwrap();
    assert!(
        !eiger.is_default && !tenderly.is_default,
        "nobody until chosen"
    );
    let chosen = c
        .set_default_client(as_caller(
            OWNER,
            SetDefaultClientRequest {
                id: tenderly.id.clone(),
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .client
        .unwrap();
    assert!(chosen.is_default);
    c.set_default_client(as_caller(
        OWNER,
        SetDefaultClientRequest {
            id: eiger.id.clone(),
        },
    ))
    .await
    .unwrap();
    let listed = c
        .list_clients(as_caller(OWNER, ListClientsRequest::default()))
        .await
        .unwrap()
        .into_inner()
        .clients;
    let defaults: Vec<&str> = listed
        .iter()
        .filter(|x| x.is_default)
        .map(|x| x.name.as_str())
        .collect();
    assert_eq!(defaults, vec!["Eiger Oy"], "moved, one at a time");
    let e = c
        .set_default_client(as_caller(
            READER,
            SetDefaultClientRequest {
                id: Uuid::new_v4().to_string(),
            },
        ))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::NotFound, "{e}");
}
