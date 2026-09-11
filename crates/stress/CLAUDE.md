# crates/stress

`tbd-stress`: stress campaigns against the ledger. Closed-loop workers keep a
client-side model of every subject they own and judge every answer against the
ledger's contract; a broken rule is a finding with the trace that led there. The
contract is `docs/chaos/stress.md`; chaos runs campaigns (`chaos stress`) around the
same stack and timeline it uses for scenarios.

- The crate never boots a ledger and never depends on `tbd-ledger` (dev-dependency for
  the integration test only): the model must not share code with the thing it checks.
  The caller hands `run` a `client::Target` per ledger (a channel and an optional
  bearer); `client::GrpcLedger` is the one implementation of `LedgerClient`.
- `campaign.rs`: the TOML schema, every table `deny_unknown_fields`. `[stack]` and
  `[[timeline]]` are opaque `toml::Table`s here; chaos parses them. A present
  `[workload.owner.mix]` lists exactly the operations that run (a missing key is 0);
  an absent one runs the balanced default.
- `model/mod.rs`: `SubjectModel`, built only from requests and responses. `learn` is
  the one place a read fills in the model: ids and stamps of tombstones the model knows
  exist (a re-driven retract answered `NotFound`) and the fate of appends whose outcome
  was unknown. Never make the model adopt what a read says otherwise.
- `model/invariants.rs`: the checkers, pure over data, one per rule in `ALL`; the
  campaign's `[invariants]` and the report's counters are keyed by these names. A new
  rule is a function here, a row in `docs/chaos/stress.md`, and a call from the worker
  that knows when it applies.
- `trace.rs`: symbolic requests (`SubjectRef`, `TimeRef`, `cursor_from`) materialised
  against a `Binding`, so a trace replays on a fresh subject; steps look up earlier
  steps by `index`, not position, because the worker keeps a bounded ring. The instants
  actually sent are pinned into the stored request, so a replay sends the same bytes
  (idempotency fingerprints include them).
- `workers/owner.rs`: the owner worker. `send` is the one path every call goes
  through: permit, timing, metrics, the step, the `clean_errors` judgement.
  `append_settled` and `retract_settled` re-drive a write after a tolerated failure
  and settle its outcome through the idempotency key or `NotFound`. An erasure window
  under `SAFE_WINDOW` (2 s) may close during the denial checks, so restore is allowed to
  find the subject gone and the cycle continues into the cascade; a real window never
  awaits the cascade.
- `replay.rs`: `Interpreter` judges a trace step by step from the request alone, with
  the same model and checkers the worker used, so `replay` reproduces a finding on any
  target; `shrink.rs` is ddmin over the steps with dependencies pinned back in, bounded
  by `[stop]`. Owner workers know *what* to send; the interpreter knows what an answer
  must have been. When a rule is added or changed, both must agree, and the lying-client
  test in `tests/it` is where a disagreement shows.
- `executor.rs`: `run` connects and hands over to `run_with_clients` (what a test with
  a lying client calls): ping the targets (a stub ledger is an error), spawn workers over
  the targets round robin, warmup, the measured phase with a snapshot a second, join,
  collect, then shrink every finding while the workers are quiet. `[stop] max_findings`
  cancels the workers early.
- `metrics.rs` is chaos's load metrics type, moved here so load runs and stress runs
  report the same `LoadSnapshot`; chaos re-exports it at `load::metrics`.

Tests: unit tests next to the code (every invariant with hand-built pages, the model's
transitions, the campaign checks, the signature normalisation); `tests/it/main.rs` boots
`tbd_ledger::serve_store` on the memory store with a zero grace window and runs short
campaigns: every enabled invariant is evaluated and holds, cancel ends the run, an
unreachable target is an error, and a client that drops the last fact of every history
page produces `history_is_everything` findings that shrink to a few steps and replay
through the liar but not through the honest client. The shipped campaigns under `stress/` run in
`mise run ci`.

When a campaign finds something, the trace decides whether the ledger or the model is
wrong. Fix the model when the ledger's answer follows from its documented contract; fix
the ledger, in its own commit with a conformance test, when it does not.
