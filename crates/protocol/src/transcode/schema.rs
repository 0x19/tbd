//! JSON Schema for the platform's messages, from the descriptors: what an
//! agent is told a tool takes (MCP `inputSchema`). The spellings are proto3
//! JSON and the same rules the `OpenAPI` document follows (`openapi.rs`); a
//! test holds the two to the same answer for every message the document
//! names.
//!
//! Nested messages are inlined, because the agents that read these schemas
//! handle a flat object best; a message that contains itself is cut at the
//! second visit with an open object. Well-known types take their JSON forms.
//! Descriptions come from the comments in the `.proto` files.

use prost_reflect::{FieldDescriptor, Kind, MessageDescriptor, MethodDescriptor};
use serde_json::{Map, Value, json};

use super::template;

/// The schema of a message as an object, every field optional (proto3).
#[must_use]
pub fn message(desc: &MessageDescriptor) -> Value {
    object(desc, &mut Vec::new())
}

/// The leading comment of an RPC in its `.proto`, trimmed; empty when the
/// file has none.
#[must_use]
pub fn method_comment(method: &MethodDescriptor) -> String {
    comment(method.parent_file().file_descriptor_proto(), method.path())
}

fn field_comment(field: &FieldDescriptor) -> String {
    comment(field.parent_file().file_descriptor_proto(), field.path())
}

fn comment(file: &prost_types::FileDescriptorProto, path: &[i32]) -> String {
    file.source_code_info
        .as_ref()
        .and_then(|info| info.location.iter().find(|l| l.path == path))
        .and_then(|l| l.leading_comments.as_deref())
        .map(|c| {
            c.lines()
                .map(str::trim)
                .filter(|l| !l.is_empty())
                .collect::<Vec<_>>()
                .join(" ")
        })
        .unwrap_or_default()
}

fn object(desc: &MessageDescriptor, stack: &mut Vec<String>) -> Value {
    let name = desc.full_name().to_owned();
    if stack.contains(&name) {
        return json!({ "type": "object", "description": format!("`{name}` (recursive; any object)") });
    }
    stack.push(name.clone());
    let mut properties = Map::new();
    for field in desc.fields() {
        let mut schema = field_schema(&field, stack);
        let text = field_comment(&field);
        if !text.is_empty()
            && let Some(o) = schema.as_object_mut()
        {
            o.insert("description".into(), Value::String(text));
        }
        properties.insert(field.name().to_owned(), schema);
    }
    stack.pop();
    json!({
        "type": "object",
        "properties": properties,
        "additionalProperties": false,
    })
}

fn field_schema(field: &FieldDescriptor, stack: &mut Vec<String>) -> Value {
    if field.is_map() {
        let value = match field.kind() {
            Kind::Message(entry) => entry.get_field_by_name("value").map(|v| leaf(&v, stack)),
            _ => None,
        };
        return match value {
            Some(v) => json!({ "type": "object", "additionalProperties": v }),
            None => json!({ "type": "object" }),
        };
    }
    let one = leaf(field, stack);
    if field.is_list() {
        return json!({ "type": "array", "items": one });
    }
    one
}

/// One value's schema: the proto3 JSON spelling of its kind.
fn leaf(field: &FieldDescriptor, stack: &mut Vec<String>) -> Value {
    match field.kind() {
        Kind::String => json!({ "type": "string" }),
        Kind::Bool => json!({ "type": "boolean" }),
        Kind::Int32 | Kind::Sint32 | Kind::Sfixed32 | Kind::Uint32 | Kind::Fixed32 => {
            json!({ "type": "integer", "format": "int32" })
        }
        // 64-bit integers travel as decimal strings (proto3 JSON).
        Kind::Int64 | Kind::Sint64 | Kind::Sfixed64 | Kind::Uint64 | Kind::Fixed64 => {
            json!({ "type": "string", "format": "int64" })
        }
        Kind::Float => json!({ "type": "number", "format": "float" }),
        Kind::Double => json!({ "type": "number", "format": "double" }),
        Kind::Bytes => json!({ "type": "string", "format": "byte" }),
        Kind::Enum(desc) => json!({
            "type": "string",
            "enum": desc.values().map(|v| v.name().to_owned()).collect::<Vec<_>>(),
        }),
        Kind::Message(desc) if template::is_timestamp(&desc) => {
            json!({ "type": "string", "format": "date-time" })
        }
        Kind::Message(desc) => well_known(&desc).unwrap_or_else(|| object(&desc, stack)),
    }
}

/// The JSON forms of the well-known types other than `Timestamp`.
fn well_known(desc: &MessageDescriptor) -> Option<Value> {
    Some(match desc.full_name() {
        "google.protobuf.Duration" => {
            json!({ "type": "string", "description": "seconds, e.g. `1.5s`" })
        }
        "google.protobuf.Struct" => json!({ "type": "object" }),
        "google.protobuf.Value" | "google.protobuf.Any" => json!({}),
        "google.protobuf.ListValue" => json!({ "type": "array" }),
        "google.protobuf.Empty" => {
            json!({ "type": "object", "properties": {}, "additionalProperties": false })
        }
        "google.protobuf.StringValue" => json!({ "type": "string" }),
        "google.protobuf.BoolValue" => json!({ "type": "boolean" }),
        "google.protobuf.Int32Value" | "google.protobuf.UInt32Value" => {
            json!({ "type": "integer" })
        }
        "google.protobuf.Int64Value" | "google.protobuf.UInt64Value" => {
            json!({ "type": "string", "format": "int64" })
        }
        "google.protobuf.FloatValue" | "google.protobuf.DoubleValue" => json!({ "type": "number" }),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pool() -> prost_reflect::DescriptorPool {
        prost_reflect::DescriptorPool::decode(tbd_proto::DESCRIPTOR_SET_ALL)
            .unwrap_or_else(|e| panic!("{e}"))
    }

    /// Every request message of every service has an object schema whose
    /// properties are exactly its fields, by proto name.
    #[test]
    fn every_request_message_is_an_object_of_its_fields() {
        let pool = pool();
        let mut seen = 0;
        for service in pool
            .services()
            .filter(|s| s.package_name().starts_with("tbd."))
        {
            for method in service.methods() {
                let input = method.input();
                let schema = message(&input);
                assert_eq!(schema["type"], "object", "{}", input.full_name());
                let props = schema["properties"]
                    .as_object()
                    .cloned()
                    .unwrap_or_default();
                let fields: Vec<String> = input.fields().map(|f| f.name().to_owned()).collect();
                let mut keys: Vec<String> = props.keys().cloned().collect();
                let mut want = fields.clone();
                keys.sort();
                want.sort();
                assert_eq!(keys, want, "{}", input.full_name());
                seen += 1;
            }
        }
        assert!(seen > 10, "the contract has RPCs: {seen}");
    }

    #[test]
    fn comments_travel_with_the_contract() {
        let pool = pool();
        let generate = pool
            .get_service_by_name("tbd.llm.v1.LlmService")
            .and_then(|s| s.methods().find(|m| m.name() == "Generate"))
            .unwrap_or_else(|| panic!("LlmService/Generate"));
        let text = method_comment(&generate);
        assert!(text.contains("streamed"), "{text:?}");
        let schema = message(&generate.input());
        assert_eq!(schema["properties"]["tier"]["type"], "string");
        assert!(
            schema["properties"]["tier"]["enum"]
                .as_array()
                .is_some_and(|e| e.len() == 3)
        );
        assert!(
            schema["properties"]["session_id"]["description"]
                .as_str()
                .is_some_and(|d| !d.is_empty()),
            "{}",
            schema["properties"]["session_id"]
        );
    }

    /// The same spellings as the `OpenAPI` document: an int64 is a string, a
    /// timestamp a date-time string, an enum a string with its names.
    #[test]
    fn spellings_match_the_openapi_rules() {
        let pool = pool();
        let budget = pool
            .get_message_by_name("tbd.llm.v1.GetBudgetResponse")
            .unwrap_or_else(|| panic!("GetBudgetResponse"));
        let schema = message(&budget);
        assert_eq!(schema["properties"]["used_today"]["type"], "string");
        assert_eq!(schema["properties"]["used_today"]["format"], "int64");
    }
}
