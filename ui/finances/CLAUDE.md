@AGENTS.md

# ui/finances

The finance UI: `ui/chaos` copied and reduced to its shell, with pages for money
instead of scenarios. Same kit (the shadcnblocks Admin Kit, `/mnt/development/0x19/admin-kit/`),
same conventions; read `ui/chaos/CLAUDE.md` for what is the kit's and what is ours.

- The API is the protocol's REST surface for `tbd.finance.v1.FinanceService`
  (`proto/tbd/finance/v1/finance.proto`, `docs/protocol/README.md`): same origin in a
  build (Envoy serves the UI at the root of `finance.*` and routes `/v1/` to the
  protocol), `http://127.0.0.1:18080` under `next dev` with a token in
  `NEXT_PUBLIC_FINANCE_TOKEN` (`NEXT_PUBLIC_FINANCE_API` overrides the host).
  `src/lib/api/schema.ts` mirrors the proto as the transcoder renders it: proto field
  names, int64 as a JSON string, every field present. Change both in the same commit.
- **Money never becomes a float.** Amounts are minor-unit strings on the wire and
  `BigInt` in `src/lib/summary.ts`; `money()` formats digits, `chartValue()` converts
  for an axis only.
- **The scope is the grant.** `Providers` fetches `ListParties`; the personal/business/
  combined toggle (`scope-toggle.tsx`, the sidebar footer, ⌘K) chooses among _those_.
  No page ever holds a party id the server did not list.
- Pages: `/` (overview: one period -- month, quarter or year, `PeriodControl` -- and every
  widget in `src/components/dashboard/` follows it: stats with the previous period, money
  in vs spent with the period lit and a year-over-year tab, a category donut, per-month
  minis, accounts with sync health, invoices, receipts, recent rows, and a year table.
  All of it is client-side slices of one 24-month `MonthlySummary` plus the list calls;
  the period maths are pure functions in `src/lib/summary.ts`), `/transactions/`
  (filters in the URL, inline recategorise = `DeclareCategory`; a row opens
  `transaction-sheet.tsx`, which fetches `GetTransaction` for the bank's record and the
  rule that claimed it, and "Make a rule from this" prefills `rule-dialog.tsx`; the
  dialog previews what a condition would claim from the newest hundred rows mentioning
  it), `/categories/` (two tabs: categories as cards with twelve months of use from
  `MonthlySummary`, edited through `category-dialog.tsx` = `UpsertCategory`; rules
  grouped by category with search and the claims-nothing filter; `rule-dialog.tsx`
  saves through `UpsertRule`, which reapplies at once),
  `/accounts/` (balances, sync state, "Fetch now" = `RefreshAccount`), `/connections/`
  (`StartConnection` sends the browser to the bank), `/connect/callback/` (the
  registered redirect: reads `state`+`code` from its URL, POSTs `CompleteConnection`,
  then scrubs the URL), `/invoices/` (drafts, preview, approve, PDF), `/clients/` (a table; a sheet per
  client with its details and line templates), `/issuer/` (one form in sections), `/connectors/`
  (one flat list drawn from the `WatchConnectors` SSE feed via `useEvents`: a pull's
  progress and outcome, a relink, a removal arrive as events; `useFetch` polls only
  while the feed is down. Link a mailbox chooses the party; the row's party select
  moves it through `ConfigureConnector`), `/connectors/callback/`, `/documents/`
  (receipts as a ledger: search, vendor and month filters and the sums come from
  `ListDocuments`; a row opens a sheet with the PDF inline, the fields with how each was
  found (`found_by`), an editor that declares corrections through `UpdateDocument`
  (including whose it is: `party_id`; `found_by.party` says whether the account that
  paid, the text or the mailbox decided), "Read again" = `ExtractDocument`, and
  "Upload" = `UploadDocument` through `upload-receipt.tsx`, which shrinks a photo to fit
  the gateway's 2 MiB body and refuses a larger PDF with the size), `/accountant/` (the company's month from
  `MonthlyReconciliation`: each transaction's need and its receipt; a policy select per
  counterparty = `SetCounterpartyPolicy`, suggestions and "Find" = `LinkDocument`, "Attach"
  on a missing row = `UploadDocument` then `LinkDocument`. Every hand-made link goes
  through one flow: a `failed_precondition` from the service means the receipt's reading
  disagrees with the charge, and a dialog shows the service's words with "Attach anyway"
  (`force`); the toast says whether the amount was checked or nothing could be read; the
  bundle -- receipts, summary.csv, missing.csv, README -- is zipped in the browser by
  `src/lib/zip.ts`, since a month of PDFs would not fit one gRPC message).
- Streams: `useEvents(url, schema, onEvent)` in `hooks.ts` wraps `EventSource` with
  credentials; every `data:` frame is one JSON message. Envoy keeps `/v1/**/events`
  open on the finance host. EventSource cannot send the dev bearer token, so in
  development the feed shows "Polling" and the list refreshes on a timer.
- **Two languages.** `src/lib/i18n/` holds the provider (`LangProvider`, the choice in
  `localStorage`, the browser's language first), `useT()` and per-namespace dictionaries in
  `messages/<ns>.ts` (`en` and `hr` maps keyed `<ns>.<slug>`; `common.*`, `nav.*`,
  `status.*` are shared). Every visible string goes through `t()`; sidebar titles and
  breadcrumbs are keys; `StatusBadge` translates wire words; `format.ts` follows the
  language for dates and counts (`money()` does not). Server reasons arrive as codes with
  arguments (`Reason`) and are translated on the page (`need.*`, `why.*`); the accountant
  bundle's file names, CSV headers and README follow the language too. API error messages
  stay English. A missing Croatian key falls back to English, a missing key shows itself.
- Static export, no `basePath`, `trailingSlash: true`; detail-less by design so far.
  `pnpm dev` is on 3004.
- Checks: `mise run ui:finances:check` (prettier, eslint, tsc) is part of `mise run ci`;
  `ui:finances:build` before the image; `ui:finances:e2e` against the local cluster.
- Served by Caddy from `devops/docker/Dockerfile.finances` (the `ui/www` pattern), never
  by the chaos binary: this host carries personal accounts.
