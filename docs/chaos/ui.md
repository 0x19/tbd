# The chaos admin UI (`ui/chaos`)

A web front end for `chaos serve`: see the stack, stop and fault instances, write and
push scenarios, run them and watch the numbers move, run validate against any target,
and read what a failure means. It is a Next.js static export served by the chaos binary
itself, so there is no Node.js process anywhere at runtime: one process, one port,
`/chaos/` for the pages and `/api/chaos/` for the data.

```
http://localhost:7700/chaos/         chaos serve on this machine (mise run chaos:serve)
http://localhost:18080/chaos/        the local cluster, through Envoy
http://chaos.<domain>/               a real environment, Envoy virtual host
```

## Pages

| Page | What it is for |
|---|---|
| Overview | Stack up or down, last validate, last run, recent failures, links into Grafana, VictoriaLogs, VictoriaMetrics, Pyroscope (and Envoy admin locally) for this environment, derived from `[links] domain` behind the public edge. The first thing to open when something looks wrong. |
| Stack | Every instance of the in-process topology with stop, start and a behaviour editor (healthy, slow, hang, error, delayed failure). Engine counters live. This is where a new feature gets exercised against faults by hand. |
| Scenarios | The scenario files. New scenario opens a template; the editor checks the TOML on every pause and saves only what checks. Run goes straight to the live run page. |
| Runs | Every run, filterable by kind, with requests, error rate, p99, checks passed and duration. Deleting a run removes its record. |
| Run | One run: stat cards, the per-second chart (req/s, p50, p99, error %), assertions with bound and observed value, the timeline as it fired, per-operation and per-target breakdown, error classes. Live while running, replayed when opened late, complete when finished. Cancel stops load and tears down. |
| Load | Ad-hoc load: rate, duration, warmup, timeout, concurrency, constant or ramp, operation weights, against the serve stack or any protocol URL. |
| Validate | The eleven checks against the config targets or any pair of URLs, with latency and detail per check. |
| Runbook | Symptom, what it means, where to look, what to do. Add an entry whenever a failure taught something. |

Every number on these pages comes from the API described in [api.md](api.md). The UI
holds no state of its own beyond what is on screen.

## Developing it

```sh
mise run chaos:serve      # terminal 1: API on :7700 with the dev topology
mise run ui:dev           # terminal 2: Next.js dev server on :3001, talks to :7700
```

`next dev` runs on 3001 because Grafana takes 3000 on this machine. In development the
pages call `http://127.0.0.1:7700/api/chaos` (set `NEXT_PUBLIC_CHAOS_API` to point
elsewhere); in a build they call `/api/chaos` on their own origin, which is why one
export works under `chaos serve`, Envoy's prefix routes and the `chaos.<domain>` host.

```sh
mise run ui:check         # prettier, eslint (React Compiler rules), tsc
mise run ui:build         # static export into ui/chaos/out
mise run chaos:serve      # picks ui/chaos/out up and serves it at /chaos
```

`mise run local:build` builds the UI first, so the chaos image in the local cluster
always carries the current pages. `mise run ci` runs `ui:check`; CI's `ui` job runs the
same plus a build.

`mise run ui:e2e` drives the deployed UI in headless Chromium (Playwright,
`ui/chaos/e2e/smoke.mjs`): every page renders real data, a fault applies from the
dialog, an engine stops and starts, a scenario run streams to the end, an ad-hoc load run
cancels, validate passes, dark mode toggles, and no console error occurs. It targets
`http://localhost:18080/chaos` (the local cluster) unless `UI_BASE` says otherwise, and
leaves screenshots under `ui/chaos/e2e/shots/`. It is not in CI because it needs a
running cluster.

## How it is put together

```
ui/chaos/src
├── app/                       one folder per page, all client components
│   ├── layout.tsx             theme, sidebar, header, toaster
│   ├── page.tsx               overview
│   ├── stack/  scenarios/  scenarios/view/  runs/  runs/view/  load/  validate/  runbook/
├── components/
│   ├── ui/                    shadcn/ui primitives (generated; regenerate, do not hand-edit)
│   ├── shell/                 app-sidebar, site-header, providers (overview + live feed context), nav
│   ├── instances-table.tsx    stack table with stop/start/fault
│   ├── behavior-dialog.tsx    the behaviour form; emits the same JSON a timeline uses
│   ├── load-chart.tsx         recharts line chart over LoadSnapshot samples
│   ├── runs-table.tsx, stat-card.tsx, status-badge.tsx, page-header.tsx
└── lib/
    ├── api/schema.ts          Zod schemas, one per type in api.md
    ├── api/client.ts          fetch wrapper (parses through Zod) and SSE subscribe
    ├── api/hooks.ts           useFetch (poll), useGlobalFeed, useStack, useRunFeed
    └── format.ts              ms, pct, ago, ...
```

Stack: Next.js 16 App Router with `output: "export"`, React 19, Tailwind CSS 4, shadcn/ui
(Base UI primitives, `render` prop instead of `asChild`), Recharts, Zod, Lucide,
next-themes, sonner. No server components do work: every page is `"use client"` and
fetches from the API, which is what makes the static export possible.

Rules the code follows:

- Every response is parsed through its Zod schema in `lib/api/schema.ts`. A field added
  to a Rust type in `crates/chaos/src/api` is added there in the same commit, and to
  [api.md](api.md).
- Dynamic pages use a query string (`/runs/view/?id=...`) rather than a dynamic route
  segment, because a static export cannot enumerate run ids. `useSearchParams` pages sit
  under `Suspense`.
- Live data comes over Server-Sent Events (`useRunFeed`, `useGlobalFeed`); lists poll.
  No websocket, no client-side store.
- Nothing is mocked. The pages render from the real API or show the error.
- The React Compiler lint rules are on (`react-hooks/set-state-in-effect`, refs during
  render). Derive state instead of syncing it in effects; `useRunFeed` shows the pattern.

## Layout

The shell is the shadcn sidebar layout used by admin dashboards such as the shadcnblocks
Admin Kit: collapsible icon sidebar with grouped navigation and a footer status, a header
with breadcrumb and theme toggle, content as cards on a responsive grid. The kit itself is
a paid download and is not vendored here; its blocks can be dropped into
`components/` later without changing the data layer.
