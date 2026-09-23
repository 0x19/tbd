# ui/cv

The gated site for the full CV: two pages wearing the public site's chrome. The header,
footer, kit (`src/components/{site-header,site-footer,kit,logo}.tsx`), motion sheet
(`src/app/site.css`) and bundled fonts (`src/fonts/`) are `ui/www`'s, copied on purpose
so this reads as the same site behind a sign-in; the public pages in the header and
footer are absolute links back to it (`src/data/site.ts`, a mirror of the public data
file's chrome facts: a copy change there is a copy change here), and the public header
carries a "Full CV" link the other way. `docs/cv/README.md` is the contract; the API
client and i18n follow `ui/finances`.

- The API is the protocol's REST surface for `tbd.cv.v1.CvService`
  (`proto/tbd/cv/v1/cv.proto`): same origin in a build (Envoy serves the UI at the
  root of `cv.*` and routes `/v1/` to the protocol; the browser sign-in on that host is
  the only way in), `http://127.0.0.1:18080` under `next dev` with a token in
  `NEXT_PUBLIC_CV_TOKEN` (`NEXT_PUBLIC_CV_API` overrides the host).
  `src/lib/api/schema.ts` mirrors the proto as the transcoder renders it; change both in
  the same commit.
- `Providers` fetches `/v1/me` once; `useMe()` is who is signed in, with `email`, `name`
  and `role`. The header shows the Requests link only for `role === "admin"`, and
  `/admin/` says so to anyone else -- the service refuses them anyway; the page is not
  the gate.
- `/` is a visitor's standing (`GetAccess`), the note and the Request button
  (`RequestAccess`; a token without an e-mail cannot ask, and the page says why), and,
  once approved, the Download button: `DownloadCv` answers base64, the page turns it into
  a Blob and clicks an anchor, as the finances UI does for invoices. The line under the
  status says whether the owner's mail went out (`notified_at`) or whether notices are
  off in this environment.
- `/admin/` is the owner's table (`ListRequests`, a status filter) with Approve, Refuse
  and Withdraw (`DecideRequest`); the service's state machine decides what is allowed,
  the page only hides what cannot apply.
- Two languages from the first string (`src/lib/i18n`, `messages/{nav,cv}.ts`); the
  toggle is the finances UI's.
- Static export, `trailingSlash: true`, no `basePath`; served by Caddy from
  `devops/docker/Dockerfile.cv-ui`. `mise run ui:cv:check` (prettier, eslint, tsc) is in
  `mise run ci`.
