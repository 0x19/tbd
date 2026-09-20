#!/usr/bin/env sh
# Build the iOS app for an environment, sign it with the team's cloud-managed
# certificate through an App Store Connect API key, and upload it to
# TestFlight. The same script runs in CI (.github/workflows/mobile-ios.yml)
# and on any Mac with Xcode; nothing here needs a person at the keyboard.
#
#   APP_STORE_CONNECT_KEY_ID      the key's id (App Store Connect > Users and Access > Integrations)
#   APP_STORE_CONNECT_ISSUER_ID   the issuer id shown on that page
#   APP_STORE_CONNECT_KEY_P8      the .p8 file's contents, base64 (or APP_STORE_CONNECT_KEY_PATH, a path)
#   APPLE_TEAM_ID                 the team id (developer.apple.com > Membership)
#   MOBILE_ENV                    which configs/mobile/<env>.json (default prod)
#   BUILD_NUMBER                  CFBundleVersion; unique per upload (CI passes the run number)
#
# Signing is Xcode's automatic signing with -allowProvisioningUpdates: with the
# API key (role Admin) Xcode registers the App ID, creates a cloud-managed
# Apple Distribution certificate and the App Store profile, and keeps them.
# Nothing is stored in a keychain and nothing is committed.
set -eu

here=$(cd "$(dirname "$0")" && pwd)
mobile=$(cd "$here/.." && pwd)
app="$mobile/apps/tbd"
env="${MOBILE_ENV:-prod}"
number="${BUILD_NUMBER:-1}"

need() { eval "v=\${$1:-}"; [ -n "$v" ] || { echo "testflight: $1 is not set" >&2; exit 2; }; }
need APP_STORE_CONNECT_KEY_ID
need APP_STORE_CONNECT_ISSUER_ID
need APPLE_TEAM_ID
command -v xcodebuild >/dev/null || { echo "testflight: xcodebuild not found; this runs on macOS with Xcode" >&2; exit 2; }

# The key on disk, where both xcodebuild and altool look for it.
keydir="$HOME/.private_keys"
key="$keydir/AuthKey_${APP_STORE_CONNECT_KEY_ID}.p8"
mkdir -p "$keydir"
if [ -n "${APP_STORE_CONNECT_KEY_PATH:-}" ]; then
  cp "$APP_STORE_CONNECT_KEY_PATH" "$key"
elif [ -n "${APP_STORE_CONNECT_KEY_P8:-}" ]; then
  printf '%s' "$APP_STORE_CONNECT_KEY_P8" | base64 -d > "$key"
else
  echo "testflight: APP_STORE_CONNECT_KEY_P8 (base64) or APP_STORE_CONNECT_KEY_PATH is required" >&2
  exit 2
fi
chmod 600 "$key"

auth="-allowProvisioningUpdates -authenticationKeyPath $key -authenticationKeyID $APP_STORE_CONNECT_KEY_ID -authenticationKeyIssuerID $APP_STORE_CONNECT_ISSUER_ID"

echo "== configuration: $env, build $number"
(cd "$mobile" && dart run tool/env.dart "$env")

echo "== flutter build ios (release, no codesign: Xcode signs the archive)"
(cd "$app" && flutter build ios --release --no-codesign \
  --build-number="$number" \
  --dart-define-from-file="$mobile/build/env.$env.json")

echo "== xcodebuild archive"
archive="$app/build/ios/archive/Runner.xcarchive"
# shellcheck disable=SC2086
xcodebuild -workspace "$app/ios/Runner.xcworkspace" -scheme Runner -configuration Release \
  -destination 'generic/platform=iOS' \
  -archivePath "$archive" \
  DEVELOPMENT_TEAM="$APPLE_TEAM_ID" CODE_SIGN_STYLE=Automatic \
  $auth archive | tail -20

echo "== export and upload to App Store Connect (TestFlight)"
# shellcheck disable=SC2086
xcodebuild -exportArchive -archivePath "$archive" \
  -exportOptionsPlist "$app/ios/ExportOptions.plist" \
  -exportPath "$app/build/ios/ipa" \
  $auth | tail -20

echo "== uploaded build $number for $env; it appears in TestFlight after Apple's processing (minutes)"
