# chaos serve: the HTTP API

`chaos serve` keeps the tool running as an HTTP API so the admin UI (`ui/chaos`) and
scripts can do everything the CLI does, plus hold a stack up between calls and follow
runs live. This page is the contract. Every path below is relative to
`[serve] base_path`, default `/api/chaos/v1`; through Envoy that is
`http://localhost:18080/api/chaos/v1/...` locally and `chaos.api.<domain>/v1/...` where a
domain exists (see [devops/envoy/README.md](../../devops/envoy/README.md)).

```
chaos serve                          # 127.0.0.1:7700, dev topology in-process
curl -s localhost:7700/api/chaos/v1/overview | jq .
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
  "queue": [ "…QueuedRun, front first" ],
  "recent_runs": [ "…RunSummary, newest first, at most 10" ],
  "last_validate": "…RunSummary or null",
  "scenarios": 4,
  "runs": 17,
  "schedules": 2,
  "schedules_enabled": 1,
  "next_schedule": "…Schedule with the earliest next_at, or null"
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
| `queue_changed` | `{"type", "queue": [QueuedRun]}` | something was queued, started or removed |
| `schedules_changed` | `{"type", "schedules": [Schedule]}` | a schedule was created, changed, deleted, fired or skipped |

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
Validate defaults to `[targets]` in the config. Both validate and explicit-target load
runs send the bearer token from `[auth]` (client credentials, fetched per run); in the
cluster the chaos pod has the `tbd-chaos` client's secret from the `chaos-auth` Secret.
The API itself is reachable only through Envoy's gate: `chaosadmin.<domain>` after the
browser login, or with a bearer token on the request.

Scenario runs start their own stack on free ports, exactly like `chaos run`; the serve
stack is untouched.

## Queue

The queue is what waits for the single run slot: "run all scenarios", a list pushed by
a script, and whatever a schedule fires. Items start in order, each when the previous
run ends; a `POST /runs` made by hand takes the slot when it is free and the queue
waits behind it. The queue is in memory: it does not survive a restart.

| Method and path | Body | Returns |
|---|---|---|
| `GET /queue` | | `[QueuedRun]`, front first |
| `POST /queue` | `{"jobs": [Job]}` | `202` + `[QueuedRun]`, one per job after expansion; the first starts at once when the slot is free |
| `DELETE /queue/{id}` | | `204`; `404` if it is not waiting (it may have started) |
| `DELETE /queue` | | `204`, drops everything waiting; the active run keeps going |

A `Job` is one of:

| JSON | Runs |
|---|---|
| `{"scenario": "<id>"}` | that scenario; `404` before anything is queued when it does not exist |
| `"all_scenarios"` | every scenario that checks and is not skipped, in id order, expanded into one item each; `422` when there is none |
| `{"load": {…}}` | an ad-hoc load run, the same body as `POST /runs`; checked before queuing |
| `{"validate": {…}}` | a validate run, the same body as `POST /validate`; validate does not take the slot, so it runs alongside whatever is active |

`QueuedRun`: `id`, `job`, `kind`, `name`, `scenario_id`, `schedule_id` (set when a
schedule queued it), `queued_at`. A queued job that cannot start when its turn comes
(its scenario was deleted meanwhile) is recorded as an `error` run so the failure is
visible, and the next item is tried.

## Schedules

A schedule queues its job on a cron expression, UTC. Five fields (`*/15 * * * *`), six
with leading seconds, or a nickname (`@hourly`, `@daily`). Serve checks once a second.
A schedule that falls due while its previous job is still queued or running is skipped
and counted, never stacked: a cron faster than the run it starts does not pile up.
Schedules are kept in `[paths] schedules` (one JSON file) and survive a restart;
`next_at` is recomputed from the start time, so fires missed while serve was down are
dropped, not replayed.

| Method and path | Body | Returns |
|---|---|---|
| `GET /schedules` | | `[Schedule]`, by id |
| `POST /schedules` | `ScheduleSpec` | `201` + `Schedule`; `422` for a cron that does not parse, an empty name, or a load job that does not check |
| `GET /schedules/{id}` | | `Schedule` |
| `PUT /schedules/{id}` | `ScheduleSpec` | `Schedule`; name, cron, job and enabled are replaced, counters stay |
| `DELETE /schedules/{id}` | | `204` |
| `POST /schedules/{id}/run` | | `202` + `[QueuedRun]`: queue its job now, whatever the cron says, enabled or not |

`ScheduleSpec`: `{"name", "cron", "job": Job, "enabled": true}` (`enabled` defaults to
`true`). `Schedule` adds `id`, `created_at`, `updated_at`, `next_at` (`null` when
disabled), `last_fired_at`, `last_skipped_at`, `fired`, `skipped`. Runs started by a
schedule carry its id in `schedule_id`, so `GET /runs` filtered on it is the
schedule's history.

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
  "schedule_id": null,
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
| `schedule_id` | the schedule that queued it, or `null` for runs started by hand or by `POST /queue` |
| `scenario` | the [`chaos run --json`](commands.md#chaos-run) object, for scenario runs |
| `load` | the final `LoadSnapshot`, for load runs |
| `validate` | the [`chaos validate --json`](commands.md#chaos-validate) report |
| `samples` | one `LoadSnapshot` per second, for charts; the last one equals the final snapshot |
| `events` | timeline actions as applied |
| `request` | what started it: the load request, or validate's targets |

`RunSummary` is the list view: `id`, `kind`, `name`, `scenario_id`, `schedule_id`, `status`,
`started_at`, `finished_at`, `duration_s`, `requests_total`, `error_rate`,
`throughput_rps`, `p50_ms`, `p90_ms`, `p99_ms`, `passed` (`[passed, total]` assertions or
checks), `error`.

Records are JSON files, one per run, under `[paths] results` (default
`.chaos/results/`, gitignored). Serve indexes the directory on start; a record still
`running` from a process that died is marked `error: interrupted`.

## UI

With `[serve] ui_dir` (or `--ui-dir`) pointing at the built UI (`ui/chaos/out`), serve
also serves it at `[serve] ui_path` (default `""`, the root of the host) with `index.html` as the fallback
for client-side routes. `mise run chaos:serve` passes `--ui-dir ui/chaos/out`.

## Through Envoy

Envoy forwards `/api/chaos/` (every version) on the edge listener, on any host, to the `chaos` cluster
with no stream timeout, so SSE works through it. The UI has its own virtual host,
`chaos.*` or `chaosadmin.*`, served at the root (`http://chaos.localhost:18080/` in the
local cluster; browsers resolve `*.localhost` to loopback by themselves), and
`chaos.api.*` maps a dedicated host to the API prefix. In the local cluster:

```sh
curl -s http://localhost:18080/api/chaos/v1/overview | jq .stack
curl -s -N http://localhost:18080/api/chaos/v1/events
```
