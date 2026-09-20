# The mobile foundation

`/mobile` is the Flutter skeleton every product's phone app starts from: sign in through
the SSO that already exists, talk to the same Envoy edge with the same tokens, carry the
same traces, look like the web UIs, and be built, checked and released the same way. It
ships with the smallest real thing that proves the whole chain: **sign in, be greeted by
the backend over REST and gRPC, sign out.** Finance will be its first tenant; no product
screen lives in it.

Working on it: [`mobile/CLAUDE.md`](../../mobile/CLAUDE.md) (the invariants) and
[`mobile/README.md`](../../mobile/README.md) (devices, environments, checks).

## A bearer client

The web UIs are browser-session-gated: Envoy's OAuth2 filter sets an ID-token cookie
and every UI host redirects to `auth.<domain>` when it is missing. A native app cannot
be that. It is a **bearer client**: it holds tokens itself and speaks only to
`api.<domain>` with `Authorization: Bearer`, the door programs already use. Envoy
verifies the token and forwards the claims as `x-jwt-payload`; nothing behind it sees a
token ([auth/README.md](../auth/README.md)). That one fact shapes everything else.

```
 phone ──── system browser ────▶ auth.<domain>   Kratos sign-in pages, Hydra /oauth2/*
   │  ◀──── tbd://callback ─────       (authorization code + PKCE, client tbd-app,
   │                                    audience=tbd-api, no consent screen)
   │
   ├── REST  GET /v1/me ─────────▶ api.<domain> ──▶ Envoy (verifies) ──▶ protocol
   └── gRPC  ProtocolService/Ping ▶ api.<domain> ──▶ Envoy (verifies) ──▶ protocol
              Authorization: Bearer <JWT>, traceparent on both
```

## Decisions

| Decision | Chosen | Why, and what it costs |
|---|---|---|
| Location | `/mobile`, a pub workspace (`apps/`, `packages/`), melos for scripts | Packages make the second product a new `apps/` entry, not a fork. Cost: one more toolchain, pinned through mise. |
| Architecture | Flutter's own guide: MVVM, feature-first; views and view models in the UI layer, repositories and services in the data layer ([docs.flutter.dev/app-architecture](https://docs.flutter.dev/app-architecture)) | The documented default, so every Flutter engineer knows where things go. Some ceremony for a hello screen; the ceremony is the point. |
| State and DI | Riverpod: providers as the graph, `Notifier`s as view models | Testable without widgets, compile-time safe. Two roots (`Env`, `Session`) are overridden at bootstrap or in a test; everything else is derived. |
| Navigation | go_router with the session as `refreshListenable` and one redirect | Deep links (`tbd://`) and guarded routes in one place. |
| Sign-in | flutter_appauth: authorization code with PKCE in the system browser against Hydra's discovery document | What Ory documents for mobile; PKCE is enforced by Hydra anyway. Never an embedded web view. `prompt=login` on every sign-in, so a shared device never reuses someone else's browser session. |
| Tokens | flutter_secure_storage (Keychain, this device, first unlock; Keystore on Android); refresh ahead of expiry and on a 401 | Never in preferences, never logged. `Session` is the only owner; the client asks it for the bearer. |
| HTTP | dio with interceptors: `traceparent`; bearer with single-flight refresh (ten refused calls, one refresh; a token already replaced is retried, not refreshed again); retries with backoff for idempotent calls only; the problem body `{code, error, details}` as a closed `AppError` set | One client, one place for every cross-cutting concern. A view switches on an error type, never a status number. |
| Contract to Dart | `buf generate` with the remote Dart plugin from `/proto` into `packages/tbd_proto`; `/v1/me` is hand-written JSON on the server, so its Dart type is hand-written too, held against `docs/protocol/openapi.json` in CI | One source of truth for messages on both sides; the one hand-written type cannot drift silently. |
| Transport | REST (with an SSE reader) through the protocol, and a gRPC channel for the proto services | Both are one bearer away; the hello uses both so the skeleton is proven for either. |
| Theme | The web kit's, generated: `tool/theme.dart` reads the `:root` and `.dark` tokens of `ui/finances/src/app/globals.css` (byte-identical across the web UIs), converts OKLCH to sRGB and writes `tbd_ui`'s `tokens.g.dart`; CI fails on drift. Inter (the invoice's own files) and Geist Mono bundled | A phone and a browser draw from one palette; a token change on the web reaches the phone with `mise run mobile:gen`. The follow-up shape is a shared `tokens.json` the CSS is also built from; that rewires four web builds and is not done. |
| Telemetry | `traceparent` on every request, one trace per request | The tap links to the spans Envoy, the protocol and the engine record. Exporting the app's own spans needs an OTLP door that does not exist; `telemetry.otlp_endpoint` is empty until it does. |
| Languages | ARB, en and hr from the first string | The web UIs are bilingual; a shell that is not would regress the standard. |
| Lints | very_good_analysis, strict, `--fatal-infos`; `dart format` | The bar clippy sets. Packages document every public member. |
| Configuration | `configs/mobile/{base,local,prod}.json`, merged by `mise run mobile:env` into the `--dart-define-from-file` the app is built with | Where every binary's config lives and the same layering rule; JSON because the app reads JSON. A public client has no secrets, so nothing there is sensitive. |
| Platforms | Android and iOS; packages carry no platform code | Web or desktop is `flutter create --platforms` on the app. |
| Chaos | Seams: `Env` points at any edge, `FakeAuthRepository` takes a machine token, `integration_test` runs against the edge an environment names | Scenarios that fault the app's backend while it runs are the next piece; the harness is what they will drive. |

## The flows

**Sign-in.** `AppAuthRepository.signIn()` opens the system browser on Hydra's
authorization endpoint (from the discovery document) with client `tbd-app`, redirect
`tbd://callback`, the registered scopes and `audience=tbd-api`, which Hydra needs to mint
a token the API accepts. Kratos shows the sign-in pages from `ui/auth`; Hydra skips
consent (first party); the browser returns on the scheme and the code is exchanged in
the app. Access, refresh and ID tokens go to the secure store; the session is
`SignedIn(principal)` with the ID token's `sub`, `email` and `name`, enough to greet.

**A request.** The bearer interceptor attaches the access token, refreshed first when it
is within a minute of expiry. On a 401 it refreshes once (single-flight) and retries; a
refresh the token endpoint refuses ends the session and the router shows sign-in. A
refresh the network loses keeps the session; that call fails on its own terms.

**The hello.** The home view model loads `GET /v1/me` over REST and `Ping` over gRPC in
parallel and shows each with its round-trip time; a failure is a sentence in the person's
language; "Ask again" repeats both. That is the ecosystem on one screen: proto, Rust,
Envoy, Dart, one token.

**Sign-out.** The store is cleared first, so the app is signed out whatever follows;
then Hydra's RP-initiated logout runs in the browser with the ID token hint and returns
on `tbd://callback/signed-out`, ending the browser session at `auth.<domain>` too.

## What is where

| Package | Holds | May depend on |
|---|---|---|
| `tbd_core` | `Env`, `Result` and `AppError`, `Tracer`, `Clock`; pure Dart | nothing of ours |
| `tbd_proto` | Dart generated from `/proto` | grpc, protobuf |
| `tbd_api` | `ApiClient` and its interceptors, the SSE reader, `GrpcEdge`, `ProtocolApi`, `Me` | core, proto |
| `tbd_auth` | `AuthRepository` (AppAuth), `TokenStore` (secure), `Session` | core |
| `tbd_ui` | the generated tokens, `TbdTheme`, `TbdColors`, the bundled faces | Flutter only |
| `tbd_testing` | `FakeAuthRepository`, `FakeProtocolApi`, `fakeJwt` | api, auth, core; tests only |
| `apps/tbd` | bootstrap, providers, router, the features | all of the above |

## Checks and CI

`mise run mobile:check` is format, analyze, the two drift checks (`Me` against
`openapi.json`, the theme against `globals.css`) and every package's tests; it is part
of `mise run ci` and the `mobile` job in CI ([ci.md](../ci.md)). `mise run mobile:e2e
<env>` runs the integration test against the edge that environment names, signed in
through the fake with a machine token from `mise run auth:token`; it needs a device or
emulator, so it is not in the gate.

## Not done, deliberately

Product screens, chaos scenarios that fault the app's backend, push, offline
persistence, OTLP export from the device, a store signing config, and social sign-in
buttons on the phone (Kratos already offers them on the login page the app opens; Sign
in with Apple becomes mandatory the day a Google button appears on iOS,
[auth/README.md](../auth/README.md)).
