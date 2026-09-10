#!/bin/sh
# Registers the OAuth2 clients this repo relies on (create or update). Runs as the
# seed-clients Job in Kubernetes and as the seed-clients service in compose.
# Env: HYDRA_ADMIN, AUTH_PUBLIC_URL, BASE_DOMAIN, CLIENT_UI_SECRET, CLIENT_CHAOS_SECRET.
set -e
until curl -fsS "$HYDRA_ADMIN/health/ready" >/dev/null; do echo "waiting for hydra"; sleep 2; done
upsert() { # $1 client id, $2 json body
  code=$(curl -sS -o /tmp/out -w '%{http_code}' -X PUT -H 'content-type: application/json' "$HYDRA_ADMIN/admin/clients/$1" -d "$2")
  if [ "$code" = 404 ]; then
    code=$(curl -sS -o /tmp/out -w '%{http_code}' -X POST -H 'content-type: application/json' "$HYDRA_ADMIN/admin/clients" -d "$2")
  fi
  case "$code" in 200|201) echo "client $1: ok ($code)";; *) echo "client $1: HTTP $code"; cat /tmp/out; exit 1;; esac
}
d="$BASE_DOMAIN"
# Browser sessions on the UI hosts: Envoy's oauth2 filter is the client.
upsert tbd-ui "{
  \"client_id\": \"tbd-ui\", \"client_name\": \"tbd web UIs\",
  \"client_secret\": \"$CLIENT_UI_SECRET\",
  \"grant_types\": [\"authorization_code\", \"refresh_token\"],
  \"response_types\": [\"code\"],
  \"scope\": \"openid offline_access email profile\",
  \"audience\": [\"tbd-ui\"],
  \"token_endpoint_auth_method\": \"client_secret_post\",
  \"redirect_uris\": [
    \"https://grafana.$d/oauth2/callback\", \"https://logs.$d/oauth2/callback\",
    \"https://profiles.$d/oauth2/callback\", \"https://metrics.$d/oauth2/callback\",
    \"https://chaosadmin.$d/oauth2/callback\",
    \"http://grafana.localhost:18080/oauth2/callback\", \"http://chaos.localhost:18080/oauth2/callback\"
  ],
  \"skip_consent\": true, \"skip_logout_consent\": true,
  \"access_token_strategy\": \"jwt\"
}"
# Machine access to the API: chaos validate/load, CI, scripts.
upsert tbd-chaos "{
  \"client_id\": \"tbd-chaos\", \"client_name\": \"chaos tool\",
  \"client_secret\": \"$CLIENT_CHAOS_SECRET\",
  \"grant_types\": [\"client_credentials\"],
  \"response_types\": [],
  \"scope\": \"tbd.api\",
  \"audience\": [\"tbd-api\"],
  \"token_endpoint_auth_method\": \"client_secret_basic\",
  \"access_token_strategy\": \"jwt\"
}"
# The future first-party app: public client, PKCE, no consent screen.
upsert tbd-app "{
  \"client_id\": \"tbd-app\", \"client_name\": \"tbd app\",
  \"grant_types\": [\"authorization_code\", \"refresh_token\"],
  \"response_types\": [\"code\"],
  \"scope\": \"openid offline_access email profile tbd.api\",
  \"audience\": [\"tbd-api\"],
  \"token_endpoint_auth_method\": \"none\",
  \"redirect_uris\": [\"tbd://callback\", \"http://localhost:3001/callback\", \"http://127.0.0.1:3001/callback\"],
  \"skip_consent\": true, \"skip_logout_consent\": true,
  \"access_token_strategy\": \"jwt\"
}"
