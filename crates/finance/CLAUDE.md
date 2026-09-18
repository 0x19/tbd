<!-- tbd new service finance --kind grpc --port 50054 --metrics-port 9468 --bacon-key f (tbd-cli 0.1.0) -->
# crates/finance

The finance service. gRPC only. Scaffolded by `tbd new service` (docs/tbd/README.md);
`Ping` is a labelled stub until the service's real RPCs land beside it.

- `lib.rs`: `serve` (binds `[server] listen`), `serve_on` (caller-supplied listener,
  default `Runtime`), `serve_with` (listener plus a `Runtime`). Tests and the chaos
  tool use the last two on port 0.
- `service.rs`: the `FinanceService` trait impl on `Finance`. Every RPC starts with
  `admit()`: count the request, start the `RequestTimer`, apply the fault handle, map
  a `Fault` to a gRPC status.
- `config.rs`: layered TOML, `configs/finance/base.toml` < `<env>.toml` < flags and
  `FINANCE_*` environment variables (`Overrides`). Every key lives in `base.toml`;
  `deny_unknown_fields` makes a mistyped key fail at start. `finance config` prints
  the effective result.
- `main.rs`: the only file that prints. Subcommands are one-shots with no telemetry:
  `config`, `import` (prototype JSON), `adopt` (a prototype consent as a connection),
  `categorise`, `session` (one signed GET, spends no allowance), `sync` (one tick, or
  `--account` a manual refresh).
- `import.rs`: the **only** place provider JSON becomes rows (`ingest_pages`,
  `ingest_balances`), audited against 2,861 real Erste rows. The importer and the
  syncer both call it; a second parser would drift into wrong money without an error.
- `banking/`: `Provider` trait (hands back JSON *unparsed*, see above), the Enable
  Banking client (`ring` RS256, pages by repeating the date window because Erste 422s a
  continuation key sent alone), `Mock` with every failure mode and a call counter,
  `connect.rs` (consent: `state` is ours and single-use, a foreign party gets `NotFound`),
  `adopt.rs` (one-time migration of the prototype's consents).
- `sync.rs`: the worker. Budget, backoff and watermark are columns on the account, decided
  under `for update skip locked` and committed *before* the network call. Three of four
  daily fetches are the scheduler's; the fourth is a person's. Measured on Erste
  (`prototype/bank/FINDINGS.md`): unattended history is 90 days whatever is asked; no
  4/day 429 seen in 12 fetches. `tick(now)` is a pure step for tests; `run` loops it.
- `invoice/`: drafts, previews, approvals (`store.rs`), the gapless counter
  (`numbering.rs`, a locked row, never a sequence), integer totals (`totals.rs`), and
  the Typst renderer (`render.rs`: template, Inter and the mark compiled in; PDF id
  and date pinned, so a render is a pure function of the document). The approval
  names the preview's content hash; a changed draft is FAILED_PRECONDITION. See
  `docs/finance/invoice.md`. `service_invoices.rs` holds the RPCs.
- `connectors/`: linked external accounts documents are pulled from (mailboxes today,
  portals later). `mod.rs` is the registry and the `Connector` trait -- adding a kind is
  one file and one line; the UI reads the registry. Credentials are sealed at rest
  (`crypto.rs`, ChaCha20-Poly1305 under `FINANCE_CONNECTOR_KEY`, bound to the row id) and
  only `store.rs` opens them, per call. `gmail.rs` links through Google OAuth (read-only
  scope) and keeps PDF attachments; a pulled message is never pulled twice, identical
  bytes are one document with several sources. `service_connectors.rs` holds the RPCs.
  Tests inject a mock kind through `serve_with_kinds`.
- `categorise.rs` + `seeds/rules.sql`: a pass clears every `inferred` categorisation and
  reapplies rules in priority order; `declared` always survives. The seed is idempotent on
  `(party_id, name)`; a rule joins to its category by slug and a typo drops it silently,
  so `tests/it/seed.rs` counts.

- Observability: `tbd_common::telemetry::grpc_request_span` is the `trace_fn`, so every
  call gets a `grpc.request` span with the caller's `traceparent` adopted and
  `trace_id` recorded; `admit()` starts the `RequestTimer`. Metrics listen on
  `[metrics] listen` (`FINANCE_METRICS_ADDR`), `None` for embedders.

Invariants:
- A stub says so on the wire: `PingResponse.stub` is `true` until a real implementation
  replaces it, and the tests assert it. Do not let a placeholder look like a
  measurement.
- `TCP_NODELAY` is set on `TcpIncoming`, not the server builder. With a caller-supplied
  listener the builder setting does nothing, and small responses stall 40 ms.
- Services never address each other directly; a caller reaches this one through
  Envoy's internal listener (`http://envoy:50051`, matched by service name).

Tests: `tests/it/main.rs` boots the server on port 0 through `support.rs` with the
shipped `configs/finance` and env `local`, and exposes the `Runtime` so tests can
inject faults and read counters. `start_with_store()` gives a fresh migrated database
(testcontainers, or `TBD_TEST_DATABASE_URL`). `banking.rs` runs the real client against
wiremock; `sync.rs` and `connect.rs` run the worker and the consent flow against the
`Mock` bank, counting its calls -- the interesting number is usually zero.
