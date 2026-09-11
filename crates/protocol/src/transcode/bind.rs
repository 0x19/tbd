//! Build the request message from the HTTP request: JSON body, then path
//! variables, then query parameters, every failure a [`Problem`] with a
//! `field` detail.

use std::collections::HashMap;

use prost_reflect::{
    DeserializeOptions, DynamicMessage, EnumDescriptor, FieldDescriptor, Kind, Value,
};

use super::{Binding, BodyRule, template};
use crate::{Code, Problem};

/// The request message for one call.
///
/// # Errors
/// A body that is not the expected JSON, a query parameter that binds nothing
/// or binds twice, a value that does not convert; each names the field.
pub fn request(
    binding: &Binding,
    vars: &HashMap<String, String>,
    query: Option<&str>,
    body: Option<&[u8]>,
) -> Result<DynamicMessage, Problem> {
    let input = binding.method.input();
    let mut msg = match (&binding.body, body) {
        (BodyRule::Whole, Some(bytes)) => whole_body(&input, bytes)?,
        (BodyRule::Field(path), Some(bytes)) => {
            let mut msg = DynamicMessage::new(input.clone());
            let leaf = path
                .last()
                .ok_or_else(|| Problem::field("body", "no field"))?;
            let Kind::Message(sub) = leaf.kind() else {
                return Err(Problem::field("body", "the body field is not a message"));
            };
            let value = whole_body(&sub, bytes)?;
            set_path(&mut msg, path, Value::Message(value));
            msg
        }
        (BodyRule::None, Some(bytes)) if !bytes.is_empty() => {
            return Err(Problem::field("body", "this route takes no body"));
        }
        (BodyRule::Whole | BodyRule::Field(_), None) | (BodyRule::None, _) => {
            DynamicMessage::new(input.clone())
        }
    };

    for var in &binding.template.vars {
        let raw = vars.get(&var.name).ok_or_else(|| {
            Problem::new(
                Code::Internal,
                format!("path variable {} missing from the match", var.name),
            )
        })?;
        let leaf = var
            .field_path
            .last()
            .ok_or_else(|| Problem::field(&var.name, "binds no field"))?;
        let value = scalar(leaf, raw, &var.name)?;
        set_path(&mut msg, &var.field_path, value);
    }

    if let Some(query) = query.filter(|q| !q.is_empty()) {
        if matches!(binding.body, BodyRule::Whole) {
            let first = form_urlencoded::parse(query.as_bytes())
                .next()
                .map(|(k, _)| k.into_owned())
                .unwrap_or_default();
            return Err(Problem::field(
                first,
                "not a query parameter: this route takes the whole request as its body",
            ));
        }
        let mut seen: HashMap<String, usize> = HashMap::new();
        for (key, raw) in form_urlencoded::parse(query.as_bytes()) {
            let key = key.into_owned();
            if binding.template.vars.iter().any(|v| v.name == key) {
                return Err(Problem::field(&key, "is bound by the path, not the query"));
            }
            if let BodyRule::Field(path) = &binding.body
                && path.first().is_some_and(|f| key.starts_with(f.name()))
            {
                return Err(Problem::field(&key, "is bound by the body, not the query"));
            }
            let path = template::field_path_any(&binding.method.input(), &key)
                .ok_or_else(|| Problem::field(&key, "unknown query parameter"))?;
            let leaf = path
                .last()
                .ok_or_else(|| Problem::field(&key, "binds no field"))?;
            let count = seen.entry(key.clone()).or_insert(0);
            *count += 1;
            if leaf.is_list() {
                let value = scalar(leaf, &raw, &key)?;
                push_path(&mut msg, &path, value);
            } else {
                if *count > 1 {
                    return Err(Problem::field(&key, "given more than once"));
                }
                let value = scalar(leaf, &raw, &key)?;
                set_path(&mut msg, &path, value);
            }
        }
    }
    Ok(msg)
}

fn whole_body(
    desc: &prost_reflect::MessageDescriptor,
    bytes: &[u8],
) -> Result<DynamicMessage, Problem> {
    let mut de = serde_json::Deserializer::from_slice(bytes);
    let options = DeserializeOptions::new().deny_unknown_fields(true);
    DynamicMessage::deserialize_with_options(desc.clone(), &mut de, &options)
        .map_err(|e| Problem::field("body", e.to_string()))
}

/// Set the leaf of a field chain, creating the intermediate messages
/// (`get_field_mut` materialises an unset singular message).
fn set_path(msg: &mut DynamicMessage, path: &[FieldDescriptor], value: Value) {
    match path {
        [] => {}
        [leaf] => msg.set_field(leaf, value),
        [first, rest @ ..] => {
            if let Value::Message(inner) = msg.get_field_mut(first) {
                set_path(inner, rest, value);
            }
        }
    }
}

/// Push onto the repeated leaf of a field chain.
fn push_path(msg: &mut DynamicMessage, path: &[FieldDescriptor], value: Value) {
    match path {
        [] => {}
        [leaf] => {
            if let Value::List(items) = msg.get_field_mut(leaf) {
                items.push(value);
            }
        }
        [first, rest @ ..] => {
            if let Value::Message(inner) = msg.get_field_mut(first) {
                push_path(inner, rest, value);
            }
        }
    }
}

/// Convert one string (a path variable or a query value) to the field's kind.
fn scalar(field: &FieldDescriptor, raw: &str, name: &str) -> Result<Value, Problem> {
    let bad = |what: &str| Problem::field(name, format!("{raw:?} is not {what}"));
    Ok(match field.kind() {
        Kind::String => Value::String(raw.to_owned()),
        Kind::Bool => match raw {
            "true" => Value::Bool(true),
            "false" => Value::Bool(false),
            _ => return Err(bad("true or false")),
        },
        Kind::Int32 | Kind::Sint32 | Kind::Sfixed32 => {
            Value::I32(raw.parse().map_err(|_| bad("a 32-bit integer"))?)
        }
        Kind::Int64 | Kind::Sint64 | Kind::Sfixed64 => {
            Value::I64(raw.parse().map_err(|_| bad("a 64-bit integer"))?)
        }
        Kind::Uint32 | Kind::Fixed32 => {
            Value::U32(raw.parse().map_err(|_| bad("an unsigned 32-bit integer"))?)
        }
        Kind::Uint64 | Kind::Fixed64 => {
            Value::U64(raw.parse().map_err(|_| bad("an unsigned 64-bit integer"))?)
        }
        Kind::Float => Value::F32(raw.parse().map_err(|_| bad("a number"))?),
        Kind::Double => Value::F64(raw.parse().map_err(|_| bad("a number"))?),
        Kind::Bytes => {
            use base64::Engine as _;
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(raw)
                .map_err(|_| bad("base64"))?;
            Value::Bytes(bytes.into())
        }
        Kind::Enum(desc) => Value::EnumNumber(enum_number(&desc, raw).ok_or_else(|| {
            Problem::field(
                name,
                format!("{raw:?} is not a value of {}", desc.full_name()),
            )
        })?),
        Kind::Message(desc) if template::is_timestamp(&desc) => {
            let ts: prost_types::Timestamp =
                raw.parse().map_err(|_| bad("an RFC 3339 timestamp"))?;
            let mut msg = DynamicMessage::new(desc);
            msg.transcode_from(&ts)
                .map_err(|e| Problem::field(name, e.to_string()))?;
            Value::Message(msg)
        }
        Kind::Message(_) => {
            return Err(Problem::field(
                name,
                "a message cannot be given as a path or query parameter",
            ));
        }
    })
}

fn enum_number(desc: &EnumDescriptor, raw: &str) -> Option<i32> {
    if let Some(value) = desc.get_value_by_name(raw) {
        return Some(value.number());
    }
    let number: i32 = raw.parse().ok()?;
    desc.get_value(number).map(|v| v.number())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transcode::{Transcoder, pool};
    use std::sync::Arc;

    fn binding(route: &str, verb: &str) -> Arc<Binding> {
        let pool = pool().unwrap_or_else(|e| panic!("{e}"));
        let listen = "127.0.0.1:0".parse().unwrap_or_else(|_| unreachable!());
        let config = crate::Config::embedded(
            listen,
            [
                ("engine".to_owned(), "http://127.0.0.1:1".to_owned()),
                ("ledger".to_owned(), "http://127.0.0.1:2".to_owned()),
            ],
        );
        let t = Transcoder::from_config(&pool, &config).unwrap_or_else(|e| panic!("{e}"));
        t.bindings()
            .iter()
            .find(|b| b.template.axum == route && b.verb.as_str() == verb)
            .cloned()
            .unwrap_or_else(|| panic!("{verb} {route}"))
    }

    fn json(msg: &DynamicMessage) -> serde_json::Value {
        serde_json::to_value(super::super::call::Out(msg)).unwrap_or_default()
    }

    #[test]
    fn query_binds_repeated_enum_timestamp_and_scalars() {
        let b = binding("/v1/ledger/subjects/{subject_id}/history", "GET");
        let vars = HashMap::from([("subject_id".to_owned(), "s-1".to_owned())]);
        let msg = request(
            &b,
            &vars,
            Some("scopes=self&scopes=other&sources=SOURCE_DECLARED&sources=2&limit=10&at=2030-01-01T00:00:00Z"),
            None,
        )
        .unwrap_or_else(|e| panic!("{e}"));
        let v = json(&msg);
        assert_eq!(v["subject_id"], "s-1");
        assert_eq!(v["scopes"], serde_json::json!(["self", "other"]));
        assert_eq!(
            v["sources"],
            serde_json::json!(["SOURCE_DECLARED", "SOURCE_DECLARED"])
        );
        assert_eq!(v["limit"], 10);
        assert_eq!(v["at"], "2030-01-01T00:00:00Z");
    }

    #[test]
    fn bad_query_parameters_name_the_field() {
        let b = binding("/v1/ledger/subjects/{subject_id}/facts", "GET");
        let vars = HashMap::from([("subject_id".to_owned(), "s-1".to_owned())]);
        let field = |q: &str| {
            let p = request(&b, &vars, Some(q), None)
                .err()
                .unwrap_or_else(|| panic!("{q}"));
            assert_eq!(p.code, Code::BadRequest, "{q}");
            match &p.details[0] {
                crate::Detail::Field { field, .. } => field.clone(),
                other => panic!("{q}: {other:?}"),
            }
        };
        assert_eq!(field("nope=1"), "nope");
        assert_eq!(field("sources=NOPE"), "sources");
        assert_eq!(field("limit=x"), "limit");
        assert_eq!(field("limit=1&limit=2"), "limit");
        assert_eq!(field("subject_id=other"), "subject_id");
    }

    #[test]
    fn a_whole_body_is_strict_and_excludes_query_parameters() {
        let b = binding("/v1/ledger/subjects/{subject_id}/facts", "POST");
        let vars = HashMap::from([("subject_id".to_owned(), "s-1".to_owned())]);
        let ok = request(
            &b,
            &vars,
            None,
            Some(br#"{"path":"profile.name","source":"SOURCE_DECLARED","consent":["self"],"value":{"version":0,"bytes":"IkFkYSI="}}"#),
        )
        .unwrap_or_else(|e| panic!("{e}"));
        let v = json(&ok);
        assert_eq!(
            v["subject_id"], "s-1",
            "the path variable is set over the body"
        );
        assert_eq!(v["path"], "profile.name");
        assert_eq!(v["value"]["bytes"], "IkFkYSI=");

        let unknown = request(&b, &vars, None, Some(br#"{"nope": 1}"#))
            .err()
            .unwrap_or_else(|| unreachable!());
        assert!(
            matches!(&unknown.details[0], crate::Detail::Field { field, .. } if field == "body")
        );
        let with_query = request(&b, &vars, Some("limit=1"), Some(b"{}"))
            .err()
            .unwrap_or_else(|| unreachable!());
        assert!(
            matches!(&with_query.details[0], crate::Detail::Field { field, .. } if field == "limit")
        );
    }
}
