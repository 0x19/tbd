<!-- tbd new service cv --kind grpc --port 50056 --metrics-port 9470 --bacon-key c (tbd-cli 0.1.0) -->
# crates/cv

The cv service: the full CV behind sign-in and the owner's approval. gRPC only.
Scaffolded by `tbd new service` (docs/tbd/README.md); `Ping` stays the labelled stub
beside the real RPCs. `docs/cv/README.md` is the contract.

- `lib.rs`: `serve`, `serve_on`, `serve_with`. With `[store] url` the service is real;
  without it every RPC but `Ping` answers `UNAVAILABLE`. The private fields
  (`private::load`) and the notifier (`notify::Notifier::new`) are each optional the
  same way, and the template is warmed once at start off the runtime.
- `service.rs`: the RPCs. Every one starts with `admit()`. Identity is
  `Principal::from_headers` on `x-jwt-payload`, as in finance; the **owner** is whoever
  the token says is `admin` (`owner()`), the first Rust handler in the tree to check a
  role -- Envoy still gates the host, this decides who may list and decide. A request
  is keyed by the verified subject; `RequestAccess` needs an e-mail in the token
  (`INVALID_ARGUMENT` otherwise). `DownloadCv` renders for the one caller, off the
  runtime under `[render] timeout_secs`, and records the download with the
  `user-agent` and the first `x-forwarded-for` the gateway forwarded.
- `store.rs`: one row per subject in `cv.requests` (migration 0028); asking again
  renews a refused or revoked row (back to `requested`, fresh clock, `notified_at`
  cleared) and leaves an approved one approved. `Decision::transition` is the whole
  state machine: approve from requested/refused/revoked, refuse from requested, revoke
  from approved; anything else is `FAILED_PRECONDITION` naming the state. Downloads
  are rows of `cv.downloads`.
- `notify.rs`: mail through the finance service's linked mailbox, over Envoy's internal
  listener (`[finance] url`), as this service's own subject `svc:cv` in an
  `x-jwt-payload` it writes itself (the internal listener neither checks nor strips it;
  that is the trust model today). The mailbox is found by address (`[notify] from`:
  linked, can send, `external_id` matches) and cached until finance says NOT_FOUND.
  Sent on a detached task after the row commits: the internal listener's 5 s per-try
  timeout would retry a slow Gmail send and send twice, so the RPC never waits on it;
  `notified_at` on the row says when the owner's mail went out. The approval mail to
  the requester is best effort (finance's `allow_to` refuses strangers in `local`).
  `svc:cv` must hold `read` on the party that owns the mailbox: `cv grant
  --owner-subject <sub>` (mise `cv:grant`) does that once.
- `render.rs`: `assets/cv.typ` and `assets/cv.json` compiled in and set through
  `tbd-render`; the JSON is written from `ui/www/src/data/site.ts` by `mise run www:cv`
  (which also renders the public PDF from the same two files) and CI checks it is
  current. `private` and `reader` go in as `sys.inputs`; without them the render is the
  public CV.
- `private.rs`: the fields only the full CV carries, from `CV_PRIVATE_JSON` (the
  `cv-private` Secret) or a path outside the repository: phone, address, references, and
  `experience` -- the private half of a position, matched to the public entry by company
  name: its paragraph replaces the public one, its lines follow the public lines -- and
  `summary` / `achievements`, which replace the public summary and lead Selected work in
  the full CV. Never in git; the `Debug` form never prints a value, and a
  test holds that.
- `config.rs`: layered TOML (`configs/cv`), `deny_unknown_fields`; `CV_*` overrides.
  `[notify] url` is the gated site; `admin_url()` derives the admin page from it.
- `main.rs`: `config`, and `grant` (the one-shot above).

Invariants:
- Private fields never enter git, a log line or an unapproved response. The public
  render carries none of them; `render::tests` and `private::tests` hold that.
- A stub says so on the wire: `PingResponse.stub` is `true`.
- `TCP_NODELAY` is set on `TcpIncoming`, not the server builder.
- Services never address each other directly; finance is reached through Envoy's
  internal listener, and the mail is the only thing this service asks of it.

Tests: `tests/it/main.rs` boots the server on port 0 with the shipped `configs/cv` and
env `local`; `start_with_store()` gives a fresh migrated database on the shared test Postgres
(`tbd_db::testing`: the one reusable container, or `TBD_TEST_DATABASE_URL`). `access.rs` is the whole flow; `mail.rs` boots a real finance
(`tbd_finance::serve_with_kinds`) with a mock mailbox on the same database and proves
the owner's mail goes out and the stranger's does not.
