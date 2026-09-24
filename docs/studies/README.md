# Studies

What was measured, what surprised us, what we would do differently. A study follows an
RFC (`rfc:` names it) and reports on the thing the RFC decided to build. Same file
rules as `docs/rfcs/README.md`: `NNNN-slug.md`, never renumbered, front matter with
scalars only, the redaction syntax, the publish switch, and the living-document rule.

```
---
title: What a 16 GB card really does
status: running
date: 2026-09-30
public: false
summary: One sentence.
rfc: 0001-the-llm-service
headline: 72 tok/s
headline_note: the fast tier at batch one, median over ten runs
---
```

- `status`: `running` while numbers are still coming in, `measured` once the
  measurements are final, `published` once the learning section is written in the
  owner's own words.
- `headline` and `headline_note`: the one number the index shows, and what it is. A
  number earns its place only when attached to a claim; leave both out rather than
  invent one.

The body, in this order: The question, What we believed, What we built, What we
measured (tables, with the method next to them), What surprised us, What we would do
differently, Glossary (the terms that were new that week, in one line each).

## The series as planned

Subject to the measurements, and the order may change; nothing here is a commitment.
Each one is baseline, implementation, measurement, what failed, result, and each
follows the RFC it measures.

1. What a 16 GB card really does: both tiers, both candidate models (RFC 0001).
2. As fast as this card goes: admission, one engine for both tiers, continuous batching
   and speculative decoding, each step against the numbers of study 1 (RFC 0001).
3. A storm at the edge, and what eBPF costs: the shield under a storm raised inside the
   machine, and the hook's own price on legitimate traffic (RFC 0003).
4. What the platform remembers: recall that finds the right entry, recall that finds a
   wrong one, and what it costs per turn (RFC 0002).
5. Watching it live: the arena's own load on the platform, and the latency from an
   event to the pixel (RFC 0004).
6. How fast can a 120 billion parameter mixture-of-experts run from DDR4: the deep
   tier's hypothesis against the stack.
7. Replacing the model server: loading the weights ourselves, then the tokenizer, the
   KV cache, batching and sampling, one layer per study.
8. Fine-tuning against recall, measured.

The engine of our own (7) is a parallel track against the baselines, never the critical
path of the service: each layer has to say what the baseline does, what ours does, where
the bottleneck was and what it did after the fix, or it is not a study.

## Index

| # | Study | Status |
|---|---|---|
| 0001 | [What a 16 GB card really does](0001-what-a-16gb-card-really-does.md) | measured |
