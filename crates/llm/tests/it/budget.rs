//! The record and the budget, on a real migrated Postgres: what a generation
//! leaves behind, how the daily limit bites, and who may continue a session.

use std::time::Duration;

use tbd_proto::llm::v1::{GenerateRequest, GetBudgetRequest, Message, Tier};
use tonic::Code;
use uuid::Uuid;

use crate::support::{self, COMPLETION_TOKENS, Fixture, OTHER, PROMPT_TOKENS, VISITOR};

const PER_GENERATION: u64 = (PROMPT_TOKENS + COMPLETION_TOKENS) as u64;

fn ask(text: &str, session_id: &str) -> GenerateRequest {
    GenerateRequest {
        messages: vec![Message {
            role: "user".into(),
            content: text.into(),
        }],
        tier: Tier::Unspecified as i32,
        max_tokens: None,
        temperature: None,
        session_id: session_id.to_owned(),
        reasoning: None,
        agent: String::new(),
        page: String::new(),
    }
}

#[tokio::test]
async fn generations_are_recorded_and_the_budget_counts_them() {
    // Fifteen tokens a day, ten per generation: the first is fine, the second
    // starts under the limit and may overrun, the third is refused.
    let (server, pool) = support::start_with_store(Fixture::Stub, |c| {
        c.budget.tokens_per_day = PER_GENERATION + 5;
    })
    .await;
    let mut client = server.client().await;
    support::wait_probed(&mut client).await;

    let (chunks, error) = support::collect(
        client
            .generate(support::as_caller(&VISITOR, ask("Hello there world", "")))
            .await
            .unwrap()
            .into_inner(),
    )
    .await;
    assert!(error.is_none());
    let first = chunks.last().unwrap();
    assert!(!first.session_id.is_empty() && !first.generation_id.is_empty());
    assert!(
        chunks
            .iter()
            .all(|c| c.generation_id == first.generation_id)
    );

    // Wait for the row to close: the finish is recorded off the stream's path.
    let row = wait_for(&pool, &first.generation_id, "ok").await;
    assert_eq!(row.subject, VISITOR.subject);
    assert_eq!(row.engine, "stub");
    assert!(row.stub);
    assert_eq!(
        row.agent, "",
        "a conversation with the bare model names no agent"
    );
    assert_eq!(row.tier, "fast");
    assert_eq!(
        (row.engine_version.as_str(), row.model_revision.as_str()),
        ("stub", "stub"),
        "the row names the build and the weights the probe read"
    );
    assert_eq!(
        (row.prompt_tokens, row.completion_tokens),
        (
            i32::try_from(PROMPT_TOKENS).unwrap(),
            i32::try_from(COMPLETION_TOKENS).unwrap()
        )
    );
    assert!(row.first_token_ms.is_some());
    assert!(row.finished_at.is_some());

    let budget = client
        .get_budget(support::as_caller(&VISITOR, GetBudgetRequest {}))
        .await
        .unwrap()
        .into_inner();
    assert!(budget.recorded && !budget.unlimited);
    assert_eq!(budget.used_today, PER_GENERATION);
    assert_eq!(budget.remaining, 5);

    let (chunks, error) = support::collect(
        client
            .generate(support::as_caller(&VISITOR, ask("second", "")))
            .await
            .unwrap()
            .into_inner(),
    )
    .await;
    assert!(
        error.is_none(),
        "an in-flight generation may overrun by one"
    );
    wait_for(&pool, &chunks.last().unwrap().generation_id, "ok").await;

    let err = client
        .generate(support::as_caller(&VISITOR, ask("third", "")))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::ResourceExhausted, "{err:?}");
    assert!(err.message().contains("00:00 UTC"), "{}", err.message());

    // Someone else's budget is their own.
    let (_, error) = support::collect(
        client
            .generate(support::as_caller(&OTHER, ask("mine", "")))
            .await
            .unwrap()
            .into_inner(),
    )
    .await;
    assert!(error.is_none());
}

#[tokio::test]
async fn a_failed_generation_keeps_the_error_and_counts_nothing() {
    let (server, pool) = support::start_with_store(Fixture::Ollama, |_| {}).await;
    let mut client = server.client().await;
    let (chunks, error) = support::collect(
        client
            .generate(support::as_caller(&VISITOR, ask("<<error>> a b c", "")))
            .await
            .unwrap()
            .into_inner(),
    )
    .await;
    assert!(error.is_some());
    let row = wait_for(&pool, &chunks[0].generation_id, "failed").await;
    assert!(!row.error.is_empty());
    assert_eq!(row.engine, "ollama");
    assert_eq!((row.prompt_tokens, row.completion_tokens), (0, 0));

    let budget = client
        .get_budget(support::as_caller(&VISITOR, GetBudgetRequest {}))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(budget.used_today, 0);
}

#[tokio::test]
async fn a_caller_who_goes_away_leaves_a_cancelled_row() {
    let (server, pool) = support::start_with_store(Fixture::Stub, |_| {}).await;
    let mut client = server.client().await;
    let mut stream = client
        .generate(support::as_caller(&VISITOR, ask("<<hang>> one", "")))
        .await
        .unwrap()
        .into_inner();
    let first = stream.message().await.unwrap().unwrap();
    let id = first.generation_id.clone();
    drop(stream);
    drop(client);
    let row = wait_for(&pool, &id, "cancelled").await;
    assert_eq!(row.error, "the caller went away");
}

#[tokio::test]
async fn a_session_is_its_owners_and_groups_their_generations() {
    let (server, pool) = support::start_with_store(Fixture::Stub, |_| {}).await;
    let mut client = server.client().await;
    let (chunks, _) = support::collect(
        client
            .generate(support::as_caller(&VISITOR, ask("first", "")))
            .await
            .unwrap()
            .into_inner(),
    )
    .await;
    let session = chunks[0].session_id.clone();

    let err = client
        .generate(support::as_caller(&OTHER, ask("theirs", &session)))
        .await
        .unwrap_err();
    assert_eq!(
        err.code(),
        Code::NotFound,
        "a foreign session does not exist"
    );

    let err = client
        .generate(support::as_caller(&VISITOR, ask("x", "not-a-uuid")))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::InvalidArgument);

    let (chunks, _) = support::collect(
        client
            .generate(support::as_caller(&VISITOR, ask("again", &session)))
            .await
            .unwrap()
            .into_inner(),
    )
    .await;
    assert_eq!(chunks[0].session_id, session);
    wait_for(&pool, &chunks[0].generation_id, "ok").await;
    let rows = tbd_llm::store::generations_of(&pool, Uuid::parse_str(&session).unwrap())
        .await
        .unwrap();
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn without_a_limit_the_budget_is_unlimited_but_still_recorded() {
    let (server, _pool) = support::start_with_store(Fixture::Llamacpp, |c| {
        c.budget.tokens_per_day = 0;
    })
    .await;
    let mut client = server.client().await;
    let (_, error) = support::collect(
        client
            .generate(support::as_caller(&VISITOR, ask("hi", "")))
            .await
            .unwrap()
            .into_inner(),
    )
    .await;
    assert!(error.is_none());
    tokio::time::sleep(Duration::from_millis(200)).await;
    let budget = client
        .get_budget(support::as_caller(&VISITOR, GetBudgetRequest {}))
        .await
        .unwrap()
        .into_inner();
    assert!(budget.unlimited && budget.recorded);
    assert_eq!(budget.used_today, PER_GENERATION);
}

#[tokio::test]
async fn an_unlimited_subject_is_recorded_but_never_refused() {
    let (server, pool) = support::start_with_store(Fixture::Stub, |c| {
        c.budget.tokens_per_day = 1;
        c.budget.unlimited_subjects = vec![OTHER.subject.to_owned()];
    })
    .await;
    let mut client = server.client().await;
    // The limited caller is refused on the second try; the instrument never is.
    let (chunks, _) = support::collect(
        client
            .generate(support::as_caller(&VISITOR, ask("one", "")))
            .await
            .unwrap()
            .into_inner(),
    )
    .await;
    wait_for(&pool, &chunks[0].generation_id, "ok").await;
    let err = client
        .generate(support::as_caller(&VISITOR, ask("two", "")))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::ResourceExhausted);
    for _ in 0..3 {
        let (chunks, error) = support::collect(
            client
                .generate(support::as_caller(&OTHER, ask("measure", "")))
                .await
                .unwrap()
                .into_inner(),
        )
        .await;
        assert!(error.is_none());
        wait_for(&pool, &chunks[0].generation_id, "ok").await;
    }
    let budget = client
        .get_budget(support::as_caller(&OTHER, GetBudgetRequest {}))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(budget.used_today, 3 * PER_GENERATION, "still recorded");
}

/// The row once it has reached `status`, or a panic after a few seconds.
async fn wait_for(pool: &sqlx::PgPool, id: &str, status: &str) -> tbd_llm::store::GenerationRow {
    let id = Uuid::parse_str(id).unwrap();
    for _ in 0..50 {
        if let Some(row) = tbd_llm::store::generation(pool, id).await.unwrap()
            && row.status == status
        {
            return row;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    let row = tbd_llm::store::generation(pool, id).await.unwrap();
    panic!("generation {id} never reached {status}: {row:?}");
}
