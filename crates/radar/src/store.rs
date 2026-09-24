//! The `radar` schema (`migrations/0031_radar.sql`): items read from the
//! sources and the digests written from them.

use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};

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
}

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
        let rows = sqlx::query(
            "select id, source, language, guid, title, url, summary, published_at
             from radar.items
             where language = $1 and published_at >= $2 and published_at < $3
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
               (week, language, lang, changed, why, drill, script, item_count, model, stub)
             values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
             on conflict (week, language, lang) do update set
               created_at = now(), changed = excluded.changed, why = excluded.why,
               drill = excluded.drill, script = excluded.script,
               item_count = excluded.item_count, model = excluded.model, stub = excluded.stub
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
        .fetch_one(&self.pool)
        .await?;
        let mut out = d.clone();
        out.id = row.get("id");
        out.created_at = Some(row.get("created_at"));
        Ok(out)
    }

    /// Digests newest week first, optionally filtered.
    ///
    /// # Errors
    /// The database refused.
    pub async fn list_digests(
        &self,
        language: &str,
        lang: &str,
        limit: i64,
    ) -> Result<Vec<DigestRow>, StoreError> {
        let rows = sqlx::query(
            "select id, week, language, lang, created_at, changed, why, drill, script,
                    item_count, model, stub
             from radar.digests
             where ($1 = '' or language = $1) and ($2 = '' or lang = $2)
             order by week desc, language, lang limit $3",
        )
        .bind(language)
        .bind(lang)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.iter().map(digest_row).collect())
    }

    /// One digest.
    ///
    /// # Errors
    /// The database refused.
    pub async fn get_digest(&self, id: i64) -> Result<Option<DigestRow>, StoreError> {
        let row = sqlx::query(
            "select id, week, language, lang, created_at, changed, why, drill, script,
                    item_count, model, stub
             from radar.digests where id = $1",
        )
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
    }
}
