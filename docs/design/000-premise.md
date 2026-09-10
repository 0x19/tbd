# 000 — Premise

Status: open

Why this repo exists, what we carry over from apex, and what we refuse to repeat.

---

## What apex was

A privacy-first proximity dating app for the Croatian market. Match on
compatibility before appearance; photos revealed only on mutual accept; only
with people physically nearby right now; 24h expiry to force real meetings.
Self-hosted ONNX inference so ML costs nothing per user.

Two checkouts exist:

- `/mnt/development/0x19/apex` — the original repo, 4 commits, last touched
  2026-05-07.
- `/opt/proximity` — the same project renamed (domain `proximity.is`),
  refactored into a 4-crate Rust workspace on 2026-05-07, deployed and still
  running. Only the marketing site actually serves traffic.

## Why we start over rather than continue

Not because the ideas were wrong. Because the codebase reached a state where
finishing it costs more than restarting.

**Neither checkout compiles.** `.gitignore` line 22 in both repos is a bare
`models` pattern, which git matches at any depth. It silently excluded
`backend/src/engine/models/` (the four ONNX wrapper structs the engine imports)
and `mobile/lib/models/` (four Dart data models imported by 12 files including
one reachable from `main.dart`). Neither directory was ever committed. Neither
exists anywhere on this machine. They are gone.

**The three prototypes never connected.** The Flutter app makes zero calls to
the backend — the generated gRPC client is unused, `firebase_*`, `grpc`,
`app_links` and `flutter_secure_storage` are declared and never imported, every
match and message comes from `demo_data.dart`, and `main.dart:44` hardcodes
`_devMode = true` to skip auth entirely. The backend has no authentication at
all: `user_id` is client-supplied on every RPC. The chaos tool, which contains
the only real ML in the project, talks to nothing.

**The intelligence was a placeholder wearing a percentage sign.**
`profile_analyzer.rs:190` derives Big Five traits by reading embedding
dimensions 0, 10, 20, 30… as proxies. Its own comment says it's a placeholder.
Those traits feed a weighted compatibility score that is surfaced to users as
"82% compatible". The number was noise, formatted convincingly.

**Refactors were started and abandoned mid-flight.** The chaos tool has two
live implementations of the same thing: `processing.rs` (labelled "legacy",
contains all the working inference) and `analyzers/` (the new pluggable
architecture, where emotion, attributes and parsing return hardcoded fake
values). `chaos analyze dir --labels` runs accuracy reports against analyzers
that always answer "male, 25, neutral". `chaos scenario run` on the same images
gives real numbers. Both ship.

**Infrastructure ran ahead of code.** SurrealDB, Redis and Ollama with
Qwen3-VL 8B are provisioned and running on this box right now. Redis has zero
keys. SurrealDB reports unhealthy. The backend has no dependency on either —
still `redis` + `ort`. The next architecture was deployed before it was written.

**Docs ran ahead of code by roughly 5×.** ~12,000 lines of design spec against
~2,700 lines of backend skeleton. `matching.md` is 1,178 lines describing a
matching system that does not exist. `docs/development/todo.md` claims 40%
complete; measured against its own specs it is nearer 15%.

## What we keep

The ideas. These are good and most of them are unusual.

- **Compatibility before appearance, with mutual reveal.** The core wager.
  Nothing else in the market does this seriously.
- **Proximity as the forcing function.** Matching only with people who are
  physically near you *now* creates urgency that no distance filter can.
- **Spark** — signal interest in someone you've seen in person without
  approaching them, and learn nothing if they don't reciprocate. Solves a real
  social problem, and it's the feature people will describe to their friends.
- **Signals to the algorithm** (`docs/system/profile-system.md`, Part 3.2).
  Letting a user say "I come across as X but I'm actually Y" and "I say I want
  casual but honestly…" directly to the matcher. This is the strongest original
  idea in the project and nothing was built for it.
- **Intentions declared upfront** — what you're looking for is shown in every
  match notification, so nobody wastes weeks on mismatched intent.
- **Anti-gaming by construction** — you never see your own score, so you can't
  optimise it. Being genuine is the only strategy available.
- **Complementary matching**, not just similarity. Real compatibility is not
  cosine distance.
- **Privacy boundaries as hard constraints**, not settings.
- **The scenario-testing pattern from chaos** — TOML scenarios with ground
  truth and pass/fail assertions on accuracy. This was the best-engineered
  thing in apex and it's a pattern worth rebuilding from scratch for whatever
  ML we end up running.

## What we drop

- **Ship-ready privacy claims that the code contradicts.** apex promised
  "never store location history — only current location". In practice
  `update_location` GEOADDs to a Redis geo set with no expiry, `cleanup_stale`
  returns `Ok(0)` with a comment claiming Redis TTL handles it (it does not for
  geo members), so locations accumulate forever and stale users show as nearby
  indefinitely. A privacy guarantee that isn't enforced by the code is a lie
  with a nice font.
- **Six task-specific ONNX models glued together.** Face detection, emotion,
  age/gender, face parsing, iris segmentation, text embedding, image encoding —
  seven models, five preprocessing pipelines, four incompatible emotion
  orderings across the repo, and a 56 MB iris model that was downloaded, loaded
  into a session and never actually run.
- **Building a matcher before we can measure whether matching works.**
- **Croatia-only as an unexamined assumption.** It may still be right. It was
  never argued.
- **`node_modules` in git** (4,020 of 4,299 tracked files), stray root
  `package.json`, two competing `mise` configs with clashing task names, six
  files hardcoding `/Volumes/Storage/...` macOS paths from a machine that no
  longer exists.

## The bet we're making

apex failed as software, not as a product idea. The specific failure was
**breadth before depth**: four surfaces started, none finished, nothing
end-to-end, no way to tell whether the central premise — that a machine can
predict who you'll enjoy sitting across from — is even true.

So the first question this repo answers is not "what should we build" in the
feature sense. It is: **what is the smallest complete thing that would tell us
whether the premise holds?** Everything else waits behind that.

See [001-open-questions.md](001-open-questions.md).
