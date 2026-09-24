# mobile

The Flutter workspace: `apps/tbd`, the shell every product starts from, and the
packages under `packages/`. Architecture and decisions: `docs/mobile/README.md`. Rules
for changing it: `CLAUDE.md` here.

## Toolchain

Flutter is pinned in the repo's `mise.toml`; `mise install` puts it on the path. Android
needs a JDK 17 and the SDK the Flutter tool asks for (`flutter doctor` says which); iOS
needs Xcode on a Mac. `mise run mobile:setup` resolves the pub workspace once for every
package (one `pubspec.lock`).

## Environments

Configuration lives in `configs/mobile/`: `base.json` (every key), `local.json` (the
local cluster through its public edge), `prod.json`. Nothing in them is secret, the app
is a public OAuth client.

```
mise run mobile:config local      # print the merged result
mise run mobile:env local         # write mobile/build/env.local.json (git-ignored)
```

The issuer in `auth.issuer` must be the one the identity stack was deployed with: every
token carries it, and a mismatch fails at the edge, not in the app (`docs/auth/README.md`).

## Running

```
mise run mobile:run local         # merges the config, then flutter run on the connected device
mise run mobile:build prod        # release APK; iOS without code signing where xcodebuild exists
```

A phone or an emulator cannot resolve `*.localhost`, so `local.json` names the cluster's
public edge. For an offline loop against k3d on the same machine, forward the edge's
ports into the Android emulator and point a copy of `local.json` at them:

```
adb reverse tcp:18080 tcp:18080
```

## Checks

```
mise run mobile:check             # format, analyze, drift checks, every package's tests (part of mise run ci)
mise run mobile:gen               # protos -> Dart, globals.css -> tokens.g.dart, ARB -> AppLocalizations
mise run mobile:e2e local         # integration_test against the edge, signed in through the fake with MOBILE_TOKEN
```

`mobile:e2e` takes a machine token from `mise run auth:token` in `MOBILE_TOKEN`; the
fake sign-in hands it to the app so the test exercises the real edge without a browser.

## Release: TestFlight from CI

No Mac is needed to put a build on an iPhone. The `mobile ios` workflow
(`.github/workflows/mobile-ios.yml`) runs on a macOS runner: it builds the app for an
environment, signs it with the team's cloud-managed distribution certificate through an
App Store Connect API key, and uploads it to TestFlight. Start it from Actions ("Run
workflow", pick `prod` or `local`) or push a `mobile-v*` tag. The build number is the
run number, so every upload is new to TestFlight.

Once, on the Apple side (Apple Developer Program membership required; a free Apple ID
cannot use TestFlight):

1. developer.apple.com, Certificates, Identifiers & Profiles, Identifiers: register an
   App ID with the explicit bundle id `hr.inorbit.tbd`.
2. App Store Connect, Apps, "+": name `tbd`, platform iOS, that bundle id, an SKU of
   your choice.
3. App Store Connect, Users and Access, Integrations, App Store Connect API: generate a
   team key with the **Admin** role (cloud signing creates the certificate with it),
   download the `.p8` once, note the Key ID and the Issuer ID.
4. GitHub, Settings, Secrets and variables, Actions: `APP_STORE_CONNECT_KEY_ID`,
   `APP_STORE_CONNECT_ISSUER_ID`, `APP_STORE_CONNECT_KEY_P8` (the file as one line:
   `base64 -i AuthKey_XXXX.p8 | tr -d '\n'`), `APPLE_TEAM_ID` (Membership details).
5. App Store Connect, the app, TestFlight: add yourself to an internal group. Install
   the TestFlight app on the phone; each processed build shows up there.

The same script runs on a Mac with Xcode: `mise run mobile:testflight prod` with the
four variables in the environment. Nothing about signing is stored in the tree; the
`.p8` lives in the secret and, at build time, under `~/.private_keys` on the runner.

## Layout

| Path | What |
|---|---|
| `apps/tbd/` | the app: bootstrap, theme and languages wired, features under `lib/features/` |
| `packages/tbd_core/` | `Env`, `Result` and `AppError`, `Tracer`, `Clock`; pure Dart |
| `packages/tbd_auth/` | sign-in, tokens, session; behind one interface with a fake |
| `packages/tbd_api/` | the client for the edge: REST, gRPC, the `Me` type |
| `packages/tbd_proto/` | Dart generated from `/proto` by buf |
| `packages/tbd_ui/` | the web kit's tokens as a theme, the bundled faces |
| `packages/tbd_testing/` | fakes and helpers for tests |
| `tool/` | `env.dart` (config merge), `theme.dart` (tokens), `openapi_check.dart` |
