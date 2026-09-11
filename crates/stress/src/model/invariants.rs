//! The ledger's contract as checks over data: a model, a request, and what
//! came back. Every checker is pure and returns the rules it found broken;
//! the worker decides when to call which. `ALL` is the catalogue a campaign's
//! `[invariants]` and the report's counters are keyed by.

use std::collections::BTreeSet;

use chrono::{DateTime, Utc};
use tbd_proto::ledger::v1::{AppendRequest, AppendResponse, Fact, RetractResponse};
use tonic::Code;
use uuid::Uuid;

use super::{ModelFact, SubjectModel, generate};
use crate::{
    client::CallError,
    trace::{datetime, fact_json},
};

/// Every invariant, in the order reports list them.
pub const ALL: &[&str] = &[
    "append_echo",
    "recorded_at_monotonic",
    "current_is_latest",
    "history_is_everything",
    "history_cut",
    "retract_semantics",
    "consent_filter",
    "path_source_filter",
    "pagination",
    "idempotency",
    "expiry",
    "erasure_denies",
    "erasure_executes",
    "cascade_tombstones_counterparty",
    "durability",
    "acked_visible",
    "current_consistent",
    "clean_refusal",
    "clean_errors",
];

/// The rules an owner worker evaluates on every run (every one but the
/// contention and fuzz rules and `durability`, which needs a fault to check).
pub const OWNER: &[&str] = &[
    "append_echo",
    "recorded_at_monotonic",
    "current_is_latest",
    "history_is_everything",
    "history_cut",
    "retract_semantics",
    "consent_filter",
    "path_source_filter",
    "pagination",
    "idempotency",
    "expiry",
    "erasure_denies",
    "erasure_executes",
    "cascade_tombstones_counterparty",
    "clean_errors",
];

/// The rules a contention worker evaluates.
pub const CONTENTION: &[&str] = &[
    "append_echo",
    "recorded_at_monotonic",
    "pagination",
    "retract_semantics",
    "acked_visible",
    "current_consistent",
    "clean_errors",
];

/// The rules a fuzz worker evaluates.
pub const FUZZ: &[&str] = &["clean_refusal", "clean_errors"];

/// One broken rule, with what was expected and what came back.
pub type Violation = crate::trace::Violation;

fn violation(
    invariant: &'static str,
    message: impl Into<String>,
    expected: serde_json::Value,
    actual: serde_json::Value,
) -> Violation {
    Violation {
        invariant,
        message: message.into(),
        expected,
        actual,
    }
}

fn model_json(f: &ModelFact) -> serde_json::Value {
    serde_json::to_value(f).unwrap_or(serde_json::Value::Null)
}

fn ids(facts: &[Fact]) -> Vec<i64> {
    facts.iter().map(|f| f.id).collect()
}

fn stamp(f: &Fact) -> Option<(DateTime<Utc>, i64)> {
    Some((f.recorded_at.as_ref().and_then(datetime)?, f.id))
}

/// Why a wire fact does not match a model row, or `None` when it does.
#[must_use]
pub fn mismatch(model: &ModelFact, wire: &Fact) -> Option<String> {
    let w = ModelFact::from_wire(wire);
    if let Some(id) = model.id
        && id != wire.id
    {
        return Some(format!("id {} != {id}", wire.id));
    }
    if let Some(r) = model.recorded_at
        && w.recorded_at.map(generate::micros) != Some(generate::micros(r))
    {
        return Some("recorded_at differs".into());
    }
    if model.path != w.path {
        return Some(format!("path {:?} != {:?}", w.path, model.path));
    }
    if model.source != w.source {
        return Some(format!("source {} != {}", w.source, model.source));
    }
    if model.value != w.value {
        return Some("value differs".into());
    }
    if model.origin != w.origin && model.id.is_some() {
        return Some("origin differs".into());
    }
    if !generate::close(model.confidence, w.confidence) {
        return Some(format!(
            "confidence {:?} != {:?}",
            w.confidence, model.confidence
        ));
    }
    if model.counterparty != w.counterparty {
        return Some("counterparty differs".into());
    }
    if let Some(o) = model.observed_at
        && w.observed_at.map(generate::micros) != Some(generate::micros(o))
    {
        return Some("observed_at differs".into());
    }
    if model.expires_at.map(generate::micros) != w.expires_at.map(generate::micros)
        && (model.expires_at.is_some() || model.id.is_some())
    {
        return Some("expires_at differs".into());
    }
    if model.consent_set() != w.consent_set() {
        return Some(format!("consent {:?} != {:?}", w.consent, model.consent));
    }
    if model.stub != w.stub {
        return Some("stub differs".into());
    }
    None
}

/// An acknowledged append echoes the request and extends the subject's order.
#[must_use]
pub fn append_echo(
    req: &AppendRequest,
    resp: &AppendResponse,
    model: &SubjectModel,
) -> Vec<Violation> {
    let mut out = Vec::new();
    let Some(fact) = &resp.fact else {
        return vec![violation(
            "append_echo",
            "append answered without a fact",
            serde_json::json!("a fact"),
            serde_json::Value::Null,
        )];
    };
    if fact.subject_id != req.subject_id {
        out.push(violation(
            "append_echo",
            "the fact names another subject",
            serde_json::json!(req.subject_id),
            serde_json::json!(fact.subject_id),
        ));
    }
    if fact.id <= 0 {
        out.push(violation(
            "append_echo",
            "the fact id is not positive",
            serde_json::json!("id > 0"),
            serde_json::json!(fact.id),
        ));
    }
    if fact.recorded_at.is_none() {
        out.push(violation(
            "append_echo",
            "the fact has no recorded_at",
            serde_json::json!("a timestamp"),
            serde_json::Value::Null,
        ));
    }
    let sent = generate::model_fact_of(req);
    if let Some(why) = mismatch(&sent, fact) {
        out.push(violation(
            "append_echo",
            format!("the fact does not echo the request: {why}"),
            model_json(&sent),
            fact_json(fact),
        ));
    }
    if !resp.replayed {
        if model.known_ids().contains(&fact.id) {
            out.push(violation(
                "append_echo",
                "a fresh append reused an id the subject already has",
                serde_json::json!("an unused id"),
                serde_json::json!(fact.id),
            ));
        }
        out.extend(monotonic_after(model, fact));
    }
    out
}

/// `(recorded_at, id)` of a new row is greater than every stamp acknowledged before it.
fn monotonic_after(model: &SubjectModel, fact: &Fact) -> Vec<Violation> {
    let Some(new) = stamp(fact) else {
        return Vec::new();
    };
    match model.last_stamp {
        Some(last) if new <= last => vec![violation(
            "recorded_at_monotonic",
            "a new row is not after the last acknowledged one",
            serde_json::json!({"after": {"recorded_at": last.0.to_rfc3339(), "id": last.1}}),
            serde_json::json!({"recorded_at": new.0.to_rfc3339(), "id": new.1}),
        )],
        _ => Vec::new(),
    }
}

/// A page is strictly ordered by `(recorded_at, id)`.
#[must_use]
pub fn page_ordered(facts: &[Fact]) -> Vec<Violation> {
    for w in facts.windows(2) {
        let (a, b) = (stamp(&w[0]), stamp(&w[1]));
        if a.is_none() || b.is_none() || a >= b {
            return vec![violation(
                "recorded_at_monotonic",
                "a page is not strictly ordered by (recorded_at, id)",
                serde_json::json!("strictly increasing"),
                serde_json::json!([fact_json(&w[0]), fact_json(&w[1])]),
            )];
        }
    }
    Vec::new()
}

/// Whether the wire rows are exactly the model rows (order aside, compared by
/// id where known, by content where not). `pending` rows may or may not be there.
fn same_set(
    invariant: &'static str,
    what: &str,
    expected: &[&ModelFact],
    pending: &[ModelFact],
    got: &[Fact],
) -> Vec<Violation> {
    let mut out = Vec::new();
    let mut unexplained: Vec<&Fact> = Vec::new();
    let mut claimed: Vec<bool> = vec![false; expected.len()];
    for w in got {
        let hit = expected.iter().enumerate().find(|(i, m)| {
            if claimed[*i] {
                return false;
            }
            if let Some(id) = m.id {
                return id == w.id;
            }
            let wm = ModelFact::from_wire(w);
            wm.is_tombstone() && wm.path == m.path && wm.source == m.source
        });
        if let Some((i, m)) = hit {
            claimed[i] = true;
            if let Some(why) = mismatch(m, w) {
                out.push(violation(
                    invariant,
                    format!("{what}: row {} differs from the model: {why}", w.id),
                    model_json(m),
                    fact_json(w),
                ));
            }
        } else {
            let wm = ModelFact::from_wire(w);
            if !pending.iter().any(|p| generate::same_content(p, &wm)) {
                unexplained.push(w);
            }
        }
    }
    for (i, m) in expected.iter().enumerate() {
        if !claimed[i] {
            out.push(violation(
                invariant,
                format!(
                    "{what}: the model holds {} {} at {} that the ledger did not return",
                    if m.is_tombstone() {
                        "a tombstone for"
                    } else {
                        "a fact"
                    },
                    m.path,
                    m.id.map_or("an unknown id".to_owned(), |id| format!("id {id}"))
                ),
                model_json(m),
                serde_json::json!({"ids": ids(got)}),
            ));
        }
    }
    if !unexplained.is_empty() {
        out.push(violation(
            invariant,
            format!(
                "{what}: the ledger returned {} row(s) the model does not hold",
                unexplained.len()
            ),
            serde_json::json!({"ids": expected.iter().filter_map(|m| m.id).collect::<Vec<_>>()}),
            serde_json::json!(unexplained.iter().map(|f| fact_json(f)).collect::<Vec<_>>()),
        ));
    }
    out
}

/// Split violations whose subject fact carries an expiry off to `expiry`.
fn tag_expiry(mut v: Vec<Violation>) -> Vec<Violation> {
    for x in &mut v {
        let touches_expiry = x.expected.get("expires_at").is_some_and(|e| !e.is_null())
            || x.actual.get("expires_at").is_some_and(|e| !e.is_null());
        if touches_expiry {
            x.invariant = "expiry";
        }
    }
    v
}

/// `Current` under every scope is the latest live valued row per key.
#[must_use]
pub fn current_is_latest(
    model: &SubjectModel,
    now: DateTime<Utc>,
    skew: chrono::Duration,
    got: &[Fact],
) -> Vec<Violation> {
    let latest = model.latest_valued(now, skew);
    let ambiguous: BTreeSet<(&str, i32)> = latest.ambiguous.iter().map(|f| f.key()).collect();
    let got: Vec<Fact> = got
        .iter()
        .filter(|f| !ambiguous.contains(&(f.path.as_str(), f.source)))
        .cloned()
        .collect();
    let mut out = page_ordered(&got);
    for f in &got {
        if f.value.is_none() {
            out.push(violation(
                "current_is_latest",
                "current returned a tombstone",
                serde_json::json!("valued rows only"),
                fact_json(f),
            ));
        }
    }
    let mut keys = BTreeSet::new();
    for f in &got {
        if !keys.insert((f.path.as_str(), f.source)) {
            out.push(violation(
                "current_is_latest",
                format!("current returned two rows for {} / {}", f.path, f.source),
                serde_json::json!("one row per (path, source)"),
                serde_json::json!(ids(&got)),
            ));
        }
    }
    let pending: Vec<ModelFact> = model.pending.iter().map(|p| p.fact.clone()).collect();
    out.extend(tag_expiry(same_set(
        "current_is_latest",
        "current",
        &latest.live,
        &pending,
        &got,
    )));
    out
}

/// `History` without a cut is every row the subject holds, tombstones included.
#[must_use]
pub fn history_is_everything(model: &SubjectModel, got: &[Fact]) -> Vec<Violation> {
    let held = model.held();
    let pending: Vec<ModelFact> = model.pending.iter().map(|p| p.fact.clone()).collect();
    let mut out = page_ordered(got);
    out.extend(same_set(
        "history_is_everything",
        "history",
        &held,
        &pending,
        got,
    ));
    out
}

/// `History` at `at` is every row recorded at or before it that is still held.
/// `None` when the model has rows with unknown stamps: the cut cannot be judged.
#[must_use]
pub fn history_cut(
    model: &SubjectModel,
    at: DateTime<Utc>,
    got: &[Fact],
) -> Option<Vec<Violation>> {
    if model.has_unknown_stamps() || !model.pending.is_empty() {
        return None;
    }
    let held = model.held_at(at);
    let mut out = page_ordered(got);
    for f in got {
        if stamp(f).is_none_or(|(r, _)| generate::micros(r) > generate::micros(at)) {
            out.push(violation(
                "history_cut",
                "a row after the cut was returned",
                serde_json::json!({"at": at.to_rfc3339()}),
                fact_json(f),
            ));
        }
    }
    out.extend(same_set("history_cut", "history at a cut", &held, &[], got));
    Some(out)
}

/// A retraction: a tombstone that fits when there was a value, `NotFound` when not.
#[must_use]
pub fn retract_semantics(
    model: &SubjectModel,
    path: &str,
    source: i32,
    result: Result<&RetractResponse, &CallError>,
) -> Vec<Violation> {
    let latest = model.valued(path, source);
    match (latest, result) {
        (Some(v), Ok(resp)) => {
            let Some(t) = &resp.tombstone else {
                return vec![violation(
                    "retract_semantics",
                    "retract answered without a tombstone",
                    serde_json::json!("a tombstone"),
                    serde_json::Value::Null,
                )];
            };
            let mut out = Vec::new();
            if t.path != path || t.source != source {
                out.push(violation(
                    "retract_semantics",
                    "the tombstone is for another key",
                    serde_json::json!({"path": path, "source": source}),
                    fact_json(t),
                ));
            }
            if t.value.is_some() {
                out.push(violation(
                    "retract_semantics",
                    "the tombstone carries a value",
                    serde_json::json!("no value"),
                    fact_json(t),
                ));
            }
            if t.counterparty_id.is_some() {
                out.push(violation(
                    "retract_semantics",
                    "the tombstone names a counterparty",
                    serde_json::json!("no counterparty"),
                    fact_json(t),
                ));
            }
            let want: BTreeSet<&str> = v.consent_set();
            let got: BTreeSet<&str> = t.consent.iter().map(String::as_str).collect();
            if want != got {
                out.push(violation(
                    "retract_semantics",
                    "the tombstone's consent is not the retracted value's",
                    serde_json::json!(v.consent),
                    serde_json::json!(t.consent),
                ));
            }
            if model.known_ids().contains(&t.id) {
                out.push(violation(
                    "retract_semantics",
                    "the tombstone reused an id",
                    serde_json::json!("an unused id"),
                    serde_json::json!(t.id),
                ));
            }
            out.extend(monotonic_after(model, t));
            out
        }
        (Some(v), Err(e)) => vec![violation(
            "retract_semantics",
            format!("retract of a valued key failed with {}", e.class()),
            serde_json::json!({"tombstone for": v.path}),
            serde_json::to_value(e).unwrap_or_default(),
        )],
        (None, Ok(resp)) => vec![violation(
            "retract_semantics",
            "retract of a key with nothing valued succeeded",
            serde_json::json!("NotFound"),
            serde_json::json!(resp.tombstone.as_ref().map(fact_json)),
        )],
        (None, Err(e)) if e.code() == Some(Code::NotFound) => Vec::new(),
        (None, Err(e)) => vec![violation(
            "retract_semantics",
            format!(
                "retract of a key with nothing valued failed with {}",
                e.class()
            ),
            serde_json::json!("NotFound"),
            serde_json::to_value(e).unwrap_or_default(),
        )],
    }
}

/// `Current` under a scope subset returns the live rows whose consent meets it.
#[must_use]
pub fn consent_filter(
    model: &SubjectModel,
    now: DateTime<Utc>,
    skew: chrono::Duration,
    scopes: &[String],
    got: &[Fact],
) -> Vec<Violation> {
    let latest = model.latest_valued(now, skew);
    let ambiguous: BTreeSet<(&str, i32)> = latest.ambiguous.iter().map(|f| f.key()).collect();
    let want: Vec<&ModelFact> = latest
        .live
        .into_iter()
        .filter(|f| f.consent.iter().any(|c| scopes.contains(c)))
        .collect();
    let got: Vec<Fact> = got
        .iter()
        .filter(|f| !ambiguous.contains(&(f.path.as_str(), f.source)))
        .cloned()
        .collect();
    let pending: Vec<ModelFact> = model.pending.iter().map(|p| p.fact.clone()).collect();
    same_set(
        "consent_filter",
        &format!("current under {scopes:?}"),
        &want,
        &pending,
        &got,
    )
}

/// Whether `path` matches a filter (`x.*` is a prefix, else exact).
fn path_matches(filter: &str, path: &str) -> bool {
    match filter.strip_suffix(".*") {
        Some(prefix) => path.starts_with(prefix) && path[prefix.len()..].starts_with('.'),
        None => filter == path,
    }
}

/// `Current` with path and source filters returns the live rows they select.
#[must_use]
pub fn path_source_filter(
    model: &SubjectModel,
    now: DateTime<Utc>,
    skew: chrono::Duration,
    paths: &[String],
    sources: &[i32],
    got: &[Fact],
) -> Vec<Violation> {
    let latest = model.latest_valued(now, skew);
    let ambiguous: BTreeSet<(&str, i32)> = latest.ambiguous.iter().map(|f| f.key()).collect();
    let want: Vec<&ModelFact> = latest
        .live
        .into_iter()
        .filter(|f| paths.is_empty() || paths.iter().any(|p| path_matches(p, &f.path)))
        .filter(|f| sources.is_empty() || sources.contains(&f.source))
        .collect();
    let got: Vec<Fact> = got
        .iter()
        .filter(|f| !ambiguous.contains(&(f.path.as_str(), f.source)))
        .cloned()
        .collect();
    let pending: Vec<ModelFact> = model.pending.iter().map(|p| p.fact.clone()).collect();
    same_set(
        "path_source_filter",
        &format!("current with paths {paths:?} sources {sources:?}"),
        &want,
        &pending,
        &got,
    )
}

/// A walk through pages: each full page is `limit` long, the last `next` is
/// empty, no id repeats, the order holds across page boundaries.
#[must_use]
pub fn pagination(pages: &[Vec<Fact>], nexts: &[String], limit: u32) -> Vec<Violation> {
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    let mut last: Option<(DateTime<Utc>, i64)> = None;
    for (i, page) in pages.iter().enumerate() {
        let is_last = i + 1 == pages.len();
        if !is_last && limit > 0 && page.len() != limit as usize {
            out.push(violation(
                "pagination",
                format!("page {i} is not full but a next cursor followed"),
                serde_json::json!({"len": limit}),
                serde_json::json!({"len": page.len()}),
            ));
        }
        if limit > 0 && page.len() > limit as usize {
            out.push(violation(
                "pagination",
                format!("page {i} is longer than the limit"),
                serde_json::json!({"at most": limit}),
                serde_json::json!({"len": page.len()}),
            ));
        }
        for f in page {
            if !seen.insert(f.id) {
                out.push(violation(
                    "pagination",
                    format!("id {} appears on two pages", f.id),
                    serde_json::json!("each id once"),
                    serde_json::json!(f.id),
                ));
            }
            if let (Some(s), Some(l)) = (stamp(f), last)
                && s <= l
            {
                out.push(violation(
                    "pagination",
                    "order breaks across a page boundary",
                    serde_json::json!({"after": {"recorded_at": l.0.to_rfc3339(), "id": l.1}}),
                    fact_json(f),
                ));
            }
            if stamp(f).is_some() {
                last = stamp(f);
            }
        }
    }
    if nexts.last().is_some_and(|n| !n.is_empty()) {
        out.push(violation(
            "pagination",
            "the last page still carries a next cursor",
            serde_json::json!(""),
            serde_json::json!(nexts.last()),
        ));
    }
    out
}

/// What an idempotency replay is expected to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayKind {
    /// Same key, same content: the earlier fact comes back, `replayed = true`.
    Same {
        /// The earlier fact's id.
        fact_id: i64,
    },
    /// Same key, other content: `Aborted`.
    Conflict,
    /// Same key after the fact was retracted: `Aborted`.
    AfterRetract,
}

/// The idempotency rules.
#[must_use]
pub fn idempotency(
    kind: ReplayKind,
    result: Result<&AppendResponse, &CallError>,
) -> Vec<Violation> {
    match (kind, result) {
        (ReplayKind::Same { fact_id }, Ok(resp)) => {
            let mut out = Vec::new();
            if !resp.replayed {
                out.push(violation(
                    "idempotency",
                    "a replay with the same key and content was not marked replayed",
                    serde_json::json!({"replayed": true}),
                    serde_json::json!({"replayed": false}),
                ));
            }
            if resp.fact.as_ref().is_none_or(|f| f.id != fact_id) {
                out.push(violation(
                    "idempotency",
                    "a replay returned another fact",
                    serde_json::json!({"id": fact_id}),
                    serde_json::json!(resp.fact.as_ref().map(fact_json)),
                ));
            }
            out
        }
        (ReplayKind::Same { .. }, Err(e)) => vec![violation(
            "idempotency",
            format!(
                "a replay with the same key and content failed with {}",
                e.class()
            ),
            serde_json::json!("the earlier fact, replayed"),
            serde_json::to_value(e).unwrap_or_default(),
        )],
        (ReplayKind::Conflict | ReplayKind::AfterRetract, Err(e))
            if e.code() == Some(Code::Aborted) =>
        {
            Vec::new()
        }
        (ReplayKind::Conflict, r) => vec![violation(
            "idempotency",
            format!(
                "a key reused with other content was answered with {} instead of Aborted",
                outcome_label(r)
            ),
            serde_json::json!("Aborted"),
            outcome_json(r),
        )],
        (ReplayKind::AfterRetract, r) => vec![violation(
            "idempotency",
            format!(
                "a key whose fact was retracted was answered with {} instead of Aborted",
                outcome_label(r)
            ),
            serde_json::json!("Aborted"),
            outcome_json(r),
        )],
    }
}

fn outcome_label(r: Result<&AppendResponse, &CallError>) -> String {
    match r {
        Ok(resp) if resp.replayed => "a replay".into(),
        Ok(_) => "a new fact".into(),
        Err(e) => e.class(),
    }
}

fn outcome_json(r: Result<&AppendResponse, &CallError>) -> serde_json::Value {
    match r {
        Ok(resp) => {
            serde_json::json!({"fact": resp.fact.as_ref().map(fact_json), "replayed": resp.replayed})
        }
        Err(e) => serde_json::to_value(e).unwrap_or_default(),
    }
}

/// A call on an erased subject is denied.
#[must_use]
pub fn denied<T>(what: &str, result: Result<&T, &CallError>) -> Vec<Violation> {
    match result {
        Err(e) if e.code() == Some(Code::FailedPrecondition) => Vec::new(),
        Err(e) => vec![violation(
            "erasure_denies",
            format!(
                "{what} on an erased subject failed with {} instead of FailedPrecondition",
                e.class()
            ),
            serde_json::json!("FailedPrecondition"),
            serde_json::to_value(e).unwrap_or_default(),
        )],
        Ok(_) => vec![violation(
            "erasure_denies",
            format!("{what} on an erased subject succeeded"),
            serde_json::json!("FailedPrecondition"),
            serde_json::json!("ok"),
        )],
    }
}

/// A second erase inside the window answers the same request.
#[must_use]
pub fn erase_idempotent(
    first: &tbd_proto::ledger::v1::EraseResponse,
    second: Result<&tbd_proto::ledger::v1::EraseResponse, &CallError>,
) -> Vec<Violation> {
    match second {
        Ok(s) if s.requested_at == first.requested_at => Vec::new(),
        Ok(s) => vec![violation(
            "erasure_denies",
            "a second erase inside the window opened a new one",
            crate::trace::erase_json(first),
            crate::trace::erase_json(s),
        )],
        Err(e) => vec![violation(
            "erasure_denies",
            format!("a second erase inside the window failed with {}", e.class()),
            crate::trace::erase_json(first),
            serde_json::to_value(e).unwrap_or_default(),
        )],
    }
}

/// After the window: reads and writes denied, restore `NotFound`.
#[must_use]
pub fn executed<T, U>(
    read: Result<&T, &CallError>,
    restore: Result<&U, &CallError>,
    append: Result<&AppendResponse, &CallError>,
) -> Vec<Violation> {
    let mut out = Vec::new();
    if !matches!(read, Err(e) if e.code() == Some(Code::FailedPrecondition)) {
        out.push(violation(
            "erasure_executes",
            "a read after the window was not denied",
            serde_json::json!("FailedPrecondition"),
            match read {
                Ok(_) => serde_json::json!("ok"),
                Err(e) => serde_json::to_value(e).unwrap_or_default(),
            },
        ));
    }
    if !matches!(restore, Err(e) if e.code() == Some(Code::NotFound)) {
        out.push(violation(
            "erasure_executes",
            "restore after the cascade did not answer NotFound",
            serde_json::json!("NotFound"),
            match restore {
                Ok(_) => serde_json::json!("ok"),
                Err(e) => serde_json::to_value(e).unwrap_or_default(),
            },
        ));
    }
    if !matches!(append, Err(e) if e.code() == Some(Code::FailedPrecondition)) {
        out.push(violation(
            "erasure_executes",
            "an append after the cascade was not denied: the id is being reused",
            serde_json::json!("FailedPrecondition"),
            outcome_json(append),
        ));
    }
    out
}

/// After a peer's cascade this subject holds one tombstone per relation that
/// named the peer, and everything else as before. Call before the model is
/// updated with `apply_counterparty_erased`.
#[must_use]
pub fn cascade_tombstones_counterparty(
    model: &SubjectModel,
    erased: Uuid,
    got: &[Fact],
) -> Vec<Violation> {
    let mut out = Vec::new();
    let wire: Vec<ModelFact> = got.iter().map(ModelFact::from_wire).collect();
    for f in model
        .facts
        .iter()
        .filter(|f| !f.is_tombstone() && f.counterparty == Some(erased))
    {
        if let Some(id) = f.id
            && let Some(still) = got.iter().find(|w| w.id == id)
        {
            out.push(violation(
                "cascade_tombstones_counterparty",
                format!(
                    "the relation {} naming the erased subject is still held",
                    f.path
                ),
                serde_json::json!("deleted, a tombstone in its place"),
                fact_json(still),
            ));
        }
    }
    let expected_keys: Vec<(String, i32, BTreeSet<String>)> = model
        .latest_naming(erased)
        .into_iter()
        .map(|((path, source), consent)| (path, source, consent.into_iter().collect()))
        .collect();
    for (path, source, consent) in &expected_keys {
        let tomb = wire.iter().find(|w| {
            w.is_tombstone()
                && w.path == *path
                && w.source == *source
                && w.counterparty.is_none()
                && !model.known_ids().contains(&w.id.unwrap_or(-1))
        });
        match tomb {
            None => out.push(violation(
                "cascade_tombstones_counterparty",
                format!("no tombstone for {path} after the counterparty's cascade"),
                serde_json::json!({"path": path, "source": source, "origin": {"cause": "counterparty_erased"}}),
                serde_json::json!(ids(got)),
            )),
            Some(t) => {
                if t.origin != serde_json::json!({"cause": "counterparty_erased"}) {
                    out.push(violation(
                        "cascade_tombstones_counterparty",
                        format!("the cascade tombstone for {path} does not carry the cause"),
                        serde_json::json!({"cause": "counterparty_erased"}),
                        t.origin.clone(),
                    ));
                }
                let got_consent: BTreeSet<String> = t.consent.iter().cloned().collect();
                if &got_consent != consent {
                    out.push(violation(
                        "cascade_tombstones_counterparty",
                        format!("the cascade tombstone for {path} does not carry the value's consent"),
                        serde_json::json!(consent),
                        serde_json::json!(t.consent),
                    ));
                }
            }
        }
    }
    // Everything not naming the peer is untouched.
    let untouched: Vec<&ModelFact> = model
        .facts
        .iter()
        .filter(|f| f.counterparty != Some(erased))
        .collect();
    for m in untouched {
        if let Some(id) = m.id
            && !got.iter().any(|w| w.id == id)
        {
            out.push(violation(
                "cascade_tombstones_counterparty",
                format!(
                    "a row not naming the erased subject disappeared: {} id {id}",
                    m.path
                ),
                model_json(m),
                serde_json::json!(ids(got)),
            ));
        }
    }
    out
}

/// A failure the ledger may never answer with, or one outside what the
/// campaign tolerates.
#[must_use]
pub fn clean_errors(e: &CallError, tolerate: &[String]) -> Option<Violation> {
    if e.is_unclean() {
        return Some(violation(
            "clean_errors",
            format!("the ledger answered {}", e.class()),
            serde_json::json!("a client-error status or an answer"),
            serde_json::to_value(e).unwrap_or_default(),
        ));
    }
    let class = e.class();
    let infrastructural = matches!(
        class.as_str(),
        "transport" | "timeout" | "unavailable" | "deadline_exceeded" | "cancelled"
    );
    if infrastructural && !tolerate.contains(&class) {
        return Some(violation(
            "clean_errors",
            format!("the call failed with {class}, which the campaign does not tolerate"),
            serde_json::json!({"tolerated": tolerate}),
            serde_json::to_value(e).unwrap_or_default(),
        ));
    }
    None
}

/// The stamp of a wire fact, when it carries one.
#[must_use]
pub fn wire_stamp(f: &Fact) -> Option<(DateTime<Utc>, i64)> {
    stamp(f)
}

/// An append acknowledged by a shared subject: an echo of the request is all a
/// contention worker can hold, so `append_echo` here compares the wire fact
/// with the request alone.
#[must_use]
pub fn append_echo_shared(req: &AppendRequest, resp: &AppendResponse) -> Vec<Violation> {
    let Some(fact) = &resp.fact else {
        return vec![violation(
            "append_echo",
            "append answered without a fact",
            serde_json::json!("a fact"),
            serde_json::Value::Null,
        )];
    };
    let mut out = Vec::new();
    if fact.id <= 0 || fact.recorded_at.is_none() {
        out.push(violation(
            "append_echo",
            "appended fact has no positive id or no recorded_at",
            serde_json::json!({"id": "> 0", "recorded_at": "set"}),
            fact_json(fact),
        ));
    }
    if let Some(why) = mismatch(&generate::model_fact_of(req), fact) {
        out.push(violation(
            "append_echo",
            format!("appended fact does not echo the request: {why}"),
            serde_json::to_value(generate::model_fact_of(req)).unwrap_or_default(),
            fact_json(fact),
        ));
    }
    out
}

/// One acknowledged write a contention worker remembers.
#[derive(Debug, Clone, PartialEq)]
pub struct Acked {
    /// The fact's id.
    pub id: i64,
    /// Its stamp.
    pub stamp: (DateTime<Utc>, i64),
    /// `(path, source)`.
    pub key: (String, i32),
}

/// `acked_visible`: every fact this worker was acknowledged is in the full
/// history, or a tombstone for its key with a later stamp is (someone
/// retracted it). Nothing else about a shared subject is knowable.
#[must_use]
pub fn acked_visible(acked: &[Acked], history: &[Fact]) -> Vec<Violation> {
    let present: BTreeSet<i64> = ids(history).into_iter().collect();
    let mut out = Vec::new();
    for a in acked {
        if present.contains(&a.id) {
            continue;
        }
        let retracted_later = history.iter().any(|f| {
            f.value.is_none()
                && (f.path.as_str(), f.source) == (a.key.0.as_str(), a.key.1)
                && stamp(f).is_some_and(|s| s > a.stamp)
        });
        if !retracted_later {
            out.push(violation(
                "acked_visible",
                format!(
                    "acknowledged fact {} on {}/{} is missing from history and no later tombstone covers it",
                    a.id,
                    a.key.0,
                    generate::source_name(a.key.1)
                ),
                serde_json::json!({"id": a.id, "path": a.key.0, "source": a.key.1}),
                serde_json::json!(ids(history)),
            ));
        }
    }
    out
}

/// `current_consistent`: with `History` read first and `Current` after, every
/// current item is either the newest valued row of its key in the snapshot, or
/// newer than everything in the snapshot (written in between); never a
/// tombstone; never a key that the snapshot shows retracted after its stamp.
#[must_use]
pub fn current_consistent(snapshot: &[Fact], current: &[Fact]) -> Vec<Violation> {
    let max_stamp = snapshot.iter().filter_map(stamp).max();
    let mut out = Vec::new();
    for c in current {
        if c.value.is_none() {
            out.push(violation(
                "current_consistent",
                format!("current shows a tombstone (id {})", c.id),
                serde_json::json!("valued facts only"),
                fact_json(c),
            ));
            continue;
        }
        let Some(cs) = stamp(c) else {
            out.push(violation(
                "current_consistent",
                format!("current item {} has no recorded_at", c.id),
                serde_json::json!("a stamp"),
                fact_json(c),
            ));
            continue;
        };
        if max_stamp.is_some_and(|m| cs > m) {
            continue; // newer than the snapshot: written between the two reads
        }
        let key = (c.path.as_str(), c.source);
        let newest_of_key = snapshot
            .iter()
            .filter(|f| (f.path.as_str(), f.source) == key)
            .filter_map(|f| stamp(f).map(|s| (s, f)))
            .max_by_key(|(s, _)| *s);
        match newest_of_key {
            Some((_, f)) if f.id == c.id => {}
            Some((_, f)) => out.push(violation(
                "current_consistent",
                format!(
                    "current shows id {} for {}/{} but the history snapshot's newest row for the key is id {} ({})",
                    c.id,
                    c.path,
                    generate::source_name(c.source),
                    f.id,
                    if f.value.is_none() { "a tombstone" } else { "valued" }
                ),
                fact_json(f),
                fact_json(c),
            )),
            None => out.push(violation(
                "current_consistent",
                format!(
                    "current shows id {} for {}/{}, not newer than the snapshot yet absent from it",
                    c.id,
                    c.path,
                    generate::source_name(c.source)
                ),
                serde_json::json!("present in the snapshot"),
                fact_json(c),
            )),
        }
    }
    out
}

/// `retract_semantics` on a shared subject: a retraction answers a tombstone
/// for the key, or `NotFound` when nothing was valued; the content it is judged
/// against is the key alone.
#[must_use]
pub fn tombstone_shape(
    path: &str,
    source: i32,
    result: Result<&RetractResponse, &CallError>,
) -> Vec<Violation> {
    match result {
        Ok(resp) => {
            let Some(t) = &resp.tombstone else {
                return vec![violation(
                    "retract_semantics",
                    "retract answered without a tombstone",
                    serde_json::json!("a tombstone"),
                    serde_json::Value::Null,
                )];
            };
            let mut out = Vec::new();
            if t.value.is_some()
                || t.path != path
                || t.source != source
                || t.counterparty_id.is_some()
            {
                out.push(violation(
                    "retract_semantics",
                    "the tombstone does not name the retracted key without a value",
                    serde_json::json!({"path": path, "source": source, "value": null}),
                    fact_json(t),
                ));
            }
            out
        }
        Err(e) if e.code() == Some(Code::NotFound) => Vec::new(),
        Err(e) => vec![violation(
            "retract_semantics",
            format!("retract failed with {}", e.class()),
            serde_json::json!("a tombstone or NotFound"),
            serde_json::to_value(e).unwrap_or_default(),
        )],
    }
}

/// `clean_refusal`: a hostile request drew one of the codes its case allows
/// (`ok` for a request that must succeed), never anything else.
#[must_use]
pub fn clean_refusal(
    case: &str,
    expect: &[&str],
    outcome: Result<(), &CallError>,
) -> Vec<Violation> {
    let class = outcome.map_or_else(CallError::class, |()| "ok".to_owned());
    if expect.contains(&class.as_str()) {
        return Vec::new();
    }
    vec![violation(
        "clean_refusal",
        format!(
            "fuzz case {case} was answered with {class} instead of {}",
            expect.join(" or ")
        ),
        serde_json::json!(expect),
        serde_json::json!(class),
    )]
}

#[cfg(test)]
mod tests {
    use tbd_proto::ledger::v1::Envelope;

    use super::*;

    fn wire(id: i64, path: &str, value: Option<&str>, secs: i64, consent: &[&str]) -> Fact {
        Fact {
            subject_id: "s".into(),
            id,
            path: path.into(),
            source: 2,
            value: value.map(|v| Envelope {
                version: 0,
                bytes: serde_json::to_vec(&serde_json::json!(v)).unwrap(),
            }),
            origin: Some(Envelope {
                version: 0,
                bytes: b"{}".to_vec(),
            }),
            confidence: Some(1.0),
            counterparty_id: None,
            observed_at: Some(prost_types::Timestamp {
                seconds: 50,
                nanos: 0,
            }),
            recorded_at: Some(prost_types::Timestamp {
                seconds: secs,
                nanos: 0,
            }),
            expires_at: None,
            consent: consent.iter().map(|s| (*s).to_owned()).collect(),
            stub: false,
        }
    }

    fn model_with(facts: &[Fact]) -> SubjectModel {
        let mut m = SubjectModel::new(Uuid::now_v7());
        for f in facts {
            m.facts.push(ModelFact::from_wire(f));
        }
        m.last_stamp = m.facts.iter().filter_map(ModelFact::stamp).max();
        m
    }

    #[test]
    fn current_and_history_agree_with_the_model_and_catch_drift() {
        let a = wire(1, "profile.name", Some("a"), 100, &["self"]);
        let b = wire(2, "profile.name", Some("b"), 101, &["self"]);
        let c = wire(3, "profile.bio", Some("c"), 102, &["engine.base"]);
        let m = model_with(&[a.clone(), b.clone(), c.clone()]);
        let now = Utc::now();
        let skew = chrono::Duration::milliseconds(500);
        assert!(current_is_latest(&m, now, skew, &[b.clone(), c.clone()]).is_empty());
        let drift = current_is_latest(&m, now, skew, &[a.clone(), c.clone()]);
        assert!(
            drift.iter().any(|v| v.message.contains("did not return")),
            "{drift:?}"
        );
        assert!(history_is_everything(&m, &[a.clone(), b.clone(), c.clone()]).is_empty());
        let missing = history_is_everything(&m, &[a.clone(), c.clone()]);
        assert_eq!(missing.len(), 1);
        let cut = history_cut(
            &m,
            DateTime::from_timestamp(101, 0).unwrap(),
            &[a.clone(), b.clone()],
        )
        .unwrap();
        assert!(cut.is_empty(), "{cut:?}");
        let late = history_cut(
            &m,
            DateTime::from_timestamp(100, 0).unwrap(),
            &[a.clone(), b.clone()],
        )
        .unwrap();
        assert!(late.iter().any(|v| v.message.contains("after the cut")));
        assert!(
            consent_filter(
                &m,
                now,
                skew,
                &["engine.base".into()],
                std::slice::from_ref(&c)
            )
            .is_empty()
        );
        assert!(
            !consent_filter(
                &m,
                now,
                skew,
                &["engine.base".into()],
                &[a.clone(), c.clone()]
            )
            .is_empty()
        );
        assert!(
            path_source_filter(
                &m,
                now,
                skew,
                &["profile.*".into()],
                &[],
                &[b.clone(), c.clone()]
            )
            .is_empty()
        );
        assert!(
            !path_source_filter(
                &m,
                now,
                skew,
                &["profile.*".into()],
                &[],
                std::slice::from_ref(&c)
            )
            .is_empty()
        );
        let _ = a;
    }

    #[test]
    fn pages_and_order() {
        let p1 = vec![
            wire(1, "a.b", Some("x"), 1, &["self"]),
            wire(2, "a.c", Some("x"), 2, &["self"]),
        ];
        let p2 = vec![wire(3, "a.d", Some("x"), 3, &["self"])];
        assert!(pagination(&[p1.clone(), p2.clone()], &["c".into(), String::new()], 2).is_empty());
        let dup = pagination(
            &[p1.clone(), vec![wire(2, "a.c", Some("x"), 2, &["self"])]],
            &["c".into(), String::new()],
            2,
        );
        assert!(dup.iter().any(|v| v.message.contains("two pages")));
        let short = pagination(&[p2.clone(), p1.clone()], &["c".into(), String::new()], 2);
        assert!(short.iter().any(|v| v.message.contains("not full")));
        assert!(!page_ordered(&[p1[1].clone(), p1[0].clone()]).is_empty());
    }

    #[test]
    fn retract_idempotency_and_erasure_rules() {
        let a = wire(1, "profile.name", Some("a"), 100, &["self", "engine.base"]);
        let m = model_with(&[a]);
        let tomb = RetractResponse {
            tombstone: Some(wire(2, "profile.name", None, 101, &["engine.base", "self"])),
        };
        assert!(retract_semantics(&m, "profile.name", 2, Ok(&tomb)).is_empty());
        let wrong = RetractResponse {
            tombstone: Some(wire(2, "profile.name", None, 101, &["self"])),
        };
        assert!(!retract_semantics(&m, "profile.name", 2, Ok(&wrong)).is_empty());
        let nf = CallError::from_status(&tonic::Status::not_found("x"));
        assert!(retract_semantics(&m, "profile.bio", 2, Err(&nf)).is_empty());
        assert!(!retract_semantics(&m, "profile.bio", 2, Ok(&tomb)).is_empty());

        let ok = AppendResponse {
            fact: Some(wire(1, "profile.name", Some("a"), 100, &["self"])),
            replayed: true,
        };
        assert!(idempotency(ReplayKind::Same { fact_id: 1 }, Ok(&ok)).is_empty());
        let aborted = CallError::from_status(&tonic::Status::aborted("x"));
        assert!(idempotency(ReplayKind::Conflict, Err(&aborted)).is_empty());
        assert!(!idempotency(ReplayKind::Conflict, Ok(&ok)).is_empty());

        let fp = CallError::from_status(&tonic::Status::failed_precondition("erased"));
        assert!(denied::<()>("current", Err(&fp)).is_empty());
        assert!(!denied("current", Ok(&())).is_empty());
        assert!(executed::<(), ()>(Err(&fp), Err(&nf), Err(&fp)).is_empty());
        assert_eq!(executed::<(), ()>(Ok(&()), Ok(&()), Ok(&ok)).len(), 3);
        assert!(
            clean_errors(&CallError::from_status(&tonic::Status::internal("x")), &[]).is_some()
        );
        assert!(clean_errors(&CallError::Timeout, &[]).is_some());
        assert!(clean_errors(&CallError::Timeout, &["timeout".into()]).is_none());
        assert!(clean_errors(&nf, &[]).is_none());
    }

    #[test]
    fn cascade_on_the_surviving_side() {
        let erased = Uuid::now_v7();
        let mut rel = wire(1, "relations.match.m1", Some("x"), 100, &["self"]);
        rel.counterparty_id = Some(erased.to_string());
        let own = wire(2, "profile.name", Some("a"), 101, &["self"]);
        let m = model_with(&[rel, own.clone()]);
        let mut tomb = wire(3, "relations.match.m1", None, 102, &["self"]);
        tomb.origin = Some(Envelope {
            version: 0,
            bytes: serde_json::to_vec(&serde_json::json!({"cause": "counterparty_erased"}))
                .unwrap(),
        });
        assert!(
            cascade_tombstones_counterparty(&m, erased, &[own.clone(), tomb.clone()]).is_empty()
        );
        let no_tomb = cascade_tombstones_counterparty(&m, erased, &[own]);
        assert!(no_tomb.iter().any(|v| v.message.contains("no tombstone")));
    }
}
