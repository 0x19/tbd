# The chaos admin UI (`ui/chaos`)

A web front end for `chaos serve`: see the stack, stop and fault instances, write and
push scenarios, run them and watch the numbers move, run validate against any target,
and read what a failure means. It is a Next.js static export served by the chaos binary
itself, so there is no Node.js process anywhere at runtime: one process, one port,
the pages at the root and `/api/chaos/v1/` for the data.

```
http://localhost:7700/               chaos serve on this machine (mise run chaos:serve)
http://chaos.localhost:18080/        the local cluster, through Envoy's chaos.* virtual host
https://chaosadmin.<domain>/         a real environment, through the public edge
```

`chaos.localhost` needs no DNS entry: browsers resolve every `*.localhost` name to
loopback. From another machine on the LAN, one `/etc/hosts` line mapping any name that
starts with `chaos.` to the server does the same. `curl` needs `-H 'Host: chaos.localhost'`.

## Pages

| Page | Kit pattern | What it is for |
|---|---|---|
| Overview | Ecommerce dashboard 1 + Developers overview | KPI strip (stack health, last validate, runs and failure rate over 24 h with deltas against the previous day), throughput of the last scenario run over the previous run of the same scenario, latency p50/p90/p99 per scenario as stacked bars, stack list, live activity, where-to-look links, recent runs. |
| Stack | Payments webhooks | Two summary cards (integrity bar per instance, engine traffic), an instance list with health dots and Stop / Start / Fault, and a detail sheet per instance with key/value rows and the session's activity. |
| Scenarios | Ecommerce product list | Title with "Run all" (queues every ready scenario) and the primary action, status tabs, search toolbar, table with last run chip and numbers, row menu (edit, runs, schedule, delete). The queue panel appears while something waits. |
| Scenario | Ecommerce order detail | Back arrow, big name with status chips, a stage strip that previews setup / warmup / load + timeline / assert / teardown with the timeline actions, the TOML editor checked as you type, and a right rail with stack, load, assertions and last run. |
| Runs | Developers events & logs | The queue panel (what waits for the slot, remove one or clear all), filter rail (kind, status, scenario with counts), search, refresh and Live, table with a totals row and a "scheduled" chip on runs a schedule started. Sidebar sub-items deep-link by kind. |
| Run | Payments delivery simulator + order lifecycle | Lifecycle strip, stat row, the per-second chart, assertions, timeline with OK/ERROR chips, a latency heat grid per operation (darker is slower), error classes, per target and service. Live while running, replayed when opened late. |
| Load | Payments delivery simulator | A policy rail (shape, operations, target; unit suffixes and info tooltips; expected requests and concurrency computed live), a load window stat row, previous load runs with a throughput sparkline, the request body. |
| Validate | Developers events & logs | Targets form, stat row, filter rail by surface and result, PASS / FAIL chips with latency and detail, totals row, previous validate runs. |
| Schedules | Ecommerce product list + settings dialog | Cron jobs: a switch per row, what runs, the preset or raw cron, next fire time, last run chip, fired and skipped counts, "Run now" and a row menu; a dialog to create or edit (name, what to run with a scenario picker or rate and duration, a preset or custom cron in UTC, enabled). `?new=scenario:<id>` opens it prefilled from a scenario's row menu. |
| Runbook | Original settings | Vertical section nav with observability links, titled entries with "look at" and "then" columns. |

The shell is the kit's: a workspace block and grouped, collapsible navigation in the
sidebar with an environment block at the bottom; an app bar with ⌘K search,
notifications (finished runs of this session), theme toggle and the environment pill;
a breadcrumb bar with the command search. ⌘K opens a command palette with pages,
"run scenario …" for every scenario that checks, and actions.

Every number on these pages comes from the API described in [api.md](api.md). The UI
holds no state of its own beyond what is on screen.

## Developing it

```sh
mise run chaos:serve      # terminal 1: API on :7700 with the dev topology
mise run ui:dev           # terminal 2: Next.js dev server on :3001, talks to :7700
```

`next dev` runs on 3001 because Grafana takes 3000 on this machine. In development the
pages call `http://127.0.0.1:7700/api/chaos/v1` (set `NEXT_PUBLIC_CHAOS_API` to point
elsewhere); in a build they call `/api/chaos/v1` on their own origin, which is why one
export works under `chaos serve`, Envoy's `chaos.*` host and the public edge.

```sh
mise run ui:check         # prettier, eslint, tsc
mise run ui:build         # static export into ui/chaos/out
mise run chaos:serve      # picks ui/chaos/out up and serves it at /
```

`mise run local:build` builds the UI first, so the chaos image in the local cluster
always carries the current pages. `mise run ci` runs `ui:check`; CI's `ui` job runs the
same plus a build.

`mise run ui:e2e` drives the deployed UI in headless Chromium (Playwright,
`ui/chaos/e2e/smoke.mjs`): every page renders real data, a fault applies from the
dialog, an engine stops and starts, a scenario run streams to the end, an ad-hoc load run
cancels, validate passes, dark mode toggles, and no console error occurs. It targets
`http://chaos.localhost:18080` (the local cluster) unless `UI_BASE` says otherwise, and
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
│   ├── shell/                 app-sidebar, site-header, command-menu (⌘K), providers (overview, live feed, activity), nav
│   ├── kit.tsx                the kit's building blocks: PageTitle, KpiStrip, StatRow, StageBar, FilterRail, HeatGrid, Sparkline, DetailList, LevelChip
│   ├── charts.tsx             monochrome recharts: CompareChart, RunChart, LatencyBars, ChartHeadline, Legend
│   ├── behavior-dialog.tsx    the behaviour form; emits the same JSON a timeline uses
│   ├── runs-table.tsx, status-badge.tsx, instances-table.tsx (describeBehavior)
└── lib/
    ├── api/schema.ts          Zod schemas, one per type in api.md
    ├── api/client.ts          fetch wrapper (parses through Zod) and SSE subscribe
    ├── api/hooks.ts           useFetch (poll), useGlobalFeed, useStack, useRunFeed
    ├── runs.ts                windows, deltas, latest run per scenario
    └── format.ts              ms, pct, ago, ...
```

Stack: the Admin Kit's own: Next.js 16 App Router with `output: "export"`, React 19,
Tailwind CSS 4, shadcn/ui on Radix (`asChild` composition), Recharts, Zod, Lucide and
Tabler icons, next-themes, sonner, cmdk, the kit's ESLint (simple-import-sort) and
Prettier (tailwind plugin). Fonts are Inter and Geist Mono, as in the kit. No server components do work: every page is `"use client"` and
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
- ESLint is the kit's: imports sorted, unused variables an error, the React Compiler
  effect rules relaxed as the kit relaxes them.

## Layout

`ui/chaos` *is* the shadcnblocks Admin Kit (https://www.shadcnblocks.com/admin-dashboard,
v2.3.0, Premium ZIP), reduced the way the kit recommends: keep the product you are
building, delete the rest. Kept from the kit, file for file: the shell
(`components/layout/`: sidebar with team switcher and collapsible nav groups, header with
⌘K, notifications and theme controls, the breadcrumb sub-header without the kit's second
search box), the theme preset
picker with its presets, the `components/ui/` primitives (shadcn on Radix), the global
CSS, ESLint and Prettier. Deleted: the five demo sub-apps, auth and error pages, their
mock data, and the packages only they used (editor, maps, PDF, drag and drop, forms,
faker). Ours on top: the pages, the data layer, `components/kit.tsx` (page title, KPI
strip, stat row, stage bar, filter rail, heat grid, detail list, level chips, all
modelled on the kit's payments, developers and ecommerce pages) and `charts.tsx`.

The kit is licensed per buyer and is not in git. To pull a kit update, diff the new
ZIP's `src/components/{ui,layout}`, `src/lib/theme-*` and `globals.css` against ours and
port what changed; everything else here is independent of the kit's demo apps.
