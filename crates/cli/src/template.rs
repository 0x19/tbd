//! Token substitution for embedded templates: `@@ident@@` becomes a value.
//!
//! No template engine: templates are Rust, TOML, YAML, proto, Jinja and
//! Markdown, all of which use braces, so `{{ }}` was never an option, and one
//! service kind needs no loops or conditionals. An unknown token or a stray
//! `@@` is an error, which is what catches a typo in a template.

use std::collections::BTreeMap;

use crate::service::Service;

/// Values for the tokens.
#[derive(Debug, Clone, Default)]
pub struct Vars(BTreeMap<String, String>);

impl Vars {
    /// The fixed token set for a service.
    #[must_use]
    pub fn for_service(service: &Service) -> Self {
        let mut v = BTreeMap::new();
        let n = &service.name;
        v.insert("name".into(), n.as_str().to_owned());
        v.insert("plural".into(), n.plural());
        v.insert("Name".into(), n.pascal());
        v.insert("NAME".into(), n.upper());
        v.insert("package".into(), service.package());
        v.insert("crate".into(), service.crate_ident());
        v.insert("proto_package".into(), service.proto_package());
        v.insert("grpc_service".into(), service.grpc_service());
        v.insert("proto_path".into(), service.proto_path());
        v.insert("port".into(), service.port.to_string());
        v.insert("metrics_port".into(), service.metrics_port.to_string());
        v.insert("image".into(), service.image());
        v.insert("bacon_key".into(), service.bacon_key.to_string());
        v.insert("cli_version".into(), crate::VERSION.to_owned());
        v.insert("marker".into(), service.marker());
        Self(v)
    }

    /// Add or replace one token.
    #[must_use]
    pub fn with(mut self, key: &str, value: impl Into<String>) -> Self {
        self.0.insert(key.to_owned(), value.into());
        self
    }

    /// Every token name.
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.0.keys().map(String::as_str)
    }
}

/// A template could not be rendered.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum TemplateError {
    /// `@@x@@` with no value for `x`.
    #[error("unknown token @@{0}@@")]
    UnknownToken(String),
    /// An `@@` with no closing `@@` on the same line.
    #[error("unterminated @@ at line {0}")]
    Unterminated(usize),
}

/// Render `src`, replacing every `@@token@@`.
///
/// # Errors
/// A token has no value, or an `@@` is never closed.
pub fn render(src: &str, vars: &Vars) -> Result<String, TemplateError> {
    let mut out = String::with_capacity(src.len());
    for (line_no, line) in src.split_inclusive('\n').enumerate() {
        let mut rest = line;
        while let Some(start) = rest.find("@@") {
            out.push_str(&rest[..start]);
            let after = &rest[start + 2..];
            let Some(end) = after.find("@@") else {
                return Err(TemplateError::Unterminated(line_no + 1));
            };
            let key = &after[..end];
            let value = vars
                .0
                .get(key)
                .ok_or_else(|| TemplateError::UnknownToken(key.to_owned()))?;
            out.push_str(value);
            rest = &after[end + 2..];
        }
        out.push_str(rest);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::{Kind, ServiceName};

    fn vars() -> Vars {
        Vars::for_service(&Service {
            name: ServiceName::parse("ledger").unwrap(),
            kind: Kind::Grpc,
            port: 50052,
            metrics_port: 9466,
            bacon_key: 'l',
        })
    }

    #[test]
    fn renders_every_token_and_leaves_braces_alone() {
        let out = render(
            "pkg @@package@@ {{ jinja }} ${{ gh }} @@Name@@Service\n",
            &vars(),
        )
        .unwrap();
        assert_eq!(out, "pkg tbd-ledger {{ jinja }} ${{ gh }} LedgerService\n");
    }

    #[test]
    fn unknown_and_unterminated_tokens_fail() {
        assert_eq!(
            render("@@nope@@", &vars()),
            Err(TemplateError::UnknownToken("nope".into()))
        );
        assert_eq!(
            render("a\n@@name", &vars()),
            Err(TemplateError::Unterminated(2))
        );
    }
}
