//! The `radar` schema (`migrations/0031_radar.sql`): items read from the
//! sources and the digests written from them.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row, types::Json};

/// An item as a source gave it, before it is stored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewItem {
    /// The source's configured name.
    pub source: String,
    /// `"go"` or `"rust"`.
    pub language: String,
    /// The source's own id for it.
    pub guid: String,
    /// Title.
    pub title: String,
    /// Link.
    pub url: String,
    /// The source's summary, trimmed; may be empty.
    pub summary: String,
    /// When it was published (the read time when the source gives none).
    pub published_at: DateTime<Utc>,
}

/// A stored item.
#[derive(Debug, Clone)]
pub struct ItemRow {
    /// Row id.
    pub id: i64,
    /// Everything the source gave.
    pub item: NewItem,
}

/// A digest, stored or about to be.
#[derive(Debug, Clone, Default)]
pub struct DigestRow {
    /// Row id; 0 before it is stored.
    pub id: i64,
    /// ISO week, e.g. `"2026-W39"`.
    pub week: String,
    /// `"go"` or `"rust"`.
    pub language: String,
    /// Reader language, `"en"` or `"hr"`.
    pub lang: String,
    /// When it was written.
    pub created_at: Option<DateTime<Utc>>,
    /// Section: what changed.
    pub changed: String,
    /// Section: why it matters.
    pub why: String,
    /// Section: the ten-minute drill.
    pub drill: String,
    /// Section: the 60-second avatar script.
    pub script: String,
    /// Items it was written from.
    pub item_count: i32,
    /// The model that wrote it.
    pub model: String,
    /// Written by the llm service's stub engine.
    pub stub: bool,
    /// Two or three sentences on the week.
    pub summary: String,
    /// The changes, Breaking first.
    pub changes: Vec<Change>,
    /// `"draft"` or `"published"`.
    pub status: String,
    /// When it was published.
    pub published_at: Option<DateTime<Utc>>,
    /// `"live"` (the weekly run) or `"archive"` (the backfill).
    pub origin: String,
}

/// How much a change matters in production: a named category, never a score.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Impact {
    /// Can break a build or a running service.
    Breaking,
    /// Changes how you work or what you reach for.
    WorthKnowing,
    /// Tooling and ergonomics.
    NiceToKnow,
}

/// One change in a digest; its `url` is one of the week's items.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Change {
    /// Title.
    pub title: String,
    /// Impact category.
    pub impact: Impact,
    /// runtime, compiler, stdlib, tooling, language or ecosystem.
    pub area: String,
    /// The source item's link.
    pub url: String,
    /// What changed.
    pub what: String,
    /// What it means for a production service.
    pub production_impact: String,
    /// Something to try.
    pub try_it: String,
}

/// The columns [`digest_row`] reads, as a literal so `concat!` can build
/// static SQL (sqlx refuses a query string built at run time).
macro_rules! digest_columns {
    () => {
        "id, week, language, lang, created_at, changed, why, drill, script, \
         item_count, model, stub, summary, changes, status, published_at, origin"
    };
}

/// A digest's review status.
pub const DRAFT: &str = "draft";
/// A digest anyone may read.
pub const PUBLISHED: &str = "published";

/// Errors from the store.
#[derive(Debug, thiserror::Error)]
#[error("store: {0}")]
pub struct StoreError(#[from] sqlx::Error);

/// The radar's tables.
#[derive(Debug, Clone)]
pub struct Store {
    pool: PgPool,
}

impl Store {
    /// A store over `pool`.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Insert items not seen before (by `(source, guid)`); returns how many
    /// were new. An item seen again keeps its first row untouched.
    ///
    /// # Errors
    /// The database refused.
    pub async fn upsert_items(&self, items: &[NewItem]) -> Result<u64, StoreError> {
        let mut new = 0;
        for i in items {
            let done = sqlx::query(
                "insert into radar.items (source, language, guid, title, url, summary, published_at)
                 values ($1, $2, $3, $4, $5, $6, $7)
                 on conflict (source, guid) do nothing",
            )
            .bind(&i.source)
            .bind(&i.language)
            .bind(&i.guid)
            .bind(&i.title)
            .bind(&i.url)
            .bind(&i.summary)
            .bind(i.published_at)
            .execute(&self.pool)
            .await?;
            new += done.rows_affected();
        }
        Ok(new)
    }

    /// Items newest first, optionally one language's.
    ///
    /// # Errors
    /// The database refused.
    pub async fn list_items(&self, language: &str, limit: i64) -> Result<Vec<ItemRow>, StoreError> {
        let rows = sqlx::query(
            "select id, source, language, guid, title, url, summary, published_at
             from radar.items where ($1 = '' or language = $1)
             order by published_at desc, id desc limit $2",
        )
        .bind(language)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.iter().map(item_row).collect())
    }

    /// One language's items published in `[from, to)`, newest first.
    ///
    /// # Errors
    /// The database refused.
    pub async fn items_between(
        &self,
        language: &str,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
        limit: i64,
    ) -> Result<Vec<ItemRow>, StoreError> {
        // One row per URL: a post read from a live feed (named by its feed id)
        // and from an archive (named by its URL) is one item.
        let rows = sqlx::query(
            "select * from (
               select distinct on (url) id, source, language, guid, title, url, summary, published_at
               from radar.items
               where language = $1 and published_at >= $2 and published_at < $3
               order by url, length(summary) desc, id
             ) one
             order by published_at desc, id desc limit $4",
        )
        .bind(language)
        .bind(from)
        .bind(to)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.iter().map(item_row).collect())
    }

    /// The earliest week the weekly run has a digest for: the backfill stops
    /// before it, so the archive never overwrites a live issue.
    ///
    /// # Errors
    /// The database refused.
    pub async fn earliest_live_week(&self) -> Result<Option<String>, StoreError> {
        let row = sqlx::query("select min(week) from radar.digests where origin = 'live'")
            .fetch_one(&self.pool)
            .await?;
        Ok(row.get::<Option<String>, _>(0))
    }

    /// Store an archive digest, published at once and marked `archive`; an
    /// existing digest for the week is left alone (`None`), so a stopped
    /// backfill resumes without rewriting anything.
    ///
    /// # Errors
    /// The database refused.
    pub async fn put_archive_digest(&self, d: &DigestRow) -> Result<Option<DigestRow>, StoreError> {
        let row = sqlx::query(
            "insert into radar.digests
               (week, language, lang, changed, why, drill, script, item_count, model, stub,
                summary, changes, status, published_at, published_by, origin)
             values ($1, $2, $3, '', '', $4, $5, $6, $7, $8, $9, $10,
                     'published', now(), 'backfill', 'archive')
             on conflict (week, language, lang) do nothing
             returning id, created_at, published_at",
        )
        .bind(&d.week)
        .bind(&d.language)
        .bind(&d.lang)
        .bind(&d.drill)
        .bind(&d.script)
        .bind(d.item_count)
        .bind(&d.model)
        .bind(d.stub)
        .bind(&d.summary)
        .bind(Json(&d.changes))
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|row| {
            let mut out = d.clone();
            out.id = row.get("id");
            out.created_at = Some(row.get("created_at"));
            out.published_at = Some(row.get("published_at"));
            PUBLISHED.clone_into(&mut out.status);
            "archive".clone_into(&mut out.origin);
            out
        }))
    }

    /// Whether the week's digest for a language and reader language exists.
    ///
    /// # Errors
    /// The database refused.
    pub async fn digest_exists(
        &self,
        week: &str,
        language: &str,
        lang: &str,
    ) -> Result<bool, StoreError> {
        let row = sqlx::query(
            "select exists(select 1 from radar.digests where week = $1 and language = $2 and lang = $3)",
        )
        .bind(week)
        .bind(language)
        .bind(lang)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.get::<bool, _>(0))
    }

    /// Store a digest, replacing the week's existing one for the same
    /// language and reader language. Returns it with its id and time.
    ///
    /// # Errors
    /// The database refused.
    pub async fn put_digest(&self, d: &DigestRow) -> Result<DigestRow, StoreError> {
        let row = sqlx::query(
            "insert into radar.digests
               (week, language, lang, changed, why, drill, script, item_count, model, stub,
                summary, changes, status, published_at, published_by)
             values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, 'draft', null, null)
             on conflict (week, language, lang) do update set
               created_at = now(), changed = excluded.changed, why = excluded.why,
               drill = excluded.drill, script = excluded.script,
               item_count = excluded.item_count, model = excluded.model, stub = excluded.stub,
               summary = excluded.summary, changes = excluded.changes,
               status = 'draft', published_at = null, published_by = null
             returning id, created_at",
        )
        .bind(&d.week)
        .bind(&d.language)
        .bind(&d.lang)
        .bind(&d.changed)
        .bind(&d.why)
        .bind(&d.drill)
        .bind(&d.script)
        .bind(d.item_count)
        .bind(&d.model)
        .bind(d.stub)
        .bind(&d.summary)
        .bind(Json(&d.changes))
        .fetch_one(&self.pool)
        .await?;
        let mut out = d.clone();
        out.id = row.get("id");
        out.created_at = Some(row.get("created_at"));
        DRAFT.clone_into(&mut out.status);
        out.published_at = None;
        Ok(out)
    }

    /// Publish a digest (`publish`) or take it back to draft; returns it, or
    /// `None` when there is no such digest.
    ///
    /// # Errors
    /// The database refused.
    pub async fn set_published(
        &self,
        id: i64,
        publish: bool,
        by: &str,
    ) -> Result<Option<DigestRow>, StoreError> {
        let row = sqlx::query(concat!(
            "update radar.digests set
               status = case when $2 then 'published' else 'draft' end,
               published_at = case when $2 then now() else null end,
               published_by = case when $2 then $3 else null end
             where id = $1
             returning ",
            digest_columns!()
        ))
        .bind(id)
        .bind(publish)
        .bind(by)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.as_ref().map(digest_row))
    }

    /// Digests newest week first, optionally filtered; drafts only when
    /// `drafts` is true.
    ///
    /// # Errors
    /// The database refused.
    pub async fn list_digests(
        &self,
        language: &str,
        lang: &str,
        limit: i64,
        drafts: bool,
    ) -> Result<Vec<DigestRow>, StoreError> {
        let rows = sqlx::query(concat!(
            "select ",
            digest_columns!(),
            " from radar.digests
             where ($1 = '' or language = $1) and ($2 = '' or lang = $2)
               and ($4 or status = 'published')
             order by week desc, language, lang limit $3"
        ))
        .bind(language)
        .bind(lang)
        .bind(limit)
        .bind(drafts)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.iter().map(digest_row).collect())
    }

    /// One digest, draft or published; the caller decides who may see a draft.
    ///
    /// # Errors
    /// The database refused.
    pub async fn get_digest(&self, id: i64) -> Result<Option<DigestRow>, StoreError> {
        let row = sqlx::query(concat!(
            "select ",
            digest_columns!(),
            " from radar.digests where id = $1"
        ))
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.as_ref().map(digest_row))
    }
}

fn item_row(r: &sqlx::postgres::PgRow) -> ItemRow {
    ItemRow {
        id: r.get("id"),
        item: NewItem {
            source: r.get("source"),
            language: r.get("language"),
            guid: r.get("guid"),
            title: r.get("title"),
            url: r.get("url"),
            summary: r.get("summary"),
            published_at: r.get("published_at"),
        },
    }
}

fn digest_row(r: &sqlx::postgres::PgRow) -> DigestRow {
    DigestRow {
        id: r.get("id"),
        week: r.get("week"),
        language: r.get("language"),
        lang: r.get("lang"),
        created_at: Some(r.get("created_at")),
        changed: r.get("changed"),
        why: r.get("why"),
        drill: r.get("drill"),
        script: r.get("script"),
        item_count: r.get("item_count"),
        model: r.get("model"),
        stub: r.get("stub"),
        summary: r.get("summary"),
        changes: r.get::<Json<Vec<Change>>, _>("changes").0,
        status: r.get("status"),
        published_at: r.get("published_at"),
        origin: r.get("origin"),
    }
}
