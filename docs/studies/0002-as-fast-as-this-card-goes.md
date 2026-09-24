---
title: As fast as this card goes
status: measured
date: 2026-09-24
public: true
lab: llm
summary: The fast tier moved from one engine to another on the same card and the same weights, measured before and after; what parallel slots, a shared cache and speculative decoding each bought, and what they cost.
rfc: 0001-the-llm-service
headline: 281 tok/s
headline_note: the fast tier over four callers at once on one card, up from 120, completion tokens a second
---

## The question

Study 0001 found the fast tier's ceiling at about 120 tokens a second, whatever the
number of callers: the engine answered one request at a time and queued the rest out of
sight. Is that the card's limit or the engine's? If it is the engine's, how much of the
card is left on the table, and which of the usual tricks (parallel slots, a shared cache,
speculative decoding) take it back?

## What we believed

That the ceiling was mostly the card: a 16 GB card with a 20 billion parameter model on
it, bandwidth-bound, one token at a time. That parallel slots would trade speed per caller
for total throughput roughly one for one. That speculative decoding, with a draft model
built for exactly this model, would be the biggest single win.

## What we built

The same weights on a second engine: the fast model's official MXFP4 file for
`llama-server`, the engine the deep tier already ran, in its own container beside the
deep tier and with the card to itself apart from the deep tier's attention. The old
engine stopped for the measurement, so neither could take the card from the other.

The instrument this time is the engine directly, not the platform: the question is the
engine, and admission (which the platform now does, RFC 0001) would have shaped the
queue on purpose. A small script sends the same four prompts over the engines' shared
chat-completions interface, streamed, 256 tokens, low reasoning effort, in batches of 1,
2 and 4 at once, three batches per level after one warm-up request. For each request:
the time to the first token of any kind, and the decode rate (completion tokens over the
time from the first token to the last). For each level: all completion tokens over the
batches' wall time.

## What we measured

| Engine and setup | Callers | First token (median) | Per caller, tok/s | All callers, tok/s |
|---|---|---|---|---|
| previous engine, one slot | 1 | 108 ms | 126 | 120 |
| | 2 | 1,162 ms | 128 | 120 |
| | 4 | 3,213 ms | 129 | 123 |
| `llama-server`, two slots | 1 | 89 ms | 171 | 162 |
| | 2 | 120 ms | 119 | 226 |
| | 4 | 1,273 ms | 116 | 222 |
| `llama-server`, four slots, 8k context each | 1 | 87 ms | 171 | 162 |
| | 2 | 119 ms | 117 | 224 |
| | 4 | 140 ms | 79 | 297 |
| `llama-server`, four slots, one shared 32k cache | 1 | 90 ms | 169 | 161 |
| | 4 | 468 ms | 79 | 281 |
| `llama-server`, one slot, with the draft model | 1 | 85 ms | 159 | 153 |

Every `llama-server` setup fits beside the deep tier: about 12 GB for the fast model,
3 GB for the deep tier's attention, a gigabyte to spare. The draft model does not fit in
that gigabyte with the full context; it ran here with one slot and an 8k context only.

## What surprised us

**The engine was the ceiling, not the card.** The same file, on the same card, answers
one caller 36% faster on `llama-server` (171 against 126 tokens a second). Nothing about
the model changed; the difference is the engine's kernels for this weight format.

**One slot hid a whole card.** With one slot, four callers get four answers one after
another: the fourth waits over three seconds for its first token while the card could
have been working on it. Two slots nearly double the total (226 against 120) and keep
the first token near a tenth of a second for two callers; four slots reach 297.

**Parallel slots are not free per caller, but they are cheap.** Each caller's speed falls
as slots fill (171 alone, about 118 with a second, 79 with three others), because the
card reads the same weights once per step for everyone. The total still rises, and a
person reads well under 79 tokens a second.

**The draft model made it slower.** EAGLE-3 speculative decoding, with the draft built
for this model, gave 159 tokens a second against 171 without it. A mixture of experts
that activates about 3.6 billion parameters a token is already cheap to step; checking a
draft's guesses costs more than the guesses save at this size. It also costs the memory
the other slots need.

**A shared cache costs a little speed at full load.** With four slots of 8k each, a long
conversation is cut at a quarter of the context. One shared 32k cache lets any caller use
all of it; the price, with all four busy, is 281 instead of 297 tokens a second and a
first token in half a second instead of a seventh.

## What we chose

`llama-server` for both tiers, the fast one with four slots over one shared 32k cache, and
admission set to match: four running, eight waiting (RFC 0001). No draft model. The
previous engine stays installed and still works as a configuration choice, but it is off:
an idle one loads a model on the first request it gets and would take the card from the
fast tier.

## What we would do differently

Measure the engine before tuning its knobs: study 0001 spent a day on the previous
engine's parallelism setting when the engine itself was the larger difference. Check
what fits beside the other tier before choosing a context length, not after.

## Next

The same comparison through the platform, admission included, with the chaos tool's
open-loop load at several rates. The deep tier's own parallelism. Speculative decoding
again on the deep tier, where a step is expensive and a draft could pay.

## Glossary

- **Slot**: one conversation an engine runs at the same time as others; each step, it
  produces one token for every busy slot.
- **KV cache**: the engine's memory of a conversation so far; shared, any slot can use
  what the others are not.
- **Speculative decoding**: a small draft model guesses several tokens ahead and the
  large one checks them in one step; it pays when checking is cheaper than generating.
- **MXFP4**: a 4-bit number format for the model's weights that this model was
  released in.
