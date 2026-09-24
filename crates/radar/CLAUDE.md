<!-- tbd new service radar --kind grpc --port 50059 --metrics-port 9472 --bacon-key r (tbd-cli 0.1.0) -->
# crates/radar

The radar service (RFC 0007, a draft in the lab): what changed in Go and Rust this
week. It reads the official sources on a timer, keeps every item it has seen, and
once a week asks the llm service for a digest per language (`go`, `rust`) and per
reader language (`en`, `hr`). The contract is `docs/quietpager/README.md`.

- `config.rs`: layered TOML (`configs/radar/base.toml` < `<env>.toml` < flags and
  `RADAR_*`). `[[sources]]` is data (atom, rss, or `github` for the issue search with
  `{since}` in the query). `validate()` runs at start. The database URL
  (`RADAR_DATABASE_URL`) and the GitHub token (`RADAR_GITHUB_TOKEN`) come from the
  environment only and are never serialised.
- `fetch.rs`: one HTTP client, a source at a time; a failing source is counted and
  logged, never fatal. Summaries are cleaned to plain text and cut on a word.
- `store.rs`: the `radar` schema (`migrations/0031_radar.sql`): `items` unique on
  `(source, guid)`, `digests` unique on `(week, language, lang)`.
- `digest.rs`: the call to the llm service as `svc:radar`, through Envoy's internal
  listener in a deployment. The llm service has no JSON mode, so the four headings in
  `HEADINGS` are the contract: an answer missing one, or with an empty section, is
  refused and nothing is stored. Reasoning chunks are dropped.
- `worker.rs`: `refresh` and `digest`, shared by the timers and the admin RPCs. The
  week written at `now` is the week of the day before, so a Monday run is last week.
- `service.rs`: reads are public; `Refresh` and `RunDigest` need the admin role on
  the caller Envoy verified. Without a store every RPC but `Ping` is `UNAVAILABLE`.

Invariants:
- A digest is written by a model and says so on every surface (`ai_written`), and
  `stub` is true when the llm's stub engine wrote it: a placeholder never looks real.
- `PingResponse.stub` is `true`, as in every service; the chaos check asserts it.
- Only official sources, and the prompt forbids inventing a release, version or date;
  a thin week says so rather than padding.
- `TCP_NODELAY` is set on `TcpIncoming`, not the server builder.
- Services never address each other directly; the llm service is reached through
  Envoy's internal listener (`http://envoy:50051`, matched by service name).

Tests: `tests/it` boots the server on port 0 with the shipped `local` config, a fresh
database on the shared test Postgres (`TBD_TEST_DATABASE_URL`), the sources served by
wiremock (external feeds, not our services), and the real llm service on its stub
engine for the digest path.
