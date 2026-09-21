//! The filing RPCs: the company's ePorezna forms listed with their headline
//! figures, and one with everything it carries.

use std::collections::BTreeMap;

use tbd_proto::finance::v1::{
    Filing, GetFilingRequest, GetFilingResponse, ListFilingsRequest, ListFilingsResponse,
};
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::{
    filings::{
        self, Form,
        store::{self, FilingRow, Filter},
    },
    service::Finance,
};

fn uuid(s: &str, field: &str) -> Result<Uuid, Status> {
    Uuid::parse_str(s).map_err(|_| Status::invalid_argument(format!("{field}: not a uuid")))
}

fn strings(v: &serde_json::Value) -> BTreeMap<String, String> {
    v.as_object()
        .map(|o| {
            o.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_owned())))
                .collect()
        })
        .unwrap_or_default()
}

/// A row on the wire; `with_body` adds every value and the rows.
pub(crate) fn filing_proto(f: FilingRow, with_body: bool) -> Filing {
    let values = strings(&f.values);
    let headline = Form::parse(&f.form)
        .map(|form| filings::headline(form, &values))
        .unwrap_or_default();
    Filing {
        id: f.document_id.to_string(),
        party_id: f.party_id.to_string(),
        form: f.form,
        schema: f.schema,
        period_from: f.period_from.map(|d| d.to_string()).unwrap_or_default(),
        period_to: f.period_to.map(|d| d.to_string()).unwrap_or_default(),
        oib: f.oib,
        obveznik: f.obveznik,
        prepared_at: f.prepared_at.map(|t| t.to_rfc3339()).unwrap_or_default(),
        author: f.author,
        filename: f.filename.unwrap_or_default(),
        report_mark: f.report_mark,
        headline: headline.into_iter().collect(),
        values: if with_body {
            values.into_iter().collect()
        } else {
            std::collections::HashMap::new()
        },
        rows_json: if with_body {
            f.rows.to_string()
        } else {
            String::new()
        },
        error: f.error.unwrap_or_default(),
        parsed_at: f.parsed_at.to_rfc3339(),
        parser_version: f.parser_version,
    }
}

impl Finance {
    pub(crate) async fn rpc_list_filings(
        &self,
        request: Request<ListFilingsRequest>,
    ) -> Result<Response<ListFilingsResponse>, Status> {
        let req = request.get_ref().clone();
        let form = if req.form.trim().is_empty() {
            None
        } else {
            Some(Form::parse(&req.form).ok_or_else(|| {
                Status::invalid_argument("form: want pd, pdv, pdv_s, zp, joppd, pd_ipo or tz")
            })?)
        };
        let year = if req.year == 0 {
            None
        } else {
            Some(
                i32::try_from(req.year)
                    .map_err(|_| Status::invalid_argument("year: out of range"))?,
            )
        };
        let (mut timer, pool, _, view) = self
            .invoice_context("FinanceService/ListFilings", &request, &req.party_ids)
            .await?;
        let filter = Filter {
            form,
            year,
            limit: i64::from(if req.limit == 0 {
                100
            } else {
                req.limit.min(500)
            }),
            offset: i64::from(req.offset),
        };
        let r = store::list(pool, &view, &filter)
            .await
            .map(|listing| ListFilingsResponse {
                filings: listing
                    .filings
                    .into_iter()
                    .map(|f| filing_proto(f, false))
                    .collect(),
                total: u32::try_from(listing.total).unwrap_or(u32::MAX),
                years: listing
                    .years
                    .into_iter()
                    .filter_map(|y| u32::try_from(y).ok())
                    .collect(),
            });
        self.done_c(&mut timer, r)
    }

    pub(crate) async fn rpc_get_filing(
        &self,
        request: Request<GetFilingRequest>,
    ) -> Result<Response<GetFilingResponse>, Status> {
        let id = uuid(&request.get_ref().id, "id")?;
        let (mut timer, pool, access, _) = self
            .invoice_context("FinanceService/GetFiling", &request, &[])
            .await?;
        let r = store::get(pool, &access, id)
            .await
            .map(|f| GetFilingResponse {
                filing: Some(filing_proto(f, true)),
            });
        self.done_c(&mut timer, r)
    }
}
