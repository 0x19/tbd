# The llm service

The layer over whatever runs the weights. Engines (Ollama, llama.cpp's `llama-server`,
later our own) are the **L1**: they hold a model and turn a prompt into tokens. The
`llm` service is the **L2**: it owns the contract, the caller's identity and daily
budget, the choice of tier, the record of every generation, the metrics and the
chaos checks. Nothing above the `[engines]` table in its configuration, and nothing in
its proto, names an engine; every engine passes one conformance suite; a new engine
is done when it passes it, not when it worked once.

## The flow

1. A caller reaches `Generate` through the protocol (`POST /v1/llm/generate/events`,
   the answer as server-sent events, or the same RPC by name on `/v1/ws`). Envoy has
   verified who they are; the service reads the forwarded payload and refuses a call
   without one (`UNAUTHENTICATED`).
2. The request names a **tier**: `fast`, the model that fits the GPU and answers a
   visitor, or `deep`, the large model that runs from memory for quality. Unspecified
   is `[engines] default_tier`. The request is bounded by `[generate]` (messages, bytes,
   `max_tokens` clamped to the cap); over is `INVALID_ARGUMENT`.
3. With a store, the caller's tokens spent today (UTC) are summed; over
   `[budget] tokens_per_day` is `RESOURCE_EXHAUSTED`, naming the count and the reset.
   One generation in flight may overrun: tokens are known only when it ends. Then a
   session row is minted or, if the request named one, touched (a session that is not
   the caller's is `NOT_FOUND`, the same answer as one that does not exist), and a
   generation row starts as `running`.
4. The tier's engine is asked. Its `timeout_secs` is the deadline from admission and
   covers everything: an engine that has not started answering is
   `DEADLINE_EXCEEDED` as the RPC's status; one that stops mid-way is
   `DEADLINE_EXCEEDED` as the last item of the stream.
5. Chunks stream back, each with `text`, a contiguous `index`, `engine`, `model`,
   `tier`, `stub`, and the session and generation ids (empty without a store). The
   last has `done` and the engine's `usage`. An engine failing **before** the first
   chunk is the RPC's status (`UNAVAILABLE` when it cannot be reached or answers 5xx,
   `FAILED_PRECONDITION` when it refuses: an unknown model, a feature it was not
   started with); failing **after** it is one error item that ends the stream.
6. The row ends `ok` with the counts and the time to the first token, `failed` with
   the reason, or `cancelled` when the caller went away first. The counts feed the
   metrics and tomorrow's budget.

**Reasoning.** A model that reasons before it answers (the fast tier's candidate does)
streams its reasoning first. The service passes it on as chunks marked `reasoning`, so a
client can show or hide it and the answer is the chunks without it; the request's
`reasoning` (unset, true, false) turns that on or off where the engine allows (Ollama's
`think`, llama-server's `enable_thinking`). Reasoning spends the token budget before any
answer appears: a short `max_tokens` with reasoning on can end with a done chunk and no
answer, which the load operation counts (`answered`) and the studies report rather than
hide.

`Embed` (`POST /v1/llm/embed`) returns one vector per input from the tier's embedding
model, not budgeted in this phase. `ListModels` (`GET /v1/llm/models`, no caller
needed: the demo's readiness) names each tier's engine and model and whether the
engine answered its last probe. `GetBudget` (`GET /v1/llm/budget`) is the caller's
day: `used_today`, `remaining`, and two flags that keep it honest, `recorded` (a store
exists, the count is real) and `unlimited` (no limit is configured, or nothing is
recorded).

## What is where

| Piece | Where |
|---|---|
| The service | `crates/llm` (`crates/llm/CLAUDE.md`), proto `proto/tbd/llm/v1/llm.proto`, REST through the protocol under `/v1/llm/` |
| The engines | `crates/llm/src/engine/`: the trait, `ollama.rs`, `llamacpp.rs`, `stub.rs`; one match on the kind, in `build` |
| The tiers | `configs/llm/base.toml` `[engines.fast]` and `[engines.deep]`: `kind`, `url`, `model`, `timeout_secs`, `embed_model`; URLs and models per environment through `LLM_FAST_URL`, `LLM_FAST_MODEL`, `LLM_DEEP_URL`, `LLM_DEEP_MODEL` |
| The record | schema `llm` in the shared app database, migration `0029_llm.sql`, Secret `llm-db` from `mise run llm:secrets`, then `mise run db:migrate` |
| The budget | `[budget] tokens_per_day` (0 is no limit); enforced only when generations are recorded |
| The probe | `[engines] probe_interval` / `probe_timeout`; `tbd_llm_engine_up` and `ListModels.up` |
| The checks | `grpc_llm_ping`, `grpc_llm_models_lists_both_tiers`, `grpc_llm_generate_unauthenticated` (`docs/chaos/kinds.md`) |

## The engines are dialled directly

Like databases, the engines are not services of this platform: they are model servers
on a machine with the weights, and the service dials their URL straight, never through
Envoy. In the local cluster they run on the host (`host.k3d.internal`); in compose,
`host.docker.internal`. The ports and models in the shipped files are today's:

| Tier | Engine | Serves | Wire |
|---|---|---|---|
| `fast` | Ollama | the model that fits the 16 GB card | `POST /api/chat` as newline-delimited JSON, `GET /api/tags`, `POST /api/embed` |
| `deep` | `llama-server` | the large mixture-of-experts model, attention on the GPU and experts in RAM | `POST /v1/chat/completions` as server-sent events with `stream_options.include_usage`, `GET /v1/models`, `POST /v1/embeddings` (needs `--embeddings`) |

A missing usage is reported as zeros, never estimated. Which model each tier serves is
one value in configuration; the RFC that chose it lives in the site's lab.

## The stub is for tests

`kind = "stub"` is the in-process engine the conformance suite and the chaos tool run
on: deterministic, scripted by markers in the last user message (`<<down>>`,
`<<refuse>>`, `<<error>>`, `<<hang>>`), and labelled `stub: true` on every chunk,
model and embedding. No shipped environment file selects it, the config test asserts
that, and the binary refuses to start with one when `TBD_ENV=production`. There is
deliberately no flag or variable that can pick an engine *kind*.

## Calling it

Through the gateway, with a bearer token or the UI cookie:

```sh
curl -N -X POST http://localhost:18080/v1/llm/generate/events \
  -H "authorization: Bearer $TOKEN" -H "content-type: application/json" \
  -d '{"messages":[{"role":"user","content":"Why is the ledger on two databases?"}],"tier":"TIER_FAST"}'
```

Each event is one chunk as JSON; an error is an `event: error` frame carrying the
protocol's problem envelope. On the multiplexed socket the frame is
`{"type":"call","id":"1","rpc":"tbd.llm.v1.LlmService/Generate","body":{...}}` and
the chunks come back as `data` frames for that id (`docs/protocol/README.md`).

## Measuring it

The chaos tool's `llm_generate` operation streams generations and meters
`prompt_tokens`, `completion_tokens`, `generations` and `ttft` beside the latency of
each stream (`docs/chaos/scenarios.md`); `scenarios/llm_baseline.toml` runs it on the
stub engines in CI, and the same scenario pointed at the deployed service
(`--target llm=…`, a higher `rate`) is the instrument the studies read tokens per
second and time to first token from.

## Not done yet

Batching across callers, a prompt or semantic cache, retrieval over the repositories
and a budget for embeddings. Each is a study of its own.
