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
- Pages: `/` (overview: in vs spent by month, a month's categories), `/transactions/`
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
  then scrubs the URL), `/invoices/` (drafts, preview, approve, PDF), `/connectors/`
  (one flat list drawn from the `WatchConnectors` SSE feed via `useEvents`: a pull's
  progress and outcome, a relink, a removal arrive as events; `useFetch` polls only
  while the feed is down. Link a mailbox chooses the party; the row's party select
  moves it through `ConfigureConnector`), `/connectors/callback/`, `/documents/`
  (receipts as a ledger: search, vendor and month filters and the sums come from
  `ListDocuments`; a row opens a sheet with the PDF inline, the fields with how each was
  found (`found_by`), an editor that declares corrections through `UpdateDocument`, and
  "Read again" = `ExtractDocument`).
- Streams: `useEvents(url, schema, onEvent)` in `hooks.ts` wraps `EventSource` with
  credentials; every `data:` frame is one JSON message. Envoy keeps `/v1/**/events`
  open on the finance host. EventSource cannot send the dev bearer token, so in
  development the feed shows "Polling" and the list refreshes on a timer.
- Static export, no `basePath`, `trailingSlash: true`; detail-less by design so far.
  `pnpm dev` is on 3004.
- Checks: `mise run ui:finances:check` (prettier, eslint, tsc) is part of `mise run ci`;
  `ui:finances:build` before the image; `ui:finances:e2e` against the local cluster.
- Served by Caddy from `devops/docker/Dockerfile.finances` (the `ui/www` pattern), never
  by the chaos binary: this host carries personal accounts.
