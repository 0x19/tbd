# Stress campaigns

A campaign puts model-checking workers on the ledger and judges every answer. Where a
scenario asks "did the error rate stay under a bound", a campaign asks "did the ledger
ever answer something its contract forbids", and when it did, writes down the exact
sequence of requests that got there. Files live in `stress/`; `mise run stress:run` runs
the directory, `mise run stress:check` parses it, and `chaos stress` is the command
([commands.md](commands.md#chaos-stress)). The harness is the `tbd-stress` crate; chaos
boots the stack, plays the timeline and reports.

## How it works

Each **owner worker** holds a few subjects nobody else writes to and a client-side model
of every one: the facts it was acknowledged, the tombstones, the erasure state, the
idempotency keys it used. The model is built only from requests and their responses,
never from reading the ledger back, so a read that disagrees with it is a finding and not
a self-fulfilling prophecy. Requests in the trace are symbolic (`own`, `peer 1`, "the
`recorded_at` of step 3"), so a trace replays on a fresh subject.

A **finding** is one broken invariant: the subject, the message, what the model expected,
what the ledger answered, and the trace of steps on that subject up to the violation.
Findings are deduplicated across runs by a signature over the invariant and the message
with ids and stamps blanked.

A failure the ledger may answer with (`NotFound` for a missing key, `FailedPrecondition`
for an erased subject, `Aborted` for a reused key) is the contract answering; each
operation knows what to expect and anything else is a finding. `Internal`, `Unknown` and
`DataLoss` are never valid. Connection failures and timeouts are findings unless the
campaign tolerates them (`[faults] tolerate`), which is how a campaign runs through a
fault timeline: a write that drew a tolerated failure is re-driven through its
idempotency key until the outcome is known, so the model never guesses.

## The file

```toml
[campaign]
name = "smoke"                      # required
description = "..."                 # what it proves
skip = false                        # skipped when a directory is run (the long ones)
duration = "3s"                     # the measured phase
warmup = "300ms"                    # same workload first; numbers discarded
seed = 1                            # every random choice derives from it
timeout = "5s"                      # per request

[stack.ledgers.ledger-1]            # a scenario's [stack]; unused when targets are given
grace = "0s"                        # empty database_url: the memory store
database_url = ""

[workload]
max_in_flight = 64                  # requests in flight across every worker
paths = ["profile.name", "profile.bio", "traits.warmth", "traits.novelty", "journal.entry", "readings.sun"]
relation_paths = ["relations.match.m1", "relations.match.m2"]
scopes = ["self", "engine.base", "tier2@persona-a"]   # consent vocabulary; a fact's consent is a non-empty subset

[workload.owner]
workers = 4                         # concurrent workers
subjects = 2                        # per worker; >= 2 enables pair relations and the cascade check
pace = "0ms"                        # think time between operations
limit = 0                           # read page size; 0 draws one of 1, 2, 3, 5, 10, 50, 1000 per read

[workload.owner.mix]                # without this table: the balanced mix below; with it, a missing key is 0
append = 6
current = 3
history = 3
history_cut = 2
retract = 1
idempotent_replay = 1
pair_relation = 1
erase_cycle = 0.2
expiring = 0.5

[faults]
tolerate = []                       # error classes a request may draw: unavailable, deadline_exceeded, transport, timeout, ...
settle = "1500ms"                   # added to the announced window before the cascade is checked
clock_skew = "500ms"                # an expiry within this of now is not compared

[[timeline]]                        # a scenario's timeline; ignored against explicit targets
at = "1s"
action = "set_behavior"
service = "ledger-1"
behavior = { type = "error", kind = "unavailable", rate = 0.3 }

[invariants]                        # every invariant is on; `name = false` switches one off
expiry = false

[stop]
max_findings = 0                    # stop early after this many; 0 = never
```

Every table is `deny_unknown_fields`; `chaos stress check` names the first problem.

### Where it runs

- **The campaign's `[stack]`** (the default): chaos boots it on free ports, the workers
  run against its ledgers, the timeline applies. `database_url` on the ledger puts it on a
  real Postgres; empty is the memory store. This is where bugs are found and reproduced.
- **Explicit targets** (`chaos stress run --target ledger=URL`, the bearer flags as for
  `validate`): no stack, the timeline is ignored, the same file runs against a deployed
  ledger through Envoy.

### The operations

| Operation | What it does | What it judges |
|---|---|---|
| `append` | one fact on a random path with a random source, value, consent subset and a fresh idempotency key | `append_echo`, `recorded_at_monotonic` |
| `current` | `Current` under every scope, walked through every page; half the time also under a scope subset; sometimes with a path or source filter | `current_is_latest`, `expiry`, `pagination`, `consent_filter`, `path_source_filter`, `recorded_at_monotonic` |
| `history` | `History` walked through every page | `history_is_everything`, `pagination`, `recorded_at_monotonic` |
| `history_cut` | `History` at the `recorded_at` of an earlier step | `history_cut` |
| `retract` | a key with a value (70 %), or one without | `retract_semantics` |
| `idempotent_replay` | an earlier append re-sent as is (a replay), mutated (a conflict), or after its fact was retracted | `idempotency` |
| `pair_relation` | a fact on a relation path naming a peer subject | `append_echo` |
| `erase_cycle` | erase, reads and writes denied, a relation from a peer denied, restore, reads equal the model; with a short window: erase again, wait, the subject is gone and every peer that named it carries a tombstone | `erasure_denies`, `erasure_executes`, `cascade_tombstones_counterparty` |
| `expiring` | a fact that expired long ago, two seconds ago, or expires in three seconds or a minute | `append_echo`, then `expiry` on reads |

An erasure window shorter than two seconds may close while the denials are checked (the
sweeper runs on its own clock), so a restore that finds the subject gone is then the
cascade, not a finding. A window of two seconds or more is real: restore must work, and
the cascade is not awaited.

### The invariants

| Name | Holds when |
|---|---|
| `append_echo` | an acknowledged append echoes every field of the request, with a positive id and a `recorded_at` |
| `recorded_at_monotonic` | `(recorded_at, id)` strictly increases per subject across acknowledged writes, and every page is strictly ordered |
| `current_is_latest` | `Current` under every scope is exactly the latest valued live fact per `(path, source)`: no tombstone, no key twice, nothing missing, nothing extra |
| `history_is_everything` | `History` is every row the subject holds, tombstones and expired facts included |
| `history_cut` | `History` at an instant is every row recorded at or before it that is still held; a retracted value is absent from every cut |
| `retract_semantics` | a retraction of a valued key answers a tombstone for that key with no value, no counterparty and the value's consent; of nothing valued, `NotFound` |
| `consent_filter` | `Current` under a scope subset returns exactly the live facts whose consent meets it |
| `path_source_filter` | path filters (`x.*` is a prefix) and source filters select what the model says they select |
| `pagination` | pages for any limit concatenate to the unpaged set: full pages before the last, no id twice, order kept across pages, an empty `next` at the end |
| `idempotency` | the same key with the same content replays the earlier fact; other content answers `Aborted`; a key whose fact was retracted answers `Aborted` |
| `expiry` | an expired fact is absent from `Current` and present in `History`; an unexpired one is present |
| `erasure_denies` | inside the window every call on the subject answers `FailedPrecondition`, a relation naming it from another subject too, a second erase is the same one, and after restore `Current` equals the model |
| `erasure_executes` | past the window reads answer `FailedPrecondition`, restore `NotFound`, and an append `FailedPrecondition`: the id is never reusable |
| `cascade_tombstones_counterparty` | after a peer's cascade the subject holds one tombstone per relation that named the peer, with the cause as origin and the newest value's consent, and everything else as before |
| `clean_errors` | never `Internal`, `Unknown` or `DataLoss`; never a dropped connection or a timeout outside `[faults] tolerate` |

## Findings, shrinking and replay

A finding is written as `<findings-dir>/<id>.json` (`--findings-dir`,
`CHAOS_FINDINGS_DIR`, default `.chaos/findings`) with everything above and its trace.
After the workers stop, every finding is **shrunk**: the trace is replayed on fresh
subjects and cut down by delta debugging (ddmin) to the shortest sequence that still
breaks the same rule with the same signature; steps a kept step depends on (a cursor, a
`recorded_at`) are pinned back in. `[stop] shrink = false` skips it;
`shrink_attempts` and `shrink_timeout` bound it per finding. The finding records
`original_len`, whether it is `shrunk`, and a `shrink_note` saying what happened (`3 of
41 steps after 27 replays`, `every step is needed`, or `not reproduced on a fresh
replay` for a race that did not recur).

`chaos stress replay <id|path> [--target ledger=URL] [--attempts N]` runs a finding's
trace against a ledger, `attempts` times for a race, and says `REPRODUCED` (exit 1) or
`not reproduced`; the outcome is appended to the finding's `replays`. Without a target the
ledger from the config or `CHAOS_LEDGER_URL` is used: a bug found on the memory store can
be checked against Postgres, or against the deployed ledger through Envoy, from the same
file.

A replay judges answers with the same model and the same checkers the worker used, from
the requests alone, so it needs nothing but the trace.

## Output

`chaos stress run` prints one block per campaign: `PASS`/`FAIL`/`SKIP`, the target and its
store, the load line and per-operation latencies (the same shape as a scenario's), the
tolerated and re-driven counts when faults were tolerated, one line per invariant with
how often it was evaluated and how often it broke, and one line per finding. `--json`
prints the array of results, each with `checks` (a map of invariant to `{passed,
violated}`) and `findings` with their traces. Exit code 1 on any finding or error.

A campaign passes when every evaluation held and nothing failed outside the checks. A
finding is never a threshold to loosen; it is either a bug in the ledger or a bug in
the model, and the trace says which.

## The shipped campaigns

| File | Proves | In CI |
|---|---|---|
| `smoke.toml` | every owner invariant on the memory store under the balanced mix, three seconds | yes |
| `erasure_cascade.toml` | the privacy rules under repetition: denial, restore, the cascade and the counterparty tombstones | yes |
| `idempotency_storm.toml` | replays, conflicts and keys after retractions never write or leak | yes |

## Debugging a finding

Run the campaign with `--json` and read the finding: `message`, `expected`, `actual`, and
`trace`, whose steps carry the symbolic request and the response as JSON. The last step
is the one that broke the rule; the earlier ones are how the subject got there. Findings
found so far in building the harness were all in the model; the ledger's answer and the
model's expectation disagreed on which row's consent a cascade tombstone carries (the
newest, as retract does), on whether a relation's replay is judged by the counterparty
check first (it is), and on whether a zero-length window is a window (it is not).
