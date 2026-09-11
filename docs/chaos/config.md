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
| `serve.base_path` | `/api/chaos/v1` | `--base-path`, `CHAOS_BASE_PATH` | prefix of every API route |
| `serve.ui_dir` | `""` | `--ui-dir`, `CHAOS_UI_DIR` | built UI to serve; empty for none |
| `auth.token` | `""` | `--token`, `CHAOS_TOKEN` | a fixed bearer token for validate and load runs; wins over the fields below |
| `auth.token_url` | `""` | `--auth-token-url`, `CHAOS_AUTH_TOKEN_URL` | OAuth2 token endpoint for the client-credentials grant; empty sends no token |
| `auth.client_id` | `tbd-chaos` | `--auth-client-id`, `CHAOS_AUTH_CLIENT_ID` | client id |
| `auth.client_secret` | `""` | `--auth-client-secret`, `CHAOS_AUTH_CLIENT_SECRET` | client secret; environment only, never a file |
| `auth.scope` | `tbd.api` | | requested scope |
| `auth.audience` | `tbd-api` | | requested audience (Envoy checks it) |
| `serve.ui_path` | `""` (root) | | where the UI is served; `""` is the root of the host, or a prefix such as `/chaos` |
| `serve.start_stack` | `true` | `--no-stack` | run `paths.topology` in-process on start |
| `paths.topology` | `topologies/dev.toml` | `--topology`, `CHAOS_TOPOLOGY` | stack for `up` and `serve` |
| `paths.scenarios` | `scenarios` | `--scenarios`, `CHAOS_SCENARIOS_DIR` | scenario files the API lists and edits |
| `paths.scenarios_seed` | `""` | `--scenarios-seed`, `CHAOS_SCENARIOS_SEED` | on every `serve` start each `*.toml` under it that `paths.scenarios` does not hold yet is copied there, existing files are never overwritten; containers set it to the image's read-only `scenarios/` and point `paths.scenarios` at a volume |
| `paths.results` | `.chaos/results` | `--results`, `CHAOS_RESULTS_DIR` | run records |
| `paths.stack` | `.chaos/stack.json` | `--stack-file`, `CHAOS_STACK_FILE` | instances added to the serve stack at runtime (replicas, new instances of any kind, [api.md](api.md#stack)); re-added on start in dependency order; one that no longer starts is dropped with a warning; containers put it on `/data` |
| `paths.campaigns` | `stress` | `--campaigns`, `CHAOS_CAMPAIGNS_DIR` | stress campaign files the API lists and edits ([stress.md](stress.md)) |
| `paths.campaigns_seed` | `""` | `--campaigns-seed`, `CHAOS_CAMPAIGNS_SEED` | seeded into `paths.campaigns` on every `serve` start like `scenarios_seed`; containers set it to the image's `stress/` |
| `paths.findings` | `.chaos/findings` | `--findings`, `CHAOS_FINDINGS_DIR` | findings from stress runs, one JSON file each; `chaos stress run` and `replay` default to it; containers put it on `/data` |
| `paths.schedules` | `.chaos/schedules.json` | `--schedules`, `CHAOS_SCHEDULES_FILE` | the schedules file (`chaos serve` cron jobs, [api.md](api.md#schedules)); created on first write; containers put it on the `/data` volume next to the results |
| `targets.<kind>` | the kind's default ([kinds.md](kinds.md)): `protocol` `http://127.0.0.1:8080`, `engine` `http://127.0.0.1:50051`, `ledger` `http://127.0.0.1:50052` | `--target <kind>=URL`, `CHAOS_<KIND>_URL` (`CHAOS_TARGETS` for several) | default for `validate` and API validate, one URL per kind with a validate target; a key that is not such a kind fails the load; through Envoy the engine and the ledger share the internal listener `http://envoy:50051`, matched by service name |
| `validate.timeout` | `5s` | `--timeout` | per-check timeout |
| `validate.ca_cert` | `""` | `--ca-cert`, `CHAOS_CA_CERT` | extra PEM root for `https://` / `wss://` targets; empty means the public roots only |
| `notify.slack.webhook` | `""` | `--slack-webhook`, `CHAOS_SLACK_WEBHOOK` | Slack incoming webhook; environment only, never a file; empty means off |
| `notify.slack.channel` | `""` | | channel shown in the UI and sent with the message (app webhooks ignore it, they are bound to a channel; legacy ones honour it); `#chaos-local`, `#chaos-dev`, `#chaos-prod` per environment |
| `notify.slack.on` | `["failed", "error"]` | | outcomes that post; production adds `cancelled` |
| `notify.slack.kinds` | `["scenario", "load", "validate"]` | | kinds that post |
| `links.domain` | `""` | `--public-domain`, `CHAOS_PUBLIC_DOMAIN` | public base domain; when set the links below are derived from the edge's hosts (`grafana.`, `logs.`, `profiles.`, `metrics.`) and `envoy_admin` is cleared |
| `links.grafana` | `""` | | UI link when there is no domain; empty hides it |
| `links.victorialogs` | `""` | | same |
| `links.metrics` | `""` | | same |
| `links.pyroscope` | `""` | | same |
| `links.envoy_admin` | `""` | | same; never derived, the edge does not expose it |
| `links.chaos` | `""` | | the public admin UI root (`https://chaosadmin.<domain>` when derived; `http://chaos.localhost:18080` in `local.toml`): run links in Slack messages |
| `links.auth` | `""` | | the sign-in host (`https://auth.<domain>` when derived): "sign out everywhere" in the user menu |

`serve.base_path` must start with `/` and not end with one; `serve.ui_path` is empty (the
root) or the same shape; they must differ.
`targets.*` must be absolute URLs and their keys registered kinds with a validate target;
`chaos config` and `GET /overview` print every such kind, defaults filled in.

## Per environment

`local.toml` adds only the links (`http://localhost:3000` and friends, the port map of
[docs/local-cluster.md](../local-cluster.md)). Behind the public edge
(`devops/edge`) those are wrong for anyone but this machine, so `mise run local:deploy`
and `local:restart` copy `BASE_DOMAIN` from `devops/edge/.env` into
`devops/k8s/overlays/local/edge.env`, which becomes the `tbd-edge` ConfigMap and sets
`CHAOS_PUBLIC_DOMAIN` on the chaos pod. The UI then links to `https://grafana.<domain>`
and friends. Targets stay at `127.0.0.1:8080` and
`:50051`, which is both `chaos up` and the compose stack; `cluster.toml` is the k3d
cluster seen from the host (`http://localhost:18080`, `http://localhost:15051`), so
`TBD_ENV=cluster mise run validate` checks it, as `mise run local:traffic` does.

In the cluster the chaos pod keeps `/data` (run records, the editable scenario copy,
`schedules.json`, `stack.json`) on a PersistentVolumeClaim (`devops/k8s/chaos/pvc.yaml`),
so restarts and image rolls keep them; the chaos pod runs with `TBD_ENV` from the `tbd-env` ConfigMap (`local`
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
