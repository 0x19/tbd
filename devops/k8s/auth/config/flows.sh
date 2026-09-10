#!/bin/sh
# Renders /config/flows.yml: e-mail based flows (recovery, verification, the one-time
# code method) are on only when an SMTP relay is configured (COURIER_SMTP_CONNECTION_URI
# from auth-secrets, `mise run auth:smtp`). Without one they stay off and Kratos never
# tries to send mail.
set -eu
out=/config/flows.yml
if [ -n "${COURIER_SMTP_CONNECTION_URI:-}" ]; then
  cat > "$out" <<YML
courier:
  smtp:
    connection_uri: $COURIER_SMTP_CONNECTION_URI
    from_address: ${COURIER_SMTP_FROM_ADDRESS:-no-reply@__BASE_DOMAIN__}
    from_name: ${COURIER_SMTP_FROM_NAME:-tbd}
selfservice:
  methods:
    code:
      enabled: true
      passwordless_enabled: false
  flows:
    recovery:
      enabled: true
      use: code
    verification:
      enabled: true
      use: code
    registration:
      after:
        password:
          hooks:
            - hook: session
            - hook: show_verification_ui
        passkey:
          hooks:
            - hook: session
            - hook: show_verification_ui
YML
  echo "flows: recovery and verification on (SMTP relay configured)"
else
  printf 'selfservice:\n  flows:\n    recovery:\n      enabled: false\n    verification:\n      enabled: false\n' > "$out"
  echo "flows: recovery and verification off (no SMTP relay)"
fi
