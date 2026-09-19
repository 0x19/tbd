//! The invoicing RPCs: thin over `invoice::store`, with the same rule as the
//! rest -- a row outside the grant is not-found, never forbidden -- and one
//! mapping that matters: a stale draft at approval is `FAILED_PRECONDITION`,
//! the code that says "preview again", not "try again".

use tbd_db::DbError;
use tbd_proto::finance::v1::{
    ApproveInvoiceRequest, ApproveInvoiceResponse, CancelInvoiceRequest, CancelInvoiceResponse,
    ClientProfile, CreateInvoiceRequest, CreateInvoiceResponse, DeleteLineTemplateRequest,
    DeleteLineTemplateResponse, GetInvoiceDocumentRequest, GetInvoiceDocumentResponse,
    GetInvoiceRequest, GetInvoiceResponse, GetIssuerRequest, GetIssuerResponse, Invoice,
    InvoiceLine, IssuerProfile, LineTemplate, ListClientsRequest, ListClientsResponse,
    ListInvoicesRequest, ListInvoicesResponse, ListLineTemplatesRequest, ListLineTemplatesResponse,
    PreviewInvoiceRequest, PreviewInvoiceResponse, UpdateInvoiceRequest, UpdateInvoiceResponse,
    UpsertClientRequest, UpsertClientResponse, UpsertIssuerRequest, UpsertIssuerResponse,
    UpsertLineTemplateRequest, UpsertLineTemplateResponse,
};
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::{
    invoice::{
        VatTreatment,
        store::{
            self, ClientInput, ClientRow, DraftInput, InvoiceError, InvoiceRow, IssuerInput,
            IssuerRow, LineInput, LineRow, TemplateInput, TemplateRow,
        },
    },
    service::{Finance, status_of},
};

/// The one place an invoicing error becomes a status.
pub(crate) fn invoice_status(e: InvoiceError) -> Status {
    match e {
        InvoiceError::Db(d) => status_of(d),
        InvoiceError::NotDraft(s) => Status::failed_precondition(format!("invoice is {s}")),
        InvoiceError::StaleDraft => {
            Status::failed_precondition("the draft changed since it was previewed; preview again")
        }
        InvoiceError::NoIssuer => Status::failed_precondition("no issuer profile; set one first"),
        InvoiceError::NoLines => Status::failed_precondition("an invoice needs at least one line"),
        InvoiceError::Render(r) => Status::internal(format!("render: {r}")),
        InvoiceError::Canonical(c) => Status::internal(format!("canonical: {c}")),
    }
}

fn uuid(s: &str, field: &str) -> Result<Uuid, Status> {
    Uuid::parse_str(s).map_err(|_| Status::invalid_argument(format!("{field}: not a uuid")))
}

fn date(s: &str, field: &str) -> Result<chrono::NaiveDate, Status> {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .map_err(|_| Status::invalid_argument(format!("{field}: want YYYY-MM-DD")))
}

fn issuer_proto(r: IssuerRow) -> IssuerProfile {
    IssuerProfile {
        party_id: r.party_id.to_string(),
        legal_name: r.legal_name,
        address_lines: r.address_lines,
        oib: r.oib,
        vat_id: r.vat_id,
        iban: r.iban,
        swift: r.swift,
        bank_name: r.bank_name,
        court: r.court,
        registration_no: r.registration_no,
        share_capital: r.share_capital,
        board_member: r.board_member,
        issued_by: r.issued_by,
        place_of_issue: r.place_of_issue,
        operator_id: r.operator_id,
        premises: r.premises,
        device: r.device,
        due_days: r.due_days,
    }
}

fn client_proto(c: ClientRow) -> ClientProfile {
    ClientProfile {
        id: c.id.to_string(),
        party_id: c.party_id.to_string(),
        name: c.name,
        address_lines: c.address_lines,
        country_code: c.country_code,
        tax_id: c.tax_id,
        vat_treatment: c.vat_treatment,
        recipients: c.recipients,
        currency: c.currency,
        archived: c.archived_at.is_some(),
    }
}

fn invoice_proto(r: InvoiceRow, lines: Vec<LineRow>) -> Invoice {
    let t =
        |v: Option<chrono::DateTime<chrono::Utc>>| v.map(|t| t.to_rfc3339()).unwrap_or_default();
    Invoice {
        id: r.id.to_string(),
        party_id: r.party_id.to_string(),
        client_id: r.client_id.to_string(),
        status: r.status,
        number: r.number.unwrap_or_default(),
        year: r.year,
        issued_at: t(r.issued_at),
        delivery_date: r.delivery_date.to_string(),
        due_date: r.due_date.to_string(),
        place_of_issue: r.place_of_issue,
        currency: r.currency,
        subtotal_minor: r.subtotal_minor,
        vat_minor: r.vat_minor,
        total_minor: r.total_minor,
        vat_treatment: r.vat_treatment,
        vat_note: r.vat_note,
        note: r.note,
        content_hash: r.content_hash.unwrap_or_default(),
        approved_at: t(r.approved_at),
        document_id: r.document_id.map(|d| d.to_string()).unwrap_or_default(),
        prefilled_from: r.prefilled_from.map(|d| d.to_string()).unwrap_or_default(),
        cancelled_at: t(r.cancelled_at),
        created_at: r.created_at.to_rfc3339(),
        updated_at: r.updated_at.to_rfc3339(),
        lines: lines
            .into_iter()
            .map(|l| InvoiceLine {
                position: l.position,
                description: l.description,
                quantity_milli: l.quantity_milli,
                unit_price_minor: l.unit_price_minor,
                amount_minor: l.amount_minor,
                template_id: l.template_id.map(|t| t.to_string()).unwrap_or_default(),
            })
            .collect(),
    }
}

fn template_proto(t: TemplateRow) -> LineTemplate {
    LineTemplate {
        id: t.id.to_string(),
        client_id: t.client_id.to_string(),
        position: t.position,
        description: t.description,
        mode: t.mode,
        quantity_milli: t.quantity_milli,
        unit_price_minor: t.unit_price_minor,
        enabled: t.enabled,
    }
}

impl Finance {
    pub(crate) async fn invoice_context(
        &self,
        route: &'static str,
        request: &Request<impl Sized>,
        party_ids: &[String],
    ) -> Result<
        (
            tbd_common::metrics::RequestTimer,
            &sqlx::PgPool,
            tbd_db::Access,
            tbd_db::Access,
        ),
        Status,
    > {
        let mut timer = self.admit(route).await?;
        match self.read_context(request, party_ids).await {
            Ok((pool, access, view)) => Ok((timer, pool, access, view)),
            Err(status) => Err(self.reject(&mut timer, status)),
        }
    }

    /// Finish an RPC: count a failure, or hand the answer back.
    fn done<T>(
        &self,
        timer: &mut tbd_common::metrics::RequestTimer,
        result: Result<T, InvoiceError>,
    ) -> Result<Response<T>, Status> {
        match result {
            Ok(v) => Ok(Response::new(v)),
            Err(e) => Err(self.reject(timer, invoice_status(e))),
        }
    }

    pub(crate) async fn rpc_get_issuer(
        &self,
        request: Request<GetIssuerRequest>,
    ) -> Result<Response<GetIssuerResponse>, Status> {
        let party = uuid(&request.get_ref().party_id, "party_id")?;
        let (mut timer, pool, access, _) = self
            .invoice_context("FinanceService/GetIssuer", &request, &[])
            .await?;
        let r = store::issuer(pool, &access, party)
            .await
            .map(|i| GetIssuerResponse {
                issuer: i.map(issuer_proto),
            });
        self.done(&mut timer, r)
    }

    pub(crate) async fn rpc_upsert_issuer(
        &self,
        request: Request<UpsertIssuerRequest>,
    ) -> Result<Response<UpsertIssuerResponse>, Status> {
        let Some(i) = request.get_ref().issuer.clone() else {
            return Err(Status::invalid_argument("issuer is required"));
        };
        let party_id = uuid(&i.party_id, "issuer.party_id")?;
        let (mut timer, pool, access, _) = self
            .invoice_context("FinanceService/UpsertIssuer", &request, &[])
            .await?;
        let input = IssuerInput {
            party_id,
            legal_name: i.legal_name,
            address_lines: i.address_lines,
            oib: i.oib,
            vat_id: i.vat_id,
            iban: i.iban,
            swift: i.swift,
            bank_name: i.bank_name,
            court: i.court,
            registration_no: i.registration_no,
            share_capital: i.share_capital,
            board_member: i.board_member,
            issued_by: i.issued_by,
            place_of_issue: i.place_of_issue,
            operator_id: if i.operator_id.is_empty() {
                "1".into()
            } else {
                i.operator_id
            },
            premises: if i.premises.is_empty() {
                "1".into()
            } else {
                i.premises
            },
            device: if i.device.is_empty() {
                "1".into()
            } else {
                i.device
            },
            due_days: if i.due_days <= 0 { 15 } else { i.due_days },
        };
        let r = store::upsert_issuer(pool, &access, input)
            .await
            .map(|i| UpsertIssuerResponse {
                issuer: Some(issuer_proto(i)),
            });
        self.done(&mut timer, r)
    }

    pub(crate) async fn rpc_list_clients(
        &self,
        request: Request<ListClientsRequest>,
    ) -> Result<Response<ListClientsResponse>, Status> {
        let party_ids = request.get_ref().party_ids.clone();
        let (mut timer, pool, _, view) = self
            .invoice_context("FinanceService/ListClients", &request, &party_ids)
            .await?;
        let r = store::clients(pool, &view)
            .await
            .map(|c| ListClientsResponse {
                clients: c.into_iter().map(client_proto).collect(),
            });
        self.done(&mut timer, r)
    }

    pub(crate) async fn rpc_upsert_client(
        &self,
        request: Request<UpsertClientRequest>,
    ) -> Result<Response<UpsertClientResponse>, Status> {
        let Some(c) = request.get_ref().client.clone() else {
            return Err(Status::invalid_argument("client is required"));
        };
        let party_id = uuid(&c.party_id, "client.party_id")?;
        let id = if c.id.is_empty() {
            None
        } else {
            Some(uuid(&c.id, "client.id")?)
        };
        let vat_treatment = VatTreatment::parse(&c.vat_treatment)
            .ok_or_else(|| Status::invalid_argument("client.vat_treatment: unknown"))?;
        let (mut timer, pool, access, _) = self
            .invoice_context("FinanceService/UpsertClient", &request, &[])
            .await?;
        let input = ClientInput {
            id,
            party_id,
            name: c.name,
            address_lines: c.address_lines,
            country_code: c.country_code,
            tax_id: c.tax_id,
            vat_treatment,
            recipients: c.recipients,
            currency: if c.currency.is_empty() {
                "EUR".into()
            } else {
                c.currency
            },
        };
        let r = store::upsert_client(pool, &access, input)
            .await
            .map(|c| UpsertClientResponse {
                client: Some(client_proto(c)),
            });
        self.done(&mut timer, r)
    }

    pub(crate) async fn rpc_list_invoices(
        &self,
        request: Request<ListInvoicesRequest>,
    ) -> Result<Response<ListInvoicesResponse>, Status> {
        let party_ids = request.get_ref().party_ids.clone();
        let (mut timer, pool, _, view) = self
            .invoice_context("FinanceService/ListInvoices", &request, &party_ids)
            .await?;
        let r = store::invoices(pool, &view)
            .await
            .map(|rows| ListInvoicesResponse {
                invoices: rows
                    .into_iter()
                    .map(|r| invoice_proto(r, Vec::new()))
                    .collect(),
            });
        self.done(&mut timer, r)
    }

    pub(crate) async fn rpc_get_invoice(
        &self,
        request: Request<GetInvoiceRequest>,
    ) -> Result<Response<GetInvoiceResponse>, Status> {
        let id = uuid(&request.get_ref().id, "id")?;
        let (mut timer, pool, access, _) = self
            .invoice_context("FinanceService/GetInvoice", &request, &[])
            .await?;
        let r = store::invoice(pool, &access, id)
            .await
            .map(|(row, lines)| GetInvoiceResponse {
                invoice: Some(invoice_proto(row, lines)),
            });
        self.done(&mut timer, r)
    }

    pub(crate) async fn rpc_create_invoice(
        &self,
        request: Request<CreateInvoiceRequest>,
    ) -> Result<Response<CreateInvoiceResponse>, Status> {
        let client = uuid(&request.get_ref().client_id, "client_id")?;
        let (mut timer, pool, access, _) = self
            .invoice_context("FinanceService/CreateInvoice", &request, &[])
            .await?;
        let r = async {
            let row = store::create_draft(pool, &access, client).await?;
            let (row, lines) = store::invoice(pool, &access, row.id).await?;
            Ok(CreateInvoiceResponse {
                invoice: Some(invoice_proto(row, lines)),
            })
        }
        .await;
        self.done(&mut timer, r)
    }

    pub(crate) async fn rpc_update_invoice(
        &self,
        request: Request<UpdateInvoiceRequest>,
    ) -> Result<Response<UpdateInvoiceResponse>, Status> {
        let req = request.get_ref().clone();
        let id = uuid(&req.id, "id")?;
        let input = DraftInput {
            delivery_date: date(&req.delivery_date, "delivery_date")?,
            due_date: date(&req.due_date, "due_date")?,
            place_of_issue: req.place_of_issue,
            note: req.note,
            lines: req
                .lines
                .into_iter()
                .map(|l| LineInput {
                    description: l.description,
                    quantity_milli: l.quantity_milli,
                    unit_price_minor: l.unit_price_minor,
                    template_id: Uuid::parse_str(&l.template_id).ok(),
                })
                .collect(),
        };
        let (mut timer, pool, access, _) = self
            .invoice_context("FinanceService/UpdateInvoice", &request, &[])
            .await?;
        let r = async {
            let row = store::update_draft(pool, &access, id, input).await?;
            let (row, lines) = store::invoice(pool, &access, row.id).await?;
            Ok(UpdateInvoiceResponse {
                invoice: Some(invoice_proto(row, lines)),
            })
        }
        .await;
        self.done(&mut timer, r)
    }

    pub(crate) async fn rpc_preview_invoice(
        &self,
        request: Request<PreviewInvoiceRequest>,
    ) -> Result<Response<PreviewInvoiceResponse>, Status> {
        let id = uuid(&request.get_ref().id, "id")?;
        let (mut timer, pool, access, _) = self
            .invoice_context("FinanceService/PreviewInvoice", &request, &[])
            .await?;
        let r = store::preview(pool, &access, id)
            .await
            .map(|p| PreviewInvoiceResponse {
                content_hash: p.content_hash,
                number: p.doc.number,
                pdf: p.pdf,
            });
        self.done(&mut timer, r)
    }

    pub(crate) async fn rpc_approve_invoice(
        &self,
        request: Request<ApproveInvoiceRequest>,
    ) -> Result<Response<ApproveInvoiceResponse>, Status> {
        let req = request.get_ref().clone();
        let id = uuid(&req.id, "id")?;
        if req.content_hash.is_empty() {
            return Err(Status::invalid_argument(
                "content_hash: the preview's hash is required",
            ));
        }
        let (mut timer, pool, access, _) = self
            .invoice_context("FinanceService/ApproveInvoice", &request, &[])
            .await?;
        let r = async {
            let done = store::approve(pool, &access, id, &req.content_hash).await?;
            let (row, lines) = store::invoice(pool, &access, done.invoice.id).await?;
            Ok(ApproveInvoiceResponse {
                invoice: Some(invoice_proto(row, lines)),
            })
        }
        .await;
        self.done(&mut timer, r)
    }

    pub(crate) async fn rpc_cancel_invoice(
        &self,
        request: Request<CancelInvoiceRequest>,
    ) -> Result<Response<CancelInvoiceResponse>, Status> {
        let req = request.get_ref().clone();
        let id = uuid(&req.id, "id")?;
        let (mut timer, pool, access, _) = self
            .invoice_context("FinanceService/CancelInvoice", &request, &[])
            .await?;
        let r = async {
            let row = store::cancel(pool, &access, id, &req.reason).await?;
            let (row, lines) = store::invoice(pool, &access, row.id).await?;
            Ok(CancelInvoiceResponse {
                invoice: Some(invoice_proto(row, lines)),
            })
        }
        .await;
        self.done(&mut timer, r)
    }

    pub(crate) async fn rpc_list_line_templates(
        &self,
        request: Request<ListLineTemplatesRequest>,
    ) -> Result<Response<ListLineTemplatesResponse>, Status> {
        let client = uuid(&request.get_ref().client_id, "client_id")?;
        let (mut timer, pool, access, _) = self
            .invoice_context("FinanceService/ListLineTemplates", &request, &[])
            .await?;
        let r = store::templates(pool, &access, client)
            .await
            .map(|t| ListLineTemplatesResponse {
                templates: t.into_iter().map(template_proto).collect(),
            });
        self.done(&mut timer, r)
    }

    pub(crate) async fn rpc_upsert_line_template(
        &self,
        request: Request<UpsertLineTemplateRequest>,
    ) -> Result<Response<UpsertLineTemplateResponse>, Status> {
        let req = request.get_ref().clone();
        let client = uuid(&req.client_id, "client_id")?;
        let Some(t) = req.template else {
            return Err(Status::invalid_argument("template is required"));
        };
        let id = if t.id.is_empty() {
            None
        } else {
            Some(uuid(&t.id, "template.id")?)
        };
        let (mut timer, pool, access, _) = self
            .invoice_context("FinanceService/UpsertLineTemplate", &request, &[])
            .await?;
        let input = TemplateInput {
            id,
            client_id: client,
            position: t.position,
            description: t.description,
            mode: t.mode,
            quantity_milli: if t.quantity_milli == 0 {
                1000
            } else {
                t.quantity_milli
            },
            unit_price_minor: t.unit_price_minor,
            enabled: t.enabled,
        };
        let r = store::upsert_template(pool, &access, input).await.map(|t| {
            UpsertLineTemplateResponse {
                template: Some(template_proto(t)),
            }
        });
        self.done(&mut timer, r)
    }

    pub(crate) async fn rpc_delete_line_template(
        &self,
        request: Request<DeleteLineTemplateRequest>,
    ) -> Result<Response<DeleteLineTemplateResponse>, Status> {
        let req = request.get_ref().clone();
        let client = uuid(&req.client_id, "client_id")?;
        let id = uuid(&req.id, "id")?;
        let (mut timer, pool, access, _) = self
            .invoice_context("FinanceService/DeleteLineTemplate", &request, &[])
            .await?;
        let r = store::delete_template(pool, &access, client, id)
            .await
            .map(|()| DeleteLineTemplateResponse {});
        self.done(&mut timer, r)
    }

    pub(crate) async fn rpc_get_invoice_document(
        &self,
        request: Request<GetInvoiceDocumentRequest>,
    ) -> Result<Response<GetInvoiceDocumentResponse>, Status> {
        let id = uuid(&request.get_ref().id, "id")?;
        let (mut timer, pool, access, _) = self
            .invoice_context("FinanceService/GetInvoiceDocument", &request, &[])
            .await?;
        let r = async {
            let (row, _) = store::invoice(pool, &access, id).await?;
            let document = row
                .document_id
                .ok_or(InvoiceError::Db(DbError::NotFound { what: "document" }))?;
            let (content_type, pdf) = store::document(pool, &access, document).await?;
            Ok(GetInvoiceDocumentResponse {
                content_type,
                pdf,
                number: row.number.unwrap_or_default(),
            })
        }
        .await;
        self.done(&mut timer, r)
    }
}
