//! The document RPCs: the receipts list with search, one with its bytes, a
//! person's corrections, and a re-read.

use chrono::NaiveDate;
use tbd_proto::finance::v1::{
    Document, DocumentSource, ExtractDocumentRequest, ExtractDocumentResponse, GetDocumentRequest,
    GetDocumentResponse, ListDocumentsRequest, ListDocumentsResponse, UpdateDocumentRequest,
    UpdateDocumentResponse, UploadDocumentRequest, UploadDocumentResponse, VendorCount,
};
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::{
    documents::{
        self,
        store::{self, Declared, DocumentRow, Filter, SourceRow, Upload},
    },
    service::Finance,
};

fn uuid(s: &str, field: &str) -> Result<Uuid, Status> {
    Uuid::parse_str(s).map_err(|_| Status::invalid_argument(format!("{field}: not a uuid")))
}

fn date(s: &str, field: &str) -> Result<Option<NaiveDate>, Status> {
    if s.trim().is_empty() {
        return Ok(None);
    }
    NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d")
        .map(Some)
        .map_err(|_| Status::invalid_argument(format!("{field}: want YYYY-MM-DD")))
}

fn t(v: Option<chrono::DateTime<chrono::Utc>>) -> String {
    v.map(|t| t.to_rfc3339()).unwrap_or_default()
}

pub(crate) fn document_proto(d: DocumentRow, sources: Vec<SourceRow>) -> Document {
    let found_by = d
        .extracted
        .as_object()
        .map(|o| {
            o.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_owned())))
                .collect()
        })
        .unwrap_or_default();
    Document {
        id: d.id.to_string(),
        party_id: d.party_id.to_string(),
        kind: d.kind,
        filename: d.filename.unwrap_or_default(),
        content_type: d.content_type,
        size_bytes: d.size_bytes,
        sha256: d.sha256,
        vendor: d.vendor.unwrap_or_default(),
        doc_date: d.doc_date.map(|d| d.to_string()).unwrap_or_default(),
        total_minor: d.total_minor.map(|m| m.to_string()).unwrap_or_default(),
        currency: d.currency.unwrap_or_default(),
        created_at: d.created_at.to_rfc3339(),
        sources: sources
            .into_iter()
            .map(|s| DocumentSource {
                connector_id: s.connector_id.map(|c| c.to_string()).unwrap_or_default(),
                external_ref: s.external_ref,
                subject: s.subject,
                sender: s.sender,
                received_at: t(s.received_at),
            })
            .collect(),
        invoice_no: d.invoice_no.unwrap_or_default(),
        extracted_at: t(d.extracted_at),
        declared: d.declared_at.is_some(),
        found_by,
    }
}

impl Finance {
    pub(crate) async fn rpc_list_documents(
        &self,
        request: Request<ListDocumentsRequest>,
    ) -> Result<Response<ListDocumentsResponse>, Status> {
        let req = request.get_ref().clone();
        let from = date(&req.from, "from")?;
        let to = date(&req.to, "to")?;
        let (mut timer, pool, _, view) = self
            .invoice_context("FinanceService/ListDocuments", &request, &req.party_ids)
            .await?;
        let filter = Filter {
            kind: req.kind,
            q: req.q.trim().to_owned(),
            from,
            to,
            vendor: req.vendor,
            limit: i64::from(if req.limit == 0 {
                100
            } else {
                req.limit.min(500)
            }),
            offset: i64::from(req.offset),
        };
        let r = store::list(pool, &view, &filter)
            .await
            .map(|listing| ListDocumentsResponse {
                documents: listing
                    .documents
                    .into_iter()
                    .map(|(d, s)| document_proto(d, s))
                    .collect(),
                total: u32::try_from(listing.total).unwrap_or(u32::MAX),
                vendors: listing
                    .vendors
                    .into_iter()
                    .map(|v| VendorCount {
                        vendor: v.vendor,
                        count: u32::try_from(v.count).unwrap_or(u32::MAX),
                    })
                    .collect(),
            });
        self.done_c(&mut timer, r)
    }

    pub(crate) async fn rpc_get_document(
        &self,
        request: Request<GetDocumentRequest>,
    ) -> Result<Response<GetDocumentResponse>, Status> {
        let id = uuid(&request.get_ref().id, "id")?;
        let (mut timer, pool, access, _) = self
            .invoice_context("FinanceService/GetDocument", &request, &[])
            .await?;
        let r = async {
            let (doc, sources) = store::get(pool, &access, id).await?;
            let bytes = store::bytes(pool, id).await?;
            Ok(GetDocumentResponse {
                document: Some(document_proto(doc, sources)),
                bytes,
            })
        }
        .await;
        self.done_c(&mut timer, r)
    }

    pub(crate) async fn rpc_update_document(
        &self,
        request: Request<UpdateDocumentRequest>,
    ) -> Result<Response<UpdateDocumentResponse>, Status> {
        let req = request.get_ref().clone();
        let id = uuid(&req.id, "id")?;
        let total_minor = if req.total_minor.trim().is_empty() {
            None
        } else {
            Some(
                req.total_minor
                    .trim()
                    .parse::<i64>()
                    .map_err(|_| Status::invalid_argument("total_minor: want an integer"))?,
            )
        };
        let currency = req.currency.trim().to_ascii_uppercase();
        if !currency.is_empty()
            && (currency.len() != 3 || !currency.chars().all(|c| c.is_ascii_alphabetic()))
        {
            return Err(Status::invalid_argument("currency: want an ISO code"));
        }
        if total_minor.is_some() && currency.is_empty() {
            return Err(Status::invalid_argument(
                "currency: required with total_minor",
            ));
        }
        let party_id = if req.party_id.trim().is_empty() {
            None
        } else {
            Some(uuid(&req.party_id, "party_id")?)
        };
        let declared = Declared {
            vendor: Some(req.vendor.trim().to_owned()).filter(|v| !v.is_empty()),
            doc_date: date(&req.doc_date, "doc_date")?,
            total_minor,
            currency: Some(currency).filter(|c| !c.is_empty()),
            invoice_no: Some(req.invoice_no.trim().to_owned()).filter(|v| !v.is_empty()),
            party_id,
        };
        let (mut timer, pool, access, _) = self
            .invoice_context("FinanceService/UpdateDocument", &request, &[])
            .await?;
        let r = store::update(pool, &access, id, &declared)
            .await
            .map(|(d, s)| UpdateDocumentResponse {
                document: Some(document_proto(d, s)),
            });
        self.done_c(&mut timer, r)
    }

    pub(crate) async fn rpc_upload_document(
        &self,
        request: Request<UploadDocumentRequest>,
    ) -> Result<Response<UploadDocumentResponse>, Status> {
        let req = request.get_ref();
        let party_id = uuid(&req.party_id, "party_id")?;
        let content_type = req.content_type.trim().to_ascii_lowercase();
        if !matches!(
            content_type.as_str(),
            "application/pdf" | "image/jpeg" | "image/png"
        ) {
            return Err(Status::invalid_argument(
                "content_type: want application/pdf, image/jpeg or image/png",
            ));
        }
        if req.bytes.is_empty() {
            return Err(Status::invalid_argument("bytes: empty"));
        }
        if content_type == "application/pdf" && !req.bytes.starts_with(b"%PDF") {
            return Err(Status::invalid_argument("bytes: not a pdf"));
        }
        let filename: String = req
            .filename
            .trim()
            .chars()
            .filter(|c| !c.is_control() && *c != '/' && *c != '\\')
            .take(120)
            .collect();
        let filename = if filename.is_empty() {
            let ext = match content_type.as_str() {
                "image/jpeg" => "jpg",
                "image/png" => "png",
                _ => "pdf",
            };
            format!("receipt.{ext}")
        } else {
            filename
        };
        let up = Upload {
            party_id,
            filename,
            content_type,
            bytes: req.bytes.clone(),
        };
        let (mut timer, pool, access, _) = self
            .invoice_context("FinanceService/UploadDocument", &request, &[])
            .await?;
        let (id, new) = match store::upload(pool, &access, &up).await {
            Ok(v) => v,
            Err(e) => return self.done_c(&mut timer, Err(e)),
        };
        if new && let Err(e) = documents::read(pool, id).await {
            return self.done_c(&mut timer, Err(e));
        }
        let r = store::get(pool, &access, id)
            .await
            .map(|(d, s)| UploadDocumentResponse {
                document: Some(document_proto(d, s)),
            });
        self.done_c(&mut timer, r)
    }

    pub(crate) async fn rpc_extract_document(
        &self,
        request: Request<ExtractDocumentRequest>,
    ) -> Result<Response<ExtractDocumentResponse>, Status> {
        let id = uuid(&request.get_ref().id, "id")?;
        let (mut timer, pool, access, _) = self
            .invoice_context("FinanceService/ExtractDocument", &request, &[])
            .await?;
        let r = async {
            // The grant first: a document outside it is not found, and is
            // not read on a stranger's say-so.
            store::get(pool, &access, id).await?;
            documents::read(pool, id).await?;
            let (doc, sources) = store::get(pool, &access, id).await?;
            Ok(ExtractDocumentResponse {
                document: Some(document_proto(doc, sources)),
            })
        }
        .await;
        self.done_c(&mut timer, r)
    }
}
