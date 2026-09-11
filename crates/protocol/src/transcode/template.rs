//! `google.api.http` path templates, the subset the gateway serves.
//!
//! Accepted: `/literal` segments and one variable per segment, `{field}` or
//! `{outer.inner}`, bound to a scalar, enum or `google.protobuf.Timestamp`
//! field of the request. Rejected at startup with a reason: `*`, `**`,
//! `{a=b/*}` sub-paths, `:verb` suffixes, text around a variable, repeated
//! or message-typed variables. The axum pattern is the template itself:
//! axum 0.8 spells a parameter `{name}` and allows dots in the name.

use prost_reflect::{FieldDescriptor, Kind, MessageDescriptor};

/// A parsed template.
#[derive(Debug, Clone)]
pub struct Template {
    /// The axum route pattern (the template verbatim).
    pub axum: String,
    /// Variables in order of appearance.
    pub vars: Vec<PathVar>,
}

/// One `{field}` variable and the request field it binds.
#[derive(Debug, Clone)]
pub struct PathVar {
    /// The name inside the braces, dotted for nested fields.
    pub name: String,
    /// The field chain from the request message to the leaf.
    pub field_path: Vec<FieldDescriptor>,
}

/// Why a template is not served.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{0}")]
pub struct TemplateError(pub String);

/// Parse a template against the request message.
///
/// # Errors
/// A form the gateway does not serve, or a variable that names no bindable
/// field.
pub fn parse(template: &str, input: &MessageDescriptor) -> Result<Template, TemplateError> {
    let Some(rest) = template.strip_prefix('/') else {
        return Err(TemplateError("must start with `/`".into()));
    };
    if template.contains('=') {
        // Checked before splitting on `/`: the sub-path in `{a=b/*}` has one.
        return Err(TemplateError(
            "`{field=...}` sub-path variables are not supported".into(),
        ));
    }
    let mut vars: Vec<PathVar> = Vec::new();
    for segment in rest.split('/') {
        if segment.is_empty() {
            return Err(TemplateError("empty segment".into()));
        }
        if segment.contains(':') {
            return Err(TemplateError("`:verb` suffixes are not supported".into()));
        }
        if segment == "*" || segment == "**" {
            return Err(TemplateError(
                "`*` and `**` segments are not supported".into(),
            ));
        }
        if let Some(inner) = segment.strip_prefix('{').and_then(|s| s.strip_suffix('}')) {
            if inner.is_empty() || inner.contains(['{', '}', '*', '/']) {
                return Err(TemplateError(format!("`{{{inner}}}` is not a field path")));
            }
            if vars.iter().any(|v| v.name == inner) {
                return Err(TemplateError(format!("`{{{inner}}}` appears twice")));
            }
            let field_path = field_path(input, inner).ok_or_else(|| {
                TemplateError(format!(
                    "`{{{inner}}}` names no scalar, enum or timestamp field of {}",
                    input.full_name()
                ))
            })?;
            vars.push(PathVar {
                name: inner.to_owned(),
                field_path,
            });
        } else if segment.contains(['{', '}']) {
            return Err(TemplateError(format!(
                "`{segment}`: one variable per segment, nothing around it"
            )));
        }
    }
    Ok(Template {
        axum: template.to_owned(),
        vars,
    })
}

/// The pattern with every variable name erased, for duplicate detection:
/// `/v1/x/{a}` and `/v1/x/{b}` are the same route to the router.
#[must_use]
pub fn normalised(axum: &str) -> String {
    let mut out = String::with_capacity(axum.len());
    let mut in_var = false;
    for c in axum.chars() {
        match c {
            '{' => {
                in_var = true;
                out.push_str("{}");
            }
            '}' => in_var = false,
            _ if in_var => {}
            _ => out.push(c),
        }
    }
    out
}

/// Resolve `outer.inner` against a message for a path variable: every step but
/// the last is a singular message field, the leaf is a singular scalar, enum
/// or timestamp.
pub(crate) fn field_path(desc: &MessageDescriptor, dotted: &str) -> Option<Vec<FieldDescriptor>> {
    resolve(desc, dotted, Leaf::Singular)
}

/// The same for a query parameter: the leaf may be repeated.
pub(crate) fn field_path_any(
    desc: &MessageDescriptor,
    dotted: &str,
) -> Option<Vec<FieldDescriptor>> {
    resolve(desc, dotted, Leaf::MaybeList)
}

/// The same for `body: "field"`: the leaf is a singular message.
pub(crate) fn field_path_message(
    desc: &MessageDescriptor,
    dotted: &str,
) -> Option<Vec<FieldDescriptor>> {
    resolve(desc, dotted, Leaf::Message)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Leaf {
    Singular,
    MaybeList,
    Message,
}

fn resolve(desc: &MessageDescriptor, dotted: &str, leaf: Leaf) -> Option<Vec<FieldDescriptor>> {
    let mut path = Vec::new();
    let mut current = desc.clone();
    let mut parts = dotted.split('.').peekable();
    while let Some(part) = parts.next() {
        let field = current
            .get_field_by_json_name(part)
            .or_else(|| current.get_field_by_name(part))?;
        if field.is_map() {
            return None;
        }
        let last = parts.peek().is_none();
        if !last {
            let Kind::Message(inner) = field.kind() else {
                return None;
            };
            if field.is_list() {
                return None;
            }
            current = inner;
            path.push(field);
            continue;
        }
        let ok = match (leaf, field.kind()) {
            (Leaf::Message, Kind::Message(inner)) => !field.is_list() && !is_timestamp(&inner),
            (Leaf::Message, _) => false,
            (_, Kind::Message(inner)) => {
                is_timestamp(&inner) && (leaf == Leaf::MaybeList || !field.is_list())
            }
            (Leaf::Singular, _) => !field.is_list(),
            (Leaf::MaybeList, _) => true,
        };
        if !ok {
            return None;
        }
        path.push(field);
    }
    Some(path)
}

pub(crate) fn is_timestamp(desc: &MessageDescriptor) -> bool {
    desc.full_name() == "google.protobuf.Timestamp"
}

#[cfg(test)]
mod tests {
    use super::*;
    use prost_reflect::DescriptorPool;

    fn input(message: &str) -> MessageDescriptor {
        let pool =
            DescriptorPool::decode(tbd_proto::DESCRIPTOR_SET_ALL).unwrap_or_else(|e| panic!("{e}"));
        pool.get_message_by_name(message)
            .unwrap_or_else(|| panic!("{message}"))
    }

    #[test]
    fn the_shipped_templates_parse() {
        let current = input("tbd.ledger.v1.CurrentRequest");
        let t = parse("/v1/ledger/subjects/{subject_id}/facts", &current)
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(t.axum, "/v1/ledger/subjects/{subject_id}/facts");
        assert_eq!(t.vars.len(), 1);
        assert_eq!(t.vars[0].name, "subject_id");
        assert_eq!(t.vars[0].field_path[0].name(), "subject_id");
        let ping = input("tbd.ledger.v1.PingRequest");
        assert!(parse("/v1/ledger/ping", &ping).is_ok());
        assert_eq!(
            normalised("/v1/ledger/subjects/{subject_id}/facts"),
            "/v1/ledger/subjects/{}/facts"
        );
    }

    #[test]
    fn unsupported_forms_are_refused_with_a_reason() {
        let current = input("tbd.ledger.v1.CurrentRequest");
        for (template, reason) in [
            ("v1/x", "must start"),
            ("/v1//x", "empty segment"),
            ("/v1/{subject_id}:cancel", "`:verb`"),
            ("/v1/*/x", "`*` and `**`"),
            ("/v1/{subject_id=subjects/*}", "sub-path"),
            ("/v1/pre{subject_id}", "one variable per segment"),
            ("/v1/{subject_id}/{subject_id}", "appears twice"),
            ("/v1/{nope}", "names no scalar"),
            ("/v1/{paths}", "names no scalar"),
        ] {
            let err = parse(template, &current)
                .err()
                .unwrap_or_else(|| panic!("{template}"));
            assert!(err.0.contains(reason), "{template}: {err}");
        }
    }

    #[test]
    fn a_timestamp_leaf_binds_but_a_message_leaf_does_not() {
        let history = input("tbd.ledger.v1.HistoryRequest");
        assert!(parse("/v1/x/{at}", &history).is_ok());
        let append = input("tbd.ledger.v1.AppendRequest");
        assert!(parse("/v1/x/{value}", &append).is_err());
        assert!(parse("/v1/x/{value.version}", &append).is_ok());
    }
}
