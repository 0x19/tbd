# chaos serve: the HTTP API

`chaos serve` keeps the tool running as an HTTP API so the admin UI (`ui/chaos`) and
scripts can do everything the CLI does, plus hold a stack up between calls and follow
runs live. This page is the contract. Every path below is relative to
`[serve] base_path`, default `/api/chaos`; through Envoy that is
`http://localhost:18080/api/chaos/...` locally and `chaos.api.<domain>/...` where a
domain exists (see [devops/envoy/README.md](../../devops/envoy/README.md)).

```
chaos serve                          # 127.0.0.1:7700, dev topology in-process
curl -s localhost:7700/api/chaos/overview | jq .
```

Errors are `{"error": "<message>"}` with `404` (unknown instance, scenario or run),
`409` (a run is already active; serve has no stack; the run is not active),
`422` (bad body, a scenario that does not check, an instance without fault injection)
or `500`. Every body is JSON; `PUT`/`POST` bodies reject unknown fields.

## Overview and events

| Method and path | Returns |
|---|---|
| `GET /healthz` | `{"status":"ok","version":"0.1.0"}` |
| `GET /overview` | everything the overview page needs, in one call (below) |
| `GET /events` | Server-Sent Events: the global feed (below) |

`GET /overview`:

```json
{
  "version": "0.1.0",
  "env": "local",
  "config_files": ["configs/chaos/base.toml", "configs/chaos/local.toml"],
  "config": { "serve": {}, "paths": {}, "targets": {}, "validate": {}, "links": {} },
  "stack": [ { "name": "engine-1", "kind": "engine", "addr": "127.0.0.1:50051",
               "running": true, "depends_on": [], "behavior": {"type": "healthy"},
               "requests": {"total": 12, "failed": 0} } ],
  "active_run": null,
  "recent_runs": [ "…RunSummary, newest first, at most 10" ],
  "last_validate": "…RunSummary or null",
  "scenarios": 4,
  "runs": 17
}
```

`config` is the effective configuration ([config.md](config.md)); `config.links` holds
the Grafana, VictoriaLogs, VictoriaMetrics, Pyroscope and (locally) Envoy admin URLs the
UI links to, derived from `[links] domain` behind the public edge. `stack` is
`null` when serve runs with `--no-stack`.

`GET /events` frames, `event:` is the `type`, `data:` the JSON:

| `type` | `data` | When |
|---|---|---|
| `run_started` | `{"type", "run": RunSummary}` | a scenario, load or validate run begins |
| `run_finished` | `{"type", "run": RunSummary}` | it ends |
| `stack_changed` | `{"type", "instances": [InstanceInfo]}` | after every stack call below |

## Stack

The stack from `[paths] topology` runs in-process for as long as serve does. These calls
are the `[[timeline]]` actions, by hand.

| Method and path | Body | Returns |
|---|---|---|
| `GET /stack` | | `[InstanceInfo]` |
| `POST /stack/{name}/stop` | | `[InstanceInfo]` |
| `POST /stack/{name}/start` | | `[InstanceInfo]`; same port as before |
| `PUT /stack/{name}/behavior` | a behaviour, e.g. `{"type":"error","kind":"unavailable","rate":0.5,"message":"x"}` | `[InstanceInfo]` |

Behaviours are the same objects as in `[[timeline]]`, in JSON: `healthy`, `slow`,
`hang`, `error`, `delayed_failure` ([scenarios.md](scenarios.md#behaviours)). Only
engines have fault injection; a protocol answers `422`.

`InstanceInfo`: `name`, `kind` (`engine`/`protocol`), `addr`, `running`,
`depends_on`, `behavior` (`null` when stopped or without fault injection),
`requests` (`{"total","failed"}`, engine-side counters, reset on restart).

## Scenarios

Scenario ids are the path under `[paths] scenarios` without `.toml`, so
`scenarios/baseline.toml` is `baseline` and `scenarios/ws/burst.toml` is `ws/burst`. Ids
take letters, digits, `_`, `-`, `.` and `/`; `..` is rejected.

| Method and path | Body | Returns |
|---|---|---|
| `GET /scenarios` | | `[ScenarioEntry]`, sorted by id |
| `GET /scenarios/{id}` | | `ScenarioEntry` + `text` (the TOML), `parsed` (the file as JSON, `null` if it does not parse), `last_run` (`RunSummary` or `null`) |
| `PUT /scenarios/{id}` | `{"text": "<toml>"}` | `ScenarioEntry`; the file is written only if it checks, else `422` with the reason |
| `DELETE /scenarios/{id}` | | `204` |
| `POST /scenarios/check` | `{"text": "<toml>"}` | `{"ok", "name", "error", "parsed"}`; never writes |

`ScenarioEntry`: `id`, `file`, `name`, `description`, `skip`, `ok` (parses and passes
`chaos check`), `error`.

Pushing a scenario from the UI is `POST /scenarios/check` while editing, then
`PUT /scenarios/{id}` to save, then `POST /runs` to run it. Saved files are ordinary
files in `scenarios/`: commit the ones worth keeping. In a container the image's
`scenarios/` is read-only, so serve works on a copy seeded into a volume
(`[paths] scenarios_seed`, [config.md](config.md)); copy the TOML out of the UI to
bring a scenario back into the repo.

## Runs

One run at a time. A second `POST /runs` while one is active answers `409`.

| Method and path | Body | Returns |
|---|---|---|
| `POST /runs` | `{"scenario": "<id>"}` | `202` + `RunSummary` (status `running`) |
| `POST /runs` | `{"name": "adhoc", "targets": [{"name","http_url"}], "load": {…}}` | `202` + `RunSummary` |
| `GET /runs?limit=50` | | `[RunSummary]`, newest first |
| `GET /runs/{id}` | | `RunRecord` |
| `GET /runs/{id}/events` | | SSE: history so far, then live, ending at `finished` |
| `POST /runs/{id}/cancel` | | `202`; `409` if not active |
| `DELETE /runs/{id}` | | `204`; `409` if active |
| `POST /validate` | `{"protocol", "engine", "timeout"}`, every field optional | `200` + `RunRecord` (kind `validate`), synchronous |

An ad-hoc load run takes the `[load]` table of a scenario as JSON (`rate`, `duration`,
`warmup`, `timeout`, `max_in_flight`, `pattern`, `operations`; see
[scenarios.md](scenarios.md#load)). `targets` defaults to every running protocol in the
serve stack; give explicit targets to load a stack serve did not start, such as the
cluster through Envoy: `[{"name":"envoy","http_url":"http://localhost:18080"}]`.
Validate defaults to `[targets]` in the config.

Scenario runs start their own stack on free ports, exactly like `chaos run`; the serve
stack is untouched.

### `GET /runs/{id}/events`

| `type` | `data` | |
|---|---|---|
| `started` | `{"run": RunSummary}` | first frame |
| `phase` | `{"id", "name"}` | `setup`, `load`, `assert`, `teardown` (scenario runs) |
| `load` | `{"id", "snapshot": LoadSnapshot}` | once per second while load runs, plus one at the end of each phase |
| `timeline` | `{"id", "event": {"at_s", "action", "error"}}` | a timeline action fired |
| `finished` | `{"run": RunRecord}` | last frame; the stream ends |

A client that connects late gets everything from `started` on, then live frames. A
finished run answers with its `finished` frame only.

### `RunRecord`

```json
{
  "id": "01a08bac-a9fd-7520-bc58-bee7e350fdf9",
  "kind": "scenario",
  "name": "error_injection",
  "scenario_id": "error_injection",
  "status": "passed",
  "started_at": "2026-09-10T14:15:43.869Z",
  "finished_at": "2026-09-10T14:15:47.532Z",
  "duration_s": 3.66,
  "scenario": { "…the chaos run --json object for this scenario" },
  "load": null,
  "validate": null,
  "samples": [ "…LoadSnapshot per second" ],
  "events": [ { "at_s": 1.0, "action": "set_behavior engine-1 Error { .. }", "error": null } ],
  "request": null,
  "error": null
}
```

| Field | Meaning |
|---|---|
| `id` | UUID v7, so ids sort by time |
| `kind` | `scenario`, `load`, `validate` |
| `status` | `running`, `passed`, `failed`, `error` (setup or a timeline action failed), `cancelled`, `completed` (load runs: nothing to pass or fail) |
| `scenario` | the [`chaos run --json`](commands.md#chaos-run) object, for scenario runs |
| `load` | the final `LoadSnapshot`, for load runs |
| `validate` | the [`chaos validate --json`](commands.md#chaos-validate) report |
| `samples` | one `LoadSnapshot` per second, for charts; the last one equals the final snapshot |
| `events` | timeline actions as applied |
| `request` | what started it: the load request, or validate's targets |

`RunSummary` is the list view: `id`, `kind`, `name`, `scenario_id`, `status`,
`started_at`, `finished_at`, `duration_s`, `requests_total`, `error_rate`, `p99_ms`,
`passed` (`[passed, total]` assertions or checks), `error`.

Records are JSON files, one per run, under `[paths] results` (default
`.chaos/results/`, gitignored). Serve indexes the directory on start; a record still
`running` from a process that died is marked `error: interrupted`.

## UI

With `[serve] ui_dir` (or `--ui-dir`) pointing at the built UI (`ui/chaos/out`), serve
also serves it at `[serve] ui_path` (default `/chaos`) with `index.html` as the fallback
for client-side routes. `mise run chaos:serve` passes `--ui-dir ui/chaos/out`.

## Through Envoy

Envoy forwards `/api/chaos/` and `/chaos/` on the edge listener to the `chaos` cluster
with no stream timeout, so SSE works through it. The virtual hosts `chaos.api.*` and
`chaos.*` map a dedicated host to the same two prefixes. In the local cluster:

```sh
curl -s http://localhost:18080/api/chaos/overview | jq .stack
curl -s -N http://localhost:18080/api/chaos/events
```
