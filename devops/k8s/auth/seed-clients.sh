#!/bin/sh
# Registers the OAuth2 clients this repo relies on (create or update). Runs as the
# seed-clients Job in Kubernetes and as the seed-clients service in compose.
# Env: HYDRA_ADMIN, AUTH_PUBLIC_URL, BASE_DOMAIN, SITE_DOMAIN (optional: the company
# site's own domain, whose browser hosts get login callbacks as well), CLIENT_UI_SECRET,
# CLIENT_CHAOS_SECRET.
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
# The same browser hosts under the site's own domain (devops/edge/sites.d/), when the
# site has one: Envoy's redirect_uri is built from the request's authority, so each
# host name a person can sign in on must be registered. The issuer stays auth.<base>.
site_uris=""
site_logout_uris=""
if [ -n "$SITE_DOMAIN" ] && [ "$SITE_DOMAIN" != "$d" ]; then
  for h in grafana logs profiles metrics chaosadmin finance cv www; do
    site_uris="$site_uris \"https://$h.$SITE_DOMAIN/oauth2/callback\","
    site_logout_uris="$site_logout_uris \"https://$h.$SITE_DOMAIN/\","
  done
fi
# Where Envoy sends the browser after a sign-out (its post_logout_redirect_uri is
# that host's root); Hydra refuses a return that is not registered.
logout_uris="$site_logout_uris"
for h in grafana logs profiles metrics chaosadmin finance cv www; do
  logout_uris="$logout_uris \"https://$h.$d/\","
done
for h in grafana chaos finance cv www; do
  logout_uris="$logout_uris \"http://$h.localhost:18080/\","
done
logout_uris="${logout_uris%,}"
# Browser sessions on the UI hosts: Envoy's oauth2 filter is the client.
upsert tbd-ui "{
  \"client_id\": \"tbd-ui\", \"client_name\": \"tbd web UIs\",
  \"client_secret\": \"$CLIENT_UI_SECRET\",
  \"grant_types\": [\"authorization_code\", \"refresh_token\"],
  \"response_types\": [\"code\"],
  \"scope\": \"openid offline_access email profile\",
  \"audience\": [\"tbd-ui\"],
  \"token_endpoint_auth_method\": \"client_secret_post\",
  \"redirect_uris\": [$site_uris
    \"https://grafana.$d/oauth2/callback\", \"https://logs.$d/oauth2/callback\",
    \"https://profiles.$d/oauth2/callback\", \"https://metrics.$d/oauth2/callback\",
    \"https://chaosadmin.$d/oauth2/callback\", \"https://finance.$d/oauth2/callback\",
    \"https://cv.$d/oauth2/callback\", \"https://www.$d/oauth2/callback\",
    \"http://grafana.localhost:18080/oauth2/callback\", \"http://chaos.localhost:18080/oauth2/callback\",
    \"http://finance.localhost:18080/oauth2/callback\", \"http://cv.localhost:18080/oauth2/callback\",
    \"http://www.localhost:18080/oauth2/callback\"
  ],
  \"post_logout_redirect_uris\": [$logout_uris],
  \"skip_consent\": true, \"skip_logout_consent\": true,
  \"access_token_strategy\": \"jwt\"
}"
# Short-lived browser tokens: a revoked session (global sign-out, role change) is
# felt on every UI host within five minutes, when Envoy's refresh fails.
code=$(curl -sS -o /tmp/out -w '%{http_code}' -X PUT -H 'content-type: application/json' "$HYDRA_ADMIN/admin/clients/tbd-ui/lifespans" -d '{
  "authorization_code_grant_access_token_lifespan": "5m",
  "authorization_code_grant_id_token_lifespan": "5m",
  "authorization_code_grant_refresh_token_lifespan": "720h",
  "refresh_token_grant_access_token_lifespan": "5m",
  "refresh_token_grant_id_token_lifespan": "5m",
  "refresh_token_grant_refresh_token_lifespan": "720h"
}')
case "$code" in 200|201) echo "client tbd-ui: lifespans set ($code)";; *) echo "client tbd-ui lifespans: HTTP $code"; cat /tmp/out; exit 1;; esac
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
# The first-party app (/mobile): public client, PKCE, no consent screen; the
# post-logout URI is where the browser returns after RP-initiated logout; Hydra
# requires it to share scheme and host with a redirect URI, hence tbd://callback/....
upsert tbd-app "{
  \"client_id\": \"tbd-app\", \"client_name\": \"tbd app\",
  \"grant_types\": [\"authorization_code\", \"refresh_token\"],
  \"response_types\": [\"code\"],
  \"scope\": \"openid offline_access email profile tbd.api\",
  \"audience\": [\"tbd-api\"],
  \"token_endpoint_auth_method\": \"none\",
  \"redirect_uris\": [\"tbd://callback\", \"http://localhost:3001/callback\", \"http://127.0.0.1:3001/callback\"],
  \"post_logout_redirect_uris\": [\"tbd://callback/signed-out\"],
  \"skip_consent\": true, \"skip_logout_consent\": true,
  \"access_token_strategy\": \"jwt\"
}"
