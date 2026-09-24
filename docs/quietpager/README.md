# Quiet Pager

Quiet Pager, an InOrbit lab (RFC 0007, a draft readable by admins in the lab): katas
from production systems graded under injected faults, and a weekly Radar of what
changed in Go and Rust. This page is the contract of its services on the platform;
the open-source grader lives in `github.com/quietpager/qp` and the katas in
`github.com/quietpager/katas`.

## The radar service

`crates/radar`, port 50059, `tbd.radar.v1.RadarService`.

| RPC | REST | Who |
|---|---|---|
| `Ping` | `GET /v1/radar/ping` | anyone (labelled stub, as in every service) |
| `ListDigests` | `GET /v1/radar/digests?language=&lang=&limit=&include_drafts=` | anyone; drafts only for the admin role |
| `GetDigest` | `GET /v1/radar/digests/{id}` | anyone; a draft is not found for anyone but an admin |
| `ListItems` | `GET /v1/radar/items?language=&limit=` | anyone |
| `Refresh` | `POST /v1/radar/refresh` | admin role |
| `PublishDigest` | `POST /v1/radar/digests/{id}/publish` (`publish` false takes it back to draft) | admin role |
| `RunDigest` | `POST /v1/radar/digests/run`: starts the week's run in the background and returns (`started`, or `already_running`); `force` rewrites the week's, `wait` waits for it (direct callers only; the edge's timeouts cut a wait) | admin role |

On the site's host (`www.`), Envoy serves the reads open with a per-address rate
limit and no identity, sends a read with `include_drafts=true` through the lab's gate
(sign-in, then the admin role; the page asks only for an admin), and gates the writes
the same way; the service checks the role again.

**Review.** Every digest the worker writes is a draft; the page calls issues
human-reviewed, and `PublishDigest` is that review. A rewrite (`RunDigest` with `force`)
sends a published digest back to draft.

**Shape.** A digest is `summary` (the week in two or three sentences), `changes`, the
drill and the avatar script. A change has a title, an impact from a fixed set
(`IMPACT_BREAKING`, `IMPACT_WORTH_KNOWING`, `IMPACT_NICE_TO_KNOW`: named categories,
never a score), an area (runtime, compiler, stdlib, tooling, language, ecosystem), its
link, what changed, production impact and a try-it. A change's link must be one of the
week's items, so the model cannot cite what it was not given; a change without every
field, with another impact, or with another link is dropped, and a digest with no valid
change is refused. Digests written before 2026-09-24's batch carry `changed` and `why`
instead.

**Sources** (`configs/radar/base.toml` `[[sources]]`, official only): the Go blog,
Go releases, accepted Go proposals (GitHub search), the Rust blog, Inside Rust, This
Week in Rust, merged Rust RFCs (GitHub search). Read every `[fetch] interval_secs`
(6 h), each on its own; a failing source does not stop the rest.

**Digests.** One run at a time, whoever starts it (the schedule or `RunDigest`). Written on `[digest] weekday` at or after `hour` (UTC, Monday 06:00),
for the seven days before, one per language and reader language, by the llm service
(`[llm] tier`, deep) as `svc:radar` within that subject's daily token budget. Each has
four sections (what changed, why it matters, a ten-minute drill, a 60-second avatar
script). An answer missing a section is refused and nothing is stored. Every digest
carries `ai_written: true`, and `stub: true` when the llm's stub engine wrote it.

**Configuration.** `RADAR_DATABASE_URL` (Secret `radar-db`), `RADAR_LLM_URL`
(configmap, Envoy's internal listener), `RADAR_GITHUB_TOKEN` (optional Secret
`radar-github`; without it GitHub is read unauthenticated). `mise run radar:secrets
[token]` creates them; `mise run db:migrate` applies `0031_radar.sql`.

**Metrics.** `tbd_radar_fetches_total{source,outcome}`,
`tbd_radar_items_new_total{source}`, `tbd_radar_digests_total{language,lang,outcome}`
(`docs/observability/metrics.md`).
