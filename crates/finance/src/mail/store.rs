//! Templates, sends, the record, and the replies: mail against Postgres.

use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use tbd_db::{Access, DbError, PartyId, map_err};
use uuid::Uuid;

use crate::connectors::{
    Attachment, Connector, Outgoing,
    crypto::Sealer,
    store::{ConnectorRow, StoreError},
};

#[allow(missing_docs)]
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TemplateRow {
    pub id: Uuid,
    pub party_id: Uuid,
    pub name: String,
    pub subject: String,
    pub body: String,
    pub to_addrs: Vec<String>,
    pub cc_addrs: Vec<String>,
    pub bcc_addrs: Vec<String>,
    pub updated_at: DateTime<Utc>,
}

#[allow(missing_docs)]
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MailRow {
    pub id: Uuid,
    pub party_id: Uuid,
    pub connector_id: Option<Uuid>,
    pub direction: String,
    pub thread_key: Option<String>,
    pub provider_id: Option<String>,
    pub message_id: Option<String>,
    pub in_reply_to: Option<String>,
    pub parent_id: Option<Uuid>,
    pub template_id: Option<Uuid>,
    pub from_addr: String,
    pub to_addrs: Vec<String>,
    pub cc_addrs: Vec<String>,
    pub bcc_addrs: Vec<String>,
    pub subject: String,
    pub body: String,
    pub status: String,
    pub error: Option<String>,
    pub sent_at: Option<DateTime<Utc>>,
    pub received_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// A document that went with a mail or came with a reply.
#[allow(missing_docs)]
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MailDocRow {
    pub mail_id: Uuid,
    pub document_id: Uuid,
    pub filename: Option<String>,
    pub content_type: String,
    pub size_bytes: i64,
}

const TEMPLATE_COLUMNS: &str =
    "id, party_id, name, subject, body, to_addrs, cc_addrs, bcc_addrs, updated_at";
const MAIL_COLUMNS: &str =
    "id, party_id, connector_id, direction, thread_key, provider_id, message_id, in_reply_to,
    parent_id, template_id, from_addr, to_addrs, cc_addrs, bcc_addrs, subject, body, status, error,
    sent_at, received_at, created_at";

fn sql(s: &str) -> sqlx::AssertSqlSafe<String> {
    sqlx::AssertSqlSafe(s.to_owned())
}

/// A template to create or change.
#[derive(Debug, Clone, Default)]
pub struct TemplateInput {
    /// Empty creates.
    pub id: Option<Uuid>,
    /// A party the caller may read.
    pub party_id: Uuid,
    /// Unique per party.
    pub name: String,
    /// With helpers.
    pub subject: String,
    /// With helpers.
    pub body: String,
    /// Default recipients.
    pub to: Vec<String>,
    /// Default copies.
    pub cc: Vec<String>,
    /// Default blind copies.
    pub bcc: Vec<String>,
}

/// The templates in view, by name.
///
/// # Errors
/// The database.
pub async fn templates(pool: &PgPool, view: &Access) -> Result<Vec<TemplateRow>, StoreError> {
    if view.is_empty() {
        return Ok(Vec::new());
    }
    Ok(sqlx::query_as::<_, TemplateRow>(sql(&format!(
        "select {TEMPLATE_COLUMNS} from finance.mail_templates where party_id = any($1) order by name"
    )))
    .bind(view.party_ids())
    .fetch_all(pool)
    .await
    .map_err(map_err)?)
}

/// Create or change a template.
///
/// # Errors
/// The party is outside the grant (not found); the name is taken; the database.
pub async fn upsert_template(
    pool: &PgPool,
    access: &Access,
    input: &TemplateInput,
) -> Result<TemplateRow, StoreError> {
    access.require(PartyId(input.party_id), "party_id")?;
    let name = input.name.trim();
    if name.is_empty() || name.len() > 120 {
        return Err(DbError::Invalid {
            field: "name",
            reason: "want 1 to 120 characters".into(),
        }
        .into());
    }
    let id = match input.id {
        Some(id) => {
            let owned: Option<(Uuid,)> = sqlx::query_as(
                "select id from finance.mail_templates where id = $1 and party_id = $2",
            )
            .bind(id)
            .bind(input.party_id)
            .fetch_optional(pool)
            .await
            .map_err(map_err)?;
            owned.ok_or(DbError::NotFound { what: "template" })?;
            id
        }
        None => Uuid::new_v4(),
    };
    let taken: Option<(Uuid,)> = sqlx::query_as(
        "select id from finance.mail_templates where party_id = $1 and name = $2 and id <> $3",
    )
    .bind(input.party_id)
    .bind(name)
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(map_err)?;
    if taken.is_some() {
        return Err(DbError::Invalid {
            field: "name",
            reason: "a template with this name exists".into(),
        }
        .into());
    }
    Ok(sqlx::query_as::<_, TemplateRow>(sql(&format!(
        "insert into finance.mail_templates (id, party_id, name, subject, body, to_addrs, cc_addrs, bcc_addrs)
         values ($1, $2, $3, $4, $5, $6, $7, $8)
         on conflict (id) do update
            set name = excluded.name, subject = excluded.subject, body = excluded.body,
                to_addrs = excluded.to_addrs, cc_addrs = excluded.cc_addrs, bcc_addrs = excluded.bcc_addrs,
                updated_at = now()
         returning {TEMPLATE_COLUMNS}"
    )))
    .bind(id)
    .bind(input.party_id)
    .bind(name)
    .bind(&input.subject)
    .bind(&input.body)
    .bind(clean_addrs(&input.to))
    .bind(clean_addrs(&input.cc))
    .bind(clean_addrs(&input.bcc))
    .fetch_one(pool)
    .await
    .map_err(map_err)?)
}

/// Remove a template. Mails sent from it keep their text; the reference goes.
///
/// # Errors
/// Not in the grant (not found); the database.
pub async fn delete_template(pool: &PgPool, access: &Access, id: Uuid) -> Result<(), StoreError> {
    let row: Option<(Uuid,)> =
        sqlx::query_as("select party_id from finance.mail_templates where id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(map_err)?;
    let (party,) = row.ok_or(DbError::NotFound { what: "template" })?;
    access.require(PartyId(party), "template")?;
    sqlx::query("delete from finance.mail_templates where id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(map_err)?;
    Ok(())
}

fn clean_addrs(v: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for a in v {
        let a = a.trim();
        if !a.is_empty() && !out.iter().any(|o| o.eq_ignore_ascii_case(a)) {
            out.push(a.to_owned());
        }
    }
    out
}

/// The address inside `Name <addr>`, or the whole thing.
fn bare_address(a: &str) -> &str {
    a.rsplit_once('<')
        .map_or(a.trim(), |(_, rest)| rest.trim_end_matches('>').trim())
}

/// A rough shape check: something, an at sign, something with a dot.
fn plausible_address(a: &str) -> bool {
    let a = bare_address(a);
    let Some((user, host)) = a.split_once('@') else {
        return false;
    };
    !user.is_empty()
        && host.contains('.')
        && !host.starts_with('.')
        && !host.ends_with('.')
        && !a.chars().any(char::is_whitespace)
}

/// What a person asks to send.
#[derive(Debug, Clone, Default)]
pub struct SendInput {
    /// A connector in the grant that may send.
    pub connector_id: Uuid,
    /// The template it was composed from, for the record.
    pub template_id: Option<Uuid>,
    /// Recipients.
    pub to: Vec<String>,
    /// Copied.
    pub cc: Vec<String>,
    /// Copied unseen.
    pub bcc: Vec<String>,
    /// Rendered.
    pub subject: String,
    /// Rendered.
    pub body: String,
    /// Rendered HTML beside the text, when the composer made one.
    pub html: Option<String>,
    /// Documents in the grant to attach.
    pub attachment_document_ids: Vec<Uuid>,
    /// A mail of ours (or a reply we hold) to answer in its thread.
    pub in_reply_to_mail_id: Option<Uuid>,
}

/// Send a mail as a connector and record it, sent or failed. Refused before
/// the provider is asked when the connector cannot send, an address is not
/// one, or `allow_to` is set and an address is not in it.
///
/// # Errors
/// Not in the grant (not found); refused (the reason); the database. A
/// provider failure is *not* an error: the row records it as `failed`.
#[allow(clippy::too_many_lines)]
pub async fn send(
    pool: &PgPool,
    access: &Access,
    sealer: &Sealer,
    kinds: &[Box<dyn Connector>],
    allow: &crate::config::Mail,
    input: &SendInput,
) -> Result<(MailRow, Vec<MailDocRow>), StoreError> {
    let connector = crate::connectors::store::get(pool, access, input.connector_id).await?;
    if connector.status != "linked" {
        return Err(StoreError::Refused(format!(
            "the mailbox is {}; link it again",
            connector.status
        )));
    }
    if !connector.can_send {
        return Err(StoreError::Refused(
            "this mailbox was linked without permission to send; link it again and allow sending"
                .into(),
        ));
    }
    let to = clean_addrs(&input.to);
    let cc = clean_addrs(&input.cc);
    let bcc = clean_addrs(&input.bcc);
    if to.is_empty() {
        return Err(StoreError::Refused("no recipient".into()));
    }
    for a in to.iter().chain(&cc).chain(&bcc) {
        if !plausible_address(a) {
            return Err(StoreError::Refused(format!("not an address: {a}")));
        }
        if !allow.allows(bare_address(a)) {
            return Err(StoreError::Refused(format!(
                "{a} is not an allowed recipient in this environment"
            )));
        }
    }
    if input.subject.trim().is_empty() {
        return Err(StoreError::Refused("no subject".into()));
    }
    let kind = kinds
        .iter()
        .find(|k| k.kind().name == connector.kind)
        .ok_or_else(|| StoreError::UnknownKind(connector.kind.clone()))?;

    // Attachments: documents the caller may read, as bytes.
    let mut attachments = Vec::new();
    let mut attached: Vec<Uuid> = Vec::new();
    for id in &input.attachment_document_ids {
        let (doc, _) = crate::documents::store::get(pool, access, *id).await?;
        let bytes = crate::documents::store::bytes(pool, *id).await?;
        attachments.push(Attachment {
            filename: doc.filename.clone().unwrap_or_else(|| format!("{id}.pdf")),
            content_type: doc.content_type.clone(),
            bytes,
        });
        attached.push(*id);
    }
    // Answering: the thread and message id of what is answered.
    let mut in_reply_to = None;
    let mut parent_id = None;
    if let Some(reply_to) = input.in_reply_to_mail_id {
        let parent = get(pool, access, reply_to).await?.0;
        if parent.connector_id != Some(connector.id) {
            return Err(StoreError::Refused(
                "the mail answered was not sent from this mailbox".into(),
            ));
        }
        in_reply_to = Some((
            parent.thread_key.clone().unwrap_or_default(),
            parent.message_id.clone().unwrap_or_default(),
        ));
        parent_id = Some(parent.id);
    }

    let creds = crate::connectors::store::open_credentials(pool, sealer, connector.id).await?;
    let outgoing = Outgoing {
        to: to.clone(),
        cc: cc.clone(),
        bcc: bcc.clone(),
        subject: input.subject.trim().to_owned(),
        text: input.body.clone(),
        html: input.html.clone().filter(|h| !h.trim().is_empty()),
        attachments,
        in_reply_to,
    };
    let sent = kind.send(&creds, &outgoing).await;
    let id = Uuid::new_v4();
    let (status, error, provider_id, thread_key, message_id) = match &sent {
        Ok(s) => (
            "sent",
            None,
            Some(s.provider_id.clone()),
            Some(s.thread_key.clone()),
            Some(s.message_id.clone()),
        ),
        Err(e) => ("failed", Some(e.to_string()), None, None, None),
    };
    let mut tx = pool.begin().await.map_err(map_err)?;
    sqlx::query(
        "insert into finance.mails
            (id, party_id, connector_id, direction, thread_key, provider_id, message_id, in_reply_to,
             parent_id, template_id, from_addr, to_addrs, cc_addrs, bcc_addrs, subject, body, body_html,
             status, error, sent_at)
         values ($1, $2, $3, 'out', $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18,
                 case when $17 = 'sent' then now() end)",
    )
    .bind(id)
    .bind(connector.party_id)
    .bind(connector.id)
    .bind(&thread_key)
    .bind(&provider_id)
    .bind(&message_id)
    .bind(outgoing.in_reply_to.as_ref().map(|(_, m)| m.clone()))
    .bind(parent_id)
    .bind(input.template_id)
    .bind(connector.external_id.clone().unwrap_or_default())
    .bind(&to)
    .bind(&cc)
    .bind(&bcc)
    .bind(&outgoing.subject)
    .bind(&outgoing.text)
    .bind(&outgoing.html)
    .bind(status)
    .bind(&error)
    .execute(&mut *tx)
    .await
    .map_err(map_err)?;
    for doc in &attached {
        sqlx::query("insert into finance.mail_documents (mail_id, document_id) values ($1, $2) on conflict do nothing")
            .bind(id)
            .bind(doc)
            .execute(&mut *tx)
            .await
            .map_err(map_err)?;
    }
    tx.commit().await.map_err(map_err)?;
    if let Err(e) = &sent {
        tracing::warn!(mail = %id, connector = %connector.id, error = %e, "mail: send failed");
    }
    get(pool, access, id).await
}

/// What a listing narrows to.
#[derive(Debug, Clone, Default)]
pub struct Filter {
    /// One mailbox, or any.
    pub connector_id: Option<Uuid>,
    /// Free text over subject, recipients, sender and body.
    pub q: String,
    /// `out`, `in`, or empty.
    pub direction: String,
    /// Page.
    pub limit: i64,
    /// Page.
    pub offset: i64,
}

/// The mails in view, newest first, with how many replies each outgoing
/// one has and its documents.
///
/// # Errors
/// The database.
pub async fn list(
    pool: &PgPool,
    view: &Access,
    filter: &Filter,
) -> Result<(Vec<(MailRow, i64, Vec<MailDocRow>)>, i64), StoreError> {
    if view.is_empty() {
        return Ok((Vec::new(), 0));
    }
    let matching = "from finance.mails m
         where m.party_id = any($1)
           and ($2::uuid is null or m.connector_id = $2)
           and ($3 = '' or m.direction = $3)
           and ($4 = '' or concat_ws(' ', m.subject, m.from_addr, array_to_string(m.to_addrs, ' '),
                                      array_to_string(m.cc_addrs, ' '), m.body) ilike '%' || $4 || '%')";
    let like = filter.q.replace('%', "\\%").replace('_', "\\_");
    let rows = sqlx::query_as::<_, MailRow>(sql(&format!(
        "select {MAIL_COLUMNS} {matching}
          order by coalesce(m.sent_at, m.received_at, m.created_at) desc
          limit $5 offset $6"
    )))
    .bind(view.party_ids())
    .bind(filter.connector_id)
    .bind(&filter.direction)
    .bind(&like)
    .bind(filter.limit.clamp(1, 500))
    .bind(filter.offset.max(0))
    .fetch_all(pool)
    .await
    .map_err(map_err)?;
    let (total,): (i64,) = sqlx::query_as(sql(&format!("select count(*) {matching}")))
        .bind(view.party_ids())
        .bind(filter.connector_id)
        .bind(&filter.direction)
        .bind(&like)
        .fetch_one(pool)
        .await
        .map_err(map_err)?;
    let ids: Vec<Uuid> = rows.iter().map(|m| m.id).collect();
    let replies: Vec<(Uuid, i64)> = sqlx::query_as(
        "select parent_id, count(*) from finance.mails where parent_id = any($1) group by parent_id",
    )
    .bind(&ids)
    .fetch_all(pool)
    .await
    .map_err(map_err)?;
    let docs = documents_of(pool, &ids).await?;
    Ok((
        rows.into_iter()
            .map(|m| {
                let n = replies
                    .iter()
                    .find(|(p, _)| *p == m.id)
                    .map_or(0, |(_, n)| *n);
                let mine = docs.iter().filter(|d| d.mail_id == m.id).cloned().collect();
                (m, n, mine)
            })
            .collect(),
        total,
    ))
}

async fn documents_of(pool: &PgPool, ids: &[Uuid]) -> Result<Vec<MailDocRow>, StoreError> {
    Ok(sqlx::query_as::<_, MailDocRow>(
        "select md.mail_id, md.document_id, d.filename, d.content_type, d.size_bytes
           from finance.mail_documents md join finance.documents d on d.id = md.document_id
          where md.mail_id = any($1)",
    )
    .bind(ids)
    .fetch_all(pool)
    .await
    .map_err(map_err)?)
}

/// One mail the caller may see, with its documents.
///
/// # Errors
/// Not in the grant (not found); the database.
pub async fn get(
    pool: &PgPool,
    access: &Access,
    id: Uuid,
) -> Result<(MailRow, Vec<MailDocRow>), StoreError> {
    let row = sqlx::query_as::<_, MailRow>(sql(&format!(
        "select {MAIL_COLUMNS} from finance.mails m where m.id = $1"
    )))
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(map_err)?
    .ok_or(DbError::NotFound { what: "mail" })?;
    access.require(PartyId(row.party_id), "mail")?;
    let docs = documents_of(pool, &[id]).await?;
    Ok((row, docs))
}

/// The thread of a mail: it, what it answers, and every reply, oldest first.
///
/// # Errors
/// Not in the grant (not found); the database.
pub async fn thread(
    pool: &PgPool,
    access: &Access,
    id: Uuid,
) -> Result<Vec<(MailRow, Vec<MailDocRow>)>, StoreError> {
    let (root, _) = get(pool, access, id).await?;
    let root_id = root.parent_id.unwrap_or(root.id);
    let rows = sqlx::query_as::<_, MailRow>(sql(&format!(
        "select {MAIL_COLUMNS} from finance.mails m
          where m.id = $1 or m.parent_id = $1
          order by coalesce(m.sent_at, m.received_at, m.created_at)"
    )))
    .bind(root_id)
    .fetch_all(pool)
    .await
    .map_err(map_err)?;
    let ids: Vec<Uuid> = rows.iter().map(|m| m.id).collect();
    let docs = documents_of(pool, &ids).await?;
    Ok(rows
        .into_iter()
        .map(|m| {
            let mine = docs.iter().filter(|d| d.mail_id == m.id).cloned().collect();
            (m, mine)
        })
        .collect())
}

/// Import what came back to the threads this connector sent in. Called at
/// the end of a pull. Each reply is stored once (by the provider's id);
/// its attachments become documents: a PDF a receipt, anything else an
/// `attachment`, each linked to the reply.
///
/// # Errors
/// The database. A provider failure on one thread is logged and skipped.
pub async fn import_replies(
    pool: &PgPool,
    sealer: &Sealer,
    kind: &dyn Connector,
    connector: &ConnectorRow,
) -> Result<usize, StoreError> {
    let threads: Vec<(Uuid, String)> = sqlx::query_as(
        "select id, thread_key from finance.mails
          where connector_id = $1 and direction = 'out' and status = 'sent' and thread_key is not null
            and coalesce(sent_at, created_at) > now() - interval '180 days'",
    )
    .bind(connector.id)
    .fetch_all(pool)
    .await
    .map_err(map_err)?;
    if threads.is_empty() {
        return Ok(0);
    }
    let creds = crate::connectors::store::open_credentials(pool, sealer, connector.id).await?;
    let known: Vec<(String,)> = sqlx::query_as(
        "select provider_id from finance.mails where connector_id = $1 and provider_id is not null",
    )
    .bind(connector.id)
    .fetch_all(pool)
    .await
    .map_err(map_err)?;
    let known: std::collections::HashSet<String> = known.into_iter().map(|(p,)| p).collect();
    let seen = |id: &str| known.contains(id);
    let mut imported = 0;
    for (parent_id, thread_key) in threads {
        let replies = match kind.replies(&creds, &thread_key, &seen).await {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!(connector = %connector.id, thread = %thread_key, error = %e, "mail: replies not read");
                continue;
            }
        };
        for reply in replies {
            if known.contains(&reply.provider_id) {
                continue;
            }
            let id = Uuid::new_v4();
            let mut tx = pool.begin().await.map_err(map_err)?;
            let inserted = sqlx::query(
                "insert into finance.mails
                    (id, party_id, connector_id, direction, thread_key, provider_id, message_id, in_reply_to,
                     parent_id, from_addr, to_addrs, cc_addrs, subject, body, status, received_at)
                 values ($1, $2, $3, 'in', $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, 'received', $14)
                 on conflict do nothing",
            )
            .bind(id)
            .bind(connector.party_id)
            .bind(connector.id)
            .bind(&reply.thread_key)
            .bind(&reply.provider_id)
            .bind(&reply.message_id)
            .bind(&reply.in_reply_to)
            .bind(parent_id)
            .bind(&reply.from)
            .bind(&reply.to)
            .bind(&reply.cc)
            .bind(&reply.subject)
            .bind(&reply.text)
            .bind(reply.received_at)
            .execute(&mut *tx)
            .await
            .map_err(map_err)?
            .rows_affected();
            if inserted == 0 {
                tx.rollback().await.map_err(map_err)?;
                continue;
            }
            for a in &reply.attachments {
                let doc = store_attachment(&mut tx, connector, &reply.provider_id, a).await?;
                sqlx::query("insert into finance.mail_documents (mail_id, document_id) values ($1, $2) on conflict do nothing")
                    .bind(id)
                    .bind(doc)
                    .execute(&mut *tx)
                    .await
                    .map_err(map_err)?;
            }
            tx.commit().await.map_err(map_err)?;
            imported += 1;
            tracing::info!(connector = %connector.id, mail = %id, from = %reply.from, "mail: reply imported");
        }
    }
    Ok(imported)
}

/// A reply's attachment as a document of the connector's party. The same
/// bytes seen before are one document; a PDF is a receipt for the reader,
/// anything else an attachment.
async fn store_attachment(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    connector: &ConnectorRow,
    provider_id: &str,
    a: &Attachment,
) -> Result<Uuid, StoreError> {
    let mut hasher = Sha256::new();
    hasher.update(&a.bytes);
    let sha = format!("{:x}", hasher.finalize());
    let existing: Option<(Uuid,)> =
        sqlx::query_as("select id from finance.documents where party_id = $1 and sha256 = $2")
            .bind(connector.party_id)
            .bind(&sha)
            .fetch_optional(&mut **tx)
            .await
            .map_err(map_err)?;
    let id = if let Some((id,)) = existing {
        id
    } else {
        {
            let id = Uuid::new_v4();
            let kind = if a.content_type.eq_ignore_ascii_case("application/pdf")
                || a.filename.to_ascii_lowercase().ends_with(".pdf")
            {
                "receipt"
            } else {
                "attachment"
            };
            sqlx::query(
                "insert into finance.documents (id, party_id, kind, sha256, content_type, size_bytes, filename)
                 values ($1, $2, $3, $4, $5, $6, $7)",
            )
            .bind(id)
            .bind(connector.party_id)
            .bind(kind)
            .bind(&sha)
            .bind(&a.content_type)
            .bind(i64::try_from(a.bytes.len()).unwrap_or(i64::MAX))
            .bind(&a.filename)
            .execute(&mut **tx)
            .await
            .map_err(map_err)?;
            sqlx::query("insert into finance.document_blobs (document_id, bytes) values ($1, $2)")
                .bind(id)
                .bind(&a.bytes)
                .execute(&mut **tx)
                .await
                .map_err(map_err)?;
            id
        }
    };
    sqlx::query(
        "insert into finance.document_sources (document_id, connector_id, external_ref, subject, sender, received_at)
         values ($1, $2, $3, $4, '', now()) on conflict do nothing",
    )
    .bind(id)
    .bind(connector.id)
    .bind(format!("{provider_id}:{}", a.filename))
    .bind(&a.filename)
    .execute(&mut **tx)
    .await
    .map_err(map_err)?;
    Ok(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn addresses_are_checked_loosely_and_deduplicated() {
        assert!(plausible_address("a@b.hr"));
        assert!(plausible_address("Ana <ana@b.hr>"));
        assert!(!plausible_address("ana"));
        assert!(!plausible_address("a@b"));
        assert!(!plausible_address("a b@c.hr"));
        assert_eq!(bare_address("Ana <ana@b.hr>"), "ana@b.hr");
        assert_eq!(
            clean_addrs(&[" a@b.hr ".into(), "A@B.hr".into(), String::new()]),
            vec!["a@b.hr".to_owned()]
        );
    }
}
