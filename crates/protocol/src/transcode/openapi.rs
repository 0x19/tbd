//! The `OpenAPI` paths and schemas of the transcoded routes, built from the
//! descriptors with utoipa's runtime builders and merged into the hand-written
//! document (first wins per path and per schema name).

use std::{collections::BTreeSet, sync::Arc};

use prost_reflect::{FieldDescriptor, Kind, MessageDescriptor};
use utoipa::openapi::{
    Components, ComponentsBuilder, Content, HttpMethod, OpenApi, OpenApiBuilder, PathItem,
    PathsBuilder, Ref, RefOr, Required, ResponseBuilder, ResponsesBuilder,
    path::{Operation, OperationBuilder, ParameterBuilder, ParameterIn},
    request_body::RequestBodyBuilder,
    schema::{Array, KnownFormat, ObjectBuilder, Schema, SchemaFormat, Type},
};

use super::{Binding, BodyRule, template};

/// The document fragment for `bindings`: one operation per binding, one
/// component per message (named by its proto full name), `Problem` for every
/// error response.
#[must_use]
pub fn document(bindings: &[Arc<Binding>]) -> OpenApi {
    let mut components = ComponentsBuilder::new();
    let mut seen = BTreeSet::new();
    let mut paths = PathsBuilder::new();
    for binding in bindings {
        let operation = operation(binding, &mut components, &mut seen);
        let Some(method) = http_method(&binding.verb) else {
            continue;
        };
        paths = paths.path(
            binding.template.axum.clone(),
            PathItem::new(method, operation),
        );
    }
    let components: Components = components.build();
    OpenApiBuilder::new()
        .paths(paths.build())
        .components(Some(components))
        .build()
}

fn http_method(verb: &http::Method) -> Option<HttpMethod> {
    Some(match *verb {
        http::Method::GET => HttpMethod::Get,
        http::Method::POST => HttpMethod::Post,
        http::Method::PUT => HttpMethod::Put,
        http::Method::DELETE => HttpMethod::Delete,
        http::Method::PATCH => HttpMethod::Patch,
        _ => return None,
    })
}

fn operation(
    binding: &Binding,
    components: &mut ComponentsBuilder,
    seen: &mut BTreeSet<String>,
) -> Operation {
    let service = binding.method.parent_service();
    let mut op = OperationBuilder::new()
        .operation_id(Some(format!(
            "{}.{}",
            service.name(),
            binding.method.name()
        )))
        .summary(Some(format!(
            "{}/{}",
            service.full_name(),
            binding.method.name()
        )))
        .description(Some(format!(
            "Transcoded from `{}` on the `{}` backend.",
            binding.grpc_path, binding.backend
        )))
        .tag(binding.backend.clone());

    let input = binding.method.input();
    let mut bound: Vec<String> = Vec::new();
    for var in &binding.template.vars {
        bound.push(var.name.clone());
        let leaf = var.field_path.last();
        op = op.parameter(
            ParameterBuilder::new()
                .name(var.name.clone())
                .parameter_in(ParameterIn::Path)
                .required(Required::True)
                .schema(leaf.map(|f| leaf_schema(f, components, seen)))
                .build(),
        );
    }
    match &binding.body {
        BodyRule::Whole => {
            let schema = message_ref(&input, components, seen);
            op = op.request_body(Some(
                RequestBodyBuilder::new()
                    .description(Some(format!(
                        "`{}`; path variables override their fields",
                        input.full_name()
                    )))
                    .required(Some(Required::True))
                    .content("application/json", Content::new(Some(schema)))
                    .build(),
            ));
        }
        BodyRule::Field(path) => {
            if let Some(leaf) = path.last() {
                bound.push(leaf.name().to_owned());
                if let Kind::Message(sub) = leaf.kind() {
                    let schema = message_ref(&sub, components, seen);
                    op = op.request_body(Some(
                        RequestBodyBuilder::new()
                            .required(Some(Required::True))
                            .content("application/json", Content::new(Some(schema)))
                            .build(),
                    ));
                }
            }
            for parameter in query_parameters(&input, &bound, components, seen) {
                op = op.parameter(parameter);
            }
        }
        BodyRule::None => {
            for parameter in query_parameters(&input, &bound, components, seen) {
                op = op.parameter(parameter);
            }
        }
    }

    let output = binding.method.output();
    let output_schema = match &binding.response_body {
        Some(field) => leaf_schema(field, components, seen),
        None => message_ref(&output, components, seen),
    };
    let ok = if binding.streaming {
        ResponseBuilder::new()
            .description(format!(
                "A stream of `{}` events, one JSON object per `data:`; a failure is an `error` event carrying `Problem`",
                output.full_name()
            ))
            .content("text/event-stream", Content::new(Some(output_schema)))
    } else {
        ResponseBuilder::new()
            .description(format!("`{}`", output.full_name()))
            .content("application/json", Content::new(Some(output_schema)))
    };
    let responses = ResponsesBuilder::new()
        .response("200", ok.build())
        .response(
            "default",
            ResponseBuilder::new()
                .description("The error envelope, with the standard gRPC-to-HTTP status")
                .content(
                    "application/json",
                    Content::new(Some(RefOr::<Schema>::Ref(Ref::from_schema_name("Problem")))),
                )
                .build(),
        )
        .build();
    op.responses(responses).build()
}

/// Query parameters: every unbound leaf field, dotted for nested messages,
/// three levels deep at most.
fn query_parameters(
    input: &MessageDescriptor,
    bound: &[String],
    components: &mut ComponentsBuilder,
    seen: &mut BTreeSet<String>,
) -> Vec<utoipa::openapi::path::Parameter> {
    let mut out = Vec::new();
    collect_query(input, "", 0, bound, components, seen, &mut out);
    out
}

fn collect_query(
    desc: &MessageDescriptor,
    prefix: &str,
    depth: u8,
    bound: &[String],
    components: &mut ComponentsBuilder,
    seen: &mut BTreeSet<String>,
    out: &mut Vec<utoipa::openapi::path::Parameter>,
) {
    for field in desc.fields() {
        let name = if prefix.is_empty() {
            field.name().to_owned()
        } else {
            format!("{prefix}.{}", field.name())
        };
        if bound.contains(&name) || field.is_map() {
            continue;
        }
        match field.kind() {
            Kind::Message(inner) if !template::is_timestamp(&inner) => {
                if depth < 3 && !field.is_list() {
                    collect_query(&inner, &name, depth + 1, bound, components, seen, out);
                }
            }
            _ => {
                let schema = leaf_schema(&field, components, seen);
                let mut parameter = ParameterBuilder::new()
                    .name(name)
                    .parameter_in(ParameterIn::Query)
                    .required(Required::False)
                    .schema(Some(schema));
                if field.is_list() {
                    parameter = parameter.explode(Some(true));
                }
                out.push(parameter.build());
            }
        }
    }
}

/// A `$ref` to the message's component, registering it (and everything it
/// references) once.
fn message_ref(
    desc: &MessageDescriptor,
    components: &mut ComponentsBuilder,
    seen: &mut BTreeSet<String>,
) -> RefOr<Schema> {
    let name = desc.full_name().to_owned();
    if seen.insert(name.clone()) {
        let mut object = ObjectBuilder::new()
            .schema_type(Type::Object)
            .description(Some(format!("`{name}`")));
        for field in desc.fields() {
            object = object.property(field.name(), field_schema(&field, components, seen));
        }
        let schema: Schema = object.build().into();
        // `ComponentsBuilder::schema` consumes the builder; rebuild through a
        // temporary to keep the borrow simple.
        let taken = std::mem::take(components);
        *components = taken.schema(name.clone(), schema);
    }
    RefOr::Ref(Ref::from_schema_name(name))
}

/// The schema of one field, arrays and maps included.
fn field_schema(
    field: &FieldDescriptor,
    components: &mut ComponentsBuilder,
    seen: &mut BTreeSet<String>,
) -> RefOr<Schema> {
    if field.is_map() {
        let value = match field.kind() {
            Kind::Message(entry) => entry
                .get_field_by_name("value")
                .map(|v| leaf_schema(&v, components, seen)),
            _ => None,
        };
        let mut object = ObjectBuilder::new().schema_type(Type::Object);
        if let Some(value) = value {
            object = object.additional_properties(Some(value));
        }
        return object.build().into();
    }
    let leaf = leaf_schema(field, components, seen);
    if field.is_list() {
        return Array::new(leaf).into();
    }
    leaf
}

/// The schema of a field's single value: proto3 JSON spellings.
fn leaf_schema(
    field: &FieldDescriptor,
    components: &mut ComponentsBuilder,
    seen: &mut BTreeSet<String>,
) -> RefOr<Schema> {
    let object = |ty: Type, format: Option<KnownFormat>| -> RefOr<Schema> {
        ObjectBuilder::new()
            .schema_type(ty)
            .format(format.map(SchemaFormat::KnownFormat))
            .build()
            .into()
    };
    match field.kind() {
        Kind::String => object(Type::String, None),
        Kind::Bool => object(Type::Boolean, None),
        Kind::Int32 | Kind::Sint32 | Kind::Sfixed32 => {
            object(Type::Integer, Some(KnownFormat::Int32))
        }
        Kind::Uint32 | Kind::Fixed32 => object(Type::Integer, Some(KnownFormat::Int32)),
        Kind::Int64 | Kind::Sint64 | Kind::Sfixed64 | Kind::Uint64 | Kind::Fixed64 => {
            // 64-bit integers travel as decimal strings (proto3 JSON).
            object(Type::String, Some(KnownFormat::Int64))
        }
        Kind::Float => object(Type::Number, Some(KnownFormat::Float)),
        Kind::Double => object(Type::Number, Some(KnownFormat::Double)),
        Kind::Bytes => object(Type::String, Some(KnownFormat::Byte)),
        Kind::Enum(desc) => ObjectBuilder::new()
            .schema_type(Type::String)
            .enum_values(Some(desc.values().map(|v| v.name().to_owned())))
            .description(Some(format!("`{}`", desc.full_name())))
            .build()
            .into(),
        Kind::Message(desc) if template::is_timestamp(&desc) => {
            object(Type::String, Some(KnownFormat::DateTime))
        }
        Kind::Message(desc) => message_ref(&desc, components, seen),
    }
}
