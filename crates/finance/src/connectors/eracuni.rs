//! The e-računi connector: the company's received invoices and its e-invoice
//! inbox at its information intermediary (E-RAČUNI d.o.o., `e-racuni.hr`),
//! pulled as documents into the same ledger the mailboxes fill.
//!
//! The API is one endpoint, `POST /WebServicesHR/API`, a JSON envelope
//! `{ username, secretKey, token, method, parameters }`, and a rate limit of
//! four requests in four seconds (e-racuni.com, "WS API"). Three credentials,
//! pasted by the person: the API user, its API secret key, the organisation's
//! web-services token. Nothing here creates or changes anything at e-računi.
//!
//! What the answers look like is not documented publicly beyond "the PDF comes
//! back as BASE64". So the client is written tolerant -- the payload is looked
//! for under the method's name, `result`, `data`, then at the top level; a list
//! is the first array in it; a file's bytes are the first base64 field with a
//! known name -- and `test` returns the raw shape of an answer that fits none
//! of that, so one round with real credentials settles the names. A guess
//! never becomes a document: what cannot be read is a provider error, not a
//! receipt with empty fields.
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use serde_json::{Value, json};

use super::{
    Auth, AuthContext, Connector, ConnectorError, Facts, Found, Kind, Linked, Purpose, Reach,
};

/// The Croatian endpoint. A row's `config.base_url` overrides it (tests).
const API_URL: &str = "https://e-racuni.hr/WebServicesHR/API";
/// Four requests in four seconds: one a second keeps under it.
const GAP: Duration = Duration::from_millis(1000);
/// A round stores at most this many; the caller pulls again for the rest.
const ROUND_CAP: usize = 50;
/// A routine pull reaches back this far past the watermark.
const OVERLAP_DAYS: i64 = 7;

/// The kind.
#[derive(Debug, Clone)]
pub struct Eracuni {
    http: reqwest::Client,
    last_call: std::sync::Arc<tokio::sync::Mutex<Option<tokio::time::Instant>>>,
}

impl Default for Eracuni {
    fn default() -> Self {
        Self::new()
    }
}

/// The three credentials, as pasted and as sealed.
#[derive(Debug)]
struct Creds {
    username: String,
    secret_key: String,
    token: String,
}

impl Creds {
    fn from(credentials: &Value) -> Result<Self, ConnectorError> {
        let field = |name: &str| {
            credentials
                .get(name)
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_owned)
                .ok_or_else(|| ConnectorError::Unlinked(format!("credentials lack {name}")))
        };
        Ok(Self {
            username: field("username")?,
            secret_key: field("secretKey")?,
            token: field("token")?,
        })
    }
}

impl Eracuni {
    /// Nothing to configure: the credentials are the person's.
    #[must_use]
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(60))
                .build()
                .unwrap_or_default(),
            last_call: std::sync::Arc::new(tokio::sync::Mutex::new(None)),
        }
    }

    fn base_url(config: &Value) -> String {
        config
            .get("base_url")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .unwrap_or(API_URL)
            .to_owned()
    }

    /// One call, paced. A 429 is waited out once; 401/403 or an envelope
    /// error about the login is the credential's fault.
    async fn call(
        &self,
        base_url: &str,
        creds: &Creds,
        method: &str,
        parameters: Value,
    ) -> Result<Value, ConnectorError> {
        let body = json!({
            "username": creds.username,
            "secretKey": creds.secret_key,
            "token": creds.token,
            "method": method,
            "parameters": parameters,
        });
        for attempt in 0..2 {
            self.pace().await;
            let resp = self
                .http
                .post(base_url)
                .json(&body)
                .send()
                .await
                .map_err(|e| ConnectorError::Provider(format!("{method}: {e}")))?;
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            if status == reqwest::StatusCode::TOO_MANY_REQUESTS && attempt == 0 {
                tokio::time::sleep(Duration::from_secs(4)).await;
                continue;
            }
            if status == reqwest::StatusCode::UNAUTHORIZED
                || status == reqwest::StatusCode::FORBIDDEN
            {
                return Err(ConnectorError::Unlinked(format!(
                    "e-računi refused the credentials ({status})"
                )));
            }
            if !status.is_success() {
                return Err(ConnectorError::Provider(format!(
                    "{method}: HTTP {status}: {}",
                    snippet(&text)
                )));
            }
            let value: Value = serde_json::from_str(&text).map_err(|_| {
                ConnectorError::Provider(format!("{method}: not JSON: {}", snippet(&text)))
            })?;
            if let Some(message) = error_in(&value) {
                let lower = message.to_ascii_lowercase();
                return Err(
                    if [
                        "login",
                        "auth",
                        "token",
                        "secret",
                        "credential",
                        "password",
                        "user",
                    ]
                    .iter()
                    .any(|w| lower.contains(w))
                    {
                        ConnectorError::Unlinked(format!("e-računi: {message}"))
                    } else {
                        ConnectorError::Provider(format!("{method}: {message}"))
                    },
                );
            }
            return Ok(payload(value, method));
        }
        Err(ConnectorError::Provider(format!(
            "{method}: rate limited twice"
        )))
    }

    /// One request a second, across the kind's calls.
    async fn pace(&self) {
        let mut last = self.last_call.lock().await;
        if let Some(at) = *last {
            let since = at.elapsed();
            if since < GAP {
                tokio::time::sleep(GAP.saturating_sub(since)).await;
            }
        }
        *last = Some(tokio::time::Instant::now());
    }

    /// The received invoices in a date window, as the API lists them.
    async fn received_list(
        &self,
        base_url: &str,
        creds: &Creds,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<Value>, ConnectorError> {
        let v = self
            .call(
                base_url,
                creds,
                "ReceivedInvoiceList",
                json!({ "dateFrom": from.to_string(), "dateTo": to.to_string() }),
            )
            .await?;
        Ok(list_in(v))
    }

    /// A document's file: the first PDF among its attachments, else the first
    /// attachment there is (the e-invoice XML).
    async fn received_file(
        &self,
        base_url: &str,
        creds: &Creds,
        document_id: &str,
    ) -> Result<Option<(String, String, Vec<u8>)>, ConnectorError> {
        let listed = self
            .call(
                base_url,
                creds,
                "ReceivedInvoiceAttachmentList",
                json!({ "documentID": document_id }),
            )
            .await?;
        let attachments = list_in(listed);
        let pick = attachments
            .iter()
            .find(|a| name_of(a).to_ascii_lowercase().ends_with(".pdf"))
            .or_else(|| attachments.first());
        let Some(att) = pick else {
            return Ok(None);
        };
        let att_id = id_of(att).unwrap_or_default();
        let got = self
            .call(
                base_url,
                creds,
                "ReceivedInvoiceAttachmentGet",
                json!({ "documentID": document_id, "attachmentID": att_id }),
            )
            .await?;
        let name = {
            let n = name_of(&got);
            if n.is_empty() { name_of(att) } else { n }
        };
        let bytes = bytes_in(&got).ok_or_else(|| {
            ConnectorError::Provider(format!(
                "ReceivedInvoiceAttachmentGet: no file in the answer ({})",
                shape(&got)
            ))
        })?;
        let name = if name.is_empty() {
            format!("{document_id}.pdf")
        } else {
            name
        };
        Ok(Some((content_type_of(&name), name, bytes)))
    }

    /// The received invoices: list, then per new one the facts and the file.
    #[allow(clippy::too_many_arguments)]
    async fn pull_received(
        &self,
        base_url: &str,
        creds: &Creds,
        from: NaiveDate,
        today: NaiveDate,
        seen: &(dyn for<'a> Fn(&'a str) -> bool + Sync),
        sink: &tokio::sync::mpsc::Sender<Found>,
        sent: &mut usize,
    ) -> Result<Reach, ConnectorError> {
        for row in self.received_list(base_url, creds, from, today).await? {
            let Some(id) = id_of(&row) else { continue };
            let external_ref = format!("received:{id}");
            if seen(&external_ref) {
                continue;
            }
            if *sent >= ROUND_CAP {
                return Ok(Reach::Truncated);
            }
            let detail = self
                .call(
                    base_url,
                    creds,
                    "ReceivedInvoiceGet",
                    json!({ "documentID": id }),
                )
                .await?;
            let facts = facts_of(&detail);
            let Some((content_type, filename, bytes)) =
                self.received_file(base_url, creds, &id).await?
            else {
                tracing::debug!(document = %id, "e-računi: received invoice without a file");
                continue;
            };
            let found = Found {
                external_ref,
                filename,
                content_type,
                bytes,
                subject: format!(
                    "{} {}",
                    facts.invoice_no.clone().unwrap_or_default(),
                    facts.vendor.clone().unwrap_or_default()
                )
                .trim()
                .to_owned(),
                sender: "e-računi".into(),
                received_at: facts
                    .doc_date
                    .and_then(|d| d.and_hms_opt(0, 0, 0))
                    .map(|t| t.and_utc()),
                facts: Some(facts),
            };
            if sink.send(found).await.is_err() {
                return Ok(Reach::Truncated);
            }
            *sent += 1;
        }
        Ok(Reach::Complete)
    }

    /// The e-invoice inbox: list, then per new entry its contents.
    #[allow(clippy::too_many_arguments)]
    async fn pull_inbox(
        &self,
        base_url: &str,
        creds: &Creds,
        from: NaiveDate,
        today: NaiveDate,
        seen: &(dyn for<'a> Fn(&'a str) -> bool + Sync),
        sink: &tokio::sync::mpsc::Sender<Found>,
        sent: &mut usize,
    ) -> Result<Reach, ConnectorError> {
        let listed = self
                .call(
                    base_url,
                    creds,
                    "DocumentInboxList",
                    json!({ "type": "PurchaseInvoice", "receivalDateFrom": from.to_string(), "receivalDateTo": today.to_string() }),
                )
                .await?;
        for row in list_in(listed) {
            let Some(id) = id_of(&row) else { continue };
            let external_ref = format!("inbox:{id}");
            if seen(&external_ref) {
                continue;
            }
            if *sent >= ROUND_CAP {
                return Ok(Reach::Truncated);
            }
            let entry = self
                .call(
                    base_url,
                    creds,
                    "DocumentInboxEntryGet",
                    json!({ "id": id }),
                )
                .await?;
            let Some(bytes) = bytes_in(&entry) else {
                tracing::debug!(entry = %id, "e-računi: inbox entry without a file");
                continue;
            };
            let name = {
                let n = name_of(&entry);
                if n.is_empty() { name_of(&row) } else { n }
            };
            let filename = if name.is_empty() {
                format!("inbox-{id}.pdf")
            } else {
                name
            };
            let facts = facts_of(&row);
            let found = Found {
                external_ref,
                content_type: content_type_of(&filename),
                filename,
                bytes,
                subject: format!(
                    "{} {}",
                    facts.invoice_no.clone().unwrap_or_default(),
                    facts.vendor.clone().unwrap_or_default()
                )
                .trim()
                .to_owned(),
                sender: "e-računi".into(),
                received_at: date_in(&row, &["receivedDate", "enteredTS", "documentDate"])
                    .and_then(|d| d.and_hms_opt(0, 0, 0))
                    .map(|t| t.and_utc()),
                facts: Some(facts),
            };
            if sink.send(found).await.is_err() {
                return Ok(Reach::Truncated);
            }
            *sent += 1;
        }
        Ok(Reach::Complete)
    }
}

#[async_trait]
impl Connector for Eracuni {
    fn kind(&self) -> Kind {
        Kind {
            name: "eracuni",
            label: "e-računi",
            description: "The company's received invoices and its e-invoice inbox at e-računi, with supplier, number, date and total as e-računi knows them.",
            auth: Auth::Token,
            consent_note: "Your e-računi API user, its API secret key and the company's web-services \
                           token (e-računi: Postavke → Web servisi). Read-only: nothing is created \
                           or changed at e-računi.",
            configured: true,
            purposes: &[Purpose::Read],
        }
    }

    /// A token kind has nowhere to send the browser: the page asks for the
    /// credentials and completes directly.
    async fn start(&self, _ctx: &AuthContext) -> Result<String, ConnectorError> {
        Ok(String::new())
    }

    /// `code` is the pasted JSON `{ "username", "secretKey", "token" }`,
    /// proven by one cheap call before it is kept.
    async fn complete(&self, _ctx: &AuthContext, code: &str) -> Result<Linked, ConnectorError> {
        let credentials: Value = serde_json::from_str(code).map_err(|_| {
            ConnectorError::Provider("paste the three values as the page asks".into())
        })?;
        let creds = Creds::from(&credentials)?;
        let today = Utc::now().date_naive();
        self.received_list(API_URL, &creds, today - chrono::Duration::days(1), today)
            .await?;
        Ok(Linked {
            credentials: json!({
                "username": creds.username,
                "secretKey": creds.secret_key,
                "token": creds.token,
            }),
            external_id: format!("{}@e-racuni", creds.username),
            label: format!("e-računi · {}", creds.username),
        })
    }

    async fn test(&self, credentials: &Value) -> Result<String, ConnectorError> {
        let creds = Creds::from(credentials)?;
        let today = Utc::now().date_naive();
        let rows = self
            .received_list(API_URL, &creds, today - chrono::Duration::days(30), today)
            .await?;
        Ok(format!(
            "{} received invoices in the last 30 days",
            rows.len()
        ))
    }

    async fn pull(
        &self,
        credentials: &Value,
        config: &Value,
        since: DateTime<Utc>,
        seen: &(dyn for<'a> Fn(&'a str) -> bool + Sync),
        sink: tokio::sync::mpsc::Sender<Found>,
    ) -> Result<Reach, ConnectorError> {
        let creds = Creds::from(credentials)?;
        let base_url = Self::base_url(config);
        let source = config
            .get("source")
            .and_then(Value::as_str)
            .unwrap_or("both");
        let today = Utc::now().date_naive();
        let from = (since - chrono::Duration::days(OVERLAP_DAYS)).date_naive();
        let mut sent = 0usize;
        if source != "inbox"
            && self
                .pull_received(&base_url, &creds, from, today, seen, &sink, &mut sent)
                .await?
                == Reach::Truncated
        {
            return Ok(Reach::Truncated);
        }
        if source != "received"
            && self
                .pull_inbox(&base_url, &creds, from, today, seen, &sink, &mut sent)
                .await?
                == Reach::Truncated
        {
            return Ok(Reach::Truncated);
        }
        Ok(Reach::Complete)
    }
}

/// The first hundred characters of a body, for an error message.
fn snippet(text: &str) -> String {
    let t = text.trim();
    if t.chars().count() > 100 {
        format!("{}…", t.chars().take(100).collect::<String>())
    } else {
        t.to_owned()
    }
}

/// A one-line description of an answer's shape, for the person reading
/// the test result: the keys, not the values.
fn shape(v: &Value) -> String {
    match v {
        Value::Object(m) => format!(
            "object with {}",
            m.keys().cloned().collect::<Vec<_>>().join(", ")
        ),
        Value::Array(a) => format!("array of {}", a.len()),
        other => format!("{other}"),
    }
}

/// An error the envelope carries, under any of the names such APIs use.
fn error_in(v: &Value) -> Option<String> {
    let m = v.as_object()?;
    for key in ["error", "errorMessage", "message"] {
        match m.get(key) {
            Some(Value::String(s)) if !s.is_empty() && key != "message" => return Some(s.clone()),
            Some(Value::Object(o)) => {
                if let Some(Value::String(s)) = o.get("message").or_else(|| o.get("description")) {
                    return Some(s.clone());
                }
            }
            _ => {}
        }
    }
    match m.get("status") {
        Some(Value::String(s)) if s.eq_ignore_ascii_case("error") => Some(
            m.get("message")
                .and_then(Value::as_str)
                .unwrap_or("error")
                .to_owned(),
        ),
        _ => None,
    }
}

/// The payload of an answer: under the method's name, `result`, `data`, or
/// the answer itself.
fn payload(v: Value, method: &str) -> Value {
    if let Value::Object(mut m) = v {
        for key in [method, "result", "data", "response"] {
            if let Some(inner) = m.remove(key) {
                return inner;
            }
        }
        Value::Object(m)
    } else {
        v
    }
}

/// The first array in a payload: the payload itself, or its first array
/// value (`ReceivedInvoice`, `items`, `list`, whatever it is called).
fn list_in(v: Value) -> Vec<Value> {
    match v {
        Value::Array(a) => a,
        Value::Object(m) => m
            .into_iter()
            .find_map(|(_, x)| match x {
                Value::Array(a) => Some(a),
                _ => None,
            })
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

fn str_in<'a>(v: &'a Value, keys: &[&str]) -> Option<&'a str> {
    keys.iter().find_map(|k| match v.get(k) {
        Some(Value::String(s)) if !s.trim().is_empty() => Some(s.as_str()),
        Some(Value::Number(n)) => {
            // An id may arrive as a number; keep it as text.
            let _ = n;
            None
        }
        _ => None,
    })
}

/// A document's id, whatever the API calls it.
fn id_of(v: &Value) -> Option<String> {
    for key in ["documentID", "documentId", "id", "ID", "sequentialNumber"] {
        match v.get(key) {
            Some(Value::String(s)) if !s.is_empty() => return Some(s.clone()),
            Some(Value::Number(n)) => return Some(n.to_string()),
            _ => {}
        }
    }
    None
}

fn name_of(v: &Value) -> String {
    str_in(v, &["fileName", "filename", "name", "attachmentName"])
        .unwrap_or_default()
        .to_owned()
}

fn content_type_of(name: &str) -> String {
    let ext = std::path::Path::new(name)
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();
    match ext.as_str() {
        "pdf" => "application/pdf".into(),
        "xml" => "application/xml".into(),
        _ => "application/octet-stream".into(),
    }
}

/// The file's bytes: the first base64 field with a known name, whitespace
/// tolerated, standard or URL-safe alphabet.
fn bytes_in(v: &Value) -> Option<Vec<u8>> {
    use base64::Engine as _;
    let data = str_in(
        v,
        &[
            "contents",
            "content",
            "data",
            "file",
            "pdf",
            "base64",
            "fileContent",
        ],
    )?;
    let clean: String = data.chars().filter(|c| !c.is_whitespace()).collect();
    base64::engine::general_purpose::STANDARD
        .decode(&clean)
        .or_else(|_| {
            base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(clean.trim_end_matches('='))
        })
        .ok()
        .filter(|b| !b.is_empty())
}

fn date_in(v: &Value, keys: &[&str]) -> Option<NaiveDate> {
    let s = str_in(v, keys)?;
    let head: String = s.chars().take(10).collect();
    NaiveDate::parse_from_str(&head, "%Y-%m-%d")
        .or_else(|_| NaiveDate::parse_from_str(&head, "%d.%m.%Y"))
        .ok()
}

/// An amount as minor units: a JSON number, or a string with either
/// separator; the last separator is the decimal one.
fn minor_in(v: &Value, keys: &[&str]) -> Option<i64> {
    for k in keys {
        match v.get(k) {
            Some(Value::Number(n)) => {
                let f = n.as_f64()?;
                #[allow(clippy::cast_possible_truncation)]
                return Some((f * 100.0).round() as i64);
            }
            Some(Value::String(s)) if !s.trim().is_empty() => {
                let t: String = s.chars().filter(|c| !c.is_whitespace()).collect();
                let at = t.rfind(['.', ',']);
                let (whole, frac) = match at {
                    Some(i) => (&t[..i], &t[i + 1..]),
                    None => (t.as_str(), ""),
                };
                let whole: String = whole
                    .chars()
                    .filter(|c| c.is_ascii_digit() || *c == '-')
                    .collect();
                let frac: String = frac
                    .chars()
                    .filter(char::is_ascii_digit)
                    .chain("00".chars())
                    .take(2)
                    .collect();
                let neg = whole.starts_with('-');
                let whole: i64 = whole.trim_start_matches('-').parse().unwrap_or(0);
                let frac: i64 = frac.parse().unwrap_or(0);
                let minor = whole * 100 + frac;
                return Some(if neg { -minor } else { minor });
            }
            _ => {}
        }
    }
    None
}

/// What e-računi says about the invoice, under the names such answers use.
fn facts_of(v: &Value) -> Facts {
    let vendor = str_in(
        v,
        &[
            "supplierName",
            "partnerName",
            "senderName",
            "customerName",
            "supplier",
            "partner",
        ],
    )
    .map(str::to_owned)
    .or_else(|| {
        v.get("supplier")
            .or_else(|| v.get("partner"))
            .and_then(|p| str_in(p, &["name", "companyName"]))
            .map(str::to_owned)
    });
    let total = minor_in(
        v,
        &[
            "totalAmount",
            "total",
            "amountTotal",
            "grossAmount",
            "amount",
            "totalWithVAT",
        ],
    )
    .map(|m| {
        (
            m,
            str_in(v, &["currency", "currencyCode"])
                .unwrap_or("EUR")
                .to_ascii_uppercase(),
        )
    });
    Facts {
        vendor,
        doc_date: date_in(v, &["date", "documentDate", "invoiceDate", "dateOfIssue"]),
        total,
        invoice_no: str_in(
            v,
            &[
                "documentNumber",
                "number",
                "invoiceNumber",
                "supplierNumber",
                "supplierInvoiceNumber",
            ],
        )
        .map(str::to_owned),
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    fn creds() -> Value {
        json!({ "username": "api", "secretKey": "s3cr3t", "token": "T0K3N" })
    }

    fn pdf_b64() -> String {
        use base64::Engine as _;
        base64::engine::general_purpose::STANDARD.encode(b"%PDF-1.4 fake")
    }

    /// A server that answers each method from a script, recording bodies.
    async fn server(answers: Vec<(&'static str, Value)>) -> wiremock::MockServer {
        use wiremock::matchers::{body_string_contains, method, path};
        let server = wiremock::MockServer::start().await;
        for (m, answer) in answers {
            wiremock::Mock::given(method("POST"))
                .and(path("/api"))
                .and(body_string_contains(format!("\"method\":\"{m}\"")))
                .respond_with(wiremock::ResponseTemplate::new(200).set_body_json(answer))
                .mount(&server)
                .await;
        }
        server
    }

    async fn calls(server: &wiremock::MockServer) -> Vec<String> {
        server
            .received_requests()
            .await
            .unwrap_or_default()
            .iter()
            .map(|r| {
                let v: Value = serde_json::from_slice(&r.body).unwrap();
                v["method"].as_str().unwrap().to_owned()
            })
            .collect()
    }

    #[test]
    fn the_payload_is_found_wherever_the_api_puts_it_and_facts_are_read() {
        let under_method = json!({ "ReceivedInvoiceList": [{ "documentID": "1" }] });
        assert_eq!(
            list_in(payload(under_method, "ReceivedInvoiceList")).len(),
            1
        );
        let under_result = json!({ "result": { "items": [{ "id": 7 }, { "id": 8 }] } });
        assert_eq!(list_in(payload(under_result, "X")).len(), 2);
        let bare = json!([{ "id": "a" }]);
        assert_eq!(list_in(payload(bare, "X")).len(), 1);

        let f = facts_of(&json!({
            "documentNumber": "2026-000042", "supplierName": "Hetzner Online GmbH",
            "date": "2026-09-03", "totalAmount": "1.234,50", "currency": "eur"
        }));
        assert_eq!(f.invoice_no.as_deref(), Some("2026-000042"));
        assert_eq!(f.vendor.as_deref(), Some("Hetzner Online GmbH"));
        assert_eq!(f.doc_date, NaiveDate::from_ymd_opt(2026, 9, 3));
        assert_eq!(f.total, Some((123_450, "EUR".into())));
        assert_eq!(minor_in(&json!({ "total": 12.5 }), &["total"]), Some(1250));
        assert_eq!(
            minor_in(&json!({ "total": "12.50" }), &["total"]),
            Some(1250)
        );
        assert_eq!(minor_in(&json!({ "total": "-3" }), &["total"]), Some(-300));

        assert_eq!(
            error_in(&json!({ "error": { "message": "Invalid login" } })).as_deref(),
            Some("Invalid login")
        );
        assert_eq!(
            error_in(&json!({ "status": "error", "message": "boom" })).as_deref(),
            Some("boom")
        );
        assert!(error_in(&json!({ "result": [] })).is_none());
    }

    #[tokio::test]
    async fn a_pull_lists_gets_and_fetches_the_pdf_paced_and_skips_what_it_saw() {
        let server = server(vec![
            (
                "ReceivedInvoiceList",
                json!({ "result": [{ "documentID": "60:1" }, { "documentID": "60:2" }] }),
            ),
            (
                "ReceivedInvoiceGet",
                json!({ "result": { "documentID": "60:1", "documentNumber": "R-1", "supplierName": "Hetzner", "date": "2026-09-02", "totalAmount": 42.0, "currency": "EUR" } }),
            ),
            (
                "ReceivedInvoiceAttachmentList",
                json!({ "result": [{ "id": "a-xml", "fileName": "r1.xml" }, { "id": "a-pdf", "fileName": "r1.pdf" }] }),
            ),
            (
                "ReceivedInvoiceAttachmentGet",
                json!({ "result": { "fileName": "r1.pdf", "contents": pdf_b64() } }),
            ),
        ])
        .await;
        let kind = Eracuni::new();
        let (tx, mut rx) = tokio::sync::mpsc::channel(8);
        let config = json!({ "base_url": format!("{}/api", server.uri()), "source": "received" });
        let started = tokio::time::Instant::now();
        let reach = kind
            .pull(
                &creds(),
                &config,
                Utc::now() - chrono::Duration::days(30),
                &|r| r == "received:60:2",
                tx,
            )
            .await
            .unwrap();
        assert_eq!(reach, Reach::Complete);
        let found = rx.recv().await.unwrap();
        assert!(rx.recv().await.is_none(), "the seen one is not fetched");
        assert_eq!(found.external_ref, "received:60:1");
        assert_eq!(found.filename, "r1.pdf");
        assert_eq!(found.content_type, "application/pdf");
        assert_eq!(found.bytes, b"%PDF-1.4 fake");
        assert_eq!(found.sender, "e-računi");
        assert_eq!(found.subject, "R-1 Hetzner");
        let facts = found.facts.unwrap();
        assert_eq!(facts.total, Some((4200, "EUR".into())));
        assert_eq!(facts.vendor.as_deref(), Some("Hetzner"));
        let seq = calls(&server).await;
        assert_eq!(
            seq,
            vec![
                "ReceivedInvoiceList",
                "ReceivedInvoiceGet",
                "ReceivedInvoiceAttachmentList",
                "ReceivedInvoiceAttachmentGet"
            ]
        );
        // Four calls, one a second: at least three seconds passed.
        assert!(
            started.elapsed() >= Duration::from_secs(3),
            "{:?}",
            started.elapsed()
        );
        // The credentials rode in the envelope, the PDF was preferred.
        let first: Value =
            serde_json::from_slice(&server.received_requests().await.unwrap()[0].body).unwrap();
        assert_eq!(first["secretKey"], "s3cr3t");
        assert_eq!(first["token"], "T0K3N");
        let get: Value =
            serde_json::from_slice(&server.received_requests().await.unwrap()[3].body).unwrap();
        assert_eq!(get["parameters"]["attachmentID"], "a-pdf");
    }

    #[tokio::test]
    async fn the_inbox_is_pulled_too_and_a_round_is_capped() {
        let entries: Vec<Value> = (0..3)
            .map(|i| json!({ "id": i, "documentNumber": format!("I-{i}"), "senderName": "Konzum", "receivedDate": "2026-09-10", "fileName": format!("i{i}.pdf") }))
            .collect();
        let server = server(vec![
            ("DocumentInboxList", json!({ "DocumentInboxList": entries })),
            ("DocumentInboxEntryGet", json!({ "contents": pdf_b64() })),
        ])
        .await;
        let kind = Eracuni::new();
        let (tx, mut rx) = tokio::sync::mpsc::channel(8);
        let config = json!({ "base_url": format!("{}/api", server.uri()), "source": "inbox" });
        let reach = kind
            .pull(&creds(), &config, Utc::now(), &|_| false, tx)
            .await
            .unwrap();
        assert_eq!(reach, Reach::Complete);
        let mut got = Vec::new();
        while let Some(f) = rx.recv().await {
            got.push(f);
        }
        assert_eq!(got.len(), 3);
        assert_eq!(got[0].external_ref, "inbox:0");
        assert_eq!(got[0].filename, "i0.pdf");
        assert_eq!(
            got[0].facts.as_ref().unwrap().vendor.as_deref(),
            Some("Konzum")
        );
        assert_eq!(
            got[0].facts.as_ref().unwrap().invoice_no.as_deref(),
            Some("I-0")
        );
    }

    #[tokio::test]
    async fn refused_credentials_unlink_and_a_wrong_shape_is_named() {
        use wiremock::matchers::{method, path};
        let server = wiremock::MockServer::start().await;
        wiremock::Mock::given(method("POST"))
            .and(path("/api"))
            .respond_with(
                wiremock::ResponseTemplate::new(200)
                    .set_body_json(json!({ "error": "Invalid API token" })),
            )
            .mount(&server)
            .await;
        let kind = Eracuni::new();
        let err = kind
            .call(
                &format!("{}/api", server.uri()),
                &Creds::from(&creds()).unwrap(),
                "ReceivedInvoiceList",
                json!({}),
            )
            .await
            .unwrap_err();
        assert!(matches!(err, ConnectorError::Unlinked(_)), "{err}");

        let odd = json!({ "weird": { "deep": true } });
        assert!(bytes_in(&odd).is_none());
        assert_eq!(shape(&odd), "object with weird");
        let missing = Creds::from(&json!({ "username": "x" })).unwrap_err();
        assert!(matches!(missing, ConnectorError::Unlinked(_)));
    }
}
