<!-- tbd new service llm --kind grpc --port 50057 --metrics-port 9471 --bacon-key m (tbd-cli 0.1.0) -->
# crates/llm

The llm service: the L2 over the engines that run the weights. Contract and
operations in `docs/llm/README.md`; this file is the non-obvious.

- `lib.rs`: `serve` / `serve_on` / `serve_with`. Validates the config, builds one
  engine per tier (`engine::build`, the only match on a kind), starts the probe task,
  opens the pool lazily when `[store] url` is set (`LLM_DATABASE_URL`), aborts the
  probe when the server stops.
- `service.rs`: the `LlmService` impl on `Llm`. Every RPC starts with `admit()`.
  `Generate` is admit → caller → bounds → tier → budget → session and row → engine,
  then `live_stream`: an `unfold` over the engine's chunks with the deadline as a
  `select!` arm, the metrics recorded from `Chunk` (never inside an engine), and a
  `Recording` that closes the row with its outcome and, on drop without a close,
  records `cancelled`. `status_of` is the one place an `EngineError` becomes a code.
- `engine/mod.rs`: the trait, `Chunk`, `EngineError`, the shared `lines` (a
  `LinesCodec` over `bytes_stream`, 1 MiB cap) and `parse_stream`, which enforces the
  contract that a stream ends after the done chunk or the first error. Each HTTP
  engine has a pure `LineParser` unit-tested on fixture lines.
- `config.rs`: `[engines]`, `[engines.fast|deep]`, `[generate]`, `[budget]`,
  `[store]`. `EngineConfig` is a plain struct with `kind: EngineKind`, not a tagged
  enum: `deny_unknown_fields` and `flatten` do not combine. `Config::stub(addr)` is
  what tests and the chaos kind run; `refuse_stub(env)` is what `main` runs.
- `probe.rs`: health per tier on an interval into a gauge and a shared flag. It never
  touches the gRPC health reporter: an engine restart must not read as an outage.
- `store.rs`: `llm.sessions`, `llm.generations`; `used_since` is the budget.

Invariants:
- No L1 detail above the engine table: the proto, the service and the config above
  `[engines]` know tiers, never Ollama or llama.cpp. A new engine is a module, a
  `EngineKind` variant, an arm in `build`, a wiremock fixture in `tests/it/support.rs`
  that honours the prompt markers, and a green conformance suite. Nothing else.
- Two error placements, both contract: `Err` from `Engine::generate` is before the
  first chunk (the RPC's status); an `Err` item on the stream is after (an error item).
- The stub says so on the wire (`stub: true` on chunks, models, embeddings), no shipped
  env file selects it (the config test asserts it), and production refuses it.
- A missing usage is zeros, never a guess. The budget counts what engines reported.
- The deadline is the tier's `timeout_secs` from admission, one clock for the dial
  and the stream. reqwest clients carry only a connect timeout on purpose.
- Engines are dialled directly, like databases; the Envoy rule does not apply.
- `TCP_NODELAY` is set on `TcpIncoming`, not the server builder.

Tests: `tests/it/conformance.rs` is one suite run three times (stub, wiremock Ollama,
wiremock llama.cpp) through `conformance_suite!`; `tests/it/budget.rs` needs Docker
(a migrated Postgres) or `TBD_TEST_DATABASE_URL`; `tests/it/main.rs` is the scaffold's
Ping, health and fault injection. Unit tests sit next to each parser.
