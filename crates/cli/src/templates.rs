//! The embedded templates: files a service is made of, and snippets inserted
//! into shared files. Paths carry tokens too.

/// A file the scaffold creates.
#[derive(Debug, Clone, Copy)]
pub struct FileTemplate {
    /// Output path, relative to the workspace root, with tokens.
    pub out: &'static str,
    /// Content, with tokens.
    pub body: &'static str,
}

macro_rules! file {
    ($out:literal, $path:literal) => {
        FileTemplate {
            out: $out,
            body: include_str!(concat!("../templates/", $path)),
        }
    };
}

/// Every file a gRPC service consists of, in the order created.
pub const GRPC: &[FileTemplate] = &[
    file!("crates/@@name@@/Cargo.toml", "crate/Cargo.toml.tmpl"),
    file!("crates/@@name@@/CLAUDE.md", "crate/CLAUDE.md.tmpl"),
    file!("crates/@@name@@/src/main.rs", "crate/src/main.rs.tmpl"),
    file!("crates/@@name@@/src/lib.rs", "crate/src/lib.rs.tmpl"),
    file!("crates/@@name@@/src/config.rs", "crate/src/config.rs.tmpl"),
    file!(
        "crates/@@name@@/src/service.rs",
        "crate/src/service.rs.tmpl"
    ),
    file!(
        "crates/@@name@@/tests/it/main.rs",
        "crate/tests/it/main.rs.tmpl"
    ),
    file!(
        "crates/@@name@@/tests/it/support.rs",
        "crate/tests/it/support.rs.tmpl"
    ),
    file!("proto/@@proto_path@@", "proto/service.proto.tmpl"),
    file!("configs/@@name@@/base.toml", "configs/base.toml.tmpl"),
    file!("configs/@@name@@/local.toml", "configs/local.toml.tmpl"),
    file!("configs/@@name@@/dev.toml", "configs/dev.toml.tmpl"),
    file!(
        "configs/@@name@@/production.toml",
        "configs/production.toml.tmpl"
    ),
    file!(
        "devops/k8s/base/@@name@@/deployment.yaml",
        "k8s/deployment.yaml.tmpl"
    ),
    file!(
        "devops/k8s/base/@@name@@/service.yaml",
        "k8s/service.yaml.tmpl"
    ),
    file!(
        "devops/k8s/base/@@name@@/kustomization.yaml",
        "k8s/kustomization.yaml.tmpl"
    ),
    file!("crates/chaos/src/kinds/@@name@@.rs", "chaos/kind.rs.tmpl"),
];

/// A snippet inserted into a shared file, by name.
#[must_use]
pub fn snippet(name: &str) -> &'static str {
    match name {
        "proto-mod" => include_str!("../templates/snippets/proto-mod.rs.tmpl"),
        "env-example" => include_str!("../templates/snippets/env-example.tmpl"),
        "configmap" => include_str!("../templates/snippets/configmap.yaml.tmpl"),
        "protocol-service" => include_str!("../templates/snippets/protocol-service.toml.tmpl"),
        "overlay-image" => include_str!("../templates/snippets/overlay-image.yaml.tmpl"),
        "overlay-patch-pull" => include_str!("../templates/snippets/overlay-patch-pull.yaml.tmpl"),
        "overlay-patch-prod" => include_str!("../templates/snippets/overlay-patch-prod.yaml.tmpl"),
        "envoy-route" => include_str!("../templates/snippets/envoy-route.yaml.tmpl"),
        "envoy-cluster" => include_str!("../templates/snippets/envoy-cluster.yaml.tmpl"),
        "compose-service" => include_str!("../templates/snippets/compose-service.yaml.tmpl"),
        "ansible-service" => include_str!("../templates/snippets/ansible-service.j2.tmpl"),
        "ci-matrix" => include_str!("../templates/snippets/ci-matrix.yaml.tmpl"),
        "mise-run" => include_str!("../templates/snippets/mise-run.toml.tmpl"),
        "mise-watch" => include_str!("../templates/snippets/mise-watch.toml.tmpl"),
        "bacon-job" => include_str!("../templates/snippets/bacon-job.toml.tmpl"),
        _ => "",
    }
}
