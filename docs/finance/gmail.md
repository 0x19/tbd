# Gmail: the Google clients behind a linked mailbox

A mailbox is linked through Google OAuth. Google's side of that is a *project* with a
consent screen, a publishing status and one or more OAuth clients; the finance service
holds a client id and secret and sends the browser to Google with them. Which project a
link goes through decides whether the link lasts.

## Why a link expires after a week

Google issues a refresh token that stops working after **seven days** to any project
whose consent screen is *External* and in **Testing** status, unless the only scopes
asked for are name, email and profile ([Google: refresh token
expiration](https://developers.google.com/identity/protocols/oauth2#expiration)). A
mailbox linked through such a project shows *expired* on `/connectors/` a week later and
has to be linked again, every week.

Two ways out, and they apply to different mailboxes:

| Consent screen | Who may consent | Verification | Refresh tokens |
|---|---|---|---|
| **Internal** (the project sits in a Google Workspace organisation) | accounts of that organisation only | none, any scopes | last |
| **External, In production**, unverified | anyone, after a one-time "unverified app" notice; Google caps it at 100 users | needed to remove the notice; the restricted read scope needs a paid security assessment | last |
| External, Testing | listed test users only | none | **die after 7 days** |

So the service knows **two clients**, and the purpose of a link chooses:

- **Read, or read and send** (`FINANCE_GOOGLE_CLIENT_ID` / `_SECRET`): the project that
  asks for `gmail.readonly` (+ `gmail.send`, + `email`). For the company's own
  Workspace mailboxes (`@inorbit.hr`) make this project *Internal* and it never
  expires. A mailbox outside the organisation can only use it while it is in Testing,
  and expires weekly.
- **Send only** (`FINANCE_GOOGLE_SEND_CLIENT_ID` / `_SECRET`, optional): a second
  project, External and *In production*, asking for `gmail.send` and `email` alone.
  Any address may link through it (an @tenderly.co, a personal gmail.com); Google shows
  the unverified notice once; the link lasts. Without this client a send-only link
  falls back to the read client and inherits its expiry.

The credential of a linked mailbox records which client minted it (`client: read |
send`); a refresh goes back to that client, because Google refuses a refresh token at
any other. A row linked before there were two clients is the read client's.

## Setting the projects up

In the Google Cloud console, the project number is the first part of the client id
(`1039289418052-….apps.googleusercontent.com`). Google calls the consent-screen pages
*Google Auth Platform* (Branding, Audience, Clients, Data access).

**The read project** (the existing one):

1. *Audience*: if **Internal** is offered, the project sits in the inorbit.hr
   organisation; choose it and Save. Done: the next relink of an `@inorbit.hr` mailbox
   lasts. If only External is offered, the project is outside the organisation: create a
   new project *under* the organisation (IAM & Admin → Create project → the
   organisation as parent), set Branding, Audience → Internal, Data access →
   `gmail.readonly`, `gmail.send`, `email`, Clients → Web application with the redirect
   URI below, and hand the new id and secret to `finance:connector-secrets`.
2. Redirect URI on the Web client: `https://finance.<domain>/connectors/callback/`
   (the service's `[connectors] redirect_url`).

**The send project** (new):

1. Create a project (any parent). Branding as the read project.
2. *Data access*: add `https://www.googleapis.com/auth/gmail.send` and `email`, nothing
   else. This is what keeps it out of the restricted-scope review.
3. *Audience*: External, then **Publish** (status *In production*). Google warns that
   verification is needed for the sensitive scope; it is needed only to remove the
   notice, not to link.
4. *Clients*: Web application, the same redirect URI. Copy the id and secret.

**The service:**

```sh
mise run finance:connector-secrets <read_id> <read_secret> <send_id> <send_secret>
kubectl -n tbd rollout restart deployment/finance
```

The task keeps the sealing key when the Secret exists, so every linked mailbox stays
linked; only the clients change. For compose and Ansible the same four values are
`FINANCE_GOOGLE_CLIENT_ID`, `FINANCE_GOOGLE_CLIENT_SECRET`,
`FINANCE_GOOGLE_SEND_CLIENT_ID`, `FINANCE_GOOGLE_SEND_CLIENT_SECRET`
(`compose.yaml`, `devops/ansible/.../compose.yaml.j2`, `.env.example`).

## Afterwards

Link the send-only mailbox again from `/connectors/` (*Link again* keeps its purpose);
Google shows "Google hasn't verified this app" once: *Advanced* → *Go to … (unsafe)*,
tick the send checkbox, Continue. Read links of `@inorbit.hr` mailboxes go through the
Internal project on their next relink. A personal gmail.com mailbox that *reads* stays
on the Testing project and keeps its weekly relink until the read project is verified;
that is the one case these two clients do not solve.
