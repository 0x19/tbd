//! The Moj-eRačun connector: the company's incoming e-invoices at its
//! information intermediary (Moj-eRačun, `moj-eracun.hr`), pulled as
//! documents into the same ledger the mailboxes fill.
//!
//! The API is documented with its request and response bodies (the "API V2"
//! reference): `POST /apis/v2/queryInbox` lists what was addressed to the
//! company as JSON rows, and `POST /apis/v2/receive` hands back one document
//! as the UBL 2.1 XML it was delivered as. Every request carries `Username`,
//! `Password`, `CompanyId` (the OIB), an optional `CompanyBu` and the
//! `SoftwareId` Moj-eRačun issued to the integrator. Nothing here changes
//! anything at Moj-eRačun: the import is not confirmed back (`notifyimport`),
//! so the accountant's own software still sees the invoice as new.
//!
//! The UBL is what makes this the good source: supplier, number, date and
//! total are read from the XML structurally and written as facts, and a
//! Croatian e-invoice usually embeds its visual PDF, which becomes the
//! document a person opens. Without one, the XML itself is the document and
//! the facts still stand.
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use serde_json::{Value, json};

use super::{
    Auth, AuthContext, Connector, ConnectorError, Facts, Found, Kind, Linked, Purpose, Reach,
};

/// Production. A row's `config.base_url` overrides it: the demo service is
/// `https://demo.moj-eracun.hr/apis/v2`.
const API_URL: &str = "https://www.moj-eracun.hr/apis/v2";
/// No published limit; one a second is polite and never the bottleneck.
const GAP: Duration = Duration::from_millis(1000);
/// A round stores at most this many; the caller pulls again for the rest.
const ROUND_CAP: usize = 50;
/// A routine pull reaches back this far past the watermark.
const OVERLAP_DAYS: i64 = 7;
/// The inbox statuses that hold a document: sent to us, and delivered.
const STATUSES: [u32; 2] = [30, 40];

/// The kind.
#[derive(Debug, Clone)]
pub struct MojEracun {
    http: reqwest::Client,
    last_call: std::sync::Arc<tokio::sync::Mutex<Option<tokio::time::Instant>>>,
}

impl Default for MojEracun {
    fn default() -> Self {
        Self::new()
    }
}

/// The credentials, as pasted and as sealed.
#[derive(Debug)]
struct Creds {
    username: String,
    password: String,
    company_id: String,
    company_bu: Option<String>,
    software_id: String,
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
        };
        let need = |name: &str| {
            field(name).ok_or_else(|| ConnectorError::Unlinked(format!("credentials lack {name}")))
        };
        Ok(Self {
            username: need("username")?,
            password: need("password")?,
            company_id: need("companyId")?,
            company_bu: field("companyBu"),
            software_id: need("softwareId")?,
        })
    }

    /// The authentication fields every request starts with.
    fn envelope(&self) -> serde_json::Map<String, Value> {
        let mut m = serde_json::Map::new();
        m.insert("Username".into(), Value::String(self.username.clone()));
        m.insert("Password".into(), Value::String(self.password.clone()));
        m.insert("CompanyId".into(), Value::String(self.company_id.clone()));
        if let Some(bu) = &self.company_bu {
            m.insert("CompanyBu".into(), Value::String(bu.clone()));
        }
        m.insert("SoftwareId".into(), Value::String(self.software_id.clone()));
        m
    }
}

/// What the UBL says about itself.
#[derive(Debug, Default)]
struct Ubl {
    facts: Facts,
    /// The embedded visual PDF, when the sender attached one.
    pdf: Option<(String, Vec<u8>)>,
}

impl MojEracun {
    /// Nothing to configure: the credentials are the company's.
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
            .trim_end_matches('/')
            .to_owned()
    }

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

    /// One POST, paced; the body as text with its status. 401 is the
    /// credentials; a JSON `message` is the service's own word for what
    /// went wrong (the reference lists them: an unknown `ElectronicId`, an
    /// archived document, a bad `StatusId`).
    async fn post(
        &self,
        base_url: &str,
        path: &str,
        body: &Value,
    ) -> Result<(reqwest::StatusCode, String), ConnectorError> {
        self.pace().await;
        let resp = self
            .http
            .post(format!("{base_url}/{path}"))
            .header("content-type", "application/json; charset=utf-8")
            .json(body)
            .send()
            .await
            .map_err(|e| ConnectorError::Provider(format!("{path}: {e}")))?;
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
            return Err(ConnectorError::Unlinked(format!(
                "Moj-eRačun refused the credentials ({status}): {}",
                message_in(&text)
            )));
        }
        if !status.is_success() {
            return Err(ConnectorError::Provider(format!(
                "{path}: HTTP {status}: {}",
                message_in(&text)
            )));
        }
        Ok((status, text))
    }

    /// The inbox rows in one status, in a date window.
    async fn query_inbox(
        &self,
        base_url: &str,
        creds: &Creds,
        status_id: u32,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<Value>, ConnectorError> {
        let mut body = creds.envelope();
        body.insert("StatusId".into(), Value::from(status_id));
        body.insert("From".into(), Value::String(from.to_string()));
        body.insert("To".into(), Value::String(to.to_string()));
        let (_, text) = self
            .post(base_url, "queryInbox", &Value::Object(body))
            .await?;
        let v: Value = serde_json::from_str(&text).map_err(|_| {
            ConnectorError::Provider(format!("queryInbox: not JSON: {}", snippet(&text)))
        })?;
        match v {
            Value::Array(rows) => Ok(rows),
            Value::Object(m) => {
                // A validation error comes back as `{ "Field": { "Value", "Messages" } }`.
                Err(ConnectorError::Provider(format!(
                    "queryInbox: {}",
                    message_in(&Value::Object(m).to_string())
                )))
            }
            other => Err(ConnectorError::Provider(format!(
                "queryInbox: unexpected answer: {}",
                snippet(&other.to_string())
            ))),
        }
    }

    /// One document, as the UBL XML it was delivered as.
    async fn receive(
        &self,
        base_url: &str,
        creds: &Creds,
        electronic_id: i64,
    ) -> Result<Vec<u8>, ConnectorError> {
        let mut body = creds.envelope();
        body.insert("ElectronicId".into(), Value::from(electronic_id));
        let (_, text) = self.post(base_url, "receive", &Value::Object(body)).await?;
        let trimmed = text.trim_start();
        if trimmed.starts_with('{') {
            return Err(ConnectorError::Provider(format!(
                "receive {electronic_id}: {}",
                message_in(&text)
            )));
        }
        if !trimmed.starts_with('<') {
            return Err(ConnectorError::Provider(format!(
                "receive {electronic_id}: not XML: {}",
                snippet(&text)
            )));
        }
        Ok(text.into_bytes())
    }
}

#[async_trait]
impl Connector for MojEracun {
    fn kind(&self) -> Kind {
        Kind {
            name: "mojeracun",
            label: "Moj-eRačun",
            description: "The company's incoming e-invoices at Moj-eRačun: the UBL each was delivered as, its embedded PDF, and supplier, number, date and total read from it.",
            auth: Auth::Token,
            consent_note: "The company's Moj-eRačun API user and password, its OIB, the business unit if \
                           one is registered, and the SoftwareId Moj-eRačun issued to the integrator \
                           (integracije@moj-eracun.hr). Read-only: documents are downloaded, nothing \
                           is confirmed, sent or changed at Moj-eRačun.",
            configured: true,
            purposes: &[Purpose::Read],
        }
    }

    /// A token kind has nowhere to send the browser: the page asks for the
    /// credentials and completes directly.
    async fn start(&self, _ctx: &AuthContext) -> Result<String, ConnectorError> {
        Ok(String::new())
    }

    /// `code` is the pasted JSON `{ username, password, companyId, companyBu?,
    /// softwareId }`, proven by one inbox query before it is kept.
    async fn complete(&self, _ctx: &AuthContext, code: &str) -> Result<Linked, ConnectorError> {
        let credentials: Value = serde_json::from_str(code)
            .map_err(|_| ConnectorError::Provider("paste the values as the page asks".into()))?;
        let creds = Creds::from(&credentials)?;
        let today = Utc::now().date_naive();
        self.query_inbox(
            API_URL,
            &creds,
            40,
            today - chrono::Duration::days(1),
            today,
        )
        .await?;
        let mut kept = json!({
            "username": creds.username,
            "password": creds.password,
            "companyId": creds.company_id,
            "softwareId": creds.software_id,
        });
        if let Some(bu) = &creds.company_bu {
            kept["companyBu"] = Value::String(bu.clone());
        }
        Ok(Linked {
            credentials: kept,
            external_id: format!("{}@moj-eracun", creds.company_id),
            label: format!("Moj-eRačun · {}", creds.company_id),
        })
    }

    async fn test(&self, credentials: &Value) -> Result<String, ConnectorError> {
        let creds = Creds::from(credentials)?;
        let today = Utc::now().date_naive();
        let from = today - chrono::Duration::days(30);
        let mut n = 0;
        for status in STATUSES {
            n += self
                .query_inbox(API_URL, &creds, status, from, today)
                .await?
                .len();
        }
        Ok(format!("{n} e-invoices in the last 30 days"))
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
        let today = Utc::now().date_naive();
        let from = (since - chrono::Duration::days(OVERLAP_DAYS)).date_naive();
        let mut rows = Vec::new();
        for status in STATUSES {
            rows.extend(
                self.query_inbox(&base_url, &creds, status, from, today)
                    .await?,
            );
        }
        let mut sent = 0usize;
        for row in rows {
            let Some(id) = row.get("ElectronicId").and_then(Value::as_i64) else {
                continue;
            };
            let external_ref = format!("mer:{id}");
            if seen(&external_ref) {
                continue;
            }
            if sent >= ROUND_CAP {
                return Ok(Reach::Truncated);
            }
            let xml = self.receive(&base_url, &creds, id).await?;
            let ubl = read_ubl(&xml);
            let number = row
                .get("DocumentNr")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
                .map(str::to_owned)
                .or_else(|| ubl.facts.invoice_no.clone())
                .unwrap_or_else(|| id.to_string());
            let sender = row
                .get("SenderBusinessName")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
                .map(str::to_owned)
                .or_else(|| ubl.facts.vendor.clone())
                .unwrap_or_default();
            let mut facts = ubl.facts;
            if facts.vendor.is_none() && !sender.is_empty() {
                facts.vendor = Some(sender.clone());
            }
            if facts.invoice_no.is_none() {
                facts.invoice_no = Some(number.clone());
            }
            let received_at = ["Delivered", "Sent"]
                .iter()
                .find_map(|k| row.get(*k).and_then(Value::as_str))
                .and_then(parse_when);
            let (filename, content_type, bytes) = match ubl.pdf {
                Some((name, pdf)) => (name, "application/pdf".to_owned(), pdf),
                None => (
                    format!("{}.xml", safe(&number)),
                    "application/xml".to_owned(),
                    xml,
                ),
            };
            let found = Found {
                external_ref,
                filename,
                content_type,
                bytes,
                subject: format!("{number} {sender}").trim().to_owned(),
                sender: "Moj-eRačun".into(),
                received_at,
                facts: Some(facts),
            };
            if sink.send(found).await.is_err() {
                return Ok(Reach::Truncated);
            }
            sent += 1;
        }
        Ok(Reach::Complete)
    }
}

/// The supplier, number, date and total of a UBL invoice or credit note,
/// and its embedded PDF, read by element name whatever the namespace
/// prefixes. Anything absent is left for the reader.
fn read_ubl(xml: &[u8]) -> Ubl {
    let Ok(text) = std::str::from_utf8(xml) else {
        return Ubl::default();
    };
    let Ok(doc) = roxmltree::Document::parse(text) else {
        return Ubl::default();
    };
    let root = doc.root_element();
    let supplier = child(root, "AccountingSupplierParty").and_then(|p| child(p, "Party"));
    let vendor = supplier
        .and_then(|p| child(p, "PartyLegalEntity"))
        .and_then(|e| text_of(child(e, "RegistrationName")))
        .or_else(|| {
            supplier
                .and_then(|p| child(p, "PartyName"))
                .and_then(|n| text_of(child(n, "Name")))
        });
    let total = child(root, "LegalMonetaryTotal")
        .and_then(|t| child(t, "PayableAmount"))
        .and_then(|amount| {
            let minor = minor_of(amount.text()?)?;
            let currency = amount
                .attribute("currencyID")
                .map(str::to_ascii_uppercase)
                .or_else(|| text_of(child(root, "DocumentCurrencyCode")))
                .unwrap_or_else(|| "EUR".into());
            Some((minor, currency))
        });
    let pdf = root
        .children()
        .filter(|c| c.is_element() && c.tag_name().name() == "AdditionalDocumentReference")
        .find_map(|r| {
            let obj =
                child(r, "Attachment").and_then(|a| child(a, "EmbeddedDocumentBinaryObject"))?;
            let mime = obj.attribute("mimeCode").unwrap_or("");
            if !mime.eq_ignore_ascii_case("application/pdf") {
                return None;
            }
            let name = obj
                .attribute("filename")
                .filter(|f| !f.is_empty())
                .map(str::to_owned)
                .or_else(|| text_of(child(r, "ID")).map(|id| format!("{}.pdf", safe(&id))))
                .unwrap_or_else(|| "e-racun.pdf".into());
            let bytes = decode_base64(obj.text()?)?;
            Some((name, bytes))
        });
    Ubl {
        facts: Facts {
            vendor,
            doc_date: text_of(child(root, "IssueDate"))
                .and_then(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok()),
            total,
            invoice_no: text_of(child(root, "ID")),
        },
        pdf,
    }
}

/// The first child element with this local name, whatever its namespace.
fn child<'a, 'i>(node: roxmltree::Node<'a, 'i>, name: &str) -> Option<roxmltree::Node<'a, 'i>> {
    node.children()
        .find(|c| c.is_element() && c.tag_name().name() == name)
}

/// The trimmed text of an element, none when empty.
fn text_of(node: Option<roxmltree::Node<'_, '_>>) -> Option<String> {
    node.and_then(|n| n.text())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
}

/// `1234.50` as minor units; UBL amounts use a dot.
fn minor_of(text: &str) -> Option<i64> {
    let t = text.trim();
    let (whole, frac) = t.split_once('.').unwrap_or((t, ""));
    let neg = whole.starts_with('-');
    let whole: i64 = whole.trim_start_matches('-').parse().ok()?;
    let frac: String = frac.chars().chain("00".chars()).take(2).collect();
    let frac: i64 = frac.parse().ok()?;
    let minor = whole * 100 + frac;
    Some(if neg { -minor } else { minor })
}

fn decode_base64(text: &str) -> Option<Vec<u8>> {
    use base64::Engine as _;
    let clean: String = text.chars().filter(|c| !c.is_whitespace()).collect();
    base64::engine::general_purpose::STANDARD
        .decode(&clean)
        .ok()
        .filter(|b| !b.is_empty())
}

/// `2016-04-18T08:13:03.177` (the service's local time, no zone) as UTC.
fn parse_when(s: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .map(|t| t.with_timezone(&Utc))
        .ok()
        .or_else(|| {
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f")
                .or_else(|_| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S"))
                .ok()
                .map(|n| n.and_utc())
        })
}

/// The service's error word: `{ "message": … }`, or a validation object
/// `{ "Field": { "Messages": [...] } }`, or the body itself.
fn message_in(text: &str) -> String {
    let Ok(v) = serde_json::from_str::<Value>(text) else {
        return snippet(text);
    };
    if let Some(m) = v.get("message").and_then(Value::as_str) {
        return m.to_owned();
    }
    if let Some(obj) = v.as_object() {
        for (field, detail) in obj {
            if let Some(msgs) = detail.get("Messages").and_then(Value::as_array) {
                let words: Vec<&str> = msgs.iter().filter_map(Value::as_str).collect();
                return format!("{field}: {}", words.join("; "));
            }
        }
    }
    snippet(text)
}

fn snippet(text: &str) -> String {
    let t = text.trim();
    if t.chars().count() > 120 {
        format!("{}…", t.chars().take(120).collect::<String>())
    } else {
        t.to_owned()
    }
}

/// A file name from a document number: letters, digits, dots and dashes.
fn safe(s: &str) -> String {
    let out: String = s
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '.' || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect();
    if out.is_empty() {
        "document".into()
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    fn creds() -> Value {
        json!({ "username": "1083", "password": "test123", "companyId": "12345678901", "softwareId": "tbd-1" })
    }

    fn ubl(number: &str, with_pdf: bool) -> String {
        use base64::Engine as _;
        let pdf = base64::engine::general_purpose::STANDARD.encode(b"%PDF-1.4 visual");
        let attachment = if with_pdf {
            format!(
                r#"<cac:AdditionalDocumentReference><cbc:ID>{number}</cbc:ID><cac:Attachment><cbc:EmbeddedDocumentBinaryObject mimeCode="application/pdf" filename="{number}.pdf">{pdf}</cbc:EmbeddedDocumentBinaryObject></cac:Attachment></cac:AdditionalDocumentReference>"#
            )
        } else {
            String::new()
        };
        format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<Invoice xmlns="urn:oasis:names:specification:ubl:schema:xsd:Invoice-2"
         xmlns:cac="urn:oasis:names:specification:ubl:schema:xsd:CommonAggregateComponents-2"
         xmlns:cbc="urn:oasis:names:specification:ubl:schema:xsd:CommonBasicComponents-2">
  <cbc:ID>{number}</cbc:ID>
  <cbc:IssueDate>2026-09-03</cbc:IssueDate>
  <cbc:DocumentCurrencyCode>EUR</cbc:DocumentCurrencyCode>
  {attachment}
  <cac:AccountingSupplierParty><cac:Party>
    <cac:PartyName><cbc:Name>Konzum plus</cbc:Name></cac:PartyName>
    <cac:PartyTaxScheme><cbc:CompanyID>HR62226620908</cbc:CompanyID></cac:PartyTaxScheme>
    <cac:PartyLegalEntity><cbc:RegistrationName>KONZUM plus d.o.o.</cbc:RegistrationName></cac:PartyLegalEntity>
  </cac:Party></cac:AccountingSupplierParty>
  <cac:LegalMonetaryTotal><cbc:PayableAmount currencyID="EUR">1234.50</cbc:PayableAmount></cac:LegalMonetaryTotal>
</Invoice>"#
        )
    }

    #[test]
    fn the_ubl_yields_the_facts_and_the_embedded_pdf() {
        let u = read_ubl(ubl("R-2026-17", true).as_bytes());
        assert_eq!(u.facts.vendor.as_deref(), Some("KONZUM plus d.o.o."));
        assert_eq!(u.facts.invoice_no.as_deref(), Some("R-2026-17"));
        assert_eq!(u.facts.doc_date, NaiveDate::from_ymd_opt(2026, 9, 3));
        assert_eq!(u.facts.total, Some((123_450, "EUR".into())));
        let (name, bytes) = u.pdf.unwrap();
        assert_eq!(name, "R-2026-17.pdf");
        assert_eq!(bytes, b"%PDF-1.4 visual");
        assert!(read_ubl(ubl("X", false).as_bytes()).pdf.is_none());
        assert!(read_ubl(b"not xml at all").facts.vendor.is_none());
        assert_eq!(minor_of("-3.5"), Some(-350));
        assert_eq!(minor_of("12"), Some(1200));
        assert_eq!(
            message_in(
                r#"{ "StatusId": { "Value": "70", "Messages": ["StatusId is not valid. Options 30, 40"] } }"#
            ),
            "StatusId: StatusId is not valid. Options 30, 40"
        );
        assert_eq!(
            message_in(r#"{ "message": "ElectronicId not found" }"#),
            "ElectronicId not found"
        );
        assert!(parse_when("2016-04-18T08:13:03.177").is_some());
    }

    #[tokio::test]
    async fn a_pull_queries_both_statuses_receives_each_new_document_and_prefers_the_pdf() {
        use wiremock::matchers::{body_string_contains, method, path};
        let server = wiremock::MockServer::start().await;
        wiremock::Mock::given(method("POST"))
            .and(path("/queryInbox"))
            .and(body_string_contains("\"StatusId\":30"))
            .respond_with(wiremock::ResponseTemplate::new(200).set_body_json(json!([
                { "ElectronicId": 1, "DocumentNr": "R-2026-17", "SenderBusinessName": "Konzum plus", "StatusId": 30, "Sent": "2026-09-03T08:13:03.177", "Delivered": null }
            ])))
            .mount(&server)
            .await;
        wiremock::Mock::given(method("POST"))
            .and(path("/queryInbox"))
            .and(body_string_contains("\"StatusId\":40"))
            .respond_with(wiremock::ResponseTemplate::new(200).set_body_json(json!([
                { "ElectronicId": 2, "DocumentNr": "A-9", "SenderBusinessName": "Hrvatski Telekom", "StatusId": 40, "Sent": "2026-09-01T10:00:00", "Delivered": "2026-09-01T10:00:05" },
                { "ElectronicId": 3, "DocumentNr": "old", "SenderBusinessName": "Seen", "StatusId": 40, "Sent": "2026-08-01T10:00:00", "Delivered": null }
            ])))
            .mount(&server)
            .await;
        wiremock::Mock::given(method("POST"))
            .and(path("/receive"))
            .and(body_string_contains("\"ElectronicId\":1"))
            .respond_with(
                wiremock::ResponseTemplate::new(200).set_body_string(ubl("R-2026-17", true)),
            )
            .mount(&server)
            .await;
        wiremock::Mock::given(method("POST"))
            .and(path("/receive"))
            .and(body_string_contains("\"ElectronicId\":2"))
            .respond_with(wiremock::ResponseTemplate::new(200).set_body_string(ubl("A-9", false)))
            .mount(&server)
            .await;

        let kind = MojEracun::new();
        let (tx, mut rx) = tokio::sync::mpsc::channel(8);
        let config = json!({ "base_url": server.uri() });
        let reach = kind
            .pull(&creds(), &config, Utc::now(), &|r| r == "mer:3", tx)
            .await
            .unwrap();
        assert_eq!(reach, Reach::Complete);
        let mut got = Vec::new();
        while let Some(f) = rx.recv().await {
            got.push(f);
        }
        assert_eq!(got.len(), 2, "the seen one is not received");
        let first = &got[0];
        assert_eq!(first.external_ref, "mer:1");
        assert_eq!(first.filename, "R-2026-17.pdf");
        assert_eq!(first.content_type, "application/pdf");
        assert_eq!(first.bytes, b"%PDF-1.4 visual");
        assert_eq!(first.subject, "R-2026-17 Konzum plus");
        assert_eq!(first.sender, "Moj-eRačun");
        let facts = first.facts.as_ref().unwrap();
        assert_eq!(
            facts.vendor.as_deref(),
            Some("KONZUM plus d.o.o."),
            "the UBL's legal name wins"
        );
        assert_eq!(facts.total, Some((123_450, "EUR".into())));
        let second = &got[1];
        assert_eq!(
            second.filename, "A-9.xml",
            "no PDF inside: the XML is the document"
        );
        assert_eq!(second.content_type, "application/xml");
        assert!(second.received_at.is_some());

        // Every request carried the credentials, capitalised as the API wants.
        let first_body: Value =
            serde_json::from_slice(&server.received_requests().await.unwrap()[0].body).unwrap();
        assert_eq!(first_body["Username"], "1083");
        assert_eq!(first_body["CompanyId"], "12345678901");
        assert_eq!(first_body["SoftwareId"], "tbd-1");
        assert_eq!(first_body["From"].as_str().unwrap().len(), 10);
        let paths: Vec<String> = server
            .received_requests()
            .await
            .unwrap()
            .iter()
            .map(|r| r.url.path().to_owned())
            .collect();
        assert_eq!(
            paths,
            ["/queryInbox", "/queryInbox", "/receive", "/receive"]
        );
    }

    #[tokio::test]
    async fn refused_credentials_unlink_and_the_services_words_are_kept() {
        use wiremock::matchers::{method, path};
        let server = wiremock::MockServer::start().await;
        wiremock::Mock::given(method("POST"))
            .and(path("/queryInbox"))
            .respond_with(
                wiremock::ResponseTemplate::new(401).set_body_json(
                    json!({ "message": "Username and/or Password are not correct" }),
                ),
            )
            .mount(&server)
            .await;
        let kind = MojEracun::new();
        let err = kind
            .query_inbox(
                &server.uri(),
                &Creds::from(&creds()).unwrap(),
                40,
                NaiveDate::from_ymd_opt(2026, 9, 1).unwrap(),
                NaiveDate::from_ymd_opt(2026, 9, 2).unwrap(),
            )
            .await
            .unwrap_err();
        assert!(
            matches!(err, ConnectorError::Unlinked(ref m) if m.contains("Password")),
            "{err}"
        );
        let missing = Creds::from(&json!({ "username": "x" })).unwrap_err();
        assert!(matches!(missing, ConnectorError::Unlinked(_)));
        let bu = Creds::from(&json!({ "username": "u", "password": "p", "companyId": "c", "companyBu": "PJ 1", "softwareId": "s" })).unwrap();
        assert_eq!(bu.envelope()["CompanyBu"], "PJ 1");
    }
}
