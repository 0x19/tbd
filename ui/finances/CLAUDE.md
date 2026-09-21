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
  `/accounts/` (balances, sync state, "Fetch now" = `RefreshAccount`; a card links to
  its bank), `/connections/` ("Banks": a strip -- banks connected, accounts fetched,
  the consent ending soonest, fetches left today -- and one `consent-card.tsx` per
  consent with its life in a sentence (`src/lib/banking.ts`: `consentState`, `nextFetch`,
  `fetchesLeft`, pure over the wire rows) and its accounts under it: balance, last and
  next fetch, fetches today of four, the last outcome, the schedule switch, Fetch now;
  Renew = `StartConnection` with the card's party and login (the callback moves the
  accounts), Fetch all, Remove = `DeleteConnection` after a confirm; `link-dialog.tsx`
  chooses party and login and says what the bank will ask; `use-account-actions.ts`
  holds fetch and switch with every outcome's toast, shared with `/accounts/`),
  `/connect/callback/` (the
  registered redirect: reads `state`+`code` from its URL, POSTs `CompleteConnection`,
  then scrubs the URL), `/invoices/` (drafts, preview, approve, PDF; the filters are page state mirrored into the
  URL with `history.replaceState`, never a router navigation, since a query-only
  `router.replace` did not re-render; one client select filters the list and is the
  client "New draft" is for, opening on the company's default client (`is_default`,
  set on `/clients/`) when the URL names none, else the client invoiced last;
  a row duplicates into a new draft, a draft row deletes after a confirm dialog, `n`
  starts a draft; the view edits a
  draft's client, VAT treatment, currency and series beside its dates and lines; on an
  issued invoice `components/invoices/payments-card.tsx` lists what settled it -- the
  matcher's bank matches and hand records -- with Undo = `UnlinkPayment` and "Record
  payment" = `RecordPayment`, either a credit picked from `ListTransactions` searched
  by the client's name or an amount and a day the bank has not shown; the list's
  Outstanding and Overdue tiles count what is still owed, `total - paid`), `/clients/` (a table with a "default" badge and a "Make default" action per row =
  `SetDefaultClient`, one per company; a sheet per client with its details and line
  templates), `/issuer/` (one form in sections), `/connectors/`
  (one flat list drawn from the `WatchConnectors` SSE feed via `useEvents`: a pull's
  progress and outcome, a relink, a removal arrive as events; `useFetch` polls only
  while the feed is down. Link a mailbox chooses the kind (a `token` kind such as
  Moj-eRačun or e-računi turns the dialog into a credential step: the kind's fields from
  `TOKEN_FIELDS`, labels `connectors.token.<kind>.<field>`, `CompleteConnector`
  with the pasted JSON as `code` and the `state` the start returned; an `eracuni` row
  has a _Source_ select in place of Gmail's filter), the **purpose** -- read
  receipts, send mail only, or both; `StartConnector.purpose` decides the consent Google
  is asked for, and the dialog says which -- and the party. A row shows two chips from
  `can_read` / `can_send`; a row that cannot read has no Pull, Filter or History, its
  line says "send only", and Link again keeps its purpose (a send-only mailbox is never
  widened from the row: that is a new link, on purpose); the row's party select moves it
  through `ConfigureConnector`), `/connectors/callback/`, `/documents/`
  (receipts as a ledger for daily use. Filters in the URL: search, vendor, month, and a
  view -- all, need a look, corrected, no amount -- over the first 200 the server
  returns for the filters (`ListDocuments`), paged by 25 on the page. `src/lib/receipts.ts`
  is the one place a receipt is judged: `statusOf` (reading, unreadable, incomplete,
  guessed, read, corrected), `needsLook`, and `totals`, which adds up only amounts read
  from a label or set by a person, so the strip at the top never counts a guess; a
  guessed amount in a row is dotted-underlined and says so, a missing one is a dash
  with a hint (`components/documents/status.tsx` holds the chip and the provenance
  word). A row opens `receipt-sheet.tsx`: the PDF large on the left, the facts on the
  right each with how it was found (`found_by`), a sentence saying why the receipt
  needs a look, the origin (the mail's subject, sender and time, "e-računi · number" for
  the intermediary's, or "uploaded"; `provider` counts as sure in `sure()`), and the
  editor in the same column, which declares corrections through `UpdateDocument`
  (including whose it is: `party_id`; `found_by.party` says whether the account that
  paid, the text or the mailbox decided), "Read again" = `ExtractDocument`, and
  "Upload" = `UploadDocument` through `upload-receipt.tsx`, which shrinks a photo to fit
  the gateway's 2 MiB body and refuses a larger PDF with the size), `/reconciliation/` (the company's month from
  `MonthlyReconciliation`: each transaction's need and its receipt; a policy select per
  counterparty = `SetCounterpartyPolicy`, suggestions and "Find" = `LinkDocument`, "Attach"
  on a missing row = `UploadDocument` then `LinkDocument`; a note under the counterparty
  (`SetTransactionNote`, Enter saves, Escape cancels, empty removes) follows its row into
  the README, both CSVs and the mail's summary. Every hand-made link goes
  through one flow: a `failed_precondition` from the service means the receipt's reading
  disagrees with the charge, and a dialog shows the service's words with "Attach anyway"
  (`force`); the toast says whether the amount was checked or nothing could be read; the
  bundle -- receipts, summary.csv, missing.csv, README -- is zipped in the browser by
  `src/lib/zip.ts`, since a month of PDFs would not fit one gRPC message), `/mail/`
  (`src/components/mail/`, the page is the orchestration: three tabs and one draft.
  Compose is `composer.tsx`: the form and `preview.tsx` -- the mail as the recipient
  sees it, header, body, attachments, the bundle's contents -- side by side at equal
  width from `xl`, two tabs below that. The sender is a linked connector with `can_send`;
  a template of that party fills recipients, subject and body; `src/lib/mail-template.ts`
  renders `{{Month}}`, `{{MonthName}}`, `{{Year}}`, `{{MonthYear}}`, `{{Company}}`,
  `{{Today}}` from a chosen month, `{{Summary}}` from a reconciliation hand-off, and
  `{{Client}}`, `{{InvoiceNumber}}`, `{{InvoiceTotal}}`, `{{IssueDate}}`, `{{DueDate}}`
  from an invoice hand-off (`HELPER_GROUPS`; `helpers-menu.tsx` inserts one at the
  caret). Recipients are checked with `looksLikeAddress` before the service does; the
  send bar names the first thing missing; receipts attach from `ListDocuments`
  (`attach-dialog.tsx`); a confirm dialog restates recipients, subject and attachments,
  then `SendMail`. Sent & replies (`sent-list.tsx`): `ListMail` with direction and
  search, a row opens `thread-sheet.tsx` from `GetMail`, Reply prefills the composer
  (`in_reply_to_mail_id`). Templates (`templates.tsx`): cards, an editor with the same
  preview filled with sample values, `UpsertMailTemplate`/`DeleteMailTemplate`.)
  Two pages hand the composer content through `sessionStorage` (`Prefill` in
  `mail-template.ts`: `stashPrefill` / `takePrefill`, read once; `draft.ts` turns it
  into a draft): "Send to the accountant" on `/reconciliation/` hands over the month's
  _rows_ (`kind: "month"`), not rendered text; "Send by mail" on an approved invoice
  (`/invoices/` row, `/invoices/view/`) hands over the invoice's number, dates, total,
  its stored PDF's `document_id` as the attachment and the client's `recipients` as the
  addresses (`kind: "invoice"`); the composer picks that party's sender and first
  template either way. `src/lib/bundle.ts` holds the pure builders both pages share --
  `summaryText()` (the README and `{{Summary}}`) and `plan()` (the same files the
  download zips: README, summary.csv, missing.csv and each receipt's name inside the
  zip, without the receipts' bytes) -- and the composer runs them again on every
  language change, so the mail, the file names and the zip follow the UI's language at
  the moment of sending; the default subject does too until something is typed. The
  composer picks that party's sender and first template, puts the summary where
  `{{Summary}}` stands or after the body, and sends the plan as `bundle`, which the
  service zips with the receipts' bytes. Nav: Accountant holds `/reconciliation/`,
  Communication holds `/mail/`.
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
