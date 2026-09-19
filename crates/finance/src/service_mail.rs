//! The mail RPCs: templates, sending as a connector, the record, and a
//! thread with its replies.

use tbd_proto::finance::v1::{
    DeleteMailTemplateRequest, DeleteMailTemplateResponse, GetMailRequest, GetMailResponse,
    ListMailRequest, ListMailResponse, ListMailTemplatesRequest, ListMailTemplatesResponse, Mail,
    MailDocument, MailTemplate, SendMailRequest, SendMailResponse, UpsertMailTemplateRequest,
    UpsertMailTemplateResponse,
};
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::{
    mail::store::{self, Filter, MailDocRow, MailRow, SendInput, TemplateInput, TemplateRow},
    service::Finance,
};

fn uuid(s: &str, field: &str) -> Result<Uuid, Status> {
    Uuid::parse_str(s).map_err(|_| Status::invalid_argument(format!("{field}: not a uuid")))
}

fn opt_uuid(s: &str, field: &str) -> Result<Option<Uuid>, Status> {
    if s.trim().is_empty() {
        Ok(None)
    } else {
        uuid(s, field).map(Some)
    }
}

fn t(v: Option<chrono::DateTime<chrono::Utc>>) -> String {
    v.map(|t| t.to_rfc3339()).unwrap_or_default()
}

fn template_proto(r: TemplateRow) -> MailTemplate {
    MailTemplate {
        id: r.id.to_string(),
        party_id: r.party_id.to_string(),
        name: r.name,
        subject: r.subject,
        body: r.body,
        to: r.to_addrs,
        cc: r.cc_addrs,
        bcc: r.bcc_addrs,
        updated_at: r.updated_at.to_rfc3339(),
    }
}

fn mail_proto(m: MailRow, replies: i64, docs: Vec<MailDocRow>) -> Mail {
    Mail {
        id: m.id.to_string(),
        party_id: m.party_id.to_string(),
        connector_id: m.connector_id.map(|c| c.to_string()).unwrap_or_default(),
        direction: m.direction,
        from: m.from_addr,
        to: m.to_addrs,
        cc: m.cc_addrs,
        bcc: m.bcc_addrs,
        subject: m.subject,
        body: m.body,
        status: m.status,
        error: m.error.unwrap_or_default(),
        sent_at: t(m.sent_at),
        received_at: t(m.received_at),
        template_id: m.template_id.map(|c| c.to_string()).unwrap_or_default(),
        parent_id: m.parent_id.map(|c| c.to_string()).unwrap_or_default(),
        replies: u32::try_from(replies).unwrap_or(u32::MAX),
        documents: docs
            .into_iter()
            .map(|d| MailDocument {
                document_id: d.document_id.to_string(),
                filename: d.filename.unwrap_or_default(),
                content_type: d.content_type,
                size_bytes: d.size_bytes,
            })
            .collect(),
        thread_key: m.thread_key.unwrap_or_default(),
    }
}

impl Finance {
    pub(crate) async fn rpc_list_mail_templates(
        &self,
        request: Request<ListMailTemplatesRequest>,
    ) -> Result<Response<ListMailTemplatesResponse>, Status> {
        let party_ids = request.get_ref().party_ids.clone();
        let (mut timer, pool, _, view) = self
            .invoice_context("FinanceService/ListMailTemplates", &request, &party_ids)
            .await?;
        let r = store::templates(pool, &view)
            .await
            .map(|rows| ListMailTemplatesResponse {
                templates: rows.into_iter().map(template_proto).collect(),
            });
        self.done_c(&mut timer, r)
    }

    pub(crate) async fn rpc_upsert_mail_template(
        &self,
        request: Request<UpsertMailTemplateRequest>,
    ) -> Result<Response<UpsertMailTemplateResponse>, Status> {
        let req = request.get_ref().clone();
        let input = TemplateInput {
            id: opt_uuid(&req.id, "id")?,
            party_id: uuid(&req.party_id, "party_id")?,
            name: req.name,
            subject: req.subject,
            body: req.body,
            to: req.to,
            cc: req.cc,
            bcc: req.bcc,
        };
        let (mut timer, pool, access, _) = self
            .invoice_context("FinanceService/UpsertMailTemplate", &request, &[])
            .await?;
        let r = store::upsert_template(pool, &access, &input)
            .await
            .map(|row| UpsertMailTemplateResponse {
                template: Some(template_proto(row)),
            });
        self.done_c(&mut timer, r)
    }

    pub(crate) async fn rpc_delete_mail_template(
        &self,
        request: Request<DeleteMailTemplateRequest>,
    ) -> Result<Response<DeleteMailTemplateResponse>, Status> {
        let id = uuid(&request.get_ref().id, "id")?;
        let (mut timer, pool, access, _) = self
            .invoice_context("FinanceService/DeleteMailTemplate", &request, &[])
            .await?;
        let r = store::delete_template(pool, &access, id)
            .await
            .map(|()| DeleteMailTemplateResponse {});
        self.done_c(&mut timer, r)
    }

    pub(crate) async fn rpc_send_mail(
        &self,
        request: Request<SendMailRequest>,
    ) -> Result<Response<SendMailResponse>, Status> {
        let req = request.get_ref().clone();
        let mut attachment_document_ids = Vec::with_capacity(req.attachment_document_ids.len());
        for id in &req.attachment_document_ids {
            attachment_document_ids.push(uuid(id, "attachment_document_ids")?);
        }
        let input = SendInput {
            connector_id: uuid(&req.connector_id, "connector_id")?,
            template_id: opt_uuid(&req.template_id, "template_id")?,
            to: req.to,
            cc: req.cc,
            bcc: req.bcc,
            subject: req.subject,
            body: req.body,
            html: Some(req.html).filter(|h| !h.trim().is_empty()),
            attachment_document_ids,
            in_reply_to_mail_id: opt_uuid(&req.in_reply_to_mail_id, "in_reply_to_mail_id")?,
        };
        let sealer = self.sealer()?;
        let kinds = self.kinds();
        let (mut timer, pool, access, _) = self
            .invoice_context("FinanceService/SendMail", &request, &[])
            .await?;
        let r = store::send(pool, &access, &sealer, &kinds, &self.mail, &input)
            .await
            .map(|(m, docs)| SendMailResponse {
                mail: Some(mail_proto(m, 0, docs)),
            });
        self.done_c(&mut timer, r)
    }

    pub(crate) async fn rpc_list_mail(
        &self,
        request: Request<ListMailRequest>,
    ) -> Result<Response<ListMailResponse>, Status> {
        let req = request.get_ref().clone();
        let filter = Filter {
            connector_id: opt_uuid(&req.connector_id, "connector_id")?,
            q: req.q.trim().to_owned(),
            direction: req.direction.trim().to_owned(),
            limit: if req.limit == 0 {
                100
            } else {
                i64::from(req.limit)
            },
            offset: i64::from(req.offset),
        };
        let (mut timer, pool, _, view) = self
            .invoice_context("FinanceService/ListMail", &request, &req.party_ids)
            .await?;
        let r = store::list(pool, &view, &filter)
            .await
            .map(|(rows, total)| ListMailResponse {
                mails: rows
                    .into_iter()
                    .map(|(m, n, docs)| mail_proto(m, n, docs))
                    .collect(),
                total: u32::try_from(total).unwrap_or(u32::MAX),
            });
        self.done_c(&mut timer, r)
    }

    pub(crate) async fn rpc_get_mail(
        &self,
        request: Request<GetMailRequest>,
    ) -> Result<Response<GetMailResponse>, Status> {
        let id = uuid(&request.get_ref().id, "id")?;
        let (mut timer, pool, access, _) = self
            .invoice_context("FinanceService/GetMail", &request, &[])
            .await?;
        let thread = match store::thread(pool, &access, id).await {
            Ok(t) => t,
            Err(e) => return self.done_c(&mut timer, Err(e)),
        };
        let mails: Vec<Mail> = thread
            .into_iter()
            .map(|(m, docs)| {
                let replies = 0;
                mail_proto(m, replies, docs)
            })
            .collect();
        let mail = mails.iter().find(|m| m.id == id.to_string()).cloned();
        self.done_c(
            &mut timer,
            Ok(GetMailResponse {
                mail,
                thread: mails,
            }),
        )
    }
}
