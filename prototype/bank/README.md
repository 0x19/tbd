# prototype/bank

Throwaway. Answers one question before anything real gets built: **can Inorbit's Erste
business *and* personal accounts be read, cheaply?**

Not held to the workspace conventions — no proto, no service, no clippy, no chaos adapter.
Python because iterating on a redirect flow is faster there and the stdlib has `sqlite3`.
What transfers to the Rust implementation is the captured JSON under `data/raw/`, not this
code. Delete the directory when `FINDINGS.md` is written.

## Setup

Uses system Python 3.13 with `cryptography`, `requests` and `PyJWT` — all already present,
no venv.

The keypair lives outside the repo at `~/.config/enablebanking/`:

```
private.key   PKCS#8, 4096-bit, mode 600 — never leaves the box, never enters git
public.crt    self-signed, uploaded to the Enable Banking control panel
```

Set the application id from the control panel:

```bash
export EB_APPLICATION_ID=<the-uuid>
```

Override `EB_PRIVATE_KEY`, `EB_BASE_URL` or `EB_COUNTRY` if needed.

## Steps

Built in order, with a hard stop after step 2.

| Step | Command | Answers |
|---|---|---|
| 1 | `python3 -m eb.token` | RS256 signing works. No network. |
| 2 | `python3 -m eb.aspsps` | **THE GATE** — does Erste expose `business` in `psu_types`? |
| 3 | *not built yet* | consent flow, business + personal |
| 4 | *not built yet* | pull accounts, balances, transactions |
| 5 | *not built yet* | load into sqlite |
| 6 | *not built yet* | the queries |

Steps 3–6 are deliberately unwritten. If step 2 says Erste has no business access, they'd be
built against a dead end — the fallback is Salt Edge, whose coverage page names
*Erste&Steiermarkische Bank d.d. Croatia* with AIS, balances, holder info and transactions.

`eb/aspsps.py` exits 0 when business access exists, 1 when it doesn't, 2 when the call failed.

## What the prototype has to find out

Beyond "does it return 200":

- Does **business** work, does **personal** work, can one app hold both consents?
- **How far back does history go?** Decides whether the real system backfills or starts today.
- Do the counterparty and remittance fields **carry the "poziv na broj"**? The deterministic
  invoice-matching design depends on it. If Erste strips it, matching degrades to amount+date
  heuristics and that part of the plan needs rewriting.
- Is `entry_reference` present and stable across two pulls of the same window? Decides whether
  the dedup key needs its content-hash fallback.
- The **real** rate limit, and what the 429 body looks like.
- Does `required_psu_headers` demand `Psu-Ip-Address`? If it does, data can only be fetched
  while the user is online — which would break unattended scheduled syncs and is a much bigger
  deal than the 4/day cap.
- **What it costs.** Restricted production should stay free with two linked accounts.

Findings go in `FINDINGS.md`, with evidence. That note decides whether the rest of
`docs/plans/finance-service.md` gets built as written, amended, or pointed at another provider.
