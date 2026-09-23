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
  a choice made with the header's EN/HR toggle (the only thing stored, `inorbit.lang`
  in `localStorage`); the country Cloudflare saw the request from, which the site's
  own `/whereami` answers with (`devops/docker/www.Caddyfile` echoes `CF-IPCountry`;
  Croatia, Bosnia, Serbia and Montenegro read Croatian); the browser's language
  (`hr`, `bs`, `sr`); English. The static export is English, so a page is
  `app/<x>/page.tsx` for the metadata and `src/components/pages/<x>.tsx` for the words:
  a new page or a new visible string goes into a dictionary in both languages, and
  a new fact into both data files. The playground pages and their tools are still
  English-only; that is the next pass, not a rule.
- **The notice bar is temporary and truthful.** `src/components/site-notice.tsx` draws
  `notice.version` from the data file and the words from `common.notice.*` above the
  header until it is dismissed; the dismissal is one `localStorage` key carrying
  `notice.version` (functional storage, no consent needed), and the bar says so in one
  sentence, naming the language choice as the other key. The site sets no cookies, so
  there is no cookie consent dialog and none should be added: a dialog that says "we use
  cookies" would be false and `/legal/` says the opposite. Empty `notice.text`
  removes the bar; a new `version` shows it again to everyone.
- **`/about/` is the person and the record, and the PDF is the same record.** The page
  opens with who, the title and `focus`, the availability line and three buttons (the
  PDF, the full CV behind the sign-in, the address), then the story, Selected work,
  every position with `experience[].highlights`, the 2007–2014 years job by job
  (`earlier`, so the timeline's compressed "Earlier" entry is left out of the
  positions), the open-source work, languages and education. `/cv/` only sends the
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
- The pipeline on the home page (`src/components/pipeline.tsx`, data in
  `pipeline` and `rails`) is a drawing of a real system, not an illustration:
  every stage and every value is something the platform actually does. It is CSS
  and hairlines rather than a diagram library — a static page should not ship a
  canvas to draw four boxes.
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
- Same bar as the other projects: `mise run ui:www:check` (prettier, eslint,
  tsc) is in `mise run ci`.
- The image is `devops/docker/Dockerfile.www` (Caddy, `devops/docker/www.Caddyfile`);
  the deployment is `devops/k8s/www/`; Envoy's public `www.*` virtual host and the
  public edge's apex block route to it.
