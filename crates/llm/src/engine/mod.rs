//! The engines: what runs the weights (the L1), behind one trait.
//!
//! The service above this module knows tiers, budgets and records; it never
//! knows an endpoint or a wire format. An engine knows exactly one wire
//! format and nothing about callers. [`build`] is the only place a
//! [`EngineKind`] is matched on, and every engine, the stub included, passes
//! the same conformance suite (`tests/it/conformance.rs`).
//!
//! Two error placements matter to callers and are part of the contract:
//! an `Err` from [`Engine::generate`] happens *before* the first chunk and
//! becomes the RPC's status; an `Err` item *on* the stream happens after
//! chunks were already sent and becomes an error item on the stream. Either
//! way the stream is over.

pub mod llamacpp;
pub mod ollama;
pub mod stub;

use std::{fmt, sync::Arc, time::Duration};

use async_trait::async_trait;
use futures::{Stream, StreamExt as _, TryStreamExt as _, stream::BoxStream};
use tokio_util::{
    codec::{FramedRead, LinesCodec},
    io::StreamReader,
};

use crate::config::{EngineConfig, EngineKind};

/// Longest line an HTTP engine may send; longer is a protocol error, not a
/// memory problem.
const MAX_LINE: usize = 1024 * 1024;

/// How long dialling an engine may take. The request itself is bounded by the
/// service's deadline, never by the client: a generation legitimately runs
/// for minutes.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

/// One turn of the conversation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    /// `system`, `user` or `assistant`.
    pub role: String,
    /// The text.
    pub content: String,
}

/// What to generate.
#[derive(Debug, Clone)]
pub struct GenerateSpec {
    /// The conversation so far.
    pub messages: Vec<Message>,
    /// Longest completion in tokens.
    pub max_tokens: Option<u32>,
    /// Sampling temperature.
    pub temperature: Option<f32>,
}

/// Tokens spent, as the engine reported them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Usage {
    /// Tokens in the prompt.
    pub prompt_tokens: u32,
    /// Tokens generated.
    pub completion_tokens: u32,
}

/// One piece of a generation.
#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    /// The text; may be empty on the final chunk.
    pub text: String,
    /// True on the last chunk.
    pub done: bool,
    /// Set on the last chunk: what the engine reported, or zeros when it
    /// reported nothing. Never invented.
    pub usage: Option<Usage>,
    /// The model the engine says it used, when it says.
    pub model: Option<String>,
}

impl Chunk {
    fn text(text: impl Into<String>, model: Option<String>) -> Self {
        Self {
            text: text.into(),
            done: false,
            usage: None,
            model,
        }
    }

    fn done(usage: Usage, model: Option<String>) -> Self {
        Self {
            text: String::new(),
            done: true,
            usage: Some(usage),
            model,
        }
    }
}

/// Why an engine did not answer.
#[derive(Debug, Clone, thiserror::Error)]
pub enum EngineError {
    /// The engine cannot be reached, answered 5xx, or went away mid-stream.
    #[error("engine unavailable: {0}")]
    Unavailable(String),
    /// The engine rejected the request (an unknown model, a feature it was
    /// not started with).
    #[error("engine refused: {0}")]
    Refused(String),
    /// The engine sent something this module does not understand.
    #[error("engine protocol: {0}")]
    Protocol(String),
    /// The service's deadline passed.
    #[error("engine timed out")]
    Timeout,
}

/// A stream of chunks; ends after the `done` chunk or the first error.
pub type ChunkStream = BoxStream<'static, Result<Chunk, EngineError>>;

/// What runs the weights.
#[async_trait]
pub trait Engine: Send + Sync + fmt::Debug + 'static {
    /// The kind, as named on the wire and in metrics.
    fn kind(&self) -> EngineKind;
    /// The model this engine is configured to serve.
    fn model(&self) -> &str;
    /// The model this engine embeds with.
    fn embed_model(&self) -> &str;
    /// True only for the test stub.
    fn stub(&self) -> bool {
        false
    }
    /// The models the engine serves; `Ok` means it is up.
    async fn health(&self) -> Result<Vec<String>, EngineError>;
    /// Start a generation. `Err` here is "before the first chunk".
    async fn generate(&self, spec: GenerateSpec) -> Result<ChunkStream, EngineError>;
    /// One vector per input, in order.
    async fn embed(&self, inputs: Vec<String>) -> Result<Vec<Vec<f32>>, EngineError>;
}

/// An engine could not be built from its configuration.
#[derive(Debug, thiserror::Error)]
#[error("engine {kind}: {reason}")]
pub struct BuildError {
    /// The kind.
    pub kind: &'static str,
    /// Why.
    pub reason: String,
}

/// The one match on the kind.
///
/// # Errors
/// The URL does not parse, or the HTTP client cannot be built.
pub fn build(cfg: &EngineConfig) -> Result<Arc<dyn Engine>, BuildError> {
    let kind = cfg.kind.as_str();
    let fail = |reason: String| BuildError { kind, reason };
    Ok(match cfg.kind {
        EngineKind::Stub => Arc::new(stub::Stub::new(&cfg.model)),
        EngineKind::Ollama => Arc::new(ollama::Ollama::new(cfg).map_err(fail)?),
        EngineKind::Llamacpp => Arc::new(llamacpp::Llamacpp::new(cfg).map_err(fail)?),
    })
}

/// The HTTP client every network engine shares the shape of.
pub(crate) fn http_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .connect_timeout(CONNECT_TIMEOUT)
        .user_agent(format!("tbd-llm/{}", tbd_common::VERSION))
        .build()
        .map_err(|e| e.to_string())
}

/// A base URL joined with a path, tolerant of a trailing slash.
pub(crate) fn join(base: &str, path: &str) -> String {
    format!(
        "{}/{}",
        base.trim_end_matches('/'),
        path.trim_start_matches('/')
    )
}

/// Send a request; a transport failure is the engine being unavailable and a
/// non-2xx answer is a refusal (4xx) or an outage (5xx), with whatever the
/// body said.
pub(crate) async fn send(req: reqwest::RequestBuilder) -> Result<reqwest::Response, EngineError> {
    let resp = req
        .send()
        .await
        .map_err(|e| EngineError::Unavailable(e.without_url().to_string()))?;
    let status = resp.status();
    if status.is_success() {
        return Ok(resp);
    }
    let body = resp.text().await.unwrap_or_default();
    let message = format!("{status}: {}", error_message(&body));
    if status.is_server_error() {
        Err(EngineError::Unavailable(message))
    } else {
        Err(EngineError::Refused(message))
    }
}

/// The message inside an engine's JSON error body, or the body itself.
pub(crate) fn error_message(body: &str) -> String {
    let trimmed = body.trim();
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(trimmed) {
        let m = v
            .get("error")
            .and_then(|e| e.get("message").or(Some(e)))
            .and_then(serde_json::Value::as_str);
        if let Some(m) = m {
            return m.to_owned();
        }
    }
    if trimmed.is_empty() {
        "no body".to_owned()
    } else {
        trimmed.chars().take(200).collect()
    }
}

/// The body as lines. An I/O error mid-body is the engine going away.
pub(crate) fn lines(
    resp: reqwest::Response,
) -> impl Stream<Item = Result<String, EngineError>> + Send + 'static {
    let reader = StreamReader::new(resp.bytes_stream().map_err(std::io::Error::other));
    FramedRead::new(reader, LinesCodec::new_with_max_length(MAX_LINE)).map_err(|e| match e {
        tokio_util::codec::LinesCodecError::MaxLineLengthExceeded => {
            EngineError::Protocol("a line longer than the cap".to_owned())
        }
        tokio_util::codec::LinesCodecError::Io(e) => {
            EngineError::Unavailable(format!("reading the response: {e}"))
        }
    })
}

/// Turns an engine's lines into chunks. Pure, so each engine's parser is
/// unit-tested on fixture lines without a socket.
pub(crate) trait LineParser: Send + 'static {
    /// The chunks one line yields; several, one or none.
    fn feed(&mut self, line: &str) -> Vec<Result<Chunk, EngineError>>;
    /// The body ended without the engine saying it was done.
    fn finish(&mut self) -> Vec<Result<Chunk, EngineError>>;
}

/// Drive a [`LineParser`] over a line stream, enforcing the contract: the
/// stream ends right after the `done` chunk or the first error, whatever else
/// the engine still had to say.
pub(crate) fn parse_stream<P: LineParser>(
    lines: impl Stream<Item = Result<String, EngineError>> + Send + 'static,
    parser: P,
) -> ChunkStream {
    struct State<P, L> {
        lines: L,
        parser: P,
        pending: std::collections::VecDeque<Result<Chunk, EngineError>>,
        over: bool,
        ended: bool,
    }
    let state = State {
        lines: Box::pin(lines),
        parser,
        pending: std::collections::VecDeque::new(),
        over: false,
        ended: false,
    };
    futures::stream::unfold(state, |mut st| async move {
        loop {
            if st.over {
                return None;
            }
            if let Some(item) = st.pending.pop_front() {
                st.over = item.is_err() || matches!(&item, Ok(c) if c.done);
                return Some((item, st));
            }
            if st.ended {
                return None;
            }
            match st.lines.next().await {
                Some(Ok(line)) => st.pending.extend(st.parser.feed(&line)),
                Some(Err(e)) => {
                    st.pending.push_back(Err(e));
                }
                None => {
                    st.ended = true;
                    st.pending.extend(st.parser.finish());
                }
            }
        }
    })
    .boxed()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_messages_come_from_the_body_when_it_is_json() {
        assert_eq!(
            error_message(r#"{"error":"model not found"}"#),
            "model not found"
        );
        assert_eq!(
            error_message(r#"{"error":{"message":"no embeddings"}}"#),
            "no embeddings"
        );
        assert_eq!(error_message("plain text"), "plain text");
        assert_eq!(error_message("  "), "no body");
    }

    #[test]
    fn join_tolerates_slashes() {
        assert_eq!(join("http://h:1/", "/api/chat"), "http://h:1/api/chat");
        assert_eq!(join("http://h:1", "api/chat"), "http://h:1/api/chat");
    }
}
