# devops/edge/sites.d

The company site under a domain of its own, beside the base domain. One file per
domain, `<domain>.caddy`, git-ignored like `.env`; the Caddyfile imports `*.caddy`
here and a glob that matches nothing imports nothing.

```
www.example.hr {
	redir https://example.hr{uri} permanent
}
example.hr {
	import site
}
# Optional: the API on the site's domain as well. Envoy matches the API on any name.
api.example.hr {
	import api
}
# Optional: the browser hosts on the site's domain (finance, cv, grafana, ...). Each
# needs its login callback registered, which `seed-clients.sh` does from SITE_DOMAIN
# (`mise run auth:deploy`, or re-run the seed-clients Job); the sign-in itself stays on
# auth.<base>.
finance.example.hr {
	import gated
}
cv.example.hr {
	import gated
}
```

The domain the site is canonical on goes into `.env` as `SITE_DOMAIN=<domain>`: the base
domain's apex and www then redirect to it, and `mise run local:build` bakes it into the
site image as the canonical URL and the sitemap's base.

DNS: `A <domain> -> the public IP`, `CNAME www.<domain> -> <domain>`, and `A api.<domain>`
if the API block is there. Create each record before its block: every failed certificate
attempt counts toward Let's Encrypt's five failed authorizations per hostname per hour. Behind
Cloudflare's proxy set the zone's SSL/TLS mode to *Full (strict)* once Caddy holds
the certificate (`mise run edge:logs`): in *Flexible* mode Cloudflare reaches the
origin over plain HTTP, Caddy redirects to HTTPS, and the browser reports too many
redirects.
