# Findings

Evidence for whether the finance plan gets built as written. Updated as the prototype runs.

## Status: gate passed. Build it.

Erste Croatia business **and** personal accounts are readable through Enable Banking's free
restricted-production tier, under one application, with 180-day consents and no contract.
2,861 real transactions pulled. Four corrections to the plan are recorded below; none of them
threatens the design.

## Confirmed

### Erste Croatia exposes business AND personal accounts

`GET /aspsps?country=HR` returns 30 ASPSPs. Erste's record, verbatim:

```json
{
  "name": "Erste & Steiermärkische Bank",
  "country": "HR",
  "psu_types": ["business", "personal"],
  "auth_methods": [
    { "psu_type": "business", "approach": "REDIRECT", "hidden_method": false },
    { "psu_type": "personal", "approach": "REDIRECT", "hidden_method": false }
  ],
  "maximum_consent_validity": 15552000,
  "beta": true,
  "bic": "ESBCHR22"
}
```

`ESBCHR22` matches the SWIFT printed on Inorbit's own invoices, so this is the right bank.

- **Business access: yes.** The premise of the whole project holds.
- **Personal access: yes.** The personal/business projection in the plan is buildable.
- **`maximum_consent_validity` is 15552000s = exactly 180 days**, as the plan assumed.
- **REDIRECT** for both, so the callback endpoint is the right design.

### No `Psu-Ip-Address` requirement

Erste's record carries **no `required_psu_headers` field at all**. This was the finding most
likely to break the design: had Erste demanded the PSU's IP, data could only be fetched while
the user was actively online, which would rule out unattended scheduled syncing entirely —
a far larger constraint than the 4/day cap.

Treat as provisional until a real data pull confirms it. Absence in the ASPSP record is
strong evidence, not proof.

### Erste's connector is flagged `beta: true`

26 of the 30 Croatian ASPSPs carry `beta: true`, so this reads as market-level labelling
rather than a specific warning about Erste. It still means the connector may change shape
without notice, which is an argument for the fixture-replay tests in the plan rather than
trusting today's response shape permanently.

### Registration costs nothing and needs no contract

Application `fd18716b-2132-49c6-89b6-384b99c92fd4`, environment **PRODUCTION**, service
Account Information. No card, no contract, no sales conversation. The control panel states
it plainly:

> You can link own accounts and use the application with these accounts. The application
> will be activated after account linking is complete. Only linked accounts can be accessed.
> For general availability please request activation.

That is the restricted-production tier the plan assumed. Confirmed as reachable by self-serve.

### RS256 signing works against the live API

`python3 -m eb.token` produces a well-formed token (3 segments, `kid` = application id,
`iss` `enablebanking.com`, `aud` `api.enablebanking.com`, 1h expiry), and the live API
**accepts it**. Proof is the shape of the rejection:

```
GET https://api.enablebanking.com/aspsps?country=HR
403 {"code":403,"message":"Application is not active"}
```

A wrong key, wrong `kid` or malformed token gives 401. A 403 on activation state means the
request authenticated and was refused on authorization. So key format (PKCS#8 from
`openssl genpkey`), signing, TLS and the request path are all verified end to end.

Errors are structured JSON with a `code`/`message` pair — the client's error mapping has
something predictable to parse.

### Certificate registration accepted a 10-year certificate

Their documented example uses `-days 365`. A cert valid to 2036 was accepted without
complaint, so annual rotation is not forced.

## Correction to the plan

**The plan had the order wrong.** `docs/plans/finance-service.md` Part 0 treats
`GET /aspsps?country=HR` as a gate that runs *before* the consent flow, on the theory that
it costs nothing and would kill the Enable Banking route early if Erste lacked business
support.

That sequencing is impossible. An application in restricted production is **inactive until
own accounts are linked**, and an inactive application gets 403 on every endpoint including
the read-only ASPSP list. The consent flow is not something the gate protects — it *is* the
gate.

Practical consequence: the question "does Erste Croatia expose business accounts" is answered
by attempting to link one, not by reading a list beforehand. The fallback plan (Salt Edge) is
unchanged but is now reached later and after more effort.

## Open — answered by linking

- [x] Does Erste Croatia appear as a linkable ASPSP? **Yes.**
- [x] Does it offer **business**? **Yes.**
- [x] Does it offer **personal**? **Yes** — both authorized and pulled.
- [x] Does `required_psu_headers` demand `Psu-Ip-Address`? **No such field on Erste's
      record.** Unattended syncing looks viable. Provisional until a data pull confirms it.
- [x] `maximum_consent_validity`: **15552000s = 180 days**, matching the plan.
- [x] Can one application hold a business *and* a personal consent simultaneously? **Yes.**
      Sessions `4493625a…` (INORBIT d.o.o.) and `2124a178…` (Nevio Vesić) are both live under
      application `fd18716b…`, each valid to 2027-03-15. The personal/business projection in
      the plan is buildable.
- [x] Note: `psu_type` in the `/auth` request does **not** decide what comes back — the
      credentials used at the bank do. A request sent with `psu_type: "personal"` returned the
      company's accounts because business credentials were used to log in. Treat `psu_type` as
      a hint about which login page to show, not a filter.

## Answered by pulling data — 2,861 real transactions

### Invoice matching works, but not where the plan expected

The plan assumed *poziv na broj* would arrive in a structured reference field. **It does not
for the payments that matter.** Every Tenderly settlement carries `reference_number: "HR99"`
— model 99, meaning *no structured reference* — and puts the invoice number in the free-text
remittance instead:

```
2026-09-04   14,500.82   TENDERLY D.O.O.   HR99 | BROJ RACUNA 9-1-1-2026
2026-08-06   25,021.67   TENDERLY D.O.O.   HR99 | BROJ RACUNA 8-1-1-2026
2026-07-03   14,872.60   TENDERLY D.O.O.   HR99 | BROJ RACUNA 7-1-1-2026
2026-06-08   14,738.18   TENDERLY D.O.O.   HR99 | BROJ RACUNA 6-1-1-2026
2026-05-05   14,336.82   TENDERLY D.O.O.   HR99 | BROJ RACUNA 5-1-1-2026
2026-04-07   14,635.15   TENDERLY D.O.O.   HR99 | BROJ RACUNA 4-1-1-2026
2026-03-05    1,698.73   TENDERLY D.O.O.   HR99 | BROJ RACUNA 3-1-1-2026
```

Seven consecutive months, identical format. So matching **is** deterministic — the amount
and the invoice number both agree — but the matcher reads
`/BROJ\s*RACUNA\s*(\d+)-(\d+)-(\d+)-(\d{4})/i` over `remittance_information`, it does not read
a field.

`reference_number` *is* populated with real structured references on other rows
(`HR68 1910-38846238650-26253`, `HR01 4049956796-202608-0`, `HR69 40002-38846238650-150`) —
mostly outgoing tax and utility payments with the OIB embedded. So both paths are real and
the matcher needs both, tried in order: structured reference first, remittance regex second.

**`remittance_information` is an array of strings**, not a string — `["HR99", "BROJ RACUNA
9-1-1-2026"]`. Element 0 is the reference model.

### `entry_reference` is always present and unique — the content hash is not needed here

2,657 transactions, 2,657 distinct `entry_reference` values, zero nulls. Erste's references
look like `506G50710R513451D`.

The plan's dedup key builds a labelled content hash with an ordinal as a fallback for when
`entry_reference` is absent or unstable. **For Erste that fallback is dead code.** Keep it —
the plan is multi-bank and other ASPSPs are not this well behaved — but the primary key path
for Erste is `entry_reference` alone, and the ordinal machinery does not need to be on the
hot path.

Not yet verified: stability of `entry_reference` across two pulls on different days. Worth one
repeat pull before relying on it.

### History depth

| Profile | Rows | Range | Span |
|---|---|---|---|
| business | 255 | 2026-03-02 → 2026-09-15 | ~6 months |
| personal | 2,606 | 2024-12-30 → 2026-09-16 | ~21 months |

Requested 24 months in both cases. Personal returned ~21, so Erste's limit is around there,
below the API's documented 24. Business returned only 6, which is almost certainly the
account's own age rather than a limit — the first row is a partial Tenderly payment
consistent with the company starting to trade then.

Backfill is therefore worth doing: there is real history to import, and the plan's
`initial_history = "730d"` is about right but will be silently truncated by the ASPSP.

### Multi-currency accounts are separate account entries

One IBAN appears once per currency, each with its own `uid`:

- `HR9224020061100925189` (INORBIT d.o.o.) — **EUR, GBP, USD, HRK**
- `HR3924020063202676456` (personal) — EUR, HRK
- `HR9524020063103277123` (personal) — EUR, HRK

Only the EUR entries hold transactions; GBP, USD and the legacy HRK accounts are all empty.
This **validates the plan's `unique (owner, iban, currency)` index** — keying accounts on
IBAN alone would have collapsed four real accounts into one.

It also explains the "duplicate" rows in the control panel: those were currencies, not
repeated link attempts.

### Counterparty names need normalisation before they can be grouped

The same person appears as `VESIĆ NEVIO`, `Vesic Nevio` and `Nevio Vesić`; the same
institution as `DRŽAVNI PRORAČUN REPUBLIKE HRVATSKE` and `DRZAVNI PRORACUN REPUBLIKE
HRVATSKE`. Anthropic bills as both `ANTHROPIC* CLAUDE SUB` and `CLAUDE.AI SUBSCRIPTION`.

Case, diacritics and word order all vary for one real counterparty. The plan's normalisation
(uppercase, collapse whitespace, strip to `[A-Z0-9 ]`) is necessary but **not sufficient** —
it fixes case and diacritics, not word order or vendor aliasing. Categorisation rules need a
counterparty-alias table, and `parties` is the natural home for it.

### Field shapes

- Amounts are **strings**: `"6.22"`. Sign is carried separately in
  `credit_debit_indicator` (`DBIT`/`CRDT`), not in the number.
- `status` is `"BOOK"`, not `"booked"`.
- `transaction_id` is always null; `entry_reference` is the identity.
- `merchant_category_code` is always null — no MCC-based categorisation available.
- `creditor.name` carries the merchant for card payments (`PLUS.EXCALIDRAW.COM`), which is
  what makes supplier-invoice matching feasible.
- `balance_after_transaction`, `bank_transaction_code`, `exchange_rate` all null.

### Rate limiting was never hit

Over 60 consecutive API calls across both profiles, including 51 paged transaction requests
on one account, produced **no 429 at all**. The documented "4 per day when the PSU is not
online" did not apply during an active session shortly after consent.

This does **not** disprove the limit — it suggests the limit applies to unattended access,
which is exactly the case the scheduled syncer will be in. The plan's budget-in-a-database-row
design stands. But the real unattended limit is still unmeasured, and the plan should not
claim otherwise until a sync runs a day after consent.

### Pagination: continuation keys are not self-contained

Erste rejects a continuation key sent on its own:

```
422 ParameterValidationException
"dateFrom in request is not the same as in continuationKey.
 Continuation key is only valid for the same getAccountTransactions parameters"
```

Every page must repeat the original `date_from`/`date_to` alongside `continuation_key`. Pages
are 51 rows.

### `GET /sessions/{id}` and `POST /sessions` disagree on shape

`POST /sessions` returns `accounts` as full objects (uid, iban, currency, name).
`GET /sessions/{id}` returns `accounts` as bare uid **strings**. Same field name, different
type. Anything reading a session must handle both or use only the creation-time response.

There is no `GET /sessions` listing at all — **405 Method Not Allowed**. Sessions cannot be
enumerated or recovered; the `session_id` from the exchange must be persisted or the consent
has to be redone.

## Infrastructure built for this

Not throwaway — these are production endpoints on `proximity.is`:

| URL | Purpose | Gate |
|---|---|---|
| `https://finance.proximity.is/connect/callback` | registered redirect URL | open (oauth2, jwt_authn, rbac all disabled for this path) |
| `https://chaosadmin.proximity.is/privacy.html` | required by registration | open |
| `https://chaosadmin.proximity.is/terms.html` | required by registration | open |

Credentials live at `~/.config/enablebanking/` — `private.key` (PKCS#8, 4096-bit, mode 600)
and `public.crt`. Neither is in git.
