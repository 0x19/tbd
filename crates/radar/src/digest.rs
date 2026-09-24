//! Writing a digest: the week's items for one language go to the llm service
//! as `svc:radar`, with a prompt that asks for four fixed headings; the answer
//! is split on them. The service has no JSON mode, so the headings are the
//! contract, and an answer missing one is refused rather than stored half.

use std::{sync::LazyLock, time::Duration};

use base64::Engine as _;
use chrono::{DateTime, Datelike, Utc};
use regex::Regex;
use tbd_proto::llm::v1::{GenerateRequest, Message, Tier, llm_service_client::LlmServiceClient};
use tonic::{
    Status,
    service::{Interceptor, interceptor::InterceptedService},
    transport::{Channel, Endpoint},
};

use crate::{
    config::Llm,
    store::{Change, DigestRow, Impact, ItemRow},
};

/// The subject the radar calls the llm service as; its daily budget is this
/// subject's.
pub const SERVICE_SUBJECT: &str = "svc:radar";

/// The four section headings, in order: the answer's contract. Sections are
/// level one or two; the changes inside `## Changes` are level three.
pub const HEADINGS: [&str; 4] = [
    "## This week",
    "## Changes",
    "## Ten-minute drill",
    "## Avatar script",
];

/// The areas a change may name; anything else is filed as `ecosystem`.
pub const AREAS: [&str; 6] = [
    "runtime",
    "compiler",
    "stdlib",
    "tooling",
    "language",
    "ecosystem",
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
    /// No change in the answer had every field, a known impact, and a link
    /// among the week's items.
    #[error("the answer has no usable change")]
    NoChanges,
    /// The answer speaks as the project ("we released") or, in Croatian,
    /// uses a Serbian form; the prompt forbids both and this holds it.
    #[error("the answer breaks the voice rules: {0:?}")]
    Voice(String),
}

/// English: the Radar reports and never claims the project's work.
static WE_DID: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\bwe(?:'ve| have)?\s+(?:just\s+)?(?:released|shipped|accepted|fixed|merged|rolled out|added|landed|published|introduced)\b")
        .unwrap_or_else(|e| unreachable!("we-did pattern: {e}"))
});
/// Croatian: the same claim ("objavili smo"), and Serbian or Bosnian forms that
/// standard Croatian replaces (tjedan, tisuća, vijesti, izvještaj, također, uvjet,
/// sigurnost).
static HR_SLIP: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(?:smo\s+(?:objavili|izdali|dodali|prihvatili|uveli|ispravili)|(?:objavili|izdali|dodali|prihvatili|uveli|ispravili)\s+smo|sedmic\w*|hiljad\w*|vest[iae]|izve[sš]taj\w*|takođe|uslov\w*|bezbednost\w*)\b")
        .unwrap_or_else(|e| unreachable!("hr pattern: {e}"))
});

/// The first phrase in `text` that breaks the voice rules for reader language
/// `lang`, if any.
#[must_use]
pub fn voice_slip(lang: &str, text: &str) -> Option<String> {
    let re = if lang == "hr" { &*HR_SLIP } else { &*WE_DID };
    re.find(text).map(|m| m.as_str().to_owned())
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

    /// Write one digest from `items` on the configured tier.
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
        self.write_on(&self.config.tier, week, language, lang, items)
            .await
    }

    /// Write one digest from `items` on `tier` (`"fast"` or `"deep"`): the
    /// archive writes on its own tier, the weekly run on `[llm] tier`.
    ///
    /// # Errors
    /// The llm service failed, timed out, or answered without the headings.
    pub async fn write_on(
        &self,
        tier: &str,
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
            tier: if tier == "fast" {
                Tier::Fast
            } else {
                Tier::Deep
            }
            .into(),
            max_tokens: Some(self.config.max_tokens),
            temperature: Some(self.config.temperature),
            session_id: String::new(),
            reasoning: Some(self.config.reasoning),
            agent: String::new(),
            page: String::new(),
        };
        let deadline = Duration::from_secs(self.config.timeout_secs);
        let mut client = self.client.clone();
        let (text, model, stub, used) = tokio::time::timeout(deadline, async move {
            let mut stream = client.generate(request).await?.into_inner();
            let mut text = String::new();
            let mut model = String::new();
            let mut stub = false;
            let mut used = 0;
            while let Some(chunk) = stream.message().await? {
                if !chunk.model.is_empty() {
                    model = chunk.model;
                }
                stub |= chunk.stub;
                if !chunk.reasoning {
                    text.push_str(&chunk.text);
                }
                if chunk.done {
                    used = chunk.usage.map_or(0, |u| u.completion_tokens);
                    break;
                }
            }
            Ok::<_, Status>((text, model, stub, used))
        })
        .await
        .map_err(|_| DigestError::Timeout(self.config.timeout_secs))??;
        let sections = parse_sections(&text).inspect_err(|e| {
            // What the answer had instead, so a refusal can be told apart from
            // a cut: the headings found, its length and the tokens it used.
            let found: Vec<&str> = text
                .lines()
                .map(str::trim_start)
                .filter(|l| l.starts_with('#') && !l.starts_with("###"))
                .collect();
            tracing::warn!(week, language, lang, error = %e, ?found, chars = text.len(), completion_tokens = used, max_tokens = self.config.max_tokens, "digest refused");
        })?;
        let links: Vec<&str> = items.iter().map(|r| r.item.url.as_str()).collect();
        let changes = parse_changes(&sections[1], &links);
        if changes.is_empty() {
            return Err(DigestError::NoChanges);
        }
        let prose = changes.iter().fold(
            format!("{}\n{}\n{}", sections[0], sections[2], sections[3]),
            |acc, c| format!("{acc}\n{}\n{}\n{}", c.what, c.production_impact, c.try_it),
        );
        if let Some(slip) = voice_slip(lang, &prose) {
            tracing::warn!(week, language, lang, slip, "digest refused");
            return Err(DigestError::Voice(slip));
        }
        Ok(DigestRow {
            week: week.to_owned(),
            language: language.to_owned(),
            lang: lang.to_owned(),
            summary: sections[0].clone(),
            changes,
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
    let (reader, style) = if lang == "hr" {
        (
            "Croatian",
            " Write standard Croatian (hrvatski standardni jezik), not Serbian or Bosnian: \
             tjedan, never sedmica; tisuća, never hiljada; the ije/je forms. Every value after a \
             label, the summary, the drill and the script are Croatian; only the four headings, \
             the label names and the Impact and Area values stay English.",
        )
    } else {
        ("English", "")
    };
    format!(
        "You write the weekly Radar for working {name} engineers: what changed in {name} this week \
         and what it means for someone who runs {name} in production. Use only the items given; \
         never invent a release, a version number, a date, a benchmark or a feature, and cite each \
         change with the exact link of the item it comes from. If the week is thin, say so \
         instead of padding. The Radar reports on the {name} project; it is not the project, \
         so never write as if we released, shipped, accepted or fixed anything: name who did \
         (the {name} team, the release, the proposal); a crate or a tool listed in This Week in \
         {name} is its authors' work, not the {name} team's. Describe only what an item's own text \
         says: when it gives only a title, say only that, and add no features, numbers or reasons \
         of your own. A point release is never Breaking. Write in {reader}.{style} Answer in Markdown with exactly these four \
         headings, in this order, written exactly as shown in English even when the text is \
         {reader}:\n\n\
         ## This week\n\
         Two or three sentences on the week as a whole.\n\n\
         ## Changes\n\
         Three to eight changes, the most important first, each as its own block:\n\
         ### <short title>\n\
         Impact: Breaking | Worth knowing | Nice to know\n\
         Area: runtime | compiler | stdlib | tooling | language | ecosystem\n\
         Link: <the item's exact link>\n\
         What changed: <one or two sentences>\n\
         Production impact: <what it means for a running service or a build>\n\
         Try it: <one concrete thing to do or check>\n\
         Breaking means it can break a build or a running service; Worth knowing means it \
         changes how you work; Nice to know is tooling and ergonomics. Keep the labels in \
         English.\n\n\
         ## Ten-minute drill\n\
         One small exercise a reader can do in ten minutes that uses something from this week.\n\n\
         ## Avatar script\n\
         About 150 words of plain spoken text for a presenter to read aloud in sixty seconds: \
         no Markdown, no links, no lists, calm, addressed to the listener as you; the {name} \
         team did the work, the presenter only reports it."
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
        .filter(|(_, line)| {
            let t = line.trim_start();
            t.starts_with('#') && !t.starts_with("###")
        })
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
const HEADING_KEYS: [&str; 4] = ["this week", "changes", "drill", "avatar script"];

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

/// The change blocks of a `## Changes` section: each `###` heading is a
/// title, followed by `Label: value` lines (a value may continue on the lines
/// after it). A block is kept only with every field, an impact among the three
/// names, and a link that is one of `links` (the week's items), so the model
/// cannot cite what it was not given; the rest are dropped and logged. Kept
/// changes are ordered Breaking, Worth knowing, Nice to know, stable within.
#[must_use]
pub fn parse_changes(section: &str, links: &[&str]) -> Vec<Change> {
    let mut blocks: Vec<(String, Vec<&str>)> = Vec::new();
    for line in section.lines() {
        let t = line.trim_start();
        if t.starts_with("###") {
            let title = t.trim_start_matches('#').replace("**", "");
            blocks.push((title.trim().to_owned(), Vec::new()));
        } else if let Some((_, lines)) = blocks.last_mut() {
            lines.push(line);
        }
    }
    let mut out: Vec<Change> = blocks
        .into_iter()
        .filter_map(|(title, lines)| {
            let change = change_from(&title, &lines, links);
            if change.is_none() {
                tracing::info!(%title, "change dropped: a field missing, an unknown impact, or a link not among the items");
            }
            change
        })
        .collect();
    out.sort_by_key(|c| c.impact);
    out
}

fn change_from(title: &str, body: &[&str], links: &[&str]) -> Option<Change> {
    let mut fields: Vec<(String, String)> = Vec::new();
    for line in body {
        let plain = line
            .trim()
            .trim_start_matches(['-', '*', ' '])
            .replace("**", "");
        let labelled = plain.split_once(':').and_then(|(label, value)| {
            let label = normalise(label);
            [
                "impact",
                "area",
                "link",
                "what changed",
                "production impact",
                "try it",
            ]
            .contains(&label.as_str())
            .then(|| (label, value.trim().to_owned()))
        });
        match labelled {
            Some(field) => fields.push(field),
            None if !plain.trim().is_empty() => {
                if let Some((_, value)) = fields.last_mut() {
                    value.push(' ');
                    value.push_str(plain.trim());
                }
            }
            None => {}
        }
    }
    let get = |key: &str| {
        fields
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.trim().to_owned())
    };
    let impact = match normalise(&get("impact")?).as_str() {
        "breaking" => Impact::Breaking,
        "worth knowing" => Impact::WorthKnowing,
        "nice to know" => Impact::NiceToKnow,
        _ => return None,
    };
    let url = first_link(&get("link")?)?;
    let same = |a: &str, b: &str| a.trim_end_matches('/') == b.trim_end_matches('/');
    if !links.iter().any(|l| same(l, &url)) {
        return None;
    }
    let area = normalise(&get("area").unwrap_or_default());
    let area = if AREAS.contains(&area.as_str()) {
        area
    } else {
        "ecosystem".to_owned()
    };
    let (what, production_impact, try_it) = (
        get("what changed")?,
        get("production impact")?,
        get("try it")?,
    );
    if title.is_empty() || what.is_empty() || production_impact.is_empty() || try_it.is_empty() {
        return None;
    }
    Some(Change {
        title: title.to_owned(),
        impact,
        area,
        url,
        what,
        production_impact,
        try_it,
    })
}

/// The first http(s) URL in a value written as a bare link, `<link>` or
/// `[text](link)`.
fn first_link(value: &str) -> Option<String> {
    let start = value.find("http://").or_else(|| value.find("https://"))?;
    let url: String = value[start..]
        .chars()
        .take_while(|c| !c.is_whitespace() && !matches!(c, ')' | '>' | ']' | '"'))
        .collect();
    Some(url.trim_end_matches(['.', ',', ';']).to_owned())
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

    const FULL: &str = "Preamble.\n## This week\nA quiet week.\n## Changes\n\
        ### Scheduler tweak\nImpact: Worth knowing\nArea: runtime\nLink: https://go.dev/a\n\
        What changed: The scheduler\nnow yields sooner.\nProduction impact: Lower tail latency.\n\
        Try it: Run your load test.\n\
        ### **CancelRequest is gone**\n- **Impact:** Breaking\n- **Area:** stdlib\n\
        - **Link:** <https://github.com/golang/go/issues/1>\n- **What changed:** Removed.\n\
        - **Production impact:** Builds fail.\n- **Try it:** grep for it.\n\
        ## Ten\u{2011}minute drill\nDo x.\n## Avatar script (60 seconds)\nHello.\n";

    const LINKS: [&str; 2] = ["https://go.dev/a", "https://github.com/golang/go/issues/1"];

    #[test]
    fn voice_slips_are_caught_and_plain_reporting_is_not() {
        assert_eq!(voice_slip("en", "This week we released Go 1.24.").as_deref(), Some("we released"));
        assert_eq!(voice_slip("en", "We've shipped a fix.").as_deref(), Some("We've shipped"));
        assert_eq!(voice_slip("en", "The Go team released 1.24; we cover it below."), None);
        assert_eq!(voice_slip("hr", "Danas smo objavili tri izdanja.").as_deref(), Some("smo objavili"));
        assert_eq!(voice_slip("hr", "Najnovije vesti iz Rusta.").as_deref(), Some("vesti"));
        assert_eq!(voice_slip("hr", "Ove sedmice").as_deref(), Some("sedmice"));
        assert_eq!(voice_slip("hr", "Takođe je izašao izveštaj.").as_deref(), Some("Takođe"));
        assert_eq!(
            voice_slip("hr", "Ovaj tjedan: vijesti, izvještaj i također sigurnost. Tim je objavio Rust 1.94."),
            None
        );
    }

    #[test]
    fn the_prompt_reports_on_the_project_and_asks_for_standard_croatian() {
        let en = system_prompt("go", "en");
        assert!(!en.contains("first person plural"));
        assert!(en.contains("never write as if we released"));
        assert!(en.contains("Describe only what an item's own text says"));
        assert!(en.contains("A point release is never Breaking"));
        assert!(!en.contains("sedmica"));
        let hr = system_prompt("rust", "hr");
        assert!(hr.contains("standard Croatian"));
        assert!(hr.contains("tjedan, never sedmica"));
    }

    #[test]
    fn sections_split_on_level_two_headings_only() {
        let s = parse_sections(FULL).unwrap();
        assert_eq!(s[0], "A quiet week.");
        assert!(s[1].starts_with("### Scheduler tweak"));
        assert_eq!(s[2], "Do x.");
        assert_eq!(s[3], "Hello.");
    }

    #[test]
    fn changes_parse_with_bold_bullets_and_continuations_breaking_first() {
        let s = parse_sections(FULL).unwrap();
        let c = parse_changes(&s[1], &LINKS);
        assert_eq!(c.len(), 2);
        assert_eq!(c[0].title, "CancelRequest is gone");
        assert_eq!(c[0].impact, Impact::Breaking);
        assert_eq!(c[0].url, "https://github.com/golang/go/issues/1");
        assert_eq!(c[1].what, "The scheduler now yields sooner.");
        assert_eq!(c[1].area, "runtime");
    }

    #[test]
    fn a_change_citing_what_it_was_not_given_is_dropped() {
        let s = parse_sections(FULL).unwrap();
        let only_one = parse_changes(&s[1], &["https://go.dev/a"]);
        assert_eq!(only_one.len(), 1);
        assert_eq!(only_one[0].title, "Scheduler tweak");
    }

    #[test]
    fn an_unknown_impact_or_a_missing_field_drops_the_change() {
        let bad = "### A\nImpact: 8.7/10\nLink: https://go.dev/a\nWhat changed: x\n\
                   Production impact: y\nTry it: z\n### B\nImpact: Breaking\nLink: https://go.dev/a\n\
                   What changed: x\nTry it: z\n";
        assert!(parse_changes(bad, &LINKS).is_empty());
    }

    #[test]
    fn a_missing_or_empty_section_is_refused() {
        assert!(matches!(
            parse_sections("## This week\nx\n## Changes\ny\n## Ten-minute drill\nz\n"),
            Err(DigestError::Missing("## Avatar script"))
        ));
        assert!(matches!(
            parse_sections(
                "## This week\n\n## Changes\ny\n## Ten-minute drill\nz\n## Avatar script\nw"
            ),
            Err(DigestError::Missing("## This week"))
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
