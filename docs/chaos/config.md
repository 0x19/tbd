# Configuration: `configs/chaos/`

chaos reads its own configuration from TOML files layered per environment. The
mechanism is generic (`tbd_common::config`) and is the template for the other
binaries: a directory per binary under `configs/`, `base.toml` plus one file per
environment.

```
configs/chaos/
├── base.toml         every key, with the defaults for a developer machine
├── local.toml        this machine: links to the local cluster's Grafana etc.
├── dev.toml          the dev cluster: targets are Envoy inside the cluster
└── production.toml   operator's machine pointing at production; serve never runs there
```

## How loading works

1. `base.toml` is read. It must exist and it lists every key.
2. `<env>.toml` is read and merged over it. Tables merge key by key; any other value
   in the environment file replaces the base value, arrays included.
3. The result is checked against the schema: unknown keys are an error.
4. Flags and their environment variables override individual fields.

Precedence, lowest to highest: `base.toml` < `<env>.toml` < flag or env var.

The environment comes from `--env` or `TBD_ENV`, default `local`. The directory comes
from `--config-dir` or `CHAOS_CONFIG_DIR`, default `configs/chaos`. A missing
environment file is an error, not a fallback: a typo cannot start the tool with base
defaults. `base` is not a valid environment name.

`chaos config` prints the effective configuration for the environment, with the files
it came from as comments. `mise run chaos:config` does the same with the repo's
`TBD_ENV`.

```sh
chaos config                       # local
chaos --env dev config             # dev
TBD_ENV=production chaos config    # production
```

## Keys

| Key | Default (`base.toml`) | Overridden by | Meaning |
|---|---|---|---|
| `serve.listen` | `127.0.0.1:7700` | `--listen`, `CHAOS_LISTEN_ADDR` | where `chaos serve` binds |
| `serve.base_path` | `/api/chaos` | `--base-path`, `CHAOS_BASE_PATH` | prefix of every API route |
| `serve.ui_dir` | `""` | `--ui-dir`, `CHAOS_UI_DIR` | built UI to serve; empty for none |
| `serve.ui_path` | `/chaos` | | where the UI is served |
| `serve.start_stack` | `true` | `--no-stack` | run `paths.topology` in-process on start |
| `paths.topology` | `topologies/dev.toml` | `--topology`, `CHAOS_TOPOLOGY` | stack for `up` and `serve` |
| `paths.scenarios` | `scenarios` | `--scenarios`, `CHAOS_SCENARIOS_DIR` | scenario files the API lists and edits |
| `paths.scenarios_seed` | `""` | `--scenarios-seed`, `CHAOS_SCENARIOS_SEED` | copied into `paths.scenarios` on `serve` start when it is missing or empty; containers set it to the image's read-only `scenarios/` and point `paths.scenarios` at a volume |
| `paths.results` | `.chaos/results` | `--results`, `CHAOS_RESULTS_DIR` | run records |
| `targets.protocol` | `http://127.0.0.1:8080` | `--protocol`, `CHAOS_PROTOCOL_URL` | default for `validate` and API validate |
| `targets.engine` | `http://127.0.0.1:50051` | `--engine`, `CHAOS_ENGINE_URL` | same, engine gRPC |
| `validate.timeout` | `5s` | `--timeout` | per-check timeout |
| `validate.ca_cert` | `""` | `--ca-cert`, `CHAOS_CA_CERT` | extra PEM root for `https://` / `wss://` targets; empty means the public roots only |
| `links.grafana` | `""` | | UI link; empty hides it |
| `links.victorialogs` | `""` | | UI link |
| `links.pyroscope` | `""` | | UI link |
| `links.envoy_admin` | `""` | | UI link |

`serve.base_path` and `serve.ui_path` must start with `/`, not end with one, and differ.
`targets.*` must be absolute URLs.

## Per environment

`local.toml` adds only the links (`http://localhost:3000` and friends, the port map of
[docs/local-cluster.md](../local-cluster.md)). Targets stay at `127.0.0.1:8080` and
`:50051`, which is both `chaos up` and the compose stack; against the k3d cluster pass
`--protocol http://localhost:18080 --engine http://localhost:15051`, as `mise run
local:traffic` does.

In the cluster the chaos pod runs with `TBD_ENV` from the `tbd-env` ConfigMap (`local`
in the local overlay, `dev` in dev) and the target env vars set to Envoy
(`CHAOS_PROTOCOL_URL=http://envoy:8080`), so `POST /validate` with no body checks the
deployed stack through the load balancer.

`dev.toml` and `production.toml` carry placeholder hosts (`*.example.invalid`) until the
domains exist; replace them in the same commit that creates the DNS names.

## Adding a key

1. Add the field to the struct in `crates/chaos/src/config.rs` and to every table it
   belongs to in `base.toml` with the developer default. Every key is in `base.toml`;
   environment files only carry differences.
2. If a flag should override it, add the flag to `serve` (or the global flags) in
   `crates/chaos/src/main.rs` with `env = "CHAOS_..."` and apply it after `load`.
3. Add the row above, and the env var to `.env.example`, `compose.yaml`,
   `devops/k8s/chaos/deployment.yaml` and the ansible template when it is set there.

## Using the loader in another binary

```rust
let loaded = tbd_common::config::load::<MyConfig>(Path::new("configs/engine"), &env)?;
// loaded.value: MyConfig, loaded.files: the two paths merged
```

`MyConfig` should be `#[serde(deny_unknown_fields)]` on every table so a misspelt key
fails at start. `tbd_common::config::merge` exposes the merge for tools that want to show
an effective document without a type.
