# ui/www

A personal site with a company footer, not an agency pitch: the reader came to
see what was built and to try a playground, not to be sold to. Keep the voice
first person and plain, and keep sales language out — no "let's talk about your
project", no capability deck, no funnel.

Static export, no server, no analytics, no tracking cookie and no third-party
embeds — keep it that way: the privacy notice on `/legal/` says so, and that
sentence is a promise. The one cookie the site sets is the language a visitor
picked (`inorbit.lang`, two letters, on the parent domain so `cv.` reads the same),
and `/legal/` names it.

The one exception is a playground that calls the platform. Break it (four live
services and a shared budget of faults) did, on the same origin (`/v1/playground/*`
and `/v1/ws`), which is why those paths are routed on the public `www` virtual host
rather than the API host: same origin, no CORS, no cookie of its own. It is paused
(2026-09-24) while it is rebuilt: `paused: true` on its `playgrounds` entry lists it
under Play as a dimmed card that links nowhere, `/playgrounds/break-it/` says it is
paused and connects to nothing, the home page and the sitemap skip it, and its code
waits whole in `src/components/playgrounds/break-it/` for the next version. `/legal/` promises that a playground which processes
what a visitor types says so on its own page — so it does, in a "what this page
sends" section. A new playground owes the reader the same paragraph.

The other exception is the lab. `/lab/` is an index of labs: each lab is one subject
built in the open (`src/data/labs.ts`: its id, page, name and what in both languages,
its workbench, whether it has a live view, its ways in), and every RFC and study names
its lab in front matter (`lab: <id>`; the build refuses an unknown one). `/lab/` shows a
card per lab, the newest status-log lines of every document ("Latest": the generator
parses each `## Status log` into `log`), and every document with a tab per lab. A lab's
own page is a static folder, `app/lab/<id>/page.tsx`, rendering
`src/components/pages/lab-home.tsx` (what it is, its live view, its ways in, its
documents, its timeline); the model lab's workbench is `/lab/llm/workbench/`.
The live parts read the arena (`src/lib/arena.ts`, `useArena`: the socket `/v1/ws`
after a `/v1/me` refresh, then `/v1/arena/events`, then a poll) and draw it with
`src/components/workbench/live-panel.tsx` (hand-drawn SVG series, no chart library);
a figure the arena leaves out shows as a dash, never a zero. The workbench
(`src/components/workbench/workbench.tsx`) keeps its sessions in `localStorage` only,
sends a turn over SSE, the socket or MCP (`src/lib/llm.ts`, one event shape for all
three), lists and runs the platform's MCP tools as cards, and says in its side column
what it sends. A model's answer is untrusted text rendered by `src/lib/markdown.ts`
(`marked`, a runtime dependency since): raw HTML shows as text, an image is its
description and is never fetched, only `http(s)` links survive (new tab, `nofollow`),
and code blocks carry their language and a copy button, and a run button for Go and
Rust (`data-run`), which sends the block to the runner (`src/lib/runner.ts`, RFC 0010)
and shows the result as a card; keep it that way, or the
`/legal/` promise and the page's safety go with it; publishing the lab means `/legal/` gains that sentence. The lab is
admins-only until its first page is published: Envoy gates the prefix on the `www`
host (sign-in, then the `admin` role), and `/v1/me` on the same host tells the page
who is signed in with a 401 instead of a redirect, so the header and footer show the
lab entry only to an admin (`src/lib/me.ts`, `navVisible` in `src/data/site.ts`).
`/account/` is gated by sign-in alone and is where signing in starts: the header's
right end (`src/components/header-user.tsx`, the CV host's widget) is a "Sign in"
button that is a plain anchor to it (a document navigation, so the gateway can
redirect; a client-side hop would fetch and could not), and once signed in it is the
person's initials with a menu: who, the role, the lab for an admin, and the sign-out,
which ends the session at the identity stack too.
Sessions are per host (the gateway's cookies are), so the CV host is its own sign-in;
the identity stack remembers the person, so the second one is a silent bounce.
The site sets no cookie for this; the one an admin carries was set by Envoy's sign-in.
Publishing the lab is one deliberate commit that touches two places: `lab.public`
in `src/data/site.ts` and the `/lab/` route's gate in `devops/envoy/envoy.yaml`,
and it revisits the `/legal/` sentence if the lab's demo then processes what a
visitor types.

The music playgrounds (`/playgrounds/tuner/`, `fretboard/`, `spectrogram/`, `chords/`,
`metronome/`, `ear/`) are the other kind: whatever they hear or play stays in the page.
`src/lib/music/` is the arithmetic with no audio in it (`pitch.ts` detection and notes,
`theory.ts` intervals, chords, voicings and the neck, `pluck.ts` a synthesised string,
`play.ts` playing those through a context) and `src/components/playgrounds/` is what they
share (`use-mic.ts` opens the microphone on request and hands each frame to a callback,
`mic-panel.tsx` the status, level bar, device picker and start/stop, `fretboard.tsx` the
neck and the chord box). Each page's paragraph says nothing leaves the browser -- keep it
true; a page that uploaded audio would break the `/legal/` promise. The metronome's
practice log is `localStorage`, per browser, and its page says so.

- **The lab's pages are markdown in the repository**, `docs/rfcs/` and
  `docs/studies/` (their READMEs are the conventions), rendered at build time by
  `tool/lab-data.ts` into the gitignored `src/generated/lab/` (`pnpm gen`, run before
  dev, build, lint and typecheck, and as `mise run www:lab`; `next dev` does not watch
  `docs/`, so re-run it after an edit). `public: true` in a page's front matter is the
  publish switch; a private page is a draft, rendered only into the gitignored
  `public/lab-private/drafts.json`, which Envoy serves to admins alone (a gate that
  does not move when the lab is published) and Caddy marks `private, no-store`;
  `src/lib/lab-drafts.ts` reads it for an admin, the lab lists stamp drafts, and
  `/lab/draft/?doc=<kind>/<slug>` renders one. Nothing private goes into
  `src/generated/`, which is compiled into the scripts. Drafts are not
  redaction-checked and may not be linked from a public page. A public page is refused, with `file:line`, when it names an
  internal host, an address, a port, a path into the machine or the deployment, a
  secret or the shape of the gate, a cluster name, a word from
  `docs/lab/redaction.json`, or loads anything from elsewhere (`tool/lab-redaction.ts`
  is the list). A fact that belongs in the narrative is withheld in the open with
  `[REDACTED: reason]` or a ` ```redacted ` block, never paraphrased away; both
  render as a black bar with the reason on hover. A published RFC is a living
  document: a change that alters what it says updates it in the same commit. The prose
  is English in both languages; only the chrome (`messages/lab.ts`) translates. The
  www image copies the three docs folders in so the same generator runs in the build.
  A static export refuses a dynamic route with no pages, so an empty list yields one
  placeholder slug that answers with the 404 (`app/lab/rfc/[slug]/page.tsx`).
- **The Radar's published issues are rendered at build time** so search engines,
  link previews and crawlers read them: `tool/radar-data.ts` (in `pnpm gen`, after
  the lab) reads the published digests from the live radar API
  (`RADAR_SOURCE_URL`, default `https://inorbit.hr`; drafts are never read) into the
  gitignored `src/generated/radar/`, `/radar/` renders the latest from it first, and
  each week has its own page, `/radar/2026-w39/`, with its title and preview card,
  listed in the sitemap. Unreachable (an offline build) writes an empty list and the
  page fetches in the browser; the build never fails for it. The browser still asks
  the service after load, for anything newer and, for an admin, the drafts (that read
  goes through the lab's gate at Envoy). A newly published issue reaches the HTML
  with the next site build: `mise run radar:site`.
- **Three sections for what was made, one rule each.** `/lab/` is what is being built
  now, with numbers and a status stamp; `/open-source/` (the `projects` data) is what
  shipped, finished, with a date and a link; `/playgrounds/` ("Play") is what to try
  for fun. A lab subject that finishes graduates to `/open-source/` with one line pointing
  back. `/work/`, `/projects/`, `/lab/break-it/` and `/lab/demo/` only send the browser on,
  kept for old links; none is in the sitemap.
- **Content lives in `src/data/site.ts`.** A copy change edits that file, and its
  Croatian twin in `src/data/site.hr.ts`. Do not inline facts into a page.
- **A `TODO` field renders as `—`** (`orDash`). Never invent a registration
  number, an address or a founding year to fill a gap. The registered seat and the
  MBS come from the court register (read 2026-09-21, the same source as `docs/nda/`).
- **`company.availability` is the one sentence a reader acts on.** It sits under the
  hero and at the top of `/about/`; keep it true (from when, for what, in what form) or
  remove it. `company.now` is the about page's "Now". There is no employer field:
  the current position is the first `experience` entry, and a position that ended
  says so in its `body` with the month.
- **Two languages, English and Croatian, from `src/lib/i18n`.** The words a page
  says are dictionaries in `src/lib/i18n/messages/` (one file per page or shared piece,
  `en` and `hr` maps keyed `<namespace>.<slug>`, read with `useT()`); the facts stay in
  `src/data/site.ts` and their Croatian in `src/data/site.hr.ts`, keyed by what
  identifies an entry (a company, a project name, a playground path, a stage), merged
  by `useSite()`. A Croatian gap reads in English, never as a key on the page and never
  silently as a wrong fact. The first language is decided on the client, in this order:
  a choice made with the header's EN/HR toggle (the only thing stored: the cookie
  `inorbit.lang` on the registrable domain, so `www.` and `cv.` share it, plus a copy
  in `localStorage` for a dev server with no domain); the country Cloudflare saw the
  request from, which the site's own `/whereami` answers with
  (`devops/docker/www.Caddyfile` echoes `CF-IPCountry`; Croatia, Bosnia, Serbia and
  Montenegro read Croatian); the browser's language (`hr`, `bs`, `sr`); English.
  `ui/cv/src/lib/i18n/index.tsx` is the same file and `cv.Caddyfile` the same route, so
  the gated site decides the same way and follows the same choice. The static export is English, so a page is
  `app/<x>/page.tsx` for the metadata and `src/components/pages/<x>.tsx` for the words:
  a new page or a new visible string goes into a dictionary in both languages, and
  a new fact into both data files. The playground pages and their tools are still
  English-only; that is the next pass, not a rule.
- **The notice bar is temporary and truthful.** `src/components/site-notice.tsx` draws
  `notice.version` from the data file and the words from `common.notice.*` above the
  header until it is dismissed; the dismissal is one `localStorage` key carrying
  `notice.version` (functional storage, no consent needed), and the bar says so in one
  sentence, naming the language cookie as the other thing kept. Both are functional and
  first-party, so there is no cookie consent dialog and none should be added: a dialog
  that says "we use cookies" for tracking would be false and `/legal/` says the opposite. Empty `notice.text`
  removes the bar; a new `version` shows it again to everyone.
- **`/about/` is the person; the PDF is the record.** The page opens with who, the
  title and `focus`, the availability line and three buttons (the PDF, the full CV
  behind the sign-in, the address), then the story (`about`) with the facts beside it,
  the path in `chapters` (eras with a sentence and where, not positions), and a
  colophon. It does not list positions, highlights, selected work, the earlier years,
  the open-source work or education: those are the PDF and `/projects/`, and repeating
  them here was the complaint that shaped the page. `/cv/` only sends the
  browser to `/about/`, kept for old links and for the PDF beside it. `mise run www:cv`
  writes `crates/cv/assets/cv.json` from `src/data/site.ts` (`tool/cv-data.ts`) and
  Typst sets `crates/cv/assets/cv.typ` into `public/cv/nevio-vesic.pdf` with the
  invoice's Inter. Both files belong to the cv service as well, which renders the
  _full_ CV from them for approved readers (`docs/cv/README.md`); the JSON and the PDF
  are committed, `ui:www:check` fails on a stale JSON, so a data change is not done
  until both are re-rendered and committed with it. Nothing in either restates an
  employer's confidential figures. The page's "Request the full CV" button is
  `cv.fullUrl`, the gated site; the public page and PDF never carry a phone number, an
  address or references.
- **`projects` are real repositories** and `playgrounds` are things that are
  actually open (or honestly marked as being built, with `href: null`). Each
  playground carries a `category` (the tab it sits under), a one-line `summary` and
  two or three `specs` for its card, in both data files; the home section and
  `/playgrounds/` draw them through `src/components/playground-tabs.tsx`, which
  features the systems piece on `/playgrounds/` and keeps the tab in the hash. Never
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
  keep it true or take it down, because an out-of-date "now" is worse than none;
  an empty list hides the section and the numbering follows.
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
- **The home page is the front door**, `src/components/pages/home.tsx`: every section
  shows something no other page shows and links out instead of restating it. Who and
  the availability line, three facts about now (no ticker of tags: a keyword wall is
  the thing the site is not), the pipeline drawing with the principles as one line
  each, four playground tiles, the lab once `lab.public` is true, the email. The
  summary paragraph, the project rows and the contact hero belong to `/about/`,
  `/projects/` and `/contact/`; putting them back here is the repetition that was
  removed. Behind the hero and behind the footer is one drawing,
  `src/components/mesh-fabric.tsx`: a mesh of points of presence grown to the whole
  width, two hand-laid tiles alternating and joined at their seams, on a 72px grid
  drawn as a pattern from the spine, faint, with a chain of blocks running through it
  on its own row (a block every second column, pointing at the one before), packets,
  ripples from the hub and breathing halos (all stop under `prefers-reduced-motion`). `Opening` hangs it from
  the top of the home page, fading down into the facts strip (where the request comes
  in from); `Closing`, wrapped round the footer in the root layout, hangs it from the
  bottom of every page, fading up (where the reply goes out to), and on the home page
  also drops the footer's top rule and leaves a small step under the last strip. The
  spine below the hero: a hairline half a gutter left of the content
  (`left-3 sm:left-4` inside each Frame) runs from the facts strip to the email, every
  section is a `Box` on it with a knock-out port where its rule meets the line, and
  one packet (`Spine`, measured with a ResizeObserver) crawls the whole way. The page
  opens and closes with the same kind of box: the facts strip under the hero is where
  the request comes in and the email strip at the end is where the reply leaves, so
  both carry `StripEdge` (a light round the outline, drops down the dividers, mirrored
  on the last one) and the spine runs from the first strip's corner to a filled port
  on the last one's. `src/lib/mesh.ts` is the arithmetic the fabric uses. A new home section
  is a `Box`, so it gets its port and its stretch of the line for free.
- The pipeline on the home page (`src/components/pipeline.tsx`, data in
  `pipeline` and `rails`) is a drawing of a real system, not an illustration:
  every stage and every value is something the platform actually does. It is drawn
  on the spine: the four layers top to bottom with the request's path through each
  (a "packet in" dot above, a "query out" dot below, both dictionary strings), and
  the two rails as vertical hairlines crossing every layer before each ends on its
  own line of text. It is CSS and hairlines rather than a diagram library — a static
  page should not ship a canvas to draw four boxes.
- **Fonts are bundled**, not fetched: `src/fonts/` holds Inter (four weights, OFL) and
  Geist Mono (two, OFL), the same files the mobile app and the invoice PDF ship, loaded
  through `next/font/local` in the root layout. A build therefore needs no network (Google
  Fonts refused a Docker build once, and that was enough), and the page loads nothing from
  a third party, which `/legal/` promises.
- The layout language is `src/components/kit.tsx`: `Frame` for the measure,
  `SectionHead` for the numbered heads, `Eyebrow`, `Reveal`, `Marquee`,
  `IndexRow`, `Tag`. Build a new page from those rather than new one-off spacing.
- `NEXT_PUBLIC_SITE_URL` decides the canonical URL, the sitemap base and whether
  robots may index the build (only `inorbit.hr` may). It is a build argument, not
  a runtime variable — a static export has no runtime. `mise run local:build` takes it
  from `SITE_DOMAIN` in `devops/edge/.env`, never from the platform's `BASE_DOMAIN`:
  the edge redirects the base domain to the site, so a build naming the base as
  canonical pointed search engines at a redirect and asked them to index nothing.
- `src/components/structured-data.tsx` is the JSON-LD (`Organization`, `Person`,
  `WebSite`) in the root layout, built from `src/data/site.ts` like everything else;
  it states only what the page states, and never an employer.
- `/legal/` carries the imprint and privacy; `/terms/` covers the site and the
  playgrounds. A playground that processes what a visitor types says so on its
  own page.
- Same bar as the other projects: `mise run ui:www:check` (the lab render and its
  redaction check, prettier, eslint, tsc) is in `mise run ci`, as the `ui-www` job.
- The image is `devops/docker/Dockerfile.www` (Caddy, `devops/docker/www.Caddyfile`);
  the deployment is `devops/k8s/www/`; Envoy's public `www.*` virtual host and the
  public edge's apex block route to it.

<!-- BEGIN:nextjs-agent-rules -->

# This is NOT the Next.js you know

This version has breaking changes — APIs, conventions, and file structure may all differ from your training data. Read the relevant guide in `node_modules/next/dist/docs/` (resolved from this file's directory; in monorepos the `next` package may not be visible from the repo root) before writing any code. Heed deprecation notices.

This block is written and re-added by `next dev` — verify at `node_modules/next/dist/server/lib/generate-agent-files.js`. Removing it from a diff only re-creates the uncommitted change; committing it with your work keeps the tree clean.

<!-- END:nextjs-agent-rules -->
