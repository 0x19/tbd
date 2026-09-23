# ui/www — the site

A personal site rather than a pitch: what I am building, what I have published,
the prototypes anyone can try, and the company details behind the invoices.
Next.js on Tailwind, with the admin UI's light/dark/system theme switcher,
exported to static files and served by Caddy in the cluster behind Envoy and the
public edge. Nothing here runs on a server.

It is the same visual language as the other surfaces — the tokens in
`src/app/globals.css` and the Inter / Geist Mono pair are the ones `ui/chaos`
and `ui/auth` use — so the site, the sign-in pages and the admin UI look like
one hand made them.

## Run it

```sh
mise run ui:www:dev      # http://localhost:3003
mise run ui:www:check    # prettier, eslint, tsc — what CI runs
pnpm build               # static export into out/
```

## Edit the content, not the pages

Everything that is a fact lives in [`src/data/site.ts`](src/data/site.ts): who I
am, the four things I work with, the public projects, the clients, the
playgrounds, how I like to build, the email and the company details. The pages
read from it, so changing a sentence never means touching layout.

Two fields are still marked `TODO` because only the company can supply them —
the registered address and the court register entry (MBS). They render as `—`
until they are filled in, so an empty field never reads as a real one.

`experience` and `achievements` are the working record behind `/about/`. They
come from the CV, and where a date is genuinely open the `when` string says so
in words rather than guessing a month.

## The mark

The monogram: the tittle of the _i_ is the body, the _O_ is the orbit. One
circle, one rounded bar, one dot on a 32-unit grid — no gradients, no type
inside it, nothing that stops working in one colour at 16 pixels.

The site draws it inline from `src/components/logo.tsx` so it inherits
`currentColor` and inverts for dark mode by itself. Everything else that needs
it reads a file:

| File                                                   | For                                                  |
| ------------------------------------------------------ | ---------------------------------------------------- |
| `public/brand/mark.svg`                                | the mark, `currentColor` — anything that can inherit |
| `public/brand/mark-black.svg`, `mark-white.svg`        | fixed colours, for documents and PDFs                |
| `public/brand/lockup.svg` and its black and white pair | mark plus the name, horizontal                       |
| `src/app/icon.svg`                                     | the browser tab                                      |
| `src/app/apple-icon.png`                               | the home-screen icon, reversed out of black          |
| `src/app/opengraph-image.png`                          | the card a shared link unfurls into                  |

**For invoices and anything printed**: use `mark-black.svg` or
`lockup-black.svg`. They are plain vectors, so a PDF keeps them sharp at any
size. The lockup sets the name in Inter via `font-family`; a generator that
cannot embed Inter should use `mark-black.svg` and set the name in the
document's own face instead of relying on the SVG's text.

Clear space around the mark is the width of the _i_ stem on every side. Never
recolour it, put it on a busy ground, or rebuild the lockup by hand — place
`lockup.svg`.

## Adding a playground

Append an entry to `playgrounds` in `src/data/site.ts`:

```ts
{ name: "Solidity playground", what: "Paste a contract, watch it parse.", href: "/play/solidity/", tag: "EVM" }
```

`href: null` means it is still being built: the card appears with a "Building"
label instead of a dead link. An empty array renders the honest empty state, so
the page never lists something that is not there.

## Where it is published

`NEXT_PUBLIC_SITE_URL` (the `SITE_URL` build argument of
`devops/docker/Dockerfile.www`) is the canonical URL and the base of
`sitemap.xml`. **Only a build whose host ends in `inorbit.hr` asks robots to
index it**; every other build — the one on the development domain, for instance
— ships `Disallow: /`. `mise run local:build` passes `SITE_DOMAIN` from
`devops/edge/.env` (the site's own domain, not the platform's `BASE_DOMAIN`, which
redirects to it); unset, the image's default `https://inorbit.hr` applies.

| Where                         | How it gets there                                                                                                                                                    |
| ----------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `http://localhost:3003/`      | `mise run ui:www:dev`                                                                                                                                                |
| `http://www.localhost:18080/` | the local cluster, through Envoy's `www.*` virtual host                                                                                                              |
| `https://<base domain>/`      | the public edge (`devops/edge/Caddyfile`) rewrites the apex Host to `www.<base domain>` so one Envoy rule serves both, and `www.<base domain>` redirects to the apex |

## Pages

| Path                          | What it is                                                                                                                              |
| ----------------------------- | --------------------------------------------------------------------------------------------------------------------------------------- |
| `/`                           | Who this is, the playgrounds, what I have built, what I am doing now, how I like to build                                               |
| `/playgrounds/`               | The prototypes. Empty until the first is up                                                                                             |
| `/projects/`                  | Every public repository, with the year it was written                                                                                   |
| `/about/`                     | The longer version: how I got here, where the time goes, where else I am                                                                |
| `/cv/`                        | The record, role by role, and the PDF (`public/cv/`, `mise run www:cv`); the full version is behind `cv.<domain>` (`docs/cv/README.md`) |
| `/contact/`                   | The address, and nothing resembling a form                                                                                              |
| `/legal/`                     | Company details (imprint, OIB) and the privacy notice                                                                                   |
| `/terms/`                     | Terms of use for the site and the playgrounds                                                                                           |
| `/robots.txt`, `/sitemap.xml` | Generated at build time from `src/data/site.ts`                                                                                         |
