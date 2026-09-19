<!-- tbd new service finance --kind grpc --port 50054 --metrics-port 9468 --bacon-key f (tbd-cli 0.1.0) -->
# crates/finance

The finance service. gRPC only. Scaffolded by `tbd new service` (docs/tbd/README.md);
`Ping` is a labelled stub until the service's real RPCs land beside it.

- `lib.rs`: `serve` (binds `[server] listen`), `serve_on` (caller-supplied listener,
  default `Runtime`), `serve_with` (listener plus a `Runtime`). Tests and the chaos
  tool use the last two on port 0.
- `service.rs`: the `FinanceService` trait impl on `Finance`. Every RPC starts with
  `admit()`: count the request, start the `RequestTimer`, apply the fault handle, map
  a `Fault` to a gRPC status.
- `config.rs`: layered TOML, `configs/finance/base.toml` < `<env>.toml` < flags and
  `FINANCE_*` environment variables (`Overrides`). Every key lives in `base.toml`;
  `deny_unknown_fields` makes a mistyped key fail at start. `finance config` prints
  the effective result.
- `main.rs`: the only file that prints. Subcommands are one-shots with no telemetry:
  `config`, `import` (prototype JSON), `adopt` (a prototype consent as a connection),
  `categorise`, `session` (one signed GET, spends no allowance), `sync` (one tick, or
  `--account` a manual refresh).
- `import.rs`: the **only** place provider JSON becomes rows (`ingest_pages`,
  `ingest_balances`), audited against 2,861 real Erste rows. The importer and the
  syncer both call it; a second parser would drift into wrong money without an error.
- `banking/`: `Provider` trait (hands back JSON *unparsed*, see above), the Enable
  Banking client (`ring` RS256, pages by repeating the date window because Erste 422s a
  continuation key sent alone), `Mock` with every failure mode and a call counter,
  `connect.rs` (consent: `state` is ours and single-use, a foreign party gets `NotFound`),
  `adopt.rs` (one-time migration of the prototype's consents).
- `sync.rs`: the worker. Budget, backoff and watermark are columns on the account, decided
  under `for update skip locked` and committed *before* the network call. Three of four
  daily fetches are the scheduler's; the fourth is a person's. Measured on Erste
  (`prototype/bank/FINDINGS.md`): unattended history is 90 days whatever is asked; the
  cap is four fetches per account per calendar day, the fifth is a 429, and a call carrying
  `Psu-Ip-Address` is counted like any other, so an attended refresh is labelled but not
  exempt. `tick(now)` is a pure step for tests; `run` loops it.
- `invoice/`: drafts, previews, approvals (`store.rs`), the gapless counter
  (`numbering.rs`, a locked row, never a sequence), integer totals (`totals.rs`), and
  the Typst renderer (`render.rs`: template, Inter and the mark compiled in; PDF id
  and date pinned, so a render is a pure function of the document). The approval
  names the preview's content hash; a changed draft is FAILED_PRECONDITION. See
  `docs/finance/invoice.md`. `service_invoices.rs` holds the RPCs.
- `connectors/`: linked external accounts documents are pulled from (mailboxes today,
  portals later). `mod.rs` is the registry and the `Connector` trait -- adding a kind is
  one file and one line; the UI reads the registry. Credentials are sealed at rest
  (`crypto.rs`, ChaCha20-Poly1305 under `FINANCE_CONNECTOR_KEY`, bound to the row id) and
  only `store.rs` opens them, per call. `gmail.rs` links through Google OAuth (read-only
  scope) and keeps PDF attachments; a pull holds a session whose access token is re-minted
  before Google's hour is up and once more if refused mid-pull (a throttled pull outlives
  the token; one died at 3,611 s and read as a broken link), so only a refused *refresh*
  marks the row expired, and a later good pull or test clears that mark; a second query (`body_query`, Gmail's own syntax)
  finds receipt mails with nothing attached, fetches the Stripe-hosted invoice a mail names
  (`/pdf` under the link) or prints the mail (`documents::mail`); a pulled message is never
  pulled twice, identical bytes are one document with several sources. `SyncConnector` opens a run row and
  returns it; the pull runs on a detached task (`store::begin_sync` / `run_sync`),
  because a mailbox takes minutes and the internal Envoy route gives a unary call
  five seconds -- a handler that pulls inline is cancelled mid-way and its run never
  finishes. A kind sends each document into a channel as it fetches it and the store
  drains the other end, counting on the run row per document, so the run is the
  progress. A kind pulls in capped rounds and says whether it reached the present; a
  run that did not is `partial` and leaves the watermark alone. One run per connector
  at a time; `serve_with` closes every run a predecessor left open at start, and one
  older than 30 minutes is closed when the next opens. `WatchConnectors` streams the
  view's connectors with their latest run, then every change (a one-second diff of
  the store per watcher) -- the page draws from it and never polls while it is up.
  `ConfigureConnector` may move a connector to another party in the grant, taking
  the documents only it pulled. `service_connectors.rs` holds the RPCs. Tests inject
  a mock kind through `serve_with_kinds`. A kind may also *send*: `capabilities` says
  so from the consent it holds (Gmail asks for `gmail.send` too; a mailbox linked before
  that is read-only until linked again, `connectors.can_send` records it), `send` builds
  the MIME itself (text, attachments, `In-Reply-To` and Gmail's `threadId` for a reply),
  and `replies` reads a thread it sent for what others answered.
- `mail/`: mail sent as a linked mailbox and kept. `store.rs` holds templates (a name per
  party; `{{Month}}`-style helpers are filled on the page, the service stores what was
  sent), `send` (refuses a mailbox that cannot send, an implausible address, no subject,
  and any recipient outside `[mail] allow_to` when that list is non-empty -- `local.toml`
  lists only the owner's two addresses so a test never reaches an accountant; production
  lists none, so nothing is guarded there; a provider failure is a `failed` row, not an
  error), the list and thread reads, and `import_replies`, which `run_sync` calls at the
  end of every pull: each reply is an `in` row under its parent and its PDF a receipt of
  the party, never imported twice (`provider_id` unique per connector). A send may carry
  the accountant's *bundle*: the page sends the text files (README, summary, missing)
  with their bytes and the receipts as document ids with the name each takes, and the
  service fetches the receipts and writes one stored zip (`zip.rs`, the browser's
  `zip.ts` in Rust: no dependency, PDFs do not compress), so a month of PDFs never has
  to fit the gateway's 2 MiB body; the zip's name is on the row (`mails.bundle`) and its
  receipts are linked like any attachment. `service_mail.rs` holds the RPCs.
- `documents/`: what a pulled receipt says. `pdf.rs` turns the bytes into text in-process
  (`pdf-extract`, fenced against its panics, off the runtime; no OCR, a scan reads as
  empty and says so). `fields.rs` reads vendor, date, amount and number out of the text
  with labelled rules, each field carrying *how* it was found (`By`: label, sender,
  first, received) so the page can show a guess as a guess; its tests are the real
  layouts (Stripe, Google, Hetzner, Medium, our own Typst). `store.rs` lists with search
  (one `ilike` over the columns, the mail's subject and sender, and the text), the vendor
  counts and the match total from one `where`; `update` declares a person's corrections
  (`declared_at`), which `write_read` never overwrites. The reader runs on every document
  as it is stored, on every unread one at start-up (`backfill`, after the run sweep), and
  on `ExtractDocument`. The reader breaks words across glyph runs, so the number is also
  sought with every space removed. `party.rs` decides whose a document is: a mailbox belongs
  to a party but its mail does not, so the party comes from the account that paid (one
  candidate party has that debit near that date), else the text (the company's name or OIB
  beats a person's name), else the mailbox; a person's choice through `UpdateDocument` is
  final, `extracted.party` records which, and a re-read keeps it. `mail.rs` prints a
  receipt mail that carried no file to a PDF (the same Typst engine and fonts as the
  invoice), so the accountant gets a page. `UploadDocument` takes what no connector can
  reach -- a PDF from a vendor's portal, a photo of a paper receipt -- as the caller's
  party, declared, deduplicated on the bytes, and read at once; a source row with no
  connector (`upload:<sha>`) says where it came from. `service_documents.rs` holds the RPCs.
- `reconcile/`: the accountant's month. `mod.rs` is two pure rule sets with their tests
  on the real August statement: *need* (what the accountant needs from us: `eracun` for an
  HR IBAN, since domestic B2B is e-invoiced; `none` for state-budget references (HR68),
  payouts to a person (HR69 40002), cash and bank fees; `receipt` for card charges and
  foreign transfers) and *score* (a receipt against a charge: the original amount the
  card was charged in, read from the remittance, then vendor token and date; `LINK` makes
  an inferred link, `SUGGEST` an offer). A person's counterparty policy or hand-made link
  overrides either; an undone inferred link stays `rejected` so the matcher does not make
  it again. A hand-made link is checked the same way: a receipt whose amount was read
  and agrees with nothing about the charge is refused (FAILED_PRECONDITION, saying what
  was read and what the charge is) unless `force`; the link's reason records `checked`,
  `forced` or `unread` (a photo nothing was read from), so the bundle can say which. `store.rs` runs the matcher greedily, best pairs first, each side once. A
  reason is a `Why`: a code with arguments (`card_original` with the amount, `days_apart`
  with the days), stored on the link as `code:arg|code`, sent as `Reason` for the page to
  say in its language, and spelled out in English beside it for other callers.
  `service_reconcile.rs` holds the RPCs.
- `categorise.rs` + `seeds/rules.sql`: a pass clears every `inferred` categorisation and
  reapplies rules in priority order; `declared` always survives. A text condition is a
  substring unless anchored (`^INA ` pins the start, ` BAR$` the end): unanchored `INA `
  claimed Lesnina, Perutnina, Fina and every "trgovina". Patterns are normalised by
  `finance.normalise` on the way in (`money::upsert_rule`), the same function the pass
  applies to the row, so punctuation agrees -- the Rust `import::normalise` turns
  `NAME-CHEAP` into `NAME CHEAP` and must not be used for patterns. `money::upsert_category`
  makes categories from the UI (slug from the name, kept on rename; archiving disables the
  rules pointing at it and reruns the pass). `GetTransaction` is the one read that carries
  the bank's raw record. The seed is idempotent on
  `(party_id, name)`; a rule joins to its category by slug and a typo drops it silently,
  so `tests/it/seed.rs` counts.

- Observability: `tbd_common::telemetry::grpc_request_span` is the `trace_fn`, so every
  call gets a `grpc.request` span with the caller's `traceparent` adopted and
  `trace_id` recorded; `admit()` starts the `RequestTimer`. Metrics listen on
  `[metrics] listen` (`FINANCE_METRICS_ADDR`), `None` for embedders.

Invariants:
- A stub says so on the wire: `PingResponse.stub` is `true` until a real implementation
  replaces it, and the tests assert it. Do not let a placeholder look like a
  measurement.
- `TCP_NODELAY` is set on `TcpIncoming`, not the server builder. With a caller-supplied
  listener the builder setting does nothing, and small responses stall 40 ms.
- Services never address each other directly; a caller reaches this one through
  Envoy's internal listener (`http://envoy:50051`, matched by service name).

Tests: `tests/it/main.rs` boots the server on port 0 through `support.rs` with the
shipped `configs/finance` and env `local`, and exposes the `Runtime` so tests can
inject faults and read counters. `start_with_store()` gives a fresh migrated database
(testcontainers, or `TBD_TEST_DATABASE_URL`). `banking.rs` runs the real client against
wiremock; `sync.rs` and `connect.rs` run the worker and the consent flow against the
`Mock` bank, counting its calls -- the interesting number is usually zero.
