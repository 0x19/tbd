#!/bin/sh
# Sets a person's role: sh role.sh EMAIL ROLE. Runs inside the cluster (a one-off
# curl pod started by `mise run auth:role`), against the Kratos admin API.
set -e
email="$1"; role="$2"
admin="${KRATOS_ADMIN:-http://kratos-admin.auth.svc:4434}"
# The identity's own id is the first "id" in the document (addresses have ids too).
id=$(curl -fsS "$admin/admin/identities?credentials_identifier=$email" | grep -o '"id":"[0-9a-f-]*"' | head -1 | cut -d'"' -f4)
if [ -z "$id" ]; then echo "no identity with e-mail $email" >&2; exit 1; fi
curl -fsS -X PATCH -H 'content-type: application/json' "$admin/admin/identities/$id" \
  -d "[{\"op\":\"add\",\"path\":\"/metadata_admin\",\"value\":{\"role\":\"$role\"}}]" >/dev/null
echo "$email ($id): role $role. Takes effect at the next sign-in; existing tokens keep their claims until they expire."
