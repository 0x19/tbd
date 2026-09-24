---
title: The grader
status: open
date: 2026-09-24
public: false
lab: quietpager
summary: qp, an open-source Rust binary that grades a learner's Go or Rust program on their own machine — a fault proxy in front, paced load and a correctness checker behind, restarts where the level says — and the kata format it reads.
---

## Problem

A kata is only worth doing if passing it means the code survives what production does to
it. Unit tests check that code compiles and answers; they do not check that a rate limiter
still refuses correctly while its clients see resets, or that a write-ahead log keeps every
acknowledged write across a kill at an arbitrary moment. The platform's chaos tool does
exactly this to our own services, but it cannot be pointed at someone else's program: its
kinds are compiled in, its faults are switches inside our services
(`crates/common/src/fault.rs`), and it has no process launcher, no proxy and no disk
faults. The grade also has to run where the code is, on the learner's machine: running
strangers' binaries on the cluster is a sandboxing project of its own.

## Proposal

One Rust binary, `qp` (github.com/quietpager/qp, Apache-2.0), with these commands:

- **`qp proxy`** — a black-box TCP fault proxy in front of any port: added latency and
  jitter, a bandwidth cap, connection resets, blackholing, stalls mid-stream. Faults come
  from a TOML schedule (`at`, `for`, `fault`, parameters) or a small local control API.
  Useful on its own against anyone's services, which is part of its value as a public tool.
- **`qp grade <kata> [--level N] -- <cmd...>`** — reads the kata, launches `<cmd>` with the
  port and data directory it names, waits for the readiness probe, puts the proxy in front,
  runs paced open-loop load (an absolute-deadline pacer and a latency histogram, small and
  re-implemented here), feeds every request and answer to the kata's checker, runs the
  level's fault schedule including SIGTERM, SIGKILL and restart, and prints a report.
- **`qp submit <report>`** — sends the report to Quiet Pager with the user's token.
- **`qp agent`** (later) — keeps an outbound connection so the site can drive live faults
  through the user's own proxy: Break it, against your own code, without opening a port.

**A kata** (github.com/quietpager/katas) is a folder:

```
rate-limiter/
  kata.toml        # name, protocol, readiness probe, port and data-dir conventions
  SPEC.md          # what to build, the wire protocol, the guarantees graded
  levels/1.toml    # load shape, fault schedule, clauses and their weights
  checker.toml     # parameters of the built-in checker for this kata
  go/  rust/       # reference implementations, built on stream
```

The checkers are built into `qp` per kata family, because correctness is a model, not
configuration:

- **Rate limiter** (kata 1, `POST /acquire` → allowed or refused): allowed requests in any
  window stay within limit × window + burst; refusals beyond the limit's tolerance count
  as over-refusal; p99 stays under the level's budget; after a restart the limit holds
  (state kept or honestly reset, as the level says).
- **Key-value store with a write-ahead log** (kata 2, `PUT/GET/DELETE`): every write
  acknowledged before a kill is readable after the restart; no read returns a value never
  written; a later level adds disk faults (EIO, failed fsync, short writes) through an
  LD_PRELOAD shim, Linux only.

**The report** is JSON: kata, level, `qp` version, the binary's hash, machine facts, per
clause `{name, passed, expected, actual, weight}`, load and latency figures, the fault
timeline, the score (weighted clauses, 0 to 100) and the verdict. It is a claim made by
the learner's machine, and the platform always shows it as one: reproducible by anyone
with the same binary and level, not verified by us.

## Alternatives considered

**Extend the chaos tool.** It has the pacer, the timeline and the assertion shapes, and
missing pieces could be added (an external-process kind, a proxy). Rejected as the home
because the open tool must build and be trusted without the private platform; the ideas
are reused, the code is re-implemented small.

**Toxiproxy for the faults.** Mature, but a second process to install and a Go dependency
for a Rust binary, and the grader needs the fault timeline in the same clock as its
measurements. Rejected; `qp proxy` is a few hundred lines on tokio.

**gRPC as the kata protocol.** Closer to production, but a barrier for a first kata in two
languages. Plain HTTP and JSON first; gRPC variants as later levels.

**Grading on the platform.** See RFC 0007: needs isolation we do not have. Rejected for now.

## Decision

Build in this order: `qp proxy` with tests against a local echo server; the process
supervisor; kata 1 with its checker and both reference implementations; `qp grade` end to
end with the report and release binaries from GitHub Actions; kata 2. `qp submit` waits
for the studio service (RFC 0007, track C).

## Publication

Public with the brand's launch, after a redaction pass. The repositories are public from
their first commit, since the code itself is the thing being shown.

## Status log

- 2026-09-24: opened with the build order and the kata format.
