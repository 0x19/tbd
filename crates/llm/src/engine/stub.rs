//! The in-process engine for tests and the chaos tool. Deterministic, fast,
//! scripted by markers in the last user message, and labelled `stub` on every
//! surface. `Config::validate` refuses it in production.
//!
//! Markers (the wiremock fakes in the conformance suite honour the same ones,
//! which is what lets one suite run against every engine):
//! - `<<down>>`: unavailable before the first chunk.
//! - `<<refuse>>`: refused before the first chunk.
//! - `<<error>>`: two chunks, then the engine goes away.
//! - `<<hang>>`: one chunk, then nothing, forever.
//!
//! Every generation opens with one reasoning chunk, as a reasoning model
//! would, so the contract that reasoning is marked and kept out of the
//! answer is exercised on every engine.

use std::time::Duration;

use async_trait::async_trait;
use futures::StreamExt as _;

use super::{Chunk, ChunkStream, Engine, EngineError, GenerateSpec, Identity, Usage};
use crate::config::EngineKind;

/// The usage every stub generation reports; the fakes report the same, so the
/// conformance assertions are shared.
pub const USAGE: Usage = Usage {
    prompt_tokens: 7,
    completion_tokens: 3,
};

/// The stub engine.
#[derive(Debug, Clone)]
pub struct Stub {
    model: String,
    embeds: bool,
}

impl Stub {
    /// A stub serving `model` (any name; `stub-model` by convention), embedding
    /// only when told to, like every engine.
    #[must_use]
    pub fn new(model: &str, embeds: bool) -> Self {
        Self {
            model: model.to_owned(),
            embeds,
        }
    }
}

fn last_user(spec: &GenerateSpec) -> String {
    spec.messages
        .iter()
        .rev()
        .find(|m| m.role == "user")
        .map(|m| m.content.clone())
        .unwrap_or_default()
}

#[async_trait]
impl Engine for Stub {
    fn kind(&self) -> EngineKind {
        EngineKind::Stub
    }

    fn model(&self) -> &str {
        &self.model
    }

    fn embed_model(&self) -> &str {
        &self.model
    }

    fn embeds(&self) -> bool {
        self.embeds
    }

    fn stub(&self) -> bool {
        true
    }

    async fn health(&self) -> Result<Vec<String>, EngineError> {
        Ok(vec![self.model.clone()])
    }

    async fn identity(&self) -> Result<Identity, EngineError> {
        Ok(Identity {
            engine_version: "stub".to_owned(),
            model_revision: "stub".to_owned(),
        })
    }

    async fn generate(&self, spec: GenerateSpec) -> Result<ChunkStream, EngineError> {
        let prompt = last_user(&spec);
        if prompt.contains("<<down>>") {
            return Err(EngineError::Unavailable("stub: down".to_owned()));
        }
        if prompt.contains("<<refuse>>") {
            return Err(EngineError::Refused("stub: refused".to_owned()));
        }
        let error = prompt.contains("<<error>>");
        let hang = prompt.contains("<<hang>>");
        let model = Some(self.model.clone());
        let words: Vec<String> = prompt
            .split_whitespace()
            .filter(|w| !w.starts_with("<<"))
            .map(str::to_owned)
            .collect();
        let words = if words.is_empty() {
            vec!["stub".to_owned()]
        } else {
            words
        };
        let take = if error || hang {
            words.len().min(if hang { 1 } else { 2 })
        } else {
            words.len()
        };
        let thought = Chunk::reasoning("thinking", model.clone());
        let stream = futures::stream::once(async move { Ok(thought) })
            .chain(
                futures::stream::iter(words.into_iter().take(take).enumerate()).then({
                    let model = model.clone();
                    move |(i, w)| {
                        let model = model.clone();
                        async move {
                            tokio::time::sleep(Duration::from_millis(1)).await;
                            let text = if i == 0 { w } else { format!(" {w}") };
                            Ok(Chunk::text(text, model))
                        }
                    }
                }),
            )
            .chain(futures::stream::once(async move {
                if error {
                    Err(EngineError::Unavailable("stub: went away".to_owned()))
                } else if hang {
                    futures::future::pending::<()>().await;
                    unreachable!("pending never resolves")
                } else {
                    Ok(Chunk::done(USAGE, model))
                }
            }));
        Ok(stream.boxed())
    }

    async fn embed(&self, inputs: Vec<String>) -> Result<Vec<Vec<f32>>, EngineError> {
        Ok(inputs
            .iter()
            .map(|s| {
                let n = f32::from(u16::try_from(s.len()).unwrap_or(u16::MAX));
                vec![n, n / 2.0, 1.0, 0.0]
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::Message;

    fn spec(text: &str) -> GenerateSpec {
        GenerateSpec {
            messages: vec![Message {
                role: "user".into(),
                content: text.into(),
            }],
            max_tokens: None,
            temperature: None,
            reasoning: None,
        }
    }

    #[tokio::test]
    async fn streams_the_words_then_a_done_chunk_with_the_fixed_usage() {
        let items: Vec<_> = Stub::new("m", true)
            .generate(spec("hello there world"))
            .await
            .unwrap_or_else(|e| panic!("{e}"))
            .collect()
            .await;
        let texts: Vec<String> = items
            .iter()
            .map(|i| i.as_ref().map(|c| c.text.clone()).unwrap_or_default())
            .collect();
        assert_eq!(texts, ["thinking", "hello", " there", " world", ""]);
        assert!(items[0].as_ref().is_ok_and(|c| c.reasoning));
        let last = items.last().and_then(|i| i.as_ref().ok()).cloned();
        assert_eq!(last.and_then(|c| c.usage), Some(USAGE));
    }

    #[tokio::test]
    async fn the_error_marker_yields_two_chunks_then_an_error() {
        let items: Vec<_> = Stub::new("m", true)
            .generate(spec("<<error>> a b c"))
            .await
            .unwrap_or_else(|e| panic!("{e}"))
            .collect()
            .await;
        assert_eq!(items.len(), 4, "reasoning, two words, the error");
        assert!(items[0].is_ok() && items[1].is_ok() && items[2].is_ok());
        assert!(matches!(items[3], Err(EngineError::Unavailable(_))));
    }

    #[tokio::test]
    async fn the_down_marker_fails_before_the_first_chunk() {
        assert!(matches!(
            Stub::new("m", true).generate(spec("<<down>>")).await.err(),
            Some(EngineError::Unavailable(_))
        ));
    }
}
