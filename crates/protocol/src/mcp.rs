//! The platform over MCP: every public RPC as a tool, at `/mcp`.
//!
//! A fourth rendering of the same registry REST, SSE and the multiplexed
//! socket are built from (`Transcoder::rpcs`), so the tools and the other
//! surfaces cannot drift apart. A tool is named `<backend>_<method>` in snake
//! case, described by the RPC's comment in the contract, and takes the request
//! message as its arguments, described by `transcode::schema`. A call is the
//! RPC, made as the caller: the identity Envoy verified travels to the backend
//! exactly as it does from a page. A streaming RPC is collected into one
//! answer under `[mcp]`'s caps; a stream of generation chunks is joined into
//! its answer text, with the model's reasoning kept apart.
//!
//! Streamable HTTP, stateless: either of the gateway's replicas answers any
//! request, and no session outlives it. The `Host` check rmcp offers against
//! DNS rebinding is off on purpose: it protects a local server reached with a
//! browser's ambient credentials, and this one is reached only with a bearer
//! token the gateway has verified, which a rebinding page does not have.

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
use tbd_common::{metrics::RequestTimer, principal::Principal};
use tonic::metadata::{Ascii, MetadataValue};

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
    /// The registry as tools: every RPC the gateway serves, in the registry's order.
    #[must_use]
    pub fn new(state: AppState, transcoder: &Transcoder) -> Self {
        let mut tools = Vec::new();
        let mut by_name = HashMap::new();
        for rpc in transcoder.rpcs().values() {
            let name = tool_name(rpc);
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
fn tool_name(rpc: &Rpc) -> String {
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
                "The platform's public RPCs as tools, named <backend>_<method>. Each call runs as the caller the gateway verified, under the caller's rights and budget. llm_generate answers a conversation; llm_list_models says which tiers are up; llm_get_budget says what is left today.",
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
        tracing::debug!(tool = %request.name, rpc = %name, caller = principal.kind_slug(), "mcp call");
        let mut timer = RequestTimer::start(TRANSPORT, name);
        let outcome = self.call(entry, payload, request.arguments).await;
        let result = match outcome {
            Ok(value) => {
                timer.set_status("ok");
                text_result(&value, false)
            }
            Err(problem) => {
                timer.set_status(problem.code.slug());
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
