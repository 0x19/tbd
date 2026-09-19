# ui/www

A personal site with a company footer, not an agency pitch: the reader came to
see what was built and to try a playground, not to be sold to. Keep the voice
first person and plain, and keep sales language out — no "let's talk about your
project", no capability deck, no funnel.

Static export, no server, no cookies and no third-party embeds — keep it that
way: the privacy notice on `/legal/` says so, and that sentence is a promise.

The one exception is a playground. `/playgrounds/break-it/` calls the gateway on
the same origin (`/v1/playground/*` and `/v1/ws`), which is why those paths are
routed on the public `www` virtual host rather than the API host: same origin,
no CORS, still no cookies. `/legal/` promises that a playground which processes
what a visitor types says so on its own page — so it does, in a "what this page
sends" section. A new playground owes the reader the same paragraph.

- **Content lives in `src/data/site.ts`.** A copy change edits that file. Do not
  inline facts into a page.
- **A `TODO` field renders as `—`** (`orDash`). Never invent a registration
  number, an address or a founding year to fill a gap.
- **`projects` are real repositories** and `playgrounds` are things that are
  actually open (or honestly marked as being built, with `href: null`). Never
  seed either with something that does not exist.
- The about page is a **person**, not a capability deck: the story, the path in
  chapters, and a colophon. A grid of service lines with stack tags is the thing
  it is not; if one creeps back, it is wrong.
- **No vanity metrics.** Repository counts, star counts and follower counts do
  not go on the page: they are about me, not about the reader, and they go stale
  in a static build. A number earns its place only when attached to a claim
  someone would act on. Project years are facts; counters are not.
- `clients` is **clients only** — who I work with, never my own projects, which
  belong in `projects` and the bio. It is the section that needs a hand on it:
  keep it true or take it down, because an out-of-date "now" is worse than none.
  A client's figures are quoted as theirs and labelled as published by them; do
  not restate someone else's marketing as a verified fact.
- The mark lives twice on purpose: inline in `src/components/logo.tsx` for the
  site (so it inherits `currentColor`) and as files under `public/brand/` for
  documents. Change the geometry in one and change it in the other, or the
  invoices stop matching the site.
- The design tokens in `src/app/globals.css` are the same file `ui/auth` and
  `ui/chaos` ship. If it changes there, change it here; do not fork the palette.
  Site-only motion and texture live in `src/app/site.css` so that file stays
  identical to the other projects', and everything in it degrades to nothing
  under `prefers-reduced-motion`.
- The pipeline on the home page (`src/components/pipeline.tsx`, data in
  `pipeline` and `rails`) is a drawing of a real system, not an illustration:
  every stage and every value is something the platform actually does. It is CSS
  and hairlines rather than a diagram library — a static page should not ship a
  canvas to draw four boxes.
- The layout language is `src/components/kit.tsx`: `Frame` for the measure,
  `SectionHead` for the numbered heads, `Eyebrow`, `Reveal`, `Marquee`,
  `IndexRow`, `Tag`. Build a new page from those rather than new one-off spacing.
- `NEXT_PUBLIC_SITE_URL` decides the canonical URL, the sitemap base and whether
  robots may index the build (only `inorbit.hr` may). It is a build argument, not
  a runtime variable — a static export has no runtime.
- `/legal/` carries the imprint and privacy; `/terms/` covers the site and the
  playgrounds. A playground that processes what a visitor types says so on its
  own page.
- Same bar as the other projects: `mise run ui:www:check` (prettier, eslint,
  tsc) is in `mise run ci`.
- The image is `devops/docker/Dockerfile.www` (Caddy, `devops/docker/www.Caddyfile`);
  the deployment is `devops/k8s/www/`; Envoy's public `www.*` virtual host and the
  public edge's apex block route to it.
