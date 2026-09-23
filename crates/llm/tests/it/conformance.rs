//! The conformance suite: one set of cases, run against every engine (the
//! stub, a wiremock playing Ollama, a wiremock playing llama.cpp). An engine
//! is done when it passes here, not when it worked once.

use std::time::Duration;

use tbd_llm::Config;
use tbd_proto::llm::v1::{
    EmbedRequest, GenerateRequest, GetBudgetRequest, ListModelsRequest, Message, Tier,
};
use tonic::{Code, Request};

use crate::support::{self, COMPLETION_TOKENS, Fixture, PROMPT_TOKENS, VISITOR};

fn ask(text: &str) -> GenerateRequest {
    GenerateRequest {
        messages: vec![Message {
            role: "user".into(),
            content: text.into(),
        }],
        tier: Tier::Unspecified as i32,
        max_tokens: None,
        temperature: None,
        session_id: String::new(),
        reasoning: None,
    }
}

async fn chunks_carry_engine_model_tier_and_stub(fixture: Fixture) {
    let name = fixture.name();
    let stub = matches!(fixture, Fixture::Stub);
    let server = support::start_on(fixture, |_| {}).await;
    let mut client = server.client().await;
    let stream = client
        .generate(support::as_caller(&VISITOR, ask("Hello there world")))
        .await
        .unwrap()
        .into_inner();
    let (chunks, error) = support::collect(stream).await;
    assert!(error.is_none(), "{error:?}");
    assert!(chunks.len() >= 2, "{chunks:?}");
    for (i, c) in chunks.iter().enumerate() {
        assert_eq!(c.index as usize, i, "indices are contiguous");
        assert_eq!(c.engine, name);
        assert!(!c.model.is_empty());
        assert_eq!(
            c.tier,
            Tier::Fast as i32,
            "unspecified routes to the default tier"
        );
        assert_eq!(
            c.stub, stub,
            "the stub flag is true for the stub and only the stub"
        );
    }
    let done: Vec<_> = chunks.iter().filter(|c| c.done).collect();
    assert_eq!(done.len(), 1, "exactly one done chunk, the last");
    assert!(chunks.last().unwrap().done);
    // Every engine reasons first in the fixtures; the answer is the chunks
    // that are not reasoning, and reasoning is marked so a client can hide it.
    assert!(
        chunks.iter().any(|c| c.reasoning && !c.text.is_empty()),
        "a reasoning chunk is marked as such: {chunks:?}"
    );
    let answer: String = chunks
        .iter()
        .filter(|c| !c.reasoning)
        .map(|c| c.text.as_str())
        .collect();
    assert_eq!(answer, "Hello there world");
}

async fn usage_counts_arrive_on_the_done_chunk(fixture: Fixture) {
    let server = support::start_on(fixture, |_| {}).await;
    let mut client = server.client().await;
    let stream = client
        .generate(support::as_caller(&VISITOR, ask("a b c")))
        .await
        .unwrap()
        .into_inner();
    let (chunks, _) = support::collect(stream).await;
    for c in &chunks {
        if c.done {
            let u = c.usage.as_ref().expect("usage on the done chunk");
            assert_eq!(
                (u.prompt_tokens, u.completion_tokens),
                (PROMPT_TOKENS, COMPLETION_TOKENS)
            );
        } else {
            assert!(c.usage.is_none(), "no usage before the end");
        }
    }
}

async fn an_engine_failure_mid_stream_is_an_error_item_after_the_chunks(fixture: Fixture) {
    let server = support::start_on(fixture, |_| {}).await;
    let mut client = server.client().await;
    let stream = client
        .generate(support::as_caller(
            &VISITOR,
            ask("<<error>> Hello there world"),
        ))
        .await
        .unwrap()
        .into_inner();
    let (chunks, error) = support::collect(stream).await;
    assert!(
        chunks.iter().filter(|c| !c.reasoning).count() == 2,
        "two answer chunks arrive before the engine goes: {chunks:?}"
    );
    assert!(chunks.iter().all(|c| !c.done));
    let status = error.expect("an error item ends the stream");
    assert!(
        matches!(status.code(), Code::Unavailable | Code::FailedPrecondition),
        "{status:?}"
    );
}

async fn an_engine_that_is_down_is_unavailable_before_the_first_chunk(fixture: Fixture) {
    let stub = matches!(fixture, Fixture::Stub);
    let server = support::start_on(fixture, |c| {
        if !stub {
            // A closed port: the dial fails, nothing is streamed.
            "http://127.0.0.1:1".clone_into(&mut c.engines.fast.url);
            "http://127.0.0.1:1".clone_into(&mut c.engines.deep.url);
        }
    })
    .await;
    let mut client = server.client().await;
    let prompt = if stub { "<<down>> hi" } else { "hi" };
    let err = client
        .generate(support::as_caller(&VISITOR, ask(prompt)))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::Unavailable, "{err:?}");
}

async fn a_refusal_is_a_failed_precondition_before_the_first_chunk(fixture: Fixture) {
    let server = support::start_on(fixture, |_| {}).await;
    let mut client = server.client().await;
    let err = client
        .generate(support::as_caller(&VISITOR, ask("<<refuse>> hi")))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::FailedPrecondition, "{err:?}");
}

async fn no_caller_is_unauthenticated(fixture: Fixture) {
    let server = support::start_on(fixture, |_| {}).await;
    let mut client = server.client().await;
    let err = client.generate(Request::new(ask("hi"))).await.unwrap_err();
    assert_eq!(err.code(), Code::Unauthenticated);
    let err = client
        .embed(Request::new(EmbedRequest {
            inputs: vec!["x".into()],
            tier: 0,
        }))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::Unauthenticated);
    let err = client
        .get_budget(Request::new(GetBudgetRequest {}))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::Unauthenticated);
}

async fn a_stuck_engine_is_a_deadline(fixture: Fixture) {
    let server = support::start_on(fixture, |c| {
        c.engines.fast.timeout_secs = 1;
        c.engines.deep.timeout_secs = 1;
    })
    .await;
    let mut client = server.client().await;
    let started = std::time::Instant::now();
    let code = match client
        .generate(support::as_caller(&VISITOR, ask("<<hang>> hi")))
        .await
    {
        // The wiremock delays its headers: the deadline hits before the first chunk.
        Err(status) => status.code(),
        // The stub sends one chunk and then nothing: the deadline hits on the stream.
        Ok(resp) => {
            let (chunks, error) = support::collect(resp.into_inner()).await;
            assert!(
                chunks.iter().filter(|c| !c.reasoning).count() <= 1,
                "{chunks:?}"
            );
            error.expect("the stream ends with the deadline").code()
        }
    };
    assert_eq!(code, Code::DeadlineExceeded);
    assert!(
        started.elapsed() < Duration::from_secs(5),
        "the deadline is the tier's timeout, not the engine's patience"
    );
}

async fn list_models_names_both_tiers_and_the_default(fixture: Fixture) {
    let name = fixture.name();
    let stub = matches!(fixture, Fixture::Stub);
    let server = support::start_on(fixture, |_| {}).await;
    let mut client = server.client().await;
    let resp = client
        .list_models(Request::new(ListModelsRequest {}))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(resp.default_tier, Tier::Fast as i32);
    let mut tiers: Vec<i32> = resp.models.iter().map(|m| m.tier).collect();
    tiers.sort_unstable();
    assert_eq!(tiers, vec![Tier::Fast as i32, Tier::Deep as i32]);
    for m in &resp.models {
        assert_eq!(m.engine, name);
        assert!(!m.model.is_empty());
        assert_eq!(m.stub, stub);
    }
}

async fn embed_returns_one_vector_per_input(fixture: Fixture) {
    let name = fixture.name();
    let server = support::start_on(fixture, |_| {}).await;
    let mut client = server.client().await;
    let resp = client
        .embed(support::as_caller(
            &VISITOR,
            EmbedRequest {
                inputs: vec!["first".into(), "second one".into()],
                tier: Tier::Fast as i32,
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(resp.vectors.len(), 2);
    assert!(resp.vectors.iter().all(|v| !v.values.is_empty()));
    assert_eq!(resp.engine, name);
    assert_eq!(resp.tier, Tier::Fast as i32);
}

async fn a_request_that_breaks_the_bounds_is_invalid(fixture: Fixture) {
    let server = support::start_on(fixture, |c| {
        c.generate.max_messages = 2;
        c.generate.max_message_len = 16;
    })
    .await;
    let mut client = server.client().await;
    let mut empty = ask("hi");
    empty.messages.clear();
    let err = client
        .generate(support::as_caller(&VISITOR, empty))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::InvalidArgument);
    let mut bad_role = ask("hi");
    bad_role.messages[0].role = "wizard".into();
    let err = client
        .generate(support::as_caller(&VISITOR, bad_role))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::InvalidArgument);
    let err = client
        .generate(support::as_caller(
            &VISITOR,
            ask("a message longer than sixteen bytes"),
        ))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::InvalidArgument);
    let mut bad_tier = ask("hi");
    bad_tier.tier = 99;
    let err = client
        .generate(support::as_caller(&VISITOR, bad_tier))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::InvalidArgument);
}

/// The deep tier reaches the deep engine: the fast tier is the stub, the deep
/// tier is the fixture, and a `TIER_DEEP` request answers with the fixture's
/// name. The one case that mixes engines, so routing itself is proven.
async fn tier_deep_routes_to_the_deep_engine(fixture: Fixture) {
    let name = fixture.name();
    let server =
        support::start_fixtures(Fixture::Stub, fixture, tbd_llm::Runtime::default(), |_| {}).await;
    let mut client = server.client().await;
    let mut deep = ask("Hello there world");
    deep.tier = Tier::Deep as i32;
    let (chunks, error) = support::collect(
        client
            .generate(support::as_caller(&VISITOR, deep))
            .await
            .unwrap()
            .into_inner(),
    )
    .await;
    assert!(error.is_none());
    assert!(
        chunks
            .iter()
            .all(|c| c.engine == name && c.tier == Tier::Deep as i32)
    );
    let (chunks, _) = support::collect(
        client
            .generate(support::as_caller(&VISITOR, ask("Hello")))
            .await
            .unwrap()
            .into_inner(),
    )
    .await;
    assert!(
        chunks
            .iter()
            .all(|c| c.engine == "stub" && c.tier == Tier::Fast as i32)
    );
}

async fn without_a_store_the_budget_says_so(fixture: Fixture) {
    let server = support::start_on(fixture, |c: &mut Config| c.budget.tokens_per_day = 10).await;
    let mut client = server.client().await;
    let resp = client
        .get_budget(support::as_caller(&VISITOR, GetBudgetRequest {}))
        .await
        .unwrap()
        .into_inner();
    assert!(
        resp.unlimited,
        "no store: nothing is counted, so nothing is limited"
    );
    assert!(!resp.recorded);
    assert_eq!(resp.tokens_per_day, 10);
    assert_eq!(resp.day.len(), 10, "YYYY-MM-DD");
}

macro_rules! conformance_suite {
    ($prefix:ident, $fixture:expr) => {
        mod $prefix {
            use super::*;
            macro_rules! case {
                ($name:ident) => {
                    #[tokio::test]
                    async fn $name() {
                        super::$name($fixture).await;
                    }
                };
            }
            case!(chunks_carry_engine_model_tier_and_stub);
            case!(usage_counts_arrive_on_the_done_chunk);
            case!(an_engine_failure_mid_stream_is_an_error_item_after_the_chunks);
            case!(an_engine_that_is_down_is_unavailable_before_the_first_chunk);
            case!(a_refusal_is_a_failed_precondition_before_the_first_chunk);
            case!(no_caller_is_unauthenticated);
            case!(a_stuck_engine_is_a_deadline);
            case!(list_models_names_both_tiers_and_the_default);
            case!(embed_returns_one_vector_per_input);
            case!(a_request_that_breaks_the_bounds_is_invalid);
            case!(tier_deep_routes_to_the_deep_engine);
            case!(without_a_store_the_budget_says_so);
        }
    };
}

conformance_suite!(stub, Fixture::Stub);
conformance_suite!(ollama, Fixture::Ollama);
conformance_suite!(llamacpp, Fixture::Llamacpp);
