//! The platform over MCP: allowlisted public RPCs as tools, at `/mcp`.
//!
//! A fourth rendering of the same registry REST, SSE and the multiplexed
//! socket are built from (`Transcoder::rpcs`), so the tools and the other
//! surfaces cannot drift apart. Default deny: only the RPCs `[mcp] tools`
//! names are tools, so a new RPC never reaches an agent by accident, and every
//! call is one audit line (who, which tool, the outcome; never the content). A tool is named `<backend>_<method>` in snake
//! case, described by the RPC's comment in the contract, and takes the request
//! message as its arguments, described by `transcode::schema`. A call is the
//! RPC, made as the caller: the identity Envoy verified travels to the backend
//! exactly as it does from a page. A streaming RPC is collected into one
//! answer under `[mcp]`'s caps; a stream of generation chunks is joined into
//! its answer text, with the model's reasoning kept apart.
//!
//! Streamable HTTP, stateless: either of the gateway's replicas answers any
//! request, and no session outlives it. Two callers reach it: an agent with a
//! bearer token on the API host, and the lab's workbench on the site's host
//! with its signed-in admin's cookie. A cookie travels with a cross-site
//! request too, so [`origin_guard`] refuses any request whose `Origin` is not
//! in `[mcp] allowed_origins` before a tool is touched; agents send no
//! `Origin`. rmcp's own `Host` check is off: behind the gateway the `Host` is
//! whatever Envoy routed on, and the origin check is the one that matters.

use std::{borrow::Cow, collections::HashMap, sync::Arc, time::Duration};

use futures::StreamExt as _;
use prost_reflect::DynamicMessage;
use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler,
    model::{
        CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock, Implementation,
        JsonObject, ListToolsResult, PaginatedRequestParams, ServerCapabilities, ServerConfig,
        Tool,
    },
    service::RequestContext,
    transport::streamable_http_server::{
        StreamableHttpServerConfig, StreamableHttpService, session::never::NeverSessionManager,
    },
};
use serde_json::{Map, Value, json};
use tbd_common::{metrics::RequestTimer, principal::Principal, telemetry::propagation};
use tonic::metadata::{Ascii, MetadataValue};
use tracing::Instrument as _;

use crate::{
    AppState,
    config::Mcp,
    error::{Code, Problem},
    invoke::{Invoked, invoke, request},
    transcode::{Rpc, Transcoder, call::Out, schema},
};

/// The path, on the API host. Hand-written, so `http::reserved_paths` carries it.
pub(crate) const PATH: &str = "/mcp";

/// The transport label of a call made here (`route` is the RPC name).
const TRANSPORT: &str = "mcp";

/// One tool: its description for `tools/list` and the RPC it calls.
#[derive(Clone)]
struct Entry {
    tool: Tool,
    rpc: Arc<Rpc>,
}

/// The MCP server: the registry as tools, and the state to call them with.
#[derive(Clone)]
pub struct Server {
    state: AppState,
    tools: Arc<Vec<Tool>>,
    by_name: Arc<HashMap<String, Entry>>,
    limits: Mcp,
}

impl Server {
    /// The allowlisted part of the registry as tools, in the registry's order.
    /// A name in `[mcp] tools` that no RPC answers to is logged and ignored.
    #[must_use]
    pub fn new(state: AppState, transcoder: &Transcoder) -> Self {
        let allowed = &state.mcp().tools;
        let mut tools = Vec::new();
        let mut by_name = HashMap::new();
        for rpc in transcoder.rpcs().values() {
            let name = tool_name(rpc);
            if !allowed.contains(&name) {
                continue;
            }
            let tool = describe(&name, rpc);
            by_name.insert(
                name,
                Entry {
                    tool: tool.clone(),
                    rpc: Arc::clone(rpc),
                },
            );
            tools.push(tool);
        }
        for name in allowed.iter().filter(|n| !by_name.contains_key(*n)) {
            tracing::warn!(tool = %name, "mcp: allowlisted tool has no RPC in this registry");
        }
        let limits = state.mcp().clone();
        Self {
            state,
            tools: Arc::new(tools),
            by_name: Arc::new(by_name),
            limits,
        }
    }

    /// The tool names, in the order `tools/list` gives them.
    #[must_use]
    pub fn tool_names(&self) -> Vec<String> {
        self.tools.iter().map(|t| t.name.to_string()).collect()
    }

    async fn call(
        &self,
        entry: &Entry,
        payload: Option<MetadataValue<Ascii>>,
        args: Option<JsonObject>,
    ) -> Result<Value, Problem> {
        let message = request(&entry.rpc, args.map(Value::Object))?;
        match invoke(&entry.rpc, &self.state, payload, message).await? {
            Invoked::Unary(response) => to_value(&response),
            Invoked::Stream(stream) => self.collect(stream).await,
        }
    }

    /// A stream as one answer: every message up to the cap and the deadline,
    /// and, when the messages carry text, that text joined (answer and
    /// reasoning apart, for a generation).
    async fn collect(
        &self,
        mut stream: tonic::Streaming<DynamicMessage>,
    ) -> Result<Value, Problem> {
        let deadline = tokio::time::sleep(self.limits.stream_timeout);
        tokio::pin!(deadline);
        let mut items: Vec<Value> = Vec::new();
        let mut cut: Option<&'static str> = None;
        loop {
            tokio::select! {
                () = &mut deadline => { cut = Some("stream_timeout"); break; }
                next = stream.next() => match next {
                    None => break,
                    Some(item) => {
                        items.push(to_value(&item?)?);
                        if items.len() >= self.limits.max_stream_items {
                            cut = Some("max_stream_items");
                            break;
                        }
                    }
                },
            }
        }
        Ok(summarise(items, cut, self.limits.stream_timeout))
    }
}

/// `<backend>_<method>` in snake case: `llm_generate`, `finance_list_transactions`.
pub(crate) fn tool_name(rpc: &Rpc) -> String {
    let mut out = format!("{}_", rpc.backend);
    for (i, ch) in rpc.method.name().chars().enumerate() {
        if ch.is_ascii_uppercase() {
            if i > 0 {
                out.push('_');
            }
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
}

fn describe(name: &str, rpc: &Rpc) -> Tool {
    let comment = schema::method_comment(&rpc.method);
    let mut description = if comment.is_empty() {
        format!("Calls {}.", rpc.name())
    } else {
        comment
    };
    if rpc.streaming {
        description.push_str(
            " Streams on the wire; this tool collects the stream into one answer (text joined, reasoning apart), up to the server's cap.",
        );
    }
    let input = match schema::message(&rpc.method.input()) {
        Value::Object(o) => o,
        _ => Map::new(),
    };
    Tool::new(
        Cow::Owned(name.to_owned()),
        Cow::Owned(description),
        Arc::new(input),
    )
}

fn to_value(message: &DynamicMessage) -> Result<Value, Problem> {
    serde_json::to_value(Out(message))
        .map_err(|error| Problem::new(Code::Internal, format!("serialize: {error}")))
}

/// What a collected stream answers. Messages with a `text` field (a
/// generation's chunks) are joined into `text`, and those marked `reasoning`
/// into `reasoning`; the last message is kept whole (it carries a
/// generation's usage). Any other stream is returned as its messages.
fn summarise(items: Vec<Value>, cut: Option<&str>, timeout: Duration) -> Value {
    let texty = !items.is_empty()
        && items
            .iter()
            .all(|i| i.get("text").is_some_and(Value::is_string));
    let mut out = Map::new();
    out.insert("messages".into(), json!(items.len()));
    if let Some(reason) = cut {
        out.insert(
            "cut".into(),
            json!(match reason {
                "stream_timeout" =>
                    format!("stopped after {}s; this is what arrived", timeout.as_secs()),
                _ => "stopped at the message cap; this is what arrived".to_owned(),
            }),
        );
    }
    if texty {
        let mut text = String::new();
        let mut reasoning = String::new();
        for item in &items {
            let t = item.get("text").and_then(Value::as_str).unwrap_or_default();
            if item
                .get("reasoning")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                reasoning.push_str(t);
            } else {
                text.push_str(t);
            }
        }
        out.insert("text".into(), json!(text));
        if !reasoning.is_empty() {
            out.insert("reasoning".into(), json!(reasoning));
        }
        if let Some(last) = items.last() {
            out.insert("last".into(), last.clone());
        }
    } else {
        out.insert("items".into(), Value::Array(items));
    }
    Value::Object(out)
}

fn text_result(value: &Value, error: bool) -> CallToolResult {
    let body = serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string());
    let content = vec![ContentBlock::text(body)];
    if error {
        CallToolResult::error(content)
    } else {
        CallToolResult::success(content)
    }
}

impl ServerHandler for Server {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("tbd", tbd_common::VERSION))
            .with_instructions(
                "A chosen set of the platform's public RPCs as tools, named <backend>_<method>. Each call runs as the caller the gateway verified, under the caller's rights and budget. llm_generate answers a conversation; llm_list_models says which tiers are up; llm_get_budget says what is left today.",
            )
    }

    // The trait's signature is async; the list is built once at start.
    #[allow(clippy::unused_async_trait_impl)]
    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        Ok(ListToolsResult::with_all_items((*self.tools).clone()))
    }

    fn get_tool(&self, name: &str) -> Option<Tool> {
        self.by_name.get(name).map(|e| e.tool.clone())
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, McpError> {
        let Some(entry) = self.by_name.get(request.name.as_ref()) else {
            return Err(McpError::invalid_params(
                format!("no tool named {}", request.name),
                None,
            ));
        };
        let parts = context.extensions.get::<http::request::Parts>();
        let principal = parts.and_then(|p| p.extensions.get::<Principal>());
        let Some(principal) = principal else {
            return Err(McpError::invalid_request(
                "unauthenticated: this gateway serves verified callers only",
                None,
            ));
        };
        let payload = parts
            .and_then(|p| p.headers.get(crate::principal::PAYLOAD_HEADER))
            .and_then(|v| MetadataValue::try_from(v.as_bytes()).ok());
        let name = entry.rpc.name();
        // rmcp runs the handler in its own task, outside the HTTP request's
        // span, so the call gets its own, parented to the same `traceparent`:
        // the backend call and the audit line join the request's trace.
        let span = tracing::info_span!(
            "mcp.call",
            mcp.tool = %request.name,
            rpc = %name,
            trace_id = tracing::field::Empty,
            enduser.id = %principal.sub,
            enduser.kind = principal.kind_slug(),
        );
        if let Some(headers) = parts.map(|p| &p.headers)
            && let Some(id) = propagation::adopt_parent(&span, &propagation::Headers(headers))
        {
            span.record("trace_id", id);
        }
        let started = std::time::Instant::now();
        let mut timer = RequestTimer::start(TRANSPORT, name.clone());
        let outcome = self
            .call(entry, payload, request.arguments)
            .instrument(span.clone())
            .await;
        let _entered = span.enter();
        let status = match &outcome {
            Ok(_) => "ok",
            Err(problem) => problem.code.slug(),
        };
        // The audit trail: who called which tool and how it ended. Never the
        // arguments or the answer, which are the caller's content.
        tracing::info!(
            target: "audit",
            surface = TRANSPORT,
            tool = %request.name,
            rpc = %name,
            subject = %principal.sub,
            caller = principal.kind_slug(),
            status,
            elapsed_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
            "mcp tool call"
        );
        timer.set_status(status);
        let result = match outcome {
            Ok(value) => text_result(&value, false),
            Err(problem) => {
                problem.log();
                let wire = problem.wire();
                text_result(
                    &json!({ "code": wire.code, "error": wire.error, "details": wire.details }),
                    true,
                )
            }
        };
        Ok(result.into())
    }
}

/// Refuse a browser request from a page not in `[mcp] allowed_origins`: 403
/// with the envelope, before rmcp reads the body. A request without an
/// `Origin` (an agent, a script) passes; the gateway's gate still applies.
pub async fn origin_guard(
    axum::extract::State(allowed): axum::extract::State<Arc<Vec<String>>>,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    use axum::response::IntoResponse as _;
    if let Some(origin) = request.headers().get(http::header::ORIGIN) {
        let origin = origin.to_str().unwrap_or_default();
        if !allowed.iter().any(|a| a == origin) {
            tracing::info!(target: "audit", surface = TRANSPORT, origin, "mcp refused a foreign origin");
            return Problem::new(Code::Forbidden, format!("origin {origin} may not call MCP"))
                .into_response();
        }
    }
    next.run(request).await
}

/// The `/mcp` service: streamable HTTP, stateless, one [`Server`] shared by
/// every request.
pub fn service(
    state: &AppState,
    transcoder: &Transcoder,
) -> StreamableHttpService<Server, NeverSessionManager> {
    let server = Server::new(state.clone(), transcoder);
    tracing::info!(tools = server.tools.len(), "mcp tools");
    let config = StreamableHttpServerConfig::default()
        .with_legacy_session_mode(false)
        .with_json_response(true)
        .with_max_request_body_bytes(state.mcp().max_body_bytes)
        .disable_allowed_hosts();
    StreamableHttpService::new(
        move || Ok(server.clone()),
        Arc::new(NeverSessionManager::default()),
        config,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every tool `base.toml` allows is an RPC of the full registry, and the
    /// code's default equals the file, so a typo cannot silently shrink the
    /// set and a rename cannot silently drop a tool.
    #[test]
    fn the_shipped_allowlist_names_real_rpcs_and_equals_the_default() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../configs/protocol");
        let (config, _) = crate::config::Config::load(&dir, "production").unwrap();
        let pool = crate::transcode::pool().unwrap();
        let transcoder = Transcoder::from_config(&pool, &config).unwrap();
        let names: Vec<String> = transcoder.rpcs().values().map(|r| tool_name(r)).collect();
        for tool in &config.mcp.tools {
            assert!(names.contains(tool), "{tool} is not an RPC; have {names:?}");
        }
        assert_eq!(config.mcp.tools, Mcp::default().tools);
    }

    #[test]
    fn a_stream_of_chunks_is_joined_with_reasoning_apart() {
        let items = vec![
            json!({"text": "thinking", "reasoning": true, "index": 0}),
            json!({"text": "Hello", "reasoning": false, "index": 1}),
            json!({"text": " there", "reasoning": false, "index": 2}),
            json!({"text": "", "done": true, "usage": {"prompt_tokens": 7}, "index": 3}),
        ];
        let out = summarise(items, None, Duration::from_secs(1));
        assert_eq!(out["text"], "Hello there");
        assert_eq!(out["reasoning"], "thinking");
        assert_eq!(out["messages"], 4);
        assert_eq!(out["last"]["done"], true);
        assert!(out.get("cut").is_none());
    }

    #[test]
    fn a_stream_without_text_is_its_messages_and_a_cut_says_so() {
        let items = vec![json!({"seq": 1}), json!({"seq": 2})];
        let out = summarise(items, Some("max_stream_items"), Duration::from_secs(1));
        assert_eq!(out["items"].as_array().map(Vec::len), Some(2));
        assert!(out["cut"].as_str().is_some_and(|c| c.contains("cap")));
    }
}
