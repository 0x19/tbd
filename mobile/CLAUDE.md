# mobile

The Flutter foundation every product's phone app starts from. One app (`apps/tbd`) and
the packages under it; a product is a new `apps/` entry or new feature packages, never
a fork of this. Read `docs/mobile/README.md` for the architecture and the decisions;
this file is what to keep true while changing it.

- **A bearer client, nothing else.** The app holds its own tokens and talks to
  `api.<domain>` only, with `Authorization: Bearer` on every call; Envoy verifies and
  forwards the claims, nothing behind it sees a token. The web UIs' browser session
  (Envoy's OAuth2 filter, the `tbd_id` cookie) does not exist here and must not be
  imitated. Sign-in is the SSO's own pages in the system browser (authorization code
  with PKCE against Hydra's discovery document, client `tbd-app`, audience `tbd-api`);
  never an embedded web view, never a password field of ours.
- **Tokens live in the platform's secure store** (`flutter_secure_storage`: Keychain,
  Keystore) and nowhere else: not in preferences, not in a log line, not in a crash
  report. `tbd_auth` is the only package that reads or writes them.
- **Configuration comes from `configs/mobile/`**, the way every binary reads
  `configs/<binary>/`: `base.json` holds every key, `local.json` / `prod.json` only what
  differs, `mise run mobile:env <env>` merges them into `mobile/build/env.<env>.json`
  (git-ignored) and the app is built with `--dart-define-from-file`. `apps/tbd/lib/app/env.dart`
  is the one file that names a define; everything else asks `Env` (`tbd_core`). No URL,
  client id or scope is a literal in code. A public client has no secrets, so nothing
  in those files is sensitive; `mise run mobile:config <env>` prints the effective
  result like `chaos config` does.
- **MVVM, feature-first** (docs.flutter.dev/app-architecture). A feature under
  `apps/tbd/lib/features/<name>/` is a view (widgets only) and a view model (a Riverpod
  `Notifier`: state, and calls into repositories). Repositories (`tbd_auth`, `tbd_api`)
  return `Result<T>` with a closed `AppError`; they never throw for an outcome a screen
  must explain, and a view never sees a raw exception message. Dependencies point one
  way: `apps` → `tbd_auth`/`tbd_api` → `tbd_core`; `tbd_ui` is widgets only; `tbd_proto`
  is generated only; `tbd_testing` is imported by tests only.
- **The theme is the web kit's.** `packages/tbd_ui/lib/src/tokens.g.dart` is generated
  by `tool/theme.dart` from `ui/finances/src/app/globals.css` (the sheet every web UI
  carries, byte for byte): OKLCH converted to sRGB, light and dark, the radius scale.
  `TbdTheme` builds Material from it; `TbdColors.of(context)` keeps the kit's names
  (`muted`, `chart`, `success`, the priorities). Do not hand-edit the tokens; change
  the CSS and run `mise run mobile:gen`, `mobile:check` fails on drift. Fonts are
  bundled (Inter, the same OTFs the invoice PDF is set in; Geist Mono under its OFL),
  never fetched at runtime.
- **Two languages from the first string.** Every visible string is a key in
  `apps/tbd/lib/l10n/app_{en,hr}.arb`; `flutter gen-l10n` (part of `mobile:gen`) writes
  `AppLocalizations`. A literal in a widget is a bug. Server error messages stay
  English, as on the web; the view maps an `AppError` to a localized sentence.
- **Contract from `/proto`.** `packages/tbd_proto/lib/gen/**` is `buf generate` with the
  remote Dart plugin (`mobile/buf.gen.yaml`); regenerate, never edit. The REST surface's
  hand-written JSON (`/v1/me`) gets a hand-written Dart type in `tbd_api` whose field
  list `tool/openapi_check.dart` holds against `docs/protocol/openapi.json`.
- **Generated files are committed** (`tokens.g.dart`, the l10n classes, `lib/gen/**`,
  `pubspec.lock`) so a checkout builds without the generators, and CI proves they are
  current.
- **Lints are the bar clippy sets:** `very_good_analysis`, strict casts and inference,
  `dart analyze --fatal-infos`. Packages document every public member; the app and
  `tool/` do not have to (their `analysis_options.yaml` says so). `dart format` at 80.
- **Stubs say so.** `Ping` answers `stub: true` and the screen prints "stub" next to it;
  a placeholder never looks like a measurement, here as everywhere in this tree.
- **Platform code stays in `apps/`.** Packages have no `android/` or `ios/`; a new
  platform is `flutter create --platforms` on the app.

Layout: `apps/tbd/lib/{main.dart, app/, features/, l10n/}`; `packages/tbd_core` (Env,
Result, AppError, Tracer, Clock; pure Dart), `tbd_auth` (AuthRepository with the AppAuth
implementation, TokenStore with the secure one, Session: the one owner of tokens and
the client's TokenSource), `tbd_api` (ApiClient and its interceptors, SSE, GrpcEdge,
ProtocolApi, Me), `tbd_proto`, `tbd_ui`, `tbd_testing` (FakeAuthRepository, every
failure mode a knob; `fakeJwt`); `tool/{env,theme,openapi_check}.dart`; `buf.gen.yaml`;
`melos.yaml` (scripts only, the mise tasks are the entry point).

The app: `app/providers.dart` is the dependency graph (two roots overridden at bootstrap
or in a test, `envProvider` and `sessionProvider`; everything else derived);
`app/router.dart` is go_router with the session as `refreshListenable` and one redirect
(unknown → splash, signed out → sign-in, signed in never sees sign-in);
`app/errors.dart` turns an `AppError` into the sentence a person reads. A feature is a
view (a `ConsumerWidget`) and a view model (a `Notifier` that projects the session or
a repository and holds the actions). The `tbd://` scheme is registered in
`android/app/build.gradle.kts` (`appAuthRedirectScheme`) and `ios/Runner/Info.plist`
(`CFBundleURLTypes`); the client `tbd-app` in `devops/k8s/auth/seed-clients.sh` lists
`tbd://callback` and `tbd://signed-out`.

Checks: `mise run mobile:check` (format, analyze, the two drift checks, every package's
tests) is part of `mise run ci`; `mobile:gen` regenerates; `mobile:run <env>` and
`mobile:build <env>` merge the config first; `mobile:e2e` runs `integration_test`
against the edge the environment names. `README.md` here has the device workflow.

Commits follow the tree's rule: by explicit path through a private index when the
checkout is shared, never `git add -A`, never a rewrite.
