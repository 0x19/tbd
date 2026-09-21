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
```

DNS: `A <domain> -> the public IP` and `CNAME www.<domain> -> <domain>`. Behind
Cloudflare's proxy set the zone's SSL/TLS mode to *Full (strict)* once Caddy holds
the certificate (`mise run edge:logs`): in *Flexible* mode Cloudflare reaches the
origin over plain HTTP, Caddy redirects to HTTPS, and the browser reports too many
redirects.
