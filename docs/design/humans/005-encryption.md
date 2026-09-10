# 005 — Encryption

Status: open — proposed for approval

The ledger and everything backed up from it hold values only as ciphertext;
outside the plaintext locations named below, no copy of a `facts` or `inputs`
value can be read without a key, and keys are released by the id plane, per
principal, per human, per scope, against the grant it already owns.

---

## The decision

[003](003-consent-and-erasure.md) scopes reads by intersecting a fact's scope ids
with the caller's grant. A check in one service is bypassed by a bug, a leaked
dump or a backup restored elsewhere. Here the bytes themselves are useless
without a key, and the key comes from the plane that owns the grant. Honest
boundary: the `humans` service is the trusted decryptor, holds released keys for
30 seconds, keeps decrypted views in Redis, decides which wraps exist, and still
filters `origin` fields in code; what is cryptographic is that **outside those
named plaintext locations, nothing but the id plane's key release can turn ledger
bytes into values**.

## Construction

```
KMS root key (one; HSM-backed, the id plane's, id/001)
 └─ human key       one per human, wrapped by the root, held ONLY by the id plane's key service; destroyable
     └─ scope key   one asymmetric pair per (human, scope id, persona where persona-bound), HPKE (RFC 9180):
        │            the PUBLIC key is held by `humans` and wraps; the PRIVATE key is wrapped by the human key,
        │            held by the key service, and released to `humans` only for a valid (principal, human, scope)
        └─ DEK      one random 256-bit key per fact and per input; wrapped once per scope public key
```

- **Per ciphertext, a data encryption key.** `value` and `origin` share one
  ciphertext and one DEK; `origin_inputs` is a second ciphertext with its own; an
  input is one more. AES-256-GCM, a fresh DEK per ciphertext so the nonce bound of
  [NIST SP 800-38D](https://csrc.nist.gov/pubs/sp/800/38/d/final) never applies to
  a shared key. Associated data: `(human_id, fact_id, path, part)` for facts, the
  fact id reserved from the sequence before encrypting; `(human_id, content_hash,
  kind)` for inputs. Every ciphertext starts with a header carrying the envelope
  format version only; algorithm and HPKE suite are bound to that version.
- **One ciphertext, many wraps, and anyone can wrap.** Each DEK is wrapped once
  per scope id in `consent`, as rows in `fact_keys (human_id, fact_id, part,
  key_scope, key_version, wrapped_dek)` ([002](002-storage.md)), using that scope's
  public key: [HPKE, RFC 9180](https://www.rfc-editor.org/rfc/rfc9180) mode base,
  suite DHKEM(X25519, HKDF-SHA256), HKDF-SHA256, AES-256-GCM, the suite bound to
  `key_version`. Wrapping needs no
  grant, so the engine can write a fact readable by `tier2@P` without being able
  to read `tier2@P` itself; unwrapping needs the private key, released only on a
  grant. Widening consent later (a lens opt-in adds `tier2@P` to
  `readings.natal.*`) is one more wrap by the service acting for the person, who
  holds `self` and so can recover the DEK. No re-encryption of values, ever.
- **`origin.inputs` is its own ciphertext** with its own DEK, wrapped for `self`
  and `engine.base` only, so an organisation's key opens `origin.model` or
  `origin.provider_id` and nothing that references another fact or input.
- **Persona is in the key.** Organisation scopes are keyed per
  `(human, scope, persona)`: a `tier2@P` wrap opens only what P exposes. Persona
  membership is enforced by which wraps exist; the service that creates and
  deletes wraps when the person edits a persona is still trusted code.
- **Raw inputs** get their own DEK each, wrapped under an `inputs` scope key,
  readable by `self` and `engine.base` ([003](003-consent-and-erasure.md)). The
  human key is released to no principal; it exists to wrap scope private keys and
  to be destroyed.
- **Who authenticates.** No service verifies tokens (`CLAUDE.md`). `humans`
  calls the id plane's key service through Envoy under its own service identity
  and asserts `(principal_sub, human_id, scope)`; the key service checks the grant
  in its own permission graph and returns the scope private key, cached by
  `humans` for at most 30 seconds. That the key service trusts `humans` as the
  decryptor is part of the honest boundary above. The KMS holds one root key;
  scope and human keys are data keys in the id plane's store, so there is no
  per-human KMS key and no KMS quota in the path (AWS, for scale: 100,000 keys
  per region, [KMS quotas](https://docs.aws.amazon.com/kms/latest/developerguide/resource-limits.html)).
  A key service in the id plane, and its 24-hour backup retention, are asks of
  that plane recorded in the [README](README.md), as are two principals with no
  person behind them: the outbox consumer inside `humans`, which holds `analytics`
  for every human, and the key service itself for rotation.
- **Revocation** is the id plane refusing to release the key. No principal ever
  holds key material, only `humans`' cache, so reads stop within the 30-second TTL
  plus the id plane's own propagation, which the scenario measures. Scope keys are
  rotated on a schedule as hygiene, not as the revocation mechanism: `humans`
  ships the wrapped DEKs for a `(human, scope)` to the key service, which holds
  both versions and returns them re-wrapped, so no retiring private key is ever
  released for the purpose.
- **Erasure** destroys the human key at window end ([003](003-consent-and-erasure.md)).
  Every scope private key beneath it, every wrapped DEK, every value ciphertext in
  every backup becomes unreadable once the id plane's key-table backups (24 hours)
  age out. Plaintext residue (`embeddings`, the clear columns) lasts the ledger's
  own backup retention, 7 days ([002](002-storage.md)). Counted from the request,
  after the 7-day grace window in [003](003-consent-and-erasure.md), that is
  **14 days**, and that is the number the enrolment text states
  ([id/006](../id/006-legal-erasure.md), requirement 3).

## What stays in clear, and where plaintext exists

| Where | What | Why |
|---|---|---|
| `facts` columns | `path`, `source`, `confidence`, `counterparty_id`, `observed_at`, `recorded_at`, `expires_at`, `consent`, `stub` | indexes, `facts_current`, expiry and the erasure cascade run on them. A path alone can reveal an opt-in (`readings.natal.*`, `engine.face`), and `counterparty_id` with `relations.*` paths exposes the match graph; both stated in the enrolment text |
| `fact_keys.key_scope`, `input_keys.key_scope` | scope ids including `tier2@<persona>` | a dump yields which personas expose which facts; stated in the enrolment text |
| `fact_inputs`, `inputs.content_hash`, `inputs.kind` | the fact-to-input graph and the input identifiers | the cascade needs the graph. `content_hash` is a **keyed** hash (HMAC under a key held by `humans`, not the human key), so a party holding the bytes of a photo it received cannot confirm the human from a dump; this closes the content-correlation channel [id/000](../id/000-account-model.md) names |
| `humans`, `erasures`, `outbox` | enrolment and erasure times, publication state | operations |
| `embeddings` | vectors in clear, owned by `ledger`, cascaded with the human | pgvector cannot search ciphertext. Text embeddings are invertible to near-verbatim text ([Morris et al. 2023](https://arxiv.org/abs/2310.06816)), so this table is treated as journal-sensitive: readable by the engine only, in the Postgres residue for the backup retention |
| Redis | the decrypted `self` and `engine.base` views, and presence | the hot path. Persistence off (`save ""`, `appendonly no`, [Redis persistence](https://redis.io/docs/latest/operate/oss_and_stack/management/persistence/)) and diskless replication (`repl-diskless-sync yes`, the 7.0 default, [Redis replication](https://redis.io/docs/latest/operate/oss_and_stack/management/replication/)), so no snapshot reaches disk; the 003 scenario asserts the data directory stays empty |
| ClickHouse | `outcomes.*` values under `analytics`, decrypted by the outbox consumer, which runs inside `humans` | Q3 evaluation needs the values; deleted per [002](002-storage.md) |
| `humans` process memory | scope private keys for 30 seconds, values in flight | the trusted decryptor, and the only holder of key material outside the id plane |

## Cost, stated

- One resolution call plus one key call per scope on the read, on a key-cache
  miss, in-cluster through Envoy, no external KMS on the path. Assumed budget:
  about 1 ms per in-cluster call and a key-cache hit rate above 95% at a 30-second
  TTL, which keeps the Postgres-miss row of [002](002-storage.md) under its 10 ms
  target; that row is marked pending measurement until the scenario runs. This is
  also why `ledger` stays in-process ([README](README.md)): a second hop per miss
  would sit on top of these calls.
- Batch jobs under `engine.base` would make one key call per human per pass, a
  million calls for a million humans, so the key service offers a batch release
  for the engine principal (one call, many humans). After that, unwraps are local
  HPKE and AES operations; the throughput figure is an estimate to measure, not a
  number we have.
- Analytics: `outcomes.*` values reach ClickHouse under an `analytics` scope
  ([003](003-consent-and-erasure.md)), because Q3 needs them; `relations.*` reach
  it as event types with ids only.
- Search over values is impossible on the ledger and goes through the engine's
  derived index. That is the point.

## Not a boolean

A compromise of the KMS root key, or of the id plane's key service, opens
everything beneath them. The `humans` process holds keys and plaintext in memory.
Embeddings are invertible to near-verbatim text and are in clear. The construction
moves the trust to two places and names them; it does not remove it.

## Unrecoverable if wrong

- **The clear-column list.** Encrypting a column later re-encrypts every row.
- **The associated data, the envelope header and the HPKE suite per
  `key_version`.** Fixed into every ciphertext and every wrap.
- **The keyed `content_hash`.** Its key is held by `humans` forever; losing it
  orphans every input reference.
- **Which fields share a ciphertext.** `value` and `origin` (minus `inputs`) share
  one, so an organisation's key opens `assurance_level` and `evidence_ref` on a
  verified fact, filtered in code; splitting them later re-encrypts every row.
- **Key granularity.** `(human, scope, persona)` can be split further, never
  merged without re-wrapping every DEK.
- **`key_version` on every wrap** is the scope key version of that wrap; the
  envelope header carries only the format version. Swapping those semantics later
  cannot be done row by row.
- **Asymmetric scope keys.** Moving to symmetric keys later, or back, re-wraps
  every `fact_keys` row.
