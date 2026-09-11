//! Descriptor-driven transcoding: an RPC annotated with `google.api.http` in
//! its `.proto` becomes a REST route (unary) or a server-sent events route
//! (server streaming), with proto3 JSON bodies, forwarded over the registered
//! backend's traced and measured transport. The contract is the proto file;
//! `docs/protocol/README.md` documents the rules.

mod bind;
pub(crate) mod call;
mod codec;
mod openapi;
pub mod template;

use std::{collections::BTreeMap, sync::Arc};

use axum::{
    Router,
    extract::{Path, RawQuery, Request, State},
    routing::{MethodFilter, on},
};
use http::{Method, uri::PathAndQuery};
use prost_reflect::{DescriptorPool, DynamicMessage, FieldDescriptor, Kind, MethodDescriptor};

use crate::{AppState, Config};

pub use template::{Template, TemplateError};

/// The full name of the method option that carries a route.
const HTTP_OPTION: &str = "google.api.http";

/// Why the transcoder could not be built. Every variant is a contract error
/// in a `.proto` or a build error, never a runtime condition.
#[derive(Debug, thiserror::Error)]
pub enum TranscodeError {
    /// The descriptor set does not decode.
    #[error("descriptor set: {0}")]
    Pool(#[from] prost_reflect::DescriptorError),
    /// `google/api/annotations.proto` is missing from the descriptor set.
    #[error(
        "the descriptor set lacks {HTTP_OPTION}: google/api/annotations.proto is not compiled in"
    )]
    MissingAnnotations,
    /// A template the gateway does not serve.
    #[error("{method}: template {template:?}: {reason}")]
    Template {
        /// `tbd.ledger.v1.LedgerService/Current`.
        method: String,
        /// The template as written.
        template: String,
        /// Why.
        reason: String,
    },
    /// `body` or `response_body` names no such field.
    #[error("{method}: {which} {field:?} is not a singular message field")]
    Field {
        /// The method.
        method: String,
        /// `body` or `response_body`.
        which: &'static str,
        /// The field named.
        field: String,
    },
    /// Client or bidirectional streaming cannot be a REST route.
    #[error("{method}: client-streaming RPCs cannot carry a google.api.http route")]
    ClientStreaming {
        /// The method.
        method: String,
    },
    /// A rule form the gateway does not serve.
    #[error("{method}: {what} is not supported")]
    Unsupported {
        /// The method.
        method: String,
        /// Which form.
        what: &'static str,
    },
    /// A server-streaming route must end in `/events` (Envoy gives those no timeout).
    #[error("{method}: a server-streaming route must end in /events, got {path}")]
    StreamingPath {
        /// The method.
        method: String,
        /// The template.
        path: String,
    },
    /// A unary route must not end in `/events`.
    #[error("{method}: only server-streaming routes may end in /events, got {path}")]
    UnaryEventsPath {
        /// The method.
        method: String,
        /// The template.
        path: String,
    },
    /// Two bindings, or a binding and a hand-written route, share a path and verb.
    #[error("{verb} {path} is bound twice: {method} and {other}")]
    Duplicate {
        /// The verb.
        verb: String,
        /// The normalised path.
        path: String,
        /// The later method.
        method: String,
        /// The earlier method or hand-written route.
        other: String,
    },
}

/// Where the request body goes.
#[derive(Debug, Clone)]
pub enum BodyRule {
    /// The route takes no body (GET, DELETE without `body`).
    None,
    /// `body: "*"`: the JSON body is the whole request message.
    Whole,
    /// `body: "field"`: the JSON body is that (message) field.
    Field(Vec<FieldDescriptor>),
}

/// One RPC bound to one HTTP verb and path.
#[derive(Debug, Clone)]
pub struct Binding {
    /// The registry backend the call goes to (`tbd.<backend>.v1`).
    pub backend: String,
    /// The RPC.
    pub method: MethodDescriptor,
    /// `/tbd.ledger.v1.LedgerService/Current`.
    pub grpc_path: PathAndQuery,
    /// The HTTP verb.
    pub verb: Method,
    /// The path template, parsed.
    pub template: Template,
    /// Where the body goes.
    pub body: BodyRule,
    /// `response_body`: answer with this field of the response instead of all of it.
    pub response_body: Option<FieldDescriptor>,
    /// Server streaming, served as SSE.
    pub streaming: bool,
}

impl Binding {
    /// `tbd.ledger.v1.LedgerService/Current`.
    #[must_use]
    pub fn rpc(&self) -> String {
        format!(
            "{}/{}",
            self.method.parent_service().full_name(),
            self.method.name()
        )
    }
}

/// Every binding the registered backends carry, in a stable order.
#[derive(Debug, Clone)]
pub struct Transcoder {
    bindings: Vec<Arc<Binding>>,
}

/// The descriptor pool of the whole contract (`tbd_proto::DESCRIPTOR_SET_ALL`).
///
/// # Errors
/// The set does not decode, which is a build error.
pub fn pool() -> Result<DescriptorPool, TranscodeError> {
    Ok(DescriptorPool::decode(tbd_proto::DESCRIPTOR_SET_ALL)?)
}

impl Transcoder {
    /// Read every `google.api.http` option of every service whose package
    /// names a registered backend (`tbd.<name>.v1` → `[services.<name>]`).
    /// Needs the registry's names only, so the `OpenAPI` document can be
    /// built without a running state.
    ///
    /// # Errors
    /// A template or rule the gateway does not serve, a duplicate route, or a
    /// descriptor set without the annotations.
    pub fn from_config(pool: &DescriptorPool, config: &Config) -> Result<Self, TranscodeError> {
        let ext = pool
            .get_extension_by_name(HTTP_OPTION)
            .ok_or(TranscodeError::MissingAnnotations)?;
        let mut bindings: Vec<Arc<Binding>> = Vec::new();
        let mut taken: BTreeMap<(String, String), String> = crate::http::reserved_paths()
            .iter()
            .flat_map(|(verb, path)| {
                [(
                    ((*verb).to_owned(), template::normalised(path)),
                    format!("the hand-written {verb} {path}"),
                )]
            })
            .collect();
        for service in pool.services() {
            let Some(backend) = backend_of(service.package_name()) else {
                continue;
            };
            let annotated: Vec<MethodDescriptor> = service
                .methods()
                .filter(|m| m.options().has_extension(&ext))
                .collect();
            if annotated.is_empty() {
                continue;
            }
            if !config.services.contains_key(backend) {
                tracing::warn!(
                    service = service.full_name(),
                    backend,
                    "annotated RPCs skipped: the backend is not in [services]"
                );
                continue;
            }
            for method in annotated {
                let rpc = format!("{}/{}", service.full_name(), method.name());
                if method.is_client_streaming() {
                    return Err(TranscodeError::ClientStreaming { method: rpc });
                }
                let options = method.options();
                let rule = options.get_extension(&ext);
                let Some(rule) = rule.as_message() else {
                    continue;
                };
                let mut rules = vec![rule.clone()];
                if let Some(more) = rule
                    .get_field_by_name("additional_bindings")
                    .and_then(|v| v.as_list().map(<[prost_reflect::Value]>::to_vec))
                {
                    rules.extend(more.iter().filter_map(|v| v.as_message().cloned()));
                }
                for rule in rules {
                    let binding = binding(backend, &method, &rpc, &rule)?;
                    let key = (
                        binding.verb.as_str().to_owned(),
                        template::normalised(&binding.template.axum),
                    );
                    if let Some(other) = taken.insert(key.clone(), rpc.clone()) {
                        return Err(TranscodeError::Duplicate {
                            verb: key.0,
                            path: key.1,
                            method: rpc,
                            other,
                        });
                    }
                    bindings.push(Arc::new(binding));
                }
            }
        }
        bindings.sort_by(|a, b| {
            (a.template.axum.as_str(), a.verb.as_str())
                .cmp(&(b.template.axum.as_str(), b.verb.as_str()))
        });
        Ok(Self { bindings })
    }

    /// The bindings, sorted by path then verb.
    #[must_use]
    pub fn bindings(&self) -> &[Arc<Binding>] {
        &self.bindings
    }

    /// One axum route per binding; the same path with several verbs merges.
    pub fn router(&self) -> Router<AppState> {
        let mut router = Router::new();
        for binding in &self.bindings {
            let filter = match binding.verb {
                Method::GET => MethodFilter::GET,
                Method::POST => MethodFilter::POST,
                Method::PUT => MethodFilter::PUT,
                Method::DELETE => MethodFilter::DELETE,
                Method::PATCH => MethodFilter::PATCH,
                _ => continue,
            };
            let captured = Arc::clone(binding);
            let handler = move |State(state): State<AppState>,
                                Path(vars): Path<std::collections::HashMap<String, String>>,
                                RawQuery(query): RawQuery,
                                request: Request| {
                let binding = Arc::clone(&captured);
                async move { call::handle(&binding, &state, vars, query, request).await }
            };
            router = router.route(&binding.template.axum, on(filter, handler));
        }
        router
    }

    /// The `OpenAPI` fragment for the bindings.
    #[must_use]
    pub fn openapi(&self) -> utoipa::openapi::OpenApi {
        openapi::document(&self.bindings)
    }
}

/// `tbd.ledger.v1` → `ledger`.
fn backend_of(package: &str) -> Option<&str> {
    let mut parts = package.split('.');
    match (parts.next(), parts.next(), parts.next(), parts.next()) {
        (Some("tbd"), Some(name), Some(version), None) if version.starts_with('v') => Some(name),
        _ => None,
    }
}

fn binding(
    backend: &str,
    method: &MethodDescriptor,
    rpc: &str,
    rule: &DynamicMessage,
) -> Result<Binding, TranscodeError> {
    let field = |name: &str| -> Option<String> {
        rule.has_field_by_name(name)
            .then(|| rule.get_field_by_name(name))
            .flatten()
            .and_then(|v| v.as_str().map(str::to_owned))
    };
    let (verb, path) = if let Some(p) = field("get") {
        (Method::GET, p)
    } else if let Some(p) = field("post") {
        (Method::POST, p)
    } else if let Some(p) = field("put") {
        (Method::PUT, p)
    } else if let Some(p) = field("delete") {
        (Method::DELETE, p)
    } else if let Some(p) = field("patch") {
        (Method::PATCH, p)
    } else {
        return Err(TranscodeError::Unsupported {
            method: rpc.to_owned(),
            what: "a custom HTTP verb",
        });
    };
    let input = method.input();
    let parsed = template::parse(&path, &input).map_err(|e| TranscodeError::Template {
        method: rpc.to_owned(),
        template: path.clone(),
        reason: e.0,
    })?;
    let body = match field("body").as_deref() {
        None | Some("") => BodyRule::None,
        Some("*") => BodyRule::Whole,
        Some(name) => {
            let path = template::field_path_message(&input, name).ok_or_else(|| {
                TranscodeError::Field {
                    method: rpc.to_owned(),
                    which: "body",
                    field: name.to_owned(),
                }
            })?;
            BodyRule::Field(path)
        }
    };
    if matches!(body, BodyRule::Whole) && matches!(verb, Method::GET | Method::DELETE) {
        return Err(TranscodeError::Unsupported {
            method: rpc.to_owned(),
            what: "a body on GET or DELETE",
        });
    }
    let response_body = match field("response_body").as_deref() {
        None | Some("") => None,
        Some(name) => Some(
            method
                .output()
                .get_field_by_name(name)
                .filter(|f| !f.is_list() && matches!(f.kind(), Kind::Message(_)))
                .ok_or_else(|| TranscodeError::Field {
                    method: rpc.to_owned(),
                    which: "response_body",
                    field: name.to_owned(),
                })?,
        ),
    };
    let streaming = method.is_server_streaming();
    let is_events = path.ends_with("/events");
    if streaming && !is_events {
        return Err(TranscodeError::StreamingPath {
            method: rpc.to_owned(),
            path,
        });
    }
    if !streaming && is_events {
        return Err(TranscodeError::UnaryEventsPath {
            method: rpc.to_owned(),
            path,
        });
    }
    let grpc_path = PathAndQuery::from_maybe_shared(format!("/{rpc}")).map_err(|_| {
        TranscodeError::Template {
            method: rpc.to_owned(),
            template: path.clone(),
            reason: "the RPC name is not a valid path".into(),
        }
    })?;
    Ok(Binding {
        backend: backend.to_owned(),
        method: method.clone(),
        grpc_path,
        verb,
        template: parsed,
        body,
        response_body,
        streaming,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(names: &[&str]) -> Config {
        let listen = "127.0.0.1:0".parse().unwrap_or_else(|_| unreachable!());
        Config::embedded(
            listen,
            names
                .iter()
                .map(|n| ((*n).to_owned(), "http://127.0.0.1:1".to_owned())),
        )
    }

    #[test]
    fn the_shipped_annotations_bind_and_only_for_registered_backends() {
        let pool = pool().unwrap_or_else(|e| panic!("{e}"));
        let t = Transcoder::from_config(&pool, &config(&["engine", "ledger"]))
            .unwrap_or_else(|e| panic!("{e}"));
        let routes: Vec<String> = t
            .bindings()
            .iter()
            .map(|b| format!("{} {}", b.verb, b.template.axum))
            .collect();
        assert!(
            routes.contains(&"GET /v1/ledger/ping".to_owned()),
            "{routes:?}"
        );
        assert!(routes.contains(&"POST /v1/ledger/subjects/{subject_id}/facts".to_owned()));
        assert!(routes.contains(&"GET /v1/ledger/subjects/{subject_id}/facts".to_owned()));
        assert!(routes.contains(&"DELETE /v1/ledger/subjects/{subject_id}".to_owned()));
        assert!(routes.contains(&"GET /v1/engine/subjects/{subject_id}/events".to_owned()));
        let events = t
            .bindings()
            .iter()
            .find(|b| b.template.axum.ends_with("/events"))
            .unwrap_or_else(|| unreachable!());
        assert!(events.streaming);
        assert_eq!(events.backend, "engine");
        assert_eq!(events.grpc_path, "/tbd.engine.v1.EngineService/Subscribe");

        let engine_only =
            Transcoder::from_config(&pool, &config(&["engine"])).unwrap_or_else(|e| panic!("{e}"));
        assert!(
            engine_only.bindings().iter().all(|b| b.backend == "engine"),
            "an unregistered backend's routes are skipped"
        );
        assert_eq!(backend_of("tbd.ledger.v1"), Some("ledger"));
        assert_eq!(backend_of("google.api"), None);
    }
}
