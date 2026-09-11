# stress

Stress campaigns run by `mise run stress:run` and by CI. Each file is a workload of
model-checking workers against a ledger, optionally a stack and a fault timeline; the
full reference is `docs/chaos/stress.md`.

Rules for files here:
- No fixed `listen` addresses in `[stack]`; campaigns run on free ports.
- Keep the CI ones short: a few seconds. A campaign that needs Postgres or runs for
  minutes sets `skip = true` and is run by hand or on a schedule.
- The header comment states the hypothesis: what the campaign proves when it passes.
  A failing campaign is a finding, never a threshold to loosen.
- `seed` is fixed, so a run is reproducible; change it on purpose, in a commit that
  says why.
- Run `mise run stress:check` after editing; it rejects unknown keys, unknown
  invariants and a `[stack]` that does not hold together.
