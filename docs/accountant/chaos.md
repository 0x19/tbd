# Chaos: replaying a fiscal year against the filings

The claim "the service produces what the accountant filed" is only worth anything if a
machine checks it on every change. This page designs that check as chaos scenario
families, using the seams `docs/chaos/extending.md` already provides, and lists the
contract pages each step changes.

## What is compared

A **golden year** is a directory:

```
golden/inorbit-2025/
  meta.toml            party, fiscal year, chart version, rule-pack versions
  opening.csv          account, debit, credit           (1.1.2025, or the migration date)
  bank.jsonl           the provider rows the importer already understands
  invoices.jsonl       issued invoices with client residence, currency, lines
  documents.jsonl      supplier documents with net, VAT treatment, account
  payroll.csv          twelve months
  assets.csv           the register
  judgements.jsonl     loan events, distribution, catch-up depreciation, prior-year costs, tax accrual
  expected/
    trial-balance.csv  account, total debit, total credit, balance   (from gfi-2025.md §1)
    bil.csv            aop, prior, current                             (§2)
    rdg.csv            aop, prior, current                             (§3)
    dop.csv            aop, prior, current                             (§4)
    pd.csv             row, amount                                     (§5)
    checks.csv         the FINA Kont rules that must pass
```

The expected files for 2025 are written from `gfi-2025.md` now, before any code, and
never edited to make a run pass. Personal data (bank rows, invoices) stays out of git:
the directory lives on the server, is named by a `[paths].golden` key, and CI scenarios
that need it are `skip = true` as `scenarios/CLAUDE.md` requires; a synthetic golden
year with the same shape and made-up counterparties is committed so CI still replays
the whole pipeline.

## Scenario families

1. **`books_invariants`** (hermetic, CI). Boots a finance instance with the in-memory
   store, posts generated entries through the load operations and asserts the ledger
   identities: Σdebit = Σcredit per entry and overall, no line with both sides, a locked
   period refuses postings, re-posting a period is idempotent, reversal cancels exactly.
   Operations: `finance_post_entry`, `finance_repost_period`, `finance_lock_period`.
   Fault behaviours on the kind (`fail_store`, `slow`) must not break any identity.
2. **`books_opening`**. Imports `opening.csv` and asserts the trial balance equals it,
   then that the balance sheet computed from the opening alone equals the prior-year
   column of `bil.csv`.
3. **`year_replay`** (the headline). Imports everything in the golden directory in date
   order through the same RPCs a person or the sync worker would use, runs
   `PostPeriod` month by month, declares the judgements, runs the year-end, computes the
   statements, and compares. Every account, every AOP, every PD row is one assertion row
   named `tb:1000`, `bil:065`, `pd:57`, so the run report lists exactly what differs and
   by how much. Tolerance is 0,00 except where FINA itself allows 0,14 (control 34), and
   that tolerance is written in `checks.csv`, not in code.
4. **`year_replay_judgements`**. The same replay with one judgement removed or changed
   at a time (no loan interest, no catch-up depreciation, distribution not declared,
   prior-year invoice booked in the year). Each variant has its own expected delta
   (for example: without interest, PD row 25 gains a hidden-distribution add-back and
   7710 is 0,00). This is how the accountant's judgements are kept as supported,
   documented behaviours rather than one-off fixes.
5. **`period_close_under_faults`**. Timeline: close month six while `fail_store` is on,
   restart the instance, close again; assert nothing posted twice and the trial balance
   equals the fault-free run.
6. **`exports`**. Export xlsx, the FINA fill result and the PD PDF; assert the FINA
   workbook re-reads with zero errors on its Kont sheet and the xlsx hash equals the
   recorded one for the golden inputs.
7. **Stress campaign `books_fuzz`**. Workers post random valid and invalid entries,
   declare and revoke judgements, close and reopen periods; invariants: the ledger
   identities, monotone period status, statements deterministic for equal inputs
   (same `inputs_hash`, same values), no unmapped account with a non-zero balance.

## What the tool needs, and where it changes

| Need | Seam | Contract pages |
|---|---|---|
| A golden directory | new `[paths].golden` key: `crates/chaos/src/config.rs`, `configs/chaos/base.toml`, `ServeArgs` flag with `env = "CHAOS_GOLDEN"` | `config.md` key table and "Adding a key" recipe, `.env.example`, `compose.yaml`, k8s configmap, ansible |
| Seeding a finance instance with a dataset | finance kind field `golden = "<name>"` next to `seed`; the kind loads the directory through the service's own import path (no second parser) | `kinds.md` regenerated, `extending.md` |
| Table-shaped comparison | new scenario table `[replay]` (`golden`, `through = "2025-12"`, `judgements = [...]`) on `ScenarioFile`, a replay phase in the executor, and an assertion family `[assertions.books]` (`expected = "expected/"`) whose `evaluate` emits one `AssertionResult` per line item | `scenarios.md` (new table, assertions), `api.md` if `RunRecord` gains a `replay` summary, `ui/chaos` Zod mirror |
| Validate checks | `grpc_finance_books_balanced`, `grpc_finance_trial_balance` on the finance kind | `kinds.md` regenerated |
| Load operations | `finance_post_entry`, `finance_repost_period`, `finance_lock_period`, `finance_statements` in a new `crates/lab/src/load/finance_ops.rs` | `scenarios.md` operations table, `extending.md` |
| Findings with a diff table | keep `tbd_stress::Finding`; put the per-line diff in `expected`/`actual` JSON arrays; the findings page renders arrays of `{key, expected, actual, delta}` as a table instead of `<pre>` | `stress.md` findings, `ui.md` |
| CI gate | `mise run chaos:run` includes the hermetic families; `year_replay` on the real golden runs on the server through `chaos serve` and posts to Slack like the other campaigns | `ci.md`, `runbook.md` |

Stale pages found by the survey (`architecture.md` module tree, `README.md` counts,
`scenarios.md` stack example) are corrected in the first commit that touches the tool.

## Order of work

Phase 1 lands the validate check `grpc_finance_books_balanced`, the operations
`finance_trial_balance` and `finance_import_opening`, and the opening slice of family 2
(`scenarios/finance_books.toml`: import, then the trial balance equals it). The posting
operations of family 1 (`finance_post_entry`, `finance_repost_period`,
`finance_lock_period`) need `PostPeriod` and land with phase 2; families 3, 4, 7 with
phase 3; 5 and 6 with phase 4. The expected files for 2025 are the first thing written,
in phase 0, from `gfi-2025.md`; `expected/opening-2025.csv` (the hand-over column,
signed) joined them in phase 1.

A scenario that needs Postgres runs against the compose database with `skip = true`, as
the one stress campaign that needs one already does: the CI scenarios job has no
database. Giving it one (`services: postgres` in `ci.yml`, a `database_url` the kind reads
from an environment variable) is a separate decision; until then the DB-backed proof of
each phase is the finance crate's integration tests.
