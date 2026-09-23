//! llama.cpp's `llama-server` over its OpenAI-compatible API:
//! `POST /v1/chat/completions` with `stream: true` as server-sent events,
//! `GET /v1/models` for health, `POST /v1/embeddings` for vectors (the server
//! must have been started with `--embeddings`, else it refuses and so do we).
//!
//! Wire shape: `data: {"choices":[{"delta":{"content":".."},"finish_reason":null}],..}`
//! per token, a chunk with `finish_reason` set at the end, then, when the
//! request asked for `stream_options.include_usage`, one more chunk with empty
//! `choices` and `usage`, then `data: [DONE]`. Builds vary: some put the
//! counts in `timings` (`prompt_n`, `predicted_n`) on the last chunk and never
//! send a usage chunk. Both are read; a missing count is zero, never a guess.

use async_trait::async_trait;
use serde::Deserialize;

use super::{
    Chunk, ChunkStream, Engine, EngineError, GenerateSpec, LineParser, Usage, http_client, join,
    lines, parse_stream, send,
};
use crate::config::{EngineConfig, EngineKind};

/// The llama.cpp engine.
#[derive(Debug, Clone)]
pub struct Llamacpp {
    http: reqwest::Client,
    url: String,
    model: String,
    embed_model: String,
}

impl Llamacpp {
    /// From `[engines.<tier>]`.
    ///
    /// # Errors
    /// The URL does not parse or the HTTP client cannot be built.
    pub fn new(cfg: &EngineConfig) -> Result<Self, String> {
        url::Url::parse(&cfg.url).map_err(|e| format!("url {:?}: {e}", cfg.url))?;
        Ok(Self {
            http: http_client()?,
            url: cfg.url.clone(),
            model: cfg.model.clone(),
            embed_model: cfg.embed_model().to_owned(),
        })
    }
}

#[derive(Deserialize)]
struct Event {
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    choices: Vec<Choice>,
    #[serde(default)]
    usage: Option<OpenAiUsage>,
    #[serde(default)]
    timings: Option<Timings>,
    #[serde(default)]
    error: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct Choice {
    #[serde(default)]
    delta: Option<Delta>,
    #[serde(default)]
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct Delta {
    #[serde(default)]
    content: Option<String>,
}

#[derive(Deserialize)]
struct OpenAiUsage {
    #[serde(default)]
    prompt_tokens: u32,
    #[serde(default)]
    completion_tokens: u32,
}

#[derive(Deserialize)]
struct Timings {
    #[serde(default)]
    prompt_n: u32,
    #[serde(default)]
    predicted_n: u32,
}

/// Parses the SSE lines.
#[derive(Debug, Default)]
pub(crate) struct Parser {
    model: Option<String>,
    /// The engine said it finished; the usage may still follow.
    finished: bool,
    /// Counts seen so far (from `timings` on the finishing chunk, say).
    usage: Option<Usage>,
    done_sent: bool,
}

impl Parser {
    fn done(&mut self) -> Chunk {
        self.done_sent = true;
        Chunk::done(self.usage.unwrap_or_default(), self.model.clone())
    }
}

impl LineParser for Parser {
    fn feed(&mut self, line: &str) -> Vec<Result<Chunk, EngineError>> {
        let line = line.trim();
        if line.is_empty() || line.starts_with(':') {
            return vec![];
        }
        let Some(data) = line.strip_prefix("data:") else {
            // `event:`/`id:` fields: nothing this parser needs.
            return vec![];
        };
        let data = data.trim();
        if data == "[DONE]" {
            return if self.done_sent {
                vec![]
            } else {
                vec![Ok(self.done())]
            };
        }
        let ev: Event = match serde_json::from_str(data) {
            Ok(e) => e,
            Err(e) => return vec![Err(EngineError::Protocol(format!("event: {e}")))],
        };
        if let Some(error) = ev.error {
            return vec![Err(EngineError::Refused(super::error_message(
                &serde_json::json!({ "error": error }).to_string(),
            )))];
        }
        if ev.model.is_some() {
            self.model = ev.model;
        }
        if let Some(u) = ev.usage {
            self.usage = Some(Usage {
                prompt_tokens: u.prompt_tokens,
                completion_tokens: u.completion_tokens,
            });
        } else if let Some(t) = ev.timings {
            self.usage = Some(Usage {
                prompt_tokens: t.prompt_n,
                completion_tokens: t.predicted_n,
            });
        }
        let mut out = Vec::new();
        for choice in ev.choices {
            let text = choice.delta.and_then(|d| d.content).unwrap_or_default();
            if !text.is_empty() {
                out.push(Ok(Chunk::text(text, self.model.clone())));
            }
            if choice.finish_reason.is_some() {
                self.finished = true;
            }
        }
        // A usage chunk after the finish, or a finishing chunk that already
        // carried the counts: the generation is complete now.
        if self.finished && self.usage.is_some() && !self.done_sent {
            out.push(Ok(self.done()));
        }
        out
    }

    fn finish(&mut self) -> Vec<Result<Chunk, EngineError>> {
        if self.done_sent {
            vec![]
        } else if self.finished {
            vec![Ok(self.done())]
        } else {
            vec![Err(EngineError::Unavailable(
                "the response ended before the engine said it was done".to_owned(),
            ))]
        }
    }
}

#[derive(Deserialize)]
struct Models {
    #[serde(default)]
    data: Vec<ModelId>,
}

#[derive(Deserialize)]
struct ModelId {
    id: String,
}

#[derive(Deserialize)]
struct Embeddings {
    #[serde(default)]
    data: Vec<EmbeddingRow>,
}

#[derive(Deserialize)]
struct EmbeddingRow {
    #[serde(default)]
    index: usize,
    #[serde(default)]
    embedding: Vec<f32>,
}

#[async_trait]
impl Engine for Llamacpp {
    fn kind(&self) -> EngineKind {
        EngineKind::Llamacpp
    }

    fn model(&self) -> &str {
        &self.model
    }

    fn embed_model(&self) -> &str {
        &self.embed_model
    }

    async fn health(&self) -> Result<Vec<String>, EngineError> {
        let resp = send(self.http.get(join(&self.url, "v1/models"))).await?;
        let models: Models = resp
            .json()
            .await
            .map_err(|e| EngineError::Protocol(format!("models: {e}")))?;
        Ok(models.data.into_iter().map(|m| m.id).collect())
    }

    async fn generate(&self, spec: GenerateSpec) -> Result<ChunkStream, EngineError> {
        let mut body = serde_json::json!({
            "model": self.model,
            "messages": spec.messages.iter().map(|m| serde_json::json!({"role": m.role, "content": m.content})).collect::<Vec<_>>(),
            "stream": true,
            "stream_options": { "include_usage": true },
        });
        if let Some(n) = spec.max_tokens {
            body["max_tokens"] = n.into();
        }
        if let Some(t) = spec.temperature {
            body["temperature"] = t.into();
        }
        let resp = send(
            self.http
                .post(join(&self.url, "v1/chat/completions"))
                .json(&body),
        )
        .await?;
        Ok(parse_stream(lines(resp), Parser::default()))
    }

    async fn embed(&self, inputs: Vec<String>) -> Result<Vec<Vec<f32>>, EngineError> {
        let body = serde_json::json!({ "model": self.embed_model, "input": inputs });
        let resp = send(self.http.post(join(&self.url, "v1/embeddings")).json(&body)).await?;
        let mut out: Embeddings = resp
            .json()
            .await
            .map_err(|e| EngineError::Protocol(format!("embeddings: {e}")))?;
        out.data.sort_by_key(|r| r.index);
        Ok(out.data.into_iter().map(|r| r.embedding).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn texts(items: &[Result<Chunk, EngineError>]) -> Vec<String> {
        items
            .iter()
            .map(|i| i.as_ref().map(|c| c.text.clone()).unwrap_or_default())
            .collect()
    }

    #[test]
    fn deltas_then_a_usage_chunk_then_done() {
        let mut p = Parser::default();
        assert_eq!(
            texts(&p.feed(
                r#"data: {"model":"m","choices":[{"delta":{"content":"Hi"},"finish_reason":null}]}"#
            )),
            ["Hi"]
        );
        assert!(p.feed(": keepalive").is_empty());
        assert!(
            p.feed(r#"data: {"choices":[{"delta":{},"finish_reason":"stop"}]}"#)
                .is_empty(),
            "finish without counts waits for the usage chunk"
        );
        let u = p.feed(r#"data: {"choices":[],"usage":{"prompt_tokens":7,"completion_tokens":3}}"#);
        assert_eq!(u.len(), 1);
        let done = u[0].as_ref().cloned().unwrap_or_else(|e| panic!("{e}"));
        assert!(done.done);
        assert_eq!(
            done.usage,
            Some(Usage {
                prompt_tokens: 7,
                completion_tokens: 3
            })
        );
        assert!(p.feed("data: [DONE]").is_empty(), "done is sent once");
    }

    #[test]
    fn timings_on_the_finishing_chunk_count_and_done_ends_without_usage() {
        let mut p = Parser::default();
        let out = p.feed(r#"data: {"choices":[{"delta":{"content":"x"},"finish_reason":"stop"}],"timings":{"prompt_n":5,"predicted_n":2}}"#);
        assert_eq!(out.len(), 2);
        let done = out[1].as_ref().cloned().unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(
            done.usage,
            Some(Usage {
                prompt_tokens: 5,
                completion_tokens: 2
            })
        );

        let mut q = Parser::default();
        q.feed(r#"data: {"choices":[{"delta":{},"finish_reason":"stop"}]}"#);
        let end = q.feed("data: [DONE]");
        assert_eq!(end.len(), 1);
        let done = end[0].as_ref().cloned().unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(
            done.usage,
            Some(Usage::default()),
            "no counts: zeros, never a guess"
        );
    }

    #[test]
    fn an_error_event_is_a_refusal_and_an_early_end_is_an_outage() {
        let mut p = Parser::default();
        assert!(matches!(
            p.feed(r#"data: {"error":{"message":"no slot"}}"#).remove(0),
            Err(EngineError::Refused(m)) if m == "no slot"
        ));
        assert!(matches!(
            Parser::default().finish().remove(0),
            Err(EngineError::Unavailable(_))
        ));
    }
}
