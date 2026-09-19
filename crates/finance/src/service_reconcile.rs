//! The reconciliation RPCs: a month for the accountant, links, policies.

use std::collections::HashMap;

use tbd_proto::finance::v1::{
    CounterpartyPolicy, DeleteCounterpartyPolicyRequest, DeleteCounterpartyPolicyResponse,
    LinkDocumentRequest, LinkDocumentResponse, LinkedDocument, MonthlyReconciliationRequest,
    MonthlyReconciliationResponse, Reason, ReconciliationRow, ReconciliationSummary,
    SetCounterpartyPolicyRequest, SetCounterpartyPolicyResponse, SetTransactionNoteRequest,
    SetTransactionNoteResponse, Transaction, UnlinkDocumentRequest, UnlinkDocumentResponse,
};
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::{
    reconcile::{
        Need, Why, decode_all, describe_all,
        store::{self, DocRow, LinkRow, PolicyRow, Row, TxRow},
    },
    service::Finance,
};

fn uuid(s: &str, field: &str) -> Result<Uuid, Status> {
    Uuid::parse_str(s).map_err(|_| Status::invalid_argument(format!("{field}: not a uuid")))
}

fn transaction_proto(t: &TxRow) -> Transaction {
    Transaction {
        id: t.id.to_string(),
        account_id: t.account_id.to_string(),
        party_id: t.party_id.to_string(),
        status: t.status.clone(),
        amount_minor: t.amount_minor,
        currency: t.currency.trim().to_owned(),
        scale: u32::try_from(t.scale).unwrap_or(2),
        booking_date: t.booking_date.to_string(),
        counterparty_name: t.counterparty_name.clone().unwrap_or_default(),
        remittance: t.remittance.clone().unwrap_or_default(),
        value_date: t.value_date.map(|d| d.to_string()).unwrap_or_default(),
        counterparty_iban: t.counterparty_iban.clone().unwrap_or_default(),
        category_id: t.category_id.map(|c| c.to_string()).unwrap_or_default(),
        category: t.category.clone().unwrap_or_default(),
        category_source: t.category_source.clone().unwrap_or_default(),
        internal: t.internal,
        reference_number: t.reference_number.clone().unwrap_or_default(),
        entry_reference: t.entry_reference.clone().unwrap_or_default(),
        // The transactions page carries more (account name, rule, times);
        // the accountant's row does not need them.
        ..Transaction::default()
    }
}

fn reason_proto(w: &Why) -> Reason {
    Reason {
        code: w.code.to_owned(),
        args: w
            .args
            .iter()
            .map(|(k, v)| ((*k).to_owned(), v.clone()))
            .collect(),
    }
}

fn doc_proto(d: &DocRow, source: &str, confidence: u32, why: &[Why]) -> LinkedDocument {
    LinkedDocument {
        document_id: d.id.to_string(),
        vendor: d.vendor.clone().unwrap_or_default(),
        doc_date: d.doc_date.map(|d| d.to_string()).unwrap_or_default(),
        total_minor: d.total_minor.map(|m| m.to_string()).unwrap_or_default(),
        currency: d.currency.clone().unwrap_or_default().trim().to_owned(),
        filename: d.filename.clone().unwrap_or_default(),
        invoice_no: d.invoice_no.clone().unwrap_or_default(),
        source: source.to_owned(),
        confidence,
        reason: describe_all(why),
        why: why.iter().map(reason_proto).collect(),
    }
}

fn link_proto(l: &LinkRow) -> LinkedDocument {
    doc_proto(
        &l.document,
        &l.source,
        u32::try_from(l.confidence).unwrap_or(0),
        &decode_all(&l.reason),
    )
}

fn row_proto(r: &Row) -> ReconciliationRow {
    ReconciliationRow {
        transaction: Some(transaction_proto(&r.tx)),
        need: r.decision.need.as_str().into(),
        need_reason: r.decision.reason(),
        need_why: Some(reason_proto(&r.decision.why)),
        status: r.status().into(),
        documents: r.documents.iter().map(link_proto).collect(),
        suggestions: r
            .suggestions
            .iter()
            .map(|(d, points, why)| doc_proto(d, "", u32::from(*points), why))
            .collect(),
        policy_id: r
            .decision
            .policy_id
            .map(|p| p.to_string())
            .unwrap_or_default(),
        original_amount_minor: r
            .original
            .as_ref()
            .map(|(m, _)| m.to_string())
            .unwrap_or_default(),
        original_currency: r
            .original
            .as_ref()
            .map(|(_, c)| c.clone())
            .unwrap_or_default(),
        note: r.note.clone(),
    }
}

fn policy_proto(p: &PolicyRow) -> CounterpartyPolicy {
    CounterpartyPolicy {
        id: p.id.to_string(),
        party_id: p.party_id.to_string(),
        r#match: p.match_normalised.clone(),
        exact: p.exact,
        policy: p.policy.clone(),
        note: p.note.clone(),
    }
}

fn summary(rows: &[Row]) -> ReconciliationSummary {
    let mut s = ReconciliationSummary {
        transactions: u32::try_from(rows.len()).unwrap_or(u32::MAX),
        ..ReconciliationSummary::default()
    };
    let mut missing: HashMap<String, i64> = HashMap::new();
    for r in rows {
        match r.decision.need {
            Need::Eracun => s.eracun += 1,
            Need::Receipt if r.documents.is_empty() => {
                s.receipt_missing += 1;
                *missing.entry(r.tx.currency.trim().to_owned()).or_default() +=
                    r.tx.amount_minor.abs();
            }
            Need::Receipt => s.receipt_covered += 1,
            Need::None => s.none += 1,
            Need::Personal => s.personal += 1,
            Need::Income => s.income += 1,
            Need::Internal => s.internal += 1,
        }
    }
    s.missing_minor = missing
        .into_iter()
        .map(|(c, m)| (c, m.to_string()))
        .collect();
    s
}

impl Finance {
    pub(crate) async fn rpc_monthly_reconciliation(
        &self,
        request: Request<MonthlyReconciliationRequest>,
    ) -> Result<Response<MonthlyReconciliationResponse>, Status> {
        let req = request.get_ref().clone();
        let party = uuid(&req.party_id, "party_id")?;
        let (mut timer, pool, access, _) = self
            .invoice_context("FinanceService/MonthlyReconciliation", &request, &[])
            .await?;
        let r = store::month(pool, &access, party, &req.month)
            .await
            .map(|(rows, policies)| MonthlyReconciliationResponse {
                summary: Some(summary(&rows)),
                rows: rows.iter().map(row_proto).collect(),
                policies: policies.iter().map(policy_proto).collect(),
            });
        self.done_c(&mut timer, r)
    }

    pub(crate) async fn rpc_link_document(
        &self,
        request: Request<LinkDocumentRequest>,
    ) -> Result<Response<LinkDocumentResponse>, Status> {
        let req = request.get_ref().clone();
        let tx = uuid(&req.transaction_id, "transaction_id")?;
        let doc = uuid(&req.document_id, "document_id")?;
        let (mut timer, pool, access, _) = self
            .invoice_context("FinanceService/LinkDocument", &request, &[])
            .await?;
        let r = store::link(pool, &access, tx, doc, req.force)
            .await
            .map(|row| LinkDocumentResponse {
                row: Some(row_proto(&row)),
            });
        self.done_c(&mut timer, r)
    }

    pub(crate) async fn rpc_unlink_document(
        &self,
        request: Request<UnlinkDocumentRequest>,
    ) -> Result<Response<UnlinkDocumentResponse>, Status> {
        let req = request.get_ref().clone();
        let tx = uuid(&req.transaction_id, "transaction_id")?;
        let doc = uuid(&req.document_id, "document_id")?;
        let (mut timer, pool, access, _) = self
            .invoice_context("FinanceService/UnlinkDocument", &request, &[])
            .await?;
        let r = store::unlink(pool, &access, tx, doc)
            .await
            .map(|row| UnlinkDocumentResponse {
                row: Some(row_proto(&row)),
            });
        self.done_c(&mut timer, r)
    }

    pub(crate) async fn rpc_set_transaction_note(
        &self,
        request: Request<SetTransactionNoteRequest>,
    ) -> Result<Response<SetTransactionNoteResponse>, Status> {
        let req = request.get_ref().clone();
        let tx = uuid(&req.transaction_id, "transaction_id")?;
        let (mut timer, pool, access, _) = self
            .invoice_context("FinanceService/SetTransactionNote", &request, &[])
            .await?;
        let r = store::set_note(pool, &access, tx, &req.note)
            .await
            .map(|row| SetTransactionNoteResponse {
                row: Some(row_proto(&row)),
            });
        self.done_c(&mut timer, r)
    }

    pub(crate) async fn rpc_set_counterparty_policy(
        &self,
        request: Request<SetCounterpartyPolicyRequest>,
    ) -> Result<Response<SetCounterpartyPolicyResponse>, Status> {
        let req = request.get_ref().clone();
        let party = uuid(&req.party_id, "party_id")?;
        let need = Need::from_policy(&req.policy).ok_or_else(|| {
            Status::invalid_argument("policy: want eracun, receipt, none or personal")
        })?;
        let (mut timer, pool, access, _) = self
            .invoice_context("FinanceService/SetCounterpartyPolicy", &request, &[])
            .await?;
        let r = store::set_policy(
            pool,
            &access,
            party,
            &req.r#match,
            req.exact,
            need,
            req.note.trim(),
        )
        .await
        .map(|p| SetCounterpartyPolicyResponse {
            policy: Some(policy_proto(&p)),
        });
        self.done_c(&mut timer, r)
    }

    pub(crate) async fn rpc_delete_counterparty_policy(
        &self,
        request: Request<DeleteCounterpartyPolicyRequest>,
    ) -> Result<Response<DeleteCounterpartyPolicyResponse>, Status> {
        let id = uuid(&request.get_ref().id, "id")?;
        let (mut timer, pool, access, _) = self
            .invoice_context("FinanceService/DeleteCounterpartyPolicy", &request, &[])
            .await?;
        let r = store::delete_policy(pool, &access, id)
            .await
            .map(|()| DeleteCounterpartyPolicyResponse {});
        self.done_c(&mut timer, r)
    }
}
