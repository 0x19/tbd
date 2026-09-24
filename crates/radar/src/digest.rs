//! Writing a digest: the week's items for one language go to the llm service
//! as `svc:radar`, with a prompt that asks for four fixed headings; the answer
//! is split on them. The service has no JSON mode, so the headings are the
//! contract, and an answer missing one is refused rather than stored half.

use std::time::Duration;

use base64::Engine as _;
use chrono::{DateTime, Datelike, Utc};
use tbd_proto::llm::v1::{GenerateRequest, Message, Tier, llm_service_client::LlmServiceClient};
use tonic::{
    Status,
    service::{Interceptor, interceptor::InterceptedService},
    transport::{Channel, Endpoint},
};

use crate::{
    config::Llm,
    store::{DigestRow, ItemRow},
};

/// The subject the radar calls the llm service as; its daily budget is this
/// subject's.
pub const SERVICE_SUBJECT: &str = "svc:radar";

/// The four headings, in order: the answer's contract.
pub const HEADINGS: [&str; 4] = [
    "## What changed",
    "## Why it matters",
    "## Ten-minute drill",
    "## Avatar script",
];

/// Why a digest was not written.
#[derive(Debug, thiserror::Error)]
pub enum DigestError {
    /// No llm service is configured.
    #[error("no llm service configured")]
    NotConfigured,
    /// The llm URL is not a URL.
    #[error("llm url: {0}")]
    Url(#[from] tonic::transport::Error),
    /// The llm service answered with an error.
    #[error("llm: {0}")]
    Llm(#[from] Status),
    /// The llm service took longer than `[llm] timeout_secs`.
    #[error("llm: no answer within {0} s")]
    Timeout(u64),
    /// The answer lacks a heading or a section is empty.
    #[error("the answer has no usable {0:?} section")]
    Missing(&'static str),
}

/// Puts the service subject on every call, the way Envoy would forward a
/// verified caller: the internal listener is the only way in, so a
/// service-to-service call carries its own claims.
#[derive(Debug, Clone, Copy)]
struct ServiceCaller;

impl Interceptor for ServiceCaller {
    fn call(&mut self, mut request: tonic::Request<()>) -> Result<tonic::Request<()>, Status> {
        let claims = serde_json::json!({ "sub": SERVICE_SUBJECT, "scp": ["tbd.llm"] });
        let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(claims.to_string());
        if let Ok(value) = payload.parse() {
            request
                .metadata_mut()
                .insert(tbd_common::principal::PAYLOAD_HEADER, value);
        }
        Ok(request)
    }
}

/// Writes digests through the llm service.
#[derive(Debug, Clone)]
pub struct Writer {
    client: LlmServiceClient<InterceptedService<Channel, ServiceCaller>>,
    config: Llm,
}

impl Writer {
    /// A writer for `[llm]`, or `None` when no URL is configured. The channel
    /// is lazy: an unreachable service fails the digest, not the start.
    ///
    /// # Errors
    /// The URL is malformed.
    pub fn new(config: &Llm) -> Result<Option<Self>, DigestError> {
        if config.url.is_empty() {
            return Ok(None);
        }
        let channel = Endpoint::from_shared(config.url.clone())?.connect_lazy();
        Ok(Some(Self {
            client: LlmServiceClient::with_interceptor(channel, ServiceCaller),
            config: config.clone(),
        }))
    }

    /// Write one digest from `items`.
    ///
    /// # Errors
    /// The llm service failed, timed out, or answered without the headings.
    pub async fn write(
        &self,
        week: &str,
        language: &str,
        lang: &str,
        items: &[ItemRow],
    ) -> Result<DigestRow, DigestError> {
        let request = GenerateRequest {
            messages: vec![
                Message {
                    role: "system".to_owned(),
                    content: system_prompt(language, lang),
                },
                Message {
                    role: "user".to_owned(),
                    content: user_prompt(week, language, items),
                },
            ],
            tier: if self.config.tier == "fast" {
                Tier::Fast
            } else {
                Tier::Deep
            }
            .into(),
            max_tokens: Some(self.config.max_tokens),
            temperature: Some(self.config.temperature),
            session_id: String::new(),
            reasoning: None,
        };
        let deadline = Duration::from_secs(self.config.timeout_secs);
        let mut client = self.client.clone();
        let (text, model, stub) = tokio::time::timeout(deadline, async move {
            let mut stream = client.generate(request).await?.into_inner();
            let mut text = String::new();
            let mut model = String::new();
            let mut stub = false;
            while let Some(chunk) = stream.message().await? {
                if !chunk.model.is_empty() {
                    model = chunk.model;
                }
                stub |= chunk.stub;
                if !chunk.reasoning {
                    text.push_str(&chunk.text);
                }
                if chunk.done {
                    break;
                }
            }
            Ok::<_, Status>((text, model, stub))
        })
        .await
        .map_err(|_| DigestError::Timeout(self.config.timeout_secs))??;
        let sections = parse_sections(&text)?;
        Ok(DigestRow {
            week: week.to_owned(),
            language: language.to_owned(),
            lang: lang.to_owned(),
            changed: sections[0].clone(),
            why: sections[1].clone(),
            drill: sections[2].clone(),
            script: sections[3].clone(),
            item_count: i32::try_from(items.len()).unwrap_or(i32::MAX),
            model,
            stub,
            ..DigestRow::default()
        })
    }
}

fn language_name(language: &str) -> &'static str {
    if language == "go" { "Go" } else { "Rust" }
}

/// The system prompt: who writes, for whom, in which language, and the four
/// headings that must appear exactly.
#[must_use]
pub fn system_prompt(language: &str, lang: &str) -> String {
    let name = language_name(language);
    let reader = if lang == "hr" { "Croatian" } else { "English" };
    format!(
        "You write the weekly Radar for working {name} engineers: what changed in {name} this week \
         and why it matters to someone who ships {name} in production. Use only the items given; \
         never invent a release, a version number, a date or a feature. If the items are thin, \
         say so briefly instead of padding. Write in {reader}. Answer in Markdown with exactly \
         these four headings, in this order, written exactly as shown in English even when the \
         text is {reader}:\n\n\
         ## What changed\n\
         Five to eight bullets, each one item, each with its link.\n\n\
         ## Why it matters\n\
         Two short paragraphs on what a production engineer should do or watch.\n\n\
         ## Ten-minute drill\n\
         One small exercise a reader can do in ten minutes that uses something from this week.\n\n\
         ## Avatar script\n\
         About 150 words of plain spoken text for a presenter to read aloud in sixty seconds: \
         no Markdown, no links, no lists, first person plural, calm."
    )
}

/// The user prompt: the week and its items, newest first.
#[must_use]
pub fn user_prompt(week: &str, language: &str, items: &[ItemRow]) -> String {
    use std::fmt::Write as _;
    let mut s = format!("{} items for week {week}:\n\n", language_name(language));
    for r in items {
        let i = &r.item;
        let _ = writeln!(
            s,
            "- [{}] {} ({}), {}",
            i.source,
            i.title,
            i.published_at.format("%Y-%m-%d"),
            i.url
        );
        if !i.summary.is_empty() {
            let _ = writeln!(s, "  {}", i.summary);
        }
    }
    s
}

/// Split an answer on [`HEADINGS`]; every section must be present and
/// non-empty. Text before the first heading is dropped.
///
/// # Errors
/// A heading is missing or its section is empty.
pub fn parse_sections(text: &str) -> Result<[String; 4], DigestError> {
    // Heading lines only (`#`, `##`, `###`), each with where it starts.
    let headings: Vec<(usize, String)> = text
        .lines()
        .scan(0usize, |pos, line| {
            let here = *pos;
            *pos += line.len() + 1;
            Some((here, line))
        })
        .filter(|(_, line)| line.trim_start().starts_with('#'))
        .map(|(at, line)| (at, normalise(line)))
        .collect();
    let mut starts = Vec::with_capacity(4);
    for (h, key) in HEADINGS.iter().zip(HEADING_KEYS) {
        let at = headings
            .iter()
            .find(|(_, line)| line.contains(key))
            .map(|(at, _)| *at)
            .ok_or(DigestError::Missing(h))?;
        starts.push((at, *h));
    }
    let mut out: [String; 4] = Default::default();
    for (n, (at, h)) in starts.iter().enumerate() {
        let body_start = text[*at..].find('\n').map_or(text.len(), |nl| at + nl + 1);
        let end = starts
            .iter()
            .map(|(s, _)| *s)
            .filter(|s| s > at)
            .min()
            .unwrap_or(text.len());
        let body = text.get(body_start..end).unwrap_or_default().trim();
        if body.is_empty() {
            return Err(DigestError::Missing(h));
        }
        body.clone_into(&mut out[n]);
    }
    Ok(out)
}

/// What identifies each heading once normalised: models write "Ten‑minute" with
/// a non-breaking hyphen, "10-minute", bold, or add "(60 seconds)", so the match
/// is on these words, not on the exact line.
const HEADING_KEYS: [&str; 4] = ["what changed", "why it matters", "drill", "avatar script"];

/// A heading line lowercased, without `#`, `*` or `_`, with every Unicode dash
/// as `-` and whitespace collapsed.
fn normalise(line: &str) -> String {
    let lowered: String = line
        .chars()
        .map(|c| match c {
            '\u{2010}'..='\u{2015}' | '\u{2212}' => '-',
            '#' | '*' | '_' => ' ',
            c => c.to_ascii_lowercase(),
        })
        .collect::<String>()
        .to_lowercase();
    lowered.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The ISO week label of a moment, e.g. `"2026-W39"`.
#[must_use]
pub fn iso_week(at: DateTime<Utc>) -> String {
    let w = at.iso_week();
    format!("{}-W{:02}", w.year(), w.week())
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    #[test]
    fn sections_split_on_the_headings_in_any_case() {
        let text = "Preamble.\n## What changed\n- a\n- b\n## why it matters\nBecause.\n\
                    ## Ten-minute drill\nDo x.\n## Avatar script (60 seconds)\nHello.\n";
        let s = parse_sections(text).unwrap();
        assert_eq!(s[0], "- a\n- b");
        assert_eq!(s[1], "Because.");
        assert_eq!(s[2], "Do x.");
        assert_eq!(s[3], "Hello.");
    }

    #[test]
    fn headings_match_despite_dashes_bold_and_numbers() {
        let text = "## **What changed**\n- a\n### Why it matters:\nb\n\
                    ## Ten\u{2011}minute drill\nc\n## Avatar script (60 seconds)\nd\n";
        let s = parse_sections(text).unwrap();
        assert_eq!(
            s,
            [
                "- a".to_owned(),
                "b".to_owned(),
                "c".to_owned(),
                "d".to_owned()
            ]
        );
        let numbered =
            "## What changed\na\n## Why it matters\nb\n## 10-minute drill\nc\n## Avatar script\nd";
        assert_eq!(parse_sections(numbered).unwrap()[2], "c");
    }

    #[test]
    fn a_missing_or_empty_section_is_refused() {
        assert!(matches!(
            parse_sections("## What changed\nx\n## Why it matters\ny\n## Ten-minute drill\nz\n"),
            Err(DigestError::Missing("## Avatar script"))
        ));
        assert!(matches!(
            parse_sections(
                "## What changed\n\n## Why it matters\ny\n## Ten-minute drill\nz\n## Avatar script\nw"
            ),
            Err(DigestError::Missing("## What changed"))
        ));
    }

    #[test]
    fn iso_week_labels() {
        let at = Utc.with_ymd_and_hms(2026, 9, 24, 12, 0, 0).unwrap();
        assert_eq!(iso_week(at), "2026-W39");
        let new_year = Utc.with_ymd_and_hms(2027, 1, 1, 0, 0, 0).unwrap();
        assert_eq!(iso_week(new_year), "2026-W53");
    }
}
