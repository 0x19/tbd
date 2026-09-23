//! Ollama over its HTTP API: `POST /api/chat` streamed as newline-delimited
//! JSON, `GET /api/tags` for health, `POST /api/embed` for vectors.
//!
//! Wire shapes (Ollama's API reference): a chat line is
//! `{"model":..,"message":{"role":"assistant","content":".."},"done":false}`;
//! the last line has `"done":true` with `prompt_eval_count` and `eval_count`;
//! an error line is `{"error":".."}`.

use async_trait::async_trait;
use serde::Deserialize;

use super::{
    Chunk, ChunkStream, Engine, EngineError, GenerateSpec, LineParser, Usage, http_client, join,
    lines, parse_stream, send,
};
use crate::config::{EngineConfig, EngineKind};

/// The Ollama engine.
#[derive(Debug, Clone)]
pub struct Ollama {
    http: reqwest::Client,
    url: String,
    model: String,
    embed_model: String,
}

impl Ollama {
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
struct ChatLine {
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    message: Option<ChatMessage>,
    #[serde(default)]
    done: bool,
    #[serde(default)]
    prompt_eval_count: Option<u32>,
    #[serde(default)]
    eval_count: Option<u32>,
    #[serde(default)]
    error: Option<String>,
}

#[derive(Deserialize)]
struct ChatMessage {
    #[serde(default)]
    content: String,
}

/// Parses `/api/chat` lines.
#[derive(Debug, Default)]
pub(crate) struct Parser {
    model: Option<String>,
}

impl LineParser for Parser {
    fn feed(&mut self, line: &str) -> Vec<Result<Chunk, EngineError>> {
        let line = line.trim();
        if line.is_empty() {
            return vec![];
        }
        let parsed: ChatLine = match serde_json::from_str(line) {
            Ok(p) => p,
            Err(e) => return vec![Err(EngineError::Protocol(format!("chat line: {e}")))],
        };
        if let Some(error) = parsed.error {
            return vec![Err(EngineError::Refused(error))];
        }
        if parsed.model.is_some() {
            self.model = parsed.model;
        }
        let mut out = Vec::new();
        let text = parsed.message.map(|m| m.content).unwrap_or_default();
        if !text.is_empty() {
            out.push(Ok(Chunk::text(text, self.model.clone())));
        }
        if parsed.done {
            let usage = Usage {
                prompt_tokens: parsed.prompt_eval_count.unwrap_or(0),
                completion_tokens: parsed.eval_count.unwrap_or(0),
            };
            out.push(Ok(Chunk::done(usage, self.model.clone())));
        }
        out
    }

    fn finish(&mut self) -> Vec<Result<Chunk, EngineError>> {
        vec![Err(EngineError::Unavailable(
            "the response ended before the engine said it was done".to_owned(),
        ))]
    }
}

#[derive(Deserialize)]
struct Tags {
    #[serde(default)]
    models: Vec<Tag>,
}

#[derive(Deserialize)]
struct Tag {
    name: String,
}

#[derive(Deserialize)]
struct Embed {
    #[serde(default)]
    embeddings: Vec<Vec<f32>>,
}

#[async_trait]
impl Engine for Ollama {
    fn kind(&self) -> EngineKind {
        EngineKind::Ollama
    }

    fn model(&self) -> &str {
        &self.model
    }

    fn embed_model(&self) -> &str {
        &self.embed_model
    }

    async fn health(&self) -> Result<Vec<String>, EngineError> {
        let resp = send(self.http.get(join(&self.url, "api/tags"))).await?;
        let tags: Tags = resp
            .json()
            .await
            .map_err(|e| EngineError::Protocol(format!("tags: {e}")))?;
        Ok(tags.models.into_iter().map(|t| t.name).collect())
    }

    async fn generate(&self, spec: GenerateSpec) -> Result<ChunkStream, EngineError> {
        let mut options = serde_json::Map::new();
        if let Some(n) = spec.max_tokens {
            options.insert("num_predict".into(), n.into());
        }
        if let Some(t) = spec.temperature {
            options.insert("temperature".into(), t.into());
        }
        let body = serde_json::json!({
            "model": self.model,
            "messages": spec.messages.iter().map(|m| serde_json::json!({"role": m.role, "content": m.content})).collect::<Vec<_>>(),
            "stream": true,
            "options": options,
        });
        let resp = send(self.http.post(join(&self.url, "api/chat")).json(&body)).await?;
        Ok(parse_stream(lines(resp), Parser::default()))
    }

    async fn embed(&self, inputs: Vec<String>) -> Result<Vec<Vec<f32>>, EngineError> {
        let body = serde_json::json!({ "model": self.embed_model, "input": inputs });
        let resp = send(self.http.post(join(&self.url, "api/embed")).json(&body)).await?;
        let out: Embed = resp
            .json()
            .await
            .map_err(|e| EngineError::Protocol(format!("embed: {e}")))?;
        Ok(out.embeddings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_lines_are_text_and_the_done_line_carries_usage() {
        let mut p = Parser::default();
        let a =
            p.feed(r#"{"model":"m","message":{"role":"assistant","content":"Hi"},"done":false}"#);
        assert_eq!(a.len(), 1);
        assert_eq!(a[0].as_ref().map(|c| c.text.as_str()).ok(), Some("Hi"));
        let b = p.feed(r#"{"model":"m","message":{"role":"assistant","content":""},"done":true,"prompt_eval_count":7,"eval_count":3}"#);
        assert_eq!(b.len(), 1);
        let done = b[0].as_ref().cloned().unwrap_or_else(|e| panic!("{e}"));
        assert!(done.done);
        assert_eq!(
            done.usage,
            Some(Usage {
                prompt_tokens: 7,
                completion_tokens: 3
            })
        );
        assert_eq!(done.model.as_deref(), Some("m"));
    }

    #[test]
    fn an_error_line_is_a_refusal_and_garbage_is_a_protocol_error() {
        let mut p = Parser::default();
        assert!(matches!(
            p.feed(r#"{"error":"model not found"}"#).remove(0),
            Err(EngineError::Refused(m)) if m == "model not found"
        ));
        assert!(matches!(
            p.feed("not json").remove(0),
            Err(EngineError::Protocol(_))
        ));
        assert!(p.feed("   ").is_empty());
    }

    #[test]
    fn a_body_that_ends_early_is_the_engine_going_away() {
        assert!(matches!(
            Parser::default().finish().remove(0),
            Err(EngineError::Unavailable(_))
        ));
    }
}
