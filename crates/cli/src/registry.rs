//! Every place a service is registered, as data. One function, one ordered
//! list; `scaffold`, `check` and `list` all iterate it.
//!
//! Order is compile order first (Cargo, proto), then runtime config, then
//! deployment, then tooling, then documentation, so a partial failure leaves
//! the most important things done. Needles never contain the port, so a
//! service scaffolded with one port is still recognised when checked with
//! another.

use std::path::PathBuf;

use crate::{
    edit::{Anchor, Edit, Match, Registration, SpliceAt},
    service::Service,
    template::{self, TemplateError, Vars},
    templates,
};

fn exact(s: &str) -> Match {
    Match::Exact(s.to_owned())
}
fn prefix(s: &str) -> Match {
    Match::Prefix(s.to_owned())
}
fn contains(s: &str) -> Match {
    Match::Contains(s.to_owned())
}

struct Builder {
    vars: Vars,
    out: Vec<Registration>,
}

impl Builder {
    fn r(&self, s: &str) -> Result<String, TemplateError> {
        template::render(s, &self.vars)
    }

    fn push(&mut self, id: &str, path: &str, needle: Option<&str>, edit: Edit) {
        self.out.push(Registration {
            id: id.to_owned(),
            path: PathBuf::from(path),
            needle: needle.map(str::to_owned),
            edit,
        });
    }

    fn after(
        &mut self,
        id: &str,
        path: &str,
        anchor: Anchor,
        lines: &str,
        needle: &str,
    ) -> Result<(), TemplateError> {
        let lines = self.r(lines)?;
        let needle = self.r(needle)?;
        self.push(id, path, Some(&needle), Edit::InsertAfter { anchor, lines });
        Ok(())
    }

    fn before(
        &mut self,
        id: &str,
        path: &str,
        anchor: Anchor,
        lines: &str,
        needle: &str,
    ) -> Result<(), TemplateError> {
        let lines = self.r(lines)?;
        let needle = self.r(needle)?;
        self.push(
            id,
            path,
            Some(&needle),
            Edit::InsertBefore { anchor, lines },
        );
        Ok(())
    }

    fn splice(
        &mut self,
        id: &str,
        path: &str,
        anchor: Anchor,
        at: SpliceAt,
        text: &str,
    ) -> Result<(), TemplateError> {
        let text = self.r(text)?;
        self.push(id, path, None, Edit::Splice { anchor, at, text });
        Ok(())
    }
}

/// Every registration for `service`, in apply order.
///
/// # Errors
/// A template or snippet uses a token that has no value.
#[allow(clippy::too_many_lines)] // the table is the point; splitting it hides the order
pub fn registrations(service: &Service) -> Result<Vec<Registration>, TemplateError> {
    let mut b = Builder {
        vars: Vars::for_service(service),
        out: Vec::new(),
    };
    let name = service.name.as_str();

    // 1. Files.
    for t in templates::GRPC {
        let out = b.r(t.out)?;
        let content = b.r(t.body)?;
        let id = format!("create:{out}");
        b.push(&id, &out, None, Edit::Create { content });
    }

    // 2. Cargo workspace.
    b.splice(
        "cargo:default-members",
        "Cargo.toml",
        Anchor::line(prefix("default-members = [")),
        SpliceAt::Before("]".into()),
        ", \"crates/@@name@@\"",
    )?;
    let key = format!("tbd-{name}");
    let dep = format!("{key:<12} = {{ path = \"crates/{name}\" }}");
    b.push(
        "cargo:dependency",
        "Cargo.toml",
        Some(&format!("{key} ")),
        Edit::InsertAfter {
            anchor: Anchor::line(Match::LastPrefix("tbd-".into())),
            lines: dep,
        },
    );

    // 3. Proto module. `crates/proto/build.rs` discovers the files itself.
    let module = b.r(templates::snippet("proto-mod"))?;
    b.push(
        "proto:module",
        "crates/proto/src/lib.rs",
        Some(&format!("pub mod {name} ")),
        Edit::AppendEof { lines: module },
    );

    // 4. Chaos: the kind module, its registry line, the dependency, the dev
    //    topology, the validate target per environment and the target env var
    //    everywhere the chaos pod is configured. rustfmt reorders `pub mod`
    //    lines at apply time (scaffold.rs), so the anchor only has to exist.
    b.after(
        "chaos:module",
        "crates/chaos/src/kinds/mod.rs",
        Anchor::line(Match::LastPrefix("pub mod ".into())),
        "pub mod @@name@@;",
        "pub mod @@name@@;",
    )?;
    b.before(
        "chaos:kind",
        "crates/chaos/src/kinds/mod.rs",
        Anchor::line(prefix("    // tbd:kinds-end")),
        "    &@@name@@::KIND,",
        "&@@name@@::KIND,",
    )?;
    b.after(
        "chaos:dependency",
        "crates/chaos/Cargo.toml",
        Anchor::line(exact("tbd-protocol.workspace = true")),
        "@@package@@.workspace = true",
        "@@package@@.workspace = true",
    )?;
    let topology = b.r("\n[stack.@@plural@@.@@name@@-1]\nlisten = \"127.0.0.1:@@port@@\"\n")?;
    b.push(
        "chaos:topology",
        "topologies/dev.toml",
        Some(&format!("[stack.{}.", service.name.plural())),
        Edit::AppendEof { lines: topology },
    );
    for (env, url) in [
        ("base", "http://127.0.0.1:@@port@@"),
        ("dev", "http://envoy:50051"),
        ("production", "https://engine.api.example.invalid"),
        ("cluster", "http://localhost:15051"),
    ] {
        b.after(
            &format!("chaos:targets:{env}"),
            &format!("configs/chaos/{env}.toml"),
            Anchor::scoped(exact("[targets]"), prefix("engine = ")),
            &format!("@@name@@ = \"{url}\""),
            "@@name@@ = \"http",
        )?;
    }
    b.after(
        "chaos:k8s-env",
        "devops/k8s/chaos/deployment.yaml",
        Anchor::scoped(
            contains("- name: CHAOS_ENGINE_URL"),
            prefix("              value: "),
        ),
        "            - name: CHAOS_@@NAME@@_URL\n              value: http://envoy:50051",
        "CHAOS_@@NAME@@_URL",
    )?;
    b.after(
        "chaos:compose-env",
        "compose.yaml",
        Anchor::line(prefix("      CHAOS_ENGINE_URL:")),
        "      CHAOS_@@NAME@@_URL: http://envoy:50051",
        "CHAOS_@@NAME@@_URL:",
    )?;
    b.after(
        "chaos:ansible-env",
        "devops/ansible/roles/tbd_app/templates/compose.yaml.j2",
        Anchor::line(prefix("      CHAOS_ENGINE_URL:")),
        "      CHAOS_@@NAME@@_URL: http://@@name@@:{{ @@name@@_port }}",
        "CHAOS_@@NAME@@_URL:",
    )?;

    // 5. Environment and config maps.
    let env = b.r(templates::snippet("env-example"))?;
    b.push(
        "env:example",
        ".env.example",
        Some(&format!("{}_LISTEN_ADDR=", service.name.upper())),
        Edit::AppendEof { lines: env },
    );
    b.after(
        "k8s:configmap",
        "devops/k8s/base/configmap.yaml",
        Anchor::line(prefix("  PROTOCOL_METRICS_ADDR:")),
        templates::snippet("configmap"),
        "@@NAME@@_LISTEN_ADDR:",
    )?;
    // The protocol's services registry: the backend table, and its URL in every
    // deployed environment next to the ledger's (through Envoy in containers,
    // direct in the ansible compose file).
    b.before(
        "protocol:registry",
        "configs/protocol/base.toml",
        Anchor::line(prefix("# tbd:services-end")),
        templates::snippet("protocol-service"),
        "[services.@@name@@]",
    )?;
    b.after(
        "protocol:k8s-env",
        "devops/k8s/base/configmap.yaml",
        Anchor::line(prefix("  PROTOCOL_LEDGER_URL:")),
        "  PROTOCOL_@@NAME@@_URL: http://envoy:50051",
        "PROTOCOL_@@NAME@@_URL:",
    )?;
    b.after(
        "protocol:compose-env",
        "compose.yaml",
        Anchor::line(prefix("      PROTOCOL_LEDGER_URL:")),
        "      PROTOCOL_@@NAME@@_URL: http://envoy:50051",
        "PROTOCOL_@@NAME@@_URL:",
    )?;
    b.after(
        "protocol:ansible-env",
        "devops/ansible/roles/tbd_app/templates/compose.yaml.j2",
        Anchor::line(prefix("      PROTOCOL_LEDGER_URL:")),
        "      PROTOCOL_@@NAME@@_URL: http://@@name@@:{{ @@name@@_port }}",
        "PROTOCOL_@@NAME@@_URL:",
    )?;
    b.after(
        "k8s:base",
        "devops/k8s/base/kustomization.yaml",
        Anchor::line(exact("  - protocol")),
        "  - @@name@@",
        "  - @@name@@",
    )?;
    for (overlay, tag, patch) in [
        ("local", "dev", "IfNotPresent"),
        ("dev", "dev", "Always"),
        ("prod", "v0.0.0-REPLACE", ""),
    ] {
        let path = format!("devops/k8s/overlays/{overlay}/kustomization.yaml");
        let image = template::render(
            templates::snippet("overlay-image"),
            &b.vars.clone().with("tag", tag),
        )?;
        b.push(
            &format!("k8s:overlay:{overlay}:image"),
            &path,
            Some(&service.image()),
            Edit::InsertBefore {
                anchor: Anchor::line(exact("patches:")),
                lines: image,
            },
        );
        let snippet = if patch.is_empty() {
            template::render(templates::snippet("overlay-patch-prod"), &b.vars)?
        } else {
            template::render(
                templates::snippet("overlay-patch-pull"),
                &b.vars.clone().with("pull_policy", patch),
            )?
        };
        b.push(
            &format!("k8s:overlay:{overlay}:patch"),
            &path,
            Some(&format!("      name: {name}\n")),
            Edit::InsertBefore {
                anchor: Anchor::line(exact("configMapGenerator:")),
                lines: snippet,
            },
        );
    }

    // 6. Envoy: internal LB route by service name, and the cluster.
    b.splice(
        "envoy:header",
        "devops/envoy/envoy.yaml",
        Anchor::line(prefix("# Clusters resolve by DNS name:")),
        SpliceAt::After("`protocol`,".into()),
        " `@@name@@`,",
    )?;
    b.before(
        "envoy:route",
        "devops/envoy/envoy.yaml",
        Anchor::scoped(
            exact("    - name: engine-lb"),
            exact("                        - match: { prefix: \"/\" }"),
        ),
        templates::snippet("envoy-route"),
        "/@@grpc_service@@/",
    )?;
    b.before(
        "envoy:cluster",
        "devops/envoy/envoy.yaml",
        Anchor::scoped(exact("  clusters:"), exact("    - name: chaos")),
        templates::snippet("envoy-cluster"),
        "    - name: @@name@@\n",
    )?;

    // 7. Compose.
    b.before(
        "compose:service",
        "compose.yaml",
        Anchor::line(exact("  envoy:")),
        templates::snippet("compose-service"),
        "@@package@@:",
    )?;
    b.after(
        "compose:envoy-depends",
        "compose.yaml",
        Anchor::scoped(exact("  envoy:"), exact("      - chaos")),
        "      - @@name@@",
        "      - @@name@@",
    )?;

    // 8. Ansible.
    b.after(
        "ansible:image",
        "devops/ansible/group_vars/all.yml",
        Anchor::line(prefix("protocol_image:")),
        "@@name@@_image: \"{{ registry }}/{{ image_org }}/@@package@@\"",
        "@@name@@_image:",
    )?;
    b.after(
        "ansible:port",
        "devops/ansible/group_vars/all.yml",
        Anchor::line(prefix("protocol_port:")),
        "@@name@@_port: @@port@@",
        "@@name@@_port:",
    )?;
    b.after(
        "ansible:pull",
        "devops/ansible/roles/tbd_app/tasks/main.yml",
        Anchor::line(exact("    - \"{{ protocol_image }}\"")),
        "    - \"{{ @@name@@_image }}\"",
        "@@name@@_image",
    )?;
    b.before(
        "ansible:compose",
        "devops/ansible/roles/tbd_app/templates/compose.yaml.j2",
        Anchor::line(prefix("{% if chaos_enabled")),
        templates::snippet("ansible-service"),
        "@@name@@_image",
    )?;
    let local = "devops/ansible/playbooks/local.yml";
    b.after(
        "ansible:local:build",
        local,
        Anchor::line(exact("        - { bin: protocol, port: 8080 }")),
        "        - { bin: @@name@@, port: @@port@@ }",
        "bin: @@name@@,",
    )?;
    b.splice(
        "ansible:local:import",
        local,
        Anchor::line(contains("k3d image import -m direct")),
        SpliceAt::End,
        " {{ registry }}/@@package@@:{{ tag }}",
    )?;
    b.splice(
        "ansible:local:restart",
        local,
        Anchor::line(contains(
            "rollout restart deployment/engine deployment/protocol",
        )),
        SpliceAt::End,
        " deployment/@@name@@",
    )?;
    b.after(
        "ansible:local:wait",
        local,
        Anchor::line(exact("        - deployment/protocol")),
        "        - deployment/@@name@@",
        "deployment/@@name@@",
    )?;

    // 9. CI matrices.
    b.after(
        "ci:matrix",
        ".github/workflows/ci.yml",
        Anchor::scoped(exact("  docker:"), exact("            port: \"7700\"")),
        templates::snippet("ci-matrix"),
        "- bin: @@name@@",
    )?;
    b.after(
        "ci:release",
        ".github/workflows/release.yml",
        Anchor::scoped(exact("  images:"), exact("            port: \"8080\"")),
        templates::snippet("ci-matrix"),
        "- bin: @@name@@",
    )?;

    // 10. mise and bacon.
    b.before(
        "mise:run",
        "mise.toml",
        Anchor::line(exact("[tasks.dev]")),
        templates::snippet("mise-run"),
        "[tasks.\"run:@@name@@\"]",
    )?;
    b.splice(
        "mise:dev",
        "mise.toml",
        Anchor::line(prefix("depends = [\"run:engine\"")),
        SpliceAt::Before("]".into()),
        ", \"run:@@name@@\"",
    )?;
    b.before(
        "mise:watch",
        "mise.toml",
        Anchor::line(contains("- docker ----")),
        templates::snippet("mise-watch"),
        "[tasks.\"watch:@@name@@\"]",
    )?;
    b.after(
        "mise:docker-build",
        "mise.toml",
        Anchor::line(contains("tbd-chaos:$IMAGE_TAG .\",")),
        "  \"docker build -f devops/docker/Dockerfile --build-arg BIN=@@name@@ --build-arg PORT=@@port@@ -t $IMAGE_REGISTRY/@@package@@:$IMAGE_TAG .\",",
        "$IMAGE_REGISTRY/@@package@@:$IMAGE_TAG .",
    )?;
    b.after(
        "mise:docker-push",
        "mise.toml",
        Anchor::line(contains(
            "docker push $IMAGE_REGISTRY/tbd-chaos:$IMAGE_TAG\"",
        )),
        "  \"docker push $IMAGE_REGISTRY/@@package@@:$IMAGE_TAG\",",
        "docker push $IMAGE_REGISTRY/@@package@@:",
    )?;
    b.after(
        "mise:local-build",
        "mise.toml",
        Anchor::line(contains("tbd-chaos:dev .")),
        "docker build -f devops/docker/Dockerfile --build-arg BIN=@@name@@ --build-arg PORT=@@port@@ -t $IMAGE_REGISTRY/@@package@@:dev .",
        "$IMAGE_REGISTRY/@@package@@:dev .",
    )?;
    b.splice(
        "mise:local-import",
        "mise.toml",
        Anchor::scoped(
            exact("[tasks.\"local:build\"]"),
            contains("k3d image import -m direct"),
        ),
        SpliceAt::After("$IMAGE_REGISTRY/tbd-chaos:dev".into()),
        " $IMAGE_REGISTRY/@@package@@:dev",
    )?;
    b.splice(
        "mise:local-deploy",
        "mise.toml",
        Anchor::line(contains(
            "rollout status deployment/engine deployment/protocol deployment/envoy deployment/chaos",
        )),
        SpliceAt::After("deployment/chaos".into()),
        " deployment/@@name@@",
    )?;
    b.splice(
        "mise:local-restart",
        "mise.toml",
        Anchor::line(contains(
            "rollout restart deployment/engine deployment/protocol deployment/chaos",
        )),
        SpliceAt::End,
        " deployment/@@name@@",
    )?;
    b.splice(
        "mise:local-restart-status",
        "mise.toml",
        Anchor::scoped(
            exact("[tasks.\"local:restart\"]"),
            contains("rollout status deployment/engine deployment/protocol deployment/chaos"),
        ),
        SpliceAt::After("deployment/chaos".into()),
        " deployment/@@name@@",
    )?;
    b.before(
        "bacon:job",
        "bacon.toml",
        Anchor::line(exact("[keybindings]")),
        templates::snippet("bacon-job"),
        "[jobs.@@name@@]",
    )?;
    b.after(
        "bacon:key",
        "bacon.toml",
        Anchor::line(prefix("p = \"job:protocol\"")),
        "@@bacon_key@@ = \"job:@@name@@\"",
        "= \"job:@@name@@\"",
    )?;

    // 11. Documentation rows.
    b.after(
        "docs:local-cluster",
        "docs/local-cluster.md",
        Anchor::line(prefix("| `tbd` | `engine` |")),
        "| `tbd` | `@@name@@` | 1 | gRPC `@@name@@` service, scaffolded by `tbd new service`; a stub until its RPCs land |",
        "| `tbd` | `@@name@@` |",
    )?;
    b.after(
        "docs:architecture:crate",
        "ARCHITECTURE.md",
        Anchor::line(prefix("| `tbd-engine` |")),
        "| `@@package@@` | `@@grpc_service@@` implementation, health, reflection; scaffolded by `tbd new service`, a stub until its RPCs land | common, proto |",
        "| `@@package@@` |",
    )?;
    b.after(
        "docs:architecture:route",
        "ARCHITECTURE.md",
        Anchor::line(prefix("| gRPC `/tbd.engine.v1.EngineService/*` |")),
        "| internal LB (50051) gRPC `/@@grpc_service@@/*` | @@name@@ | none, retries on connect failure and `UNAVAILABLE` |",
        "`/@@grpc_service@@/*`",
    )?;
    b.after(
        "docs:readme:piece",
        "README.md",
        Anchor::line(prefix("| **engine** |")),
        "| **@@name@@** | gRPC service scaffolded by `tbd new service`: health, reflection, metrics, one labelled-stub `Ping` until its real RPCs land. Port @@port@@. |",
        "| **@@name@@** |",
    )?;
    let tree = format!(
        "  {:<12}the {name} service (lib + bin + tests/it)",
        format!("{name}/")
    );
    b.push(
        "docs:readme:tree",
        "README.md",
        Some(&format!("  {name}/ ")),
        Edit::InsertAfter {
            anchor: Anchor::line(prefix("  engine/     ")),
            lines: tree,
        },
    );
    b.before(
        "docs:index",
        "docs/README.md",
        Anchor::line(prefix("| A check failed in CI")),
        "| You are working on the `@@name@@` service | [../crates/@@name@@/CLAUDE.md](../crates/@@name@@/CLAUDE.md) |",
        "| You are working on the `@@name@@` service |",
    )?;
    b.splice(
        "docs:k8s:tree",
        "devops/k8s/README.md",
        Anchor::line(prefix("base/            engine, protocol")),
        SpliceAt::After("protocol".into()),
        ", @@name@@",
    )?;
    for overlay in ["local", "dev"] {
        b.splice(
            &format!("docs:k8s:overlay:{overlay}"),
            "devops/k8s/README.md",
            Anchor::line(prefix(&format!("| `{overlay}` | engine "))),
            SpliceAt::After("envoy 2".into()),
            ", @@name@@ 1",
        )?;
    }
    b.after(
        "docs:ci:image",
        "docs/ci.md",
        Anchor::line(exact("- `ghcr.io/<ORG>/tbd-chaos`")),
        "- `ghcr.io/<ORG>/@@package@@`",
        "`ghcr.io/<ORG>/@@package@@`",
    )?;

    Ok(b.out)
}

/// The steps the scaffold cannot do safely and prints as a checklist.
pub const CHECKLIST: &[&str] = &[
    "cargo check -p @@package@@                      # updates Cargo.lock",
    "mise run chaos:docs                              # docs/chaos/kinds.md gains the @@name@@ kind",
    "kustomize build devops/k8s/overlays/local > /dev/null && docker compose config -q && mise run envoy:validate",
    "mise run ci",
];
