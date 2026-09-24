//! Reading the sources: Atom and RSS feeds (`feed-rs`), and GitHub's issue
//! search. Every source is read on its own; one failing never stops the rest.

use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::{
    config::{Fetch, SourceKind, SourceSpec},
    store::NewItem,
};

/// Why a source could not be read.
#[derive(Debug, thiserror::Error)]
pub enum FetchError {
    /// The request failed or the server said no.
    #[error("http: {0}")]
    Http(#[from] reqwest::Error),
    /// The body was not a feed.
    #[error("feed: {0}")]
    Feed(String),
    /// The body was not the JSON GitHub sends.
    #[error("github: {0}")]
    Github(String),
}

/// Reads sources over one HTTP client.
#[derive(Debug, Clone)]
pub struct Fetcher {
    client: reqwest::Client,
    config: Fetch,
}

impl Fetcher {
    /// A fetcher with the `[fetch]` settings.
    ///
    /// # Errors
    /// The HTTP client could not be built.
    pub fn new(config: Fetch) -> Result<Self, FetchError> {
        let client = reqwest::Client::builder()
            .user_agent(concat!("tbd-radar/", env!("CARGO_PKG_VERSION")))
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()?;
        Ok(Self { client, config })
    }

    /// Read one source, newest first, at most `max_items_per_source` items.
    ///
    /// # Errors
    /// The source could not be fetched or parsed.
    pub async fn read(
        &self,
        source: &SourceSpec,
        now: DateTime<Utc>,
    ) -> Result<Vec<NewItem>, FetchError> {
        let mut items = match source.kind {
            SourceKind::Atom | SourceKind::Rss => self.read_feed(source, now).await?,
            SourceKind::Github => self.read_github(source, now).await?,
        };
        items.sort_by_key(|i| std::cmp::Reverse(i.published_at));
        items.truncate(self.config.max_items_per_source);
        Ok(items)
    }

    async fn read_feed(
        &self,
        source: &SourceSpec,
        now: DateTime<Utc>,
    ) -> Result<Vec<NewItem>, FetchError> {
        let body = self
            .client
            .get(&source.url)
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?;
        parse_feed(source, &body, now, self.config.max_summary_chars)
    }

    async fn read_github(
        &self,
        source: &SourceSpec,
        now: DateTime<Utc>,
    ) -> Result<Vec<NewItem>, FetchError> {
        let since = (now - chrono::Duration::days(i64::from(source.lookback_days)))
            .format("%Y-%m-%d")
            .to_string();
        let query = source.query.replace("{since}", &since);
        let per_page = self.config.max_items_per_source.to_string();
        let mut req = self
            .client
            .get(&source.url)
            .query(&[
                ("q", query.as_str()),
                ("sort", "updated"),
                ("order", "desc"),
                ("per_page", per_page.as_str()),
            ])
            .header("accept", "application/vnd.github+json")
            .header("x-github-api-version", "2022-11-28");
        if !self.config.github_token.is_empty() {
            req = req.bearer_auth(&self.config.github_token);
        }
        let body = req.send().await?.error_for_status()?.bytes().await?;
        parse_github(source, &body, now, self.config.max_summary_chars)
    }
}

/// Items from an Atom or RSS document.
///
/// # Errors
/// The document is not a feed.
pub fn parse_feed(
    source: &SourceSpec,
    body: &[u8],
    now: DateTime<Utc>,
    max_summary: usize,
) -> Result<Vec<NewItem>, FetchError> {
    let feed = feed_rs::parser::parse(body).map_err(|e| FetchError::Feed(e.to_string()))?;
    Ok(feed
        .entries
        .into_iter()
        .filter_map(|e| {
            let url = e.links.first().map(|l| l.href.clone())?;
            let title = e.title.map(|t| t.content).unwrap_or_default();
            if title.trim().is_empty() {
                return None;
            }
            let summary = e
                .summary
                .map(|s| s.content)
                .or_else(|| e.content.and_then(|c| c.body))
                .unwrap_or_default();
            Some(NewItem {
                source: source.name.clone(),
                language: source.language.clone(),
                guid: if e.id.is_empty() { url.clone() } else { e.id },
                title: clean(&title, 300),
                url,
                summary: clean(&summary, max_summary),
                published_at: e.published.or(e.updated).unwrap_or(now),
            })
        })
        .collect())
}

#[derive(Deserialize)]
struct Search {
    items: Vec<Issue>,
}

#[derive(Deserialize)]
struct Issue {
    html_url: String,
    title: String,
    #[serde(default)]
    body: Option<String>,
    #[serde(default)]
    closed_at: Option<DateTime<Utc>>,
    updated_at: DateTime<Utc>,
    #[serde(default)]
    pull_request: Option<PullRequest>,
}

#[derive(Deserialize)]
struct PullRequest {
    #[serde(default)]
    merged_at: Option<DateTime<Utc>>,
}

/// Items from a GitHub search response. A merged pull request is dated by
/// its merge, a closed issue by its close, anything else by its last update.
///
/// # Errors
/// The body is not a search response.
pub fn parse_github(
    source: &SourceSpec,
    body: &[u8],
    _now: DateTime<Utc>,
    max_summary: usize,
) -> Result<Vec<NewItem>, FetchError> {
    let search: Search =
        serde_json::from_slice(body).map_err(|e| FetchError::Github(e.to_string()))?;
    Ok(search
        .items
        .into_iter()
        .map(|i| {
            let when = i
                .pull_request
                .and_then(|p| p.merged_at)
                .or(i.closed_at)
                .unwrap_or(i.updated_at);
            NewItem {
                source: source.name.clone(),
                language: source.language.clone(),
                guid: i.html_url.clone(),
                title: clean(&i.title, 300),
                url: i.html_url,
                summary: clean(i.body.as_deref().unwrap_or_default(), max_summary),
                published_at: when,
            }
        })
        .collect())
}

/// Plain text from a summary that may be HTML or Markdown: tags dropped,
/// the common entities decoded, whitespace collapsed, cut at `max` characters
/// on a word boundary with an ellipsis.
#[must_use]
pub fn clean(raw: &str, max: usize) -> String {
    let mut text = String::with_capacity(raw.len());
    let mut in_tag = false;
    for c in raw.chars() {
        match c {
            '<' => in_tag = true,
            '>' if in_tag => {
                in_tag = false;
                text.push(' ');
            }
            _ if !in_tag => text.push(c),
            _ => {}
        }
    }
    let text = text
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'");
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if text.chars().count() <= max {
        return text;
    }
    let cut: String = text.chars().take(max).collect();
    let cut = cut.rsplit_once(' ').map_or(cut.as_str(), |(head, _)| head);
    format!("{cut}…")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source(kind: SourceKind) -> SourceSpec {
        SourceSpec {
            name: "s".to_owned(),
            kind,
            language: "go".to_owned(),
            url: "http://x".to_owned(),
            query: String::new(),
            lookback_days: 14,
        }
    }

    #[test]
    fn clean_strips_tags_entities_and_cuts_on_a_word() {
        assert_eq!(
            clean("<p>Go&nbsp;1.27 is <b>out</b> &amp; fast</p>", 100),
            "Go 1.27 is out & fast"
        );
        assert_eq!(clean("one two three four", 9), "one two…");
    }

    #[test]
    fn atom_entries_become_items() {
        let atom = br#"<?xml version="1.0" encoding="utf-8"?>
<feed xmlns="http://www.w3.org/2005/Atom"><title>Go</title><id>go</id><updated>2026-09-20T00:00:00Z</updated>
<entry><title>Go 1.27 is released</title><id>tag:go.dev,2026:1.27</id>
<link href="https://go.dev/blog/go1.27"/><updated>2026-09-20T10:00:00Z</updated>
<summary>&lt;p&gt;Iterators everywhere.&lt;/p&gt;</summary></entry>
<entry><title></title><id>x</id><link href="https://go.dev/blog/x"/><updated>2026-09-19T10:00:00Z</updated></entry>
</feed>"#;
        let items = parse_feed(&source(SourceKind::Atom), atom, Utc::now(), 100).unwrap();
        assert_eq!(items.len(), 1, "an entry without a title is skipped");
        assert_eq!(items[0].title, "Go 1.27 is released");
        assert_eq!(items[0].guid, "tag:go.dev,2026:1.27");
        assert_eq!(items[0].summary, "Iterators everywhere.");
        assert_eq!(
            items[0].published_at.to_rfc3339(),
            "2026-09-20T10:00:00+00:00"
        );
    }

    #[test]
    fn github_merged_pull_request_is_dated_by_its_merge() {
        let json = br#"{"total_count":1,"items":[{"html_url":"https://github.com/rust-lang/rfcs/pull/1",
            "title":"RFC: x","body":"**Summary** things","closed_at":"2026-09-10T00:00:00Z",
            "updated_at":"2026-09-12T00:00:00Z","pull_request":{"merged_at":"2026-09-09T12:00:00Z"}}]}"#;
        let items = parse_github(&source(SourceKind::Github), json, Utc::now(), 100).unwrap();
        assert_eq!(
            items[0].published_at.to_rfc3339(),
            "2026-09-09T12:00:00+00:00"
        );
        assert_eq!(items[0].guid, "https://github.com/rust-lang/rfcs/pull/1");
    }
}
