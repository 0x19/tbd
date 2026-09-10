#!/bin/sh
# Renders /config/oidc.yml: the social sign-in providers whose credentials exist in
# auth-secrets (GOOGLE_CLIENT_ID/SECRET, GITHUB_CLIENT_ID/SECRET). None configured:
# the oidc method is off and the pages show password and passkey only.
set -eu
out=/config/oidc.yml
providers=""
if [ -n "${GOOGLE_CLIENT_ID:-}" ] && [ -n "${GOOGLE_CLIENT_SECRET:-}" ]; then
  providers="$providers
        - id: google
          provider: google
          label: Google
          client_id: $GOOGLE_CLIENT_ID
          client_secret: $GOOGLE_CLIENT_SECRET
          mapper_url: file:///etc/config/kratos/oidc.google.jsonnet
          scope: [email, profile]
          requested_claims:
            id_token:
              email: { essential: true }
              email_verified: { essential: true }"
fi
if [ -n "${GITHUB_CLIENT_ID:-}" ] && [ -n "${GITHUB_CLIENT_SECRET:-}" ]; then
  providers="$providers
        - id: github
          provider: github
          label: GitHub
          client_id: $GITHUB_CLIENT_ID
          client_secret: $GITHUB_CLIENT_SECRET
          mapper_url: file:///etc/config/kratos/oidc.github.jsonnet
          scope: [user:email]"
fi
if [ -z "$providers" ]; then
  printf 'selfservice:\n  methods:\n    oidc:\n      enabled: false\n' > "$out"
  echo "oidc: no provider credentials; social sign-in off"
else
  printf 'selfservice:\n  methods:\n    oidc:\n      enabled: true\n      config:\n        providers:%s\n' "$providers" > "$out"
  echo "oidc: enabled for:$(echo "$providers" | grep -E '^ *- id: ' | awk '{print $NF}' | tr '\n' ' ')"
fi
