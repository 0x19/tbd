//! The sources' archives, for the backfill: the live feeds carry ten recent
//! entries at most, so past weeks come from each source's own archive page
//! (parsed with anchored patterns, tested against fixtures of the real pages)
//! and from GitHub's search over a date range. Every reader returns dated
//! items inside `[from, to)`; nothing here writes.

use std::{sync::LazyLock, time::Duration};

use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use regex::Regex;

use crate::{
    config::{Archive as ArchiveConfig, Fetch},
    fetch::{FetchError, clean},
    store::NewItem,
};

/// Reads the archives over one HTTP client.
#[derive(Debug, Clone)]
pub struct Archive {
    client: reqwest::Client,
    config: ArchiveConfig,
    github_token: String,
    max_summary: usize,
}

static GO_BLOG: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?s)<a href="(/blog/[^"]+)">([^<]+)</a>,\s*<span class="date">(\d{1,2} [A-Za-z]+ \d{4})</span>.*?<p class="blogsummary">\s*(.*?)\s*</p>"#,
    )
    .unwrap_or_else(|e| unreachable!("go blog pattern: {e}"))
});
static GO_MAJOR: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"<h2 id="(go[\d.]+)">go[\d.]+ \(released (\d{4}-\d{2}-\d{2})\)</h2>"#)
        .unwrap_or_else(|e| unreachable!("go major pattern: {e}"))
});
static GO_MINOR: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?s)<p id="(go[\d.]+)">\s*go[\d.]+\s*\(released (\d{4}-\d{2}-\d{2})\)(.*?)</p>"#)
        .unwrap_or_else(|e| unreachable!("go minor pattern: {e}"))
});
static RUST_POST: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"<a href="(https://blog\.rust-lang\.org/(?:inside-rust/)?(\d{4})/(\d{2})/(\d{2})/[^"]+)">([^<]+)</a>"#,
    )
    .unwrap_or_else(|e| unreachable!("rust post pattern: {e}"))
});
static TWIR_ISSUE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?s)datetime="(\d{4}-\d{2}-\d{2})T[^"]*"[^<]*</time>.{0,400}?<a href="(https://this-week-in-rust\.org/blog/[^"]+)">(This Week in Rust \d+)</a>"#,
    )
    .unwrap_or_else(|e| unreachable!("twir pattern: {e}"))
});
static LINK: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"<a href="(https?://[^"]+)">([^<]+)</a>"#)
        .unwrap_or_else(|e| unreachable!("link pattern: {e}"))
});

fn at_midnight(d: NaiveDate) -> DateTime<Utc> {
    Utc.from_utc_datetime(&d.and_hms_opt(12, 0, 0).unwrap_or_default())
}

fn inside(when: DateTime<Utc>, from: DateTime<Utc>, to: DateTime<Utc>) -> bool {
    when >= from && when < to
}

fn item(
    source: &str,
    language: &str,
    url: String,
    title: &str,
    summary: String,
    when: DateTime<Utc>,
) -> NewItem {
    NewItem {
        source: source.to_owned(),
        language: language.to_owned(),
        guid: url.clone(),
        title: clean(title, 300),
        url,
        summary,
        published_at: when,
    }
}

/// Posts from `go.dev/blog/all`, with the index's own summaries.
#[must_use]
pub fn parse_go_blog(
    html: &str,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
    max_summary: usize,
) -> Vec<NewItem> {
    GO_BLOG
        .captures_iter(html)
        .filter_map(|c| {
            let when = at_midnight(NaiveDate::parse_from_str(&c[3], "%d %B %Y").ok()?);
            inside(when, from, to).then(|| {
                item(
                    "go-blog",
                    "go",
                    format!("https://go.dev{}", &c[1]),
                    &c[2],
                    clean(&c[4], max_summary),
                    when,
                )
            })
        })
        .collect()
}

/// Releases from `go.dev/doc/devel/release`: major releases and minor
/// revisions, each with the page's own note as its summary.
#[must_use]
pub fn parse_go_releases(
    html: &str,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
    max_summary: usize,
) -> Vec<NewItem> {
    let base = "https://go.dev/doc/devel/release";
    let majors = GO_MAJOR.captures_iter(html).filter_map(|c| {
        let when = at_midnight(NaiveDate::parse_from_str(&c[2], "%Y-%m-%d").ok()?);
        inside(when, from, to).then(|| {
            let v = c[1].to_owned();
            item(
                "go-releases",
                "go",
                format!("{base}#{v}"),
                &format!("{v} released"),
                format!("{v} is a major release of Go."),
                when,
            )
        })
    });
    let minors = GO_MINOR.captures_iter(html).filter_map(|c| {
        let when = at_midnight(NaiveDate::parse_from_str(&c[2], "%Y-%m-%d").ok()?);
        inside(when, from, to).then(|| {
            let v = c[1].to_owned();
            item(
                "go-releases",
                "go",
                format!("{base}#{v}"),
                &format!("{v} released"),
                clean(&format!("{v} {}", &c[3]), max_summary),
                when,
            )
        })
    });
    majors.chain(minors).collect()
}

/// Posts from the Rust blog's index, or Inside Rust's, dated by their path.
#[must_use]
pub fn parse_rust_index(
    html: &str,
    source: &str,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Vec<NewItem> {
    let inside_rust = source == "inside-rust";
    RUST_POST
        .captures_iter(html)
        .filter(|c| c[1].contains("/inside-rust/") == inside_rust)
        .filter_map(|c| {
            let d = NaiveDate::from_ymd_opt(
                c[2].parse().ok()?,
                c[3].parse().ok()?,
                c[4].parse().ok()?,
            )?;
            let when = at_midnight(d);
            inside(when, from, to)
                .then(|| item(source, "rust", c[1].to_owned(), &c[5], String::new(), when))
        })
        .collect()
}

/// This Week in Rust issues from its archive page: (date, url, title).
#[must_use]
pub fn parse_twir_archive(
    html: &str,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Vec<(DateTime<Utc>, String, String)> {
    TWIR_ISSUE
        .captures_iter(html)
        .filter_map(|c| {
            let when = at_midnight(NaiveDate::parse_from_str(&c[1], "%Y-%m-%d").ok()?);
            inside(when, from, to).then(|| (when, c[2].to_owned(), c[3].to_owned()))
        })
        .collect()
}

/// The text of the section headed by the element with `id`, up to the next
/// heading of the same kind.
fn section<'a>(html: &'a str, id: &str) -> Option<&'a str> {
    let at = html.find(&format!("id=\"{id}\""))?;
    let rest = &html[at..];
    let body_start = rest.find("</h")?;
    let rest = &rest[body_start..];
    let end = rest[4..].find("<h").map_or(rest.len(), |e| e + 4);
    Some(&rest[..end])
}

/// One This Week in Rust issue: its summary (the official links, the crate of
/// the week, the tooling updates) and the official links as items of their own,
/// dated at the issue, so a digest can cite them.
#[must_use]
pub fn parse_twir_issue(
    html: &str,
    url: &str,
    title: &str,
    when: DateTime<Utc>,
    max_summary: usize,
) -> Vec<NewItem> {
    let links = |id: &str| -> Vec<(String, String)> {
        section(html, id)
            .map(|s| {
                LINK.captures_iter(s)
                    .map(|c| (c[1].to_owned(), clean(&c[2], 200)))
                    .collect()
            })
            .unwrap_or_default()
    };
    let official = links("official");
    let tooling = links("projecttooling-updates");
    let crate_of_week = section(html, "crate-of-the-week")
        .map(|s| {
            clean(
                s.trim_start_matches(|c| c != '>').trim_start_matches('>'),
                200,
            )
        })
        .unwrap_or_default();
    let mut summary = Vec::new();
    if !official.is_empty() {
        summary.push(format!(
            "Official: {}.",
            official
                .iter()
                .map(|(_, t)| t.as_str())
                .collect::<Vec<_>>()
                .join("; ")
        ));
    }
    if !crate_of_week.is_empty() {
        summary.push(format!("Crate of the week: {crate_of_week}"));
    }
    if !tooling.is_empty() {
        summary.push(format!(
            "Tooling: {}.",
            tooling
                .iter()
                .map(|(_, t)| t.as_str())
                .collect::<Vec<_>>()
                .join("; ")
        ));
    }
    let mut out = vec![item(
        "this-week-in-rust",
        "rust",
        url.to_owned(),
        title,
        clean(&summary.join(" "), max_summary),
        when,
    )];
    out.extend(
        official
            .into_iter()
            .map(|(u, t)| item("this-week-in-rust", "rust", u, &t, String::new(), when)),
    );
    out
}

impl Archive {
    /// An archive reader with the `[archive]` and `[fetch]` settings.
    ///
    /// # Errors
    /// The HTTP client could not be built.
    pub fn new(config: ArchiveConfig, fetch: &Fetch) -> Result<Self, FetchError> {
        let client = reqwest::Client::builder()
            .user_agent(concat!(
                "tbd-radar/",
                env!("CARGO_PKG_VERSION"),
                " (archive)"
            ))
            .timeout(Duration::from_secs(fetch.timeout_secs))
            .build()?;
        Ok(Self {
            client,
            config,
            github_token: fetch.github_token.clone(),
            max_summary: fetch.max_summary_chars,
        })
    }

    async fn page(&self, url: &str) -> Result<String, FetchError> {
        Ok(self
            .client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?)
    }

    async fn pause(&self) {
        tokio::time::sleep(Duration::from_millis(self.config.pause_ms)).await;
    }

    /// Every archive's items inside `[from, to)`. A failing source is logged
    /// and the rest are still read.
    pub async fn read(&self, from: DateTime<Utc>, to: DateTime<Utc>) -> Vec<NewItem> {
        let mut out = Vec::new();
        let max = self.max_summary;
        let mut step = |name: &str, r: Result<Vec<NewItem>, FetchError>| match r {
            Ok(items) => {
                tracing::info!(source = name, items = items.len(), "archive read");
                out.extend(items);
            }
            Err(e) => tracing::warn!(source = name, error = %e, "archive source failed"),
        };
        step(
            "go-blog",
            self.page(&self.config.go_blog)
                .await
                .map(|h| parse_go_blog(&h, from, to, max)),
        );
        step(
            "go-releases",
            self.page(&self.config.go_releases)
                .await
                .map(|h| parse_go_releases(&h, from, to, max)),
        );
        step(
            "rust-blog",
            self.page(&self.config.rust_blog)
                .await
                .map(|h| parse_rust_index(&h, "rust-blog", from, to)),
        );
        step(
            "inside-rust",
            self.page(&self.config.inside_rust)
                .await
                .map(|h| parse_rust_index(&h, "inside-rust", from, to)),
        );
        step(
            "go-proposals-accepted",
            self.github(
                "go-proposals-accepted",
                "go",
                &self.config.go_proposals_query,
                from,
                to,
            )
            .await,
        );
        step(
            "rust-rfcs-merged",
            self.github(
                "rust-rfcs-merged",
                "rust",
                &self.config.rust_rfcs_query,
                from,
                to,
            )
            .await,
        );
        step("this-week-in-rust", self.twir(from, to).await);
        out
    }

    async fn twir(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<NewItem>, FetchError> {
        let issues = parse_twir_archive(&self.page(&self.config.twir).await?, from, to);
        let mut out = Vec::new();
        for (when, url, title) in issues {
            self.pause().await;
            match self.page(&url).await {
                Ok(html) => out.extend(parse_twir_issue(
                    &html,
                    &url,
                    &title,
                    when,
                    self.max_summary,
                )),
                Err(e) => {
                    tracing::warn!(%url, error = %e, "twir issue failed; kept without a summary");
                }
            }
        }
        Ok(out)
    }

    /// GitHub's issue search over the whole range, paged (100 a page), paced
    /// for the search rate limit.
    async fn github(
        &self,
        name: &str,
        language: &str,
        query: &str,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<NewItem>, FetchError> {
        let q = query
            .replace("{from}", &from.format("%Y-%m-%d").to_string())
            .replace(
                "{to}",
                &(to - chrono::Duration::days(1))
                    .format("%Y-%m-%d")
                    .to_string(),
            );
        let source = crate::config::SourceSpec {
            name: name.to_owned(),
            kind: crate::config::SourceKind::Github,
            language: language.to_owned(),
            url: self.config.github.clone(),
            query: q.clone(),
            lookback_days: 0,
        };
        let mut out = Vec::new();
        for page in 1..=self.config.max_github_pages {
            let mut req = self
                .client
                .get(&self.config.github)
                .query(&[
                    ("q", q.as_str()),
                    ("per_page", "100"),
                    ("page", &page.to_string()),
                ])
                .header("accept", "application/vnd.github+json")
                .header("x-github-api-version", "2022-11-28");
            if !self.github_token.is_empty() {
                req = req.bearer_auth(&self.github_token);
            }
            let body = req.send().await?.error_for_status()?.bytes().await?;
            let items = crate::fetch::parse_github(&source, &body, Utc::now(), self.max_summary)?;
            let n = items.len();
            out.extend(
                items
                    .into_iter()
                    .filter(|i| inside(i.published_at, from, to)),
            );
            if n < 100 {
                break;
            }
            // Unauthenticated search allows ten requests a minute.
            tokio::time::sleep(Duration::from_secs(if self.github_token.is_empty() {
                7
            } else {
                2
            }))
            .await;
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn range() -> (DateTime<Utc>, DateTime<Utc>) {
        (
            Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2027, 1, 1, 0, 0, 0).unwrap(),
        )
    }

    const GO_BLOG_HTML: &str = include_str!("../tests/fixtures/archive/go_blog.html");
    const GO_RELEASES_HTML: &str = include_str!("../tests/fixtures/archive/go_releases.html");
    const RUST_BLOG_HTML: &str = include_str!("../tests/fixtures/archive/rust_blog.html");
    const INSIDE_RUST_HTML: &str = include_str!("../tests/fixtures/archive/inside_rust.html");
    const TWIR_ARCHIVE_HTML: &str = include_str!("../tests/fixtures/archive/twir_archive.html");
    const TWIR_ISSUE_HTML: &str = include_str!("../tests/fixtures/archive/twir_issue.html");

    #[test]
    fn go_blog_posts_in_range_with_their_summaries() {
        let (from, to) = range();
        let items = parse_go_blog(GO_BLOG_HTML, from, to, 300);
        assert_eq!(
            items.len(),
            4,
            "three 2026 posts and one 2025 post; the 2024 one is outside"
        );
        assert!(
            items
                .iter()
                .all(|i| i.url.starts_with("https://go.dev/blog/") && !i.summary.is_empty())
        );
        assert!(items.iter().all(|i| i.published_at >= from));
    }

    #[test]
    fn go_releases_major_and_minor_with_notes() {
        let (from, to) = range();
        let items = parse_go_releases(GO_RELEASES_HTML, from, to, 400);
        assert!(items.iter().any(|i| i.title == "go1.27.0 released"));
        let minor = items
            .iter()
            .find(|i| i.title == "go1.27.1 released")
            .unwrap();
        assert!(minor.summary.contains("fixes to cgo"), "{}", minor.summary);
        assert_eq!(minor.url, "https://go.dev/doc/devel/release#go1.27.1");
        assert!(
            items.iter().any(|i| i.title == "go1.24.0 released"
                && i.published_at.format("%Y").to_string() == "2025"),
            "go1.24.0, released in February 2025, is inside the range"
        );
    }

    #[test]
    fn rust_indexes_split_by_blog_and_date() {
        let (from, to) = range();
        let blog = parse_rust_index(RUST_BLOG_HTML, "rust-blog", from, to);
        assert_eq!(
            blog.len(),
            6,
            "four 2026 and two 2025 posts; 2024 is outside"
        );
        assert!(blog.iter().all(|i| !i.url.contains("/inside-rust/")));
        let inside_rust = parse_rust_index(INSIDE_RUST_HTML, "inside-rust", from, to);
        assert_eq!(inside_rust.len(), 4);
        assert!(inside_rust.iter().all(|i| i.url.contains("/inside-rust/")));
    }

    #[test]
    fn twir_archive_and_an_issue_with_its_official_links() {
        let (from, to) = range();
        let issues = parse_twir_archive(TWIR_ARCHIVE_HTML, from, to);
        assert_eq!(
            issues.len(),
            4,
            "three 2026 issues and one 2025; 2024 is outside"
        );
        let (when, url, title) = &issues[0];
        let items = parse_twir_issue(TWIR_ISSUE_HTML, url, title, *when, 600);
        assert_eq!(items[0].title, "This Week in Rust 670");
        assert!(
            items[0].summary.starts_with("Official: Be alert"),
            "{}",
            items[0].summary
        );
        assert!(items[0].summary.contains("Crate of the week"));
        assert!(
            items
                .iter()
                .any(|i| i.url.contains("github-actions-leaking-secrets")),
            "official links are items"
        );
    }
}
