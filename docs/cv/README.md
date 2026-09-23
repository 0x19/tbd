# The full CV, behind sign-in and approval

The public site (`inorbit.hr/cv/`) carries the CV without contact details, and a
public PDF rendered from the same data. The **full** CV -- phone, postal address,
references -- is handed out by the `cv` service on `cv.<domain>` to people who signed
in and whom the owner approved, as a PDF rendered for that one reader with their name
and the date on every page. Every download is recorded.

## The flow

1. A visitor opens `https://cv.<domain>/` and is sent to the sign-in (Envoy's OAuth2
   filter, the same Ory stack as every browser host; registration is open, Google and
   GitHub sign in with one click). Their token carries `email`, `name` and a `role`
   (`viewer` unless the owner set one).
2. Signed in, they see their standing (`GET /v1/cv/access`) and ask with a note
   (`POST /v1/cv/access`). The request is one row per person, keyed by the verified
   subject; asking again after a refusal or a revocation renews it.
3. The owner is told by **mail**, sent through the mailbox linked in the finance app
   (its `SendMail` RPC, over Envoy's internal listener) to `[notify] to`, with the
   note and a link to the admin page. The row records when that mail went out
   (`notified_at`); the mail is sent off the request, so a slow mailbox never holds
   the answer and never sends twice.
4. On `https://cv.<domain>/admin/` (the token's role must be `admin`:
   `mise run auth:role EMAIL admin`), the owner approves, refuses or revokes
   (`POST /v1/cv/requests/{id}/decide`). An approval mails the person, best effort:
   an environment whose finance `allow_to` list is set refuses strangers, and that is
   fine.
5. Approved, the person downloads (`GET /v1/cv/document`): the PDF is rendered at that
   moment by Typst with the reader's name, e-mail and the date in every page's footer,
   and a row in `cv.downloads` keeps the time, the user agent and the address the
   gateway saw.

## What is where

| Piece | Where |
|---|---|
| The service | `crates/cv` (`crates/cv/CLAUDE.md`), proto `proto/tbd/cv/v1/cv.proto`, REST through the protocol under `/v1/cv/` |
| The data | `ui/www/src/data/site.ts` is the source; `mise run www:cv` writes `crates/cv/assets/cv.json` and renders the public PDF; CI checks the JSON is current |
| The template | `crates/cv/assets/cv.typ`; `private` and `reader` inputs make it the full version |
| The private fields | Secret `cv-private`, key `private.json`, from `mise run cv:secrets <path>`; the file lives outside the repository (`~/Documents/cv/private.json`), shape below |
| The database | schema `cv` in the shared app database, migration `0028_cv.sql`, Secret `cv-db` (also from `cv:secrets`), `mise run db:migrate` |
| The mail | `[finance] url` (`CV_FINANCE_URL=http://envoy:50051`), `[notify] from/to/url` (`configs/cv/local.toml`), and the one-time `mise run cv:grant <owner subject>` |
| The pages | `ui/cv` (`/` and `/admin/`), host `cv.*` in `devops/envoy/envoy.yaml`, `cv.{$BASE_DOMAIN}` in the edge Caddyfile |

`private.json`:

```json
{
  "phone": "+385 ...",
  "address": "Street 1, 10000 City, Croatia",
  "references": [
    { "name": "Ann Example", "role": "CTO, Example Ltd", "contact": "ann@example.com" }
  ]
}
```

## Setting it up

1. DNS: `A cv.<domain>` beside the other hosts; the edge gets its certificate itself.
2. `mise run cv:secrets ~/Documents/cv/private.json` creates `cv-db` and `cv-private`.
3. `mise run db:migrate` applies `0028_cv.sql`.
4. Sign in to the finance app once as the owner (so the user row exists), read your
   subject from `GET /v1/me`, then `mise run cv:grant <subject>`: the service's own
   subject `svc:cv` may now see the party that owns the mailbox, and only read it.
5. `mise run auth:deploy` re-seeds the browser client with the `cv.` callbacks.
6. Roll `cv`, `cv-ui`, the protocol (the new routes) and the edge.

## Not done

Telling the owner any other way than mail; a chat with visitors, which this service
is the natural home for later; more than one document.
