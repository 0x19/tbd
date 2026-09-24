---
title: What a 16 GB card really does
status: measured
date: 2026-09-24
public: true
summary: Both tiers of the llm service measured with the chaos tool, on the processor while the card was dark and then on the card; what the numbers say about the model, the engine, the request, and the platform in front of them.
rfc: 0001-the-llm-service
headline: 117 tok/s
headline_note: the fast tier's ceiling on the card, completion tokens a second, all callers together
---

## The question

How many tokens a second does each tier give a visitor, how long until the first one
appears, how many visitors at once before that gets worse, and which part of the stack
sets each limit: the model, the engine that runs it, the request itself, or the platform
in front of them?

## What we believed

That the card would set the ceiling and everything in front of it would be invisible.
That a 48-token answer is a short answer. That the interesting numbers would be the
engine's.

## What we built

The chaos tool's `llm_generate` and `llm_generate_deep` operations: one short prompt per
request out of four, streamed to the done chunk, the whole stream timed, and beside that
timing the meters a latency cannot carry: prompt and completion tokens, generations that
reached the end, generations whose answer was not empty, and the time to the first chunk
(`ttft`). Load is open loop on an absolute clock: requests are sent at the configured rate
whether or not earlier ones have finished, so a slow engine shows up as latency and queue,
never as a quietly lower rate. Runs are sixty seconds after a five-second warm-up (two
minutes for the slow ones), against the deployed service through the platform's internal
load balancer, with the engines on the host. The record names, for every generation,
the engine's build and the model's revision, so this table can be rerun against the same
weights and the same code.

## What we measured

**The fast tier** is a 20-billion-parameter mixture-of-experts model (a few billion
active per token, 4-bit) served by Ollama. The first series ran while the card was dark
(a kernel and driver mismatch; the driver's log shows the card coming back at 00:22 on
the 24th, after these runs), so the engine ran on the 24-core processor. The second ran
on the card. Requests asked for 48 tokens.

| Where | Offered rate | Done | Failed | Latency p50 | Latency p99 | TTFT p50 | Completion tok/s | Answered |
|---|---|---|---|---|---|---|---|---|
| processor | 1/s | 59 | 0 | 0.69 s | 0.80 s | 0.32 s | 47 | 4 of 59 |
| processor | 2/s | 119 | 0 | 0.70 s | 0.72 s | 0.33 s | 95 | 6 of 119 |
| processor | 4/s, before the fix | 147 | 92 | 5.2 s | 5.4 s | 4.8 s | 109 | 4 of 147 |
| processor | 4/s, after the fix | 162 | 0 | 13.7 s | 14.9 s | 13.3 s | 105 | 6 of 162 |
| card | 1/s | 59 | 0 | 0.69 s | 0.81 s | 0.32 s | 47 | 5 of 59 |
| card | 2/s | 119 | 0 | 0.68 s | 0.75 s | 0.32 s | 95 | 7 of 119 |
| card | 4/s | 204 | 0 | 18.7 s | 27.2 s | 18.3 s | 112 | 10 of 204 |
| card | 8/s | 205 | 0 | 27.0 s | 27.1 s | 26.6 s | 113 | 11 of 205 |

Then with 256 tokens per request, on the card (the average completion came out at 200):

| Where | Offered rate | Done | Failed | Latency p50 | Latency p99 | TTFT p50 | Completion tok/s | Answered |
|---|---|---|---|---|---|---|---|---|
| card | 0.5/s | 29 | 0 | 2.4 s | 3.1 s | 0.49 s | 97 | 22 of 29 |
| card | 2/s | 67 | 0 | 42.0 s | 54.4 s | 40.9 s | 117 | 47 of 67 |

**The deep tier** is the 117-billion-parameter mixture-of-experts model, about 5 billion
active per token, at its native 4-bit, served by llama.cpp with the experts in system
memory and the attention on the card, beside the fast tier's model. Measured without a
queue, one request every five seconds:

| Tokens per request | Offered rate | Done | Failed | Latency p50 | Latency p99 | TTFT p50 | Completion tok/s | Answered |
|---|---|---|---|---|---|---|---|---|
| 48 | 0.2/s | 23 | 0 | 3.2 s | 4.6 s | 0.77 s | 9.2 | 9 of 23 |
| 256 | 0.1/s | 11 | 0 | 13.9 s | 26.5 s | 0.70 s | 14.7 | 10 of 11 |

Offered at one and two a second, the deep tier only queued: latencies of thirty to sixty
seconds and more, every generation still finishing. The table above is the one that
says what a generation costs.

**Parallelism**, the setting the first runs had left at its default. Ollama serves one
request at a time unless told otherwise; told to serve two, the fast model needs a second
context on the card. With the deep tier beside it there was no room, the model spilled a
sixth of itself to the processor, and everything collapsed. With the deep tier stopped,
two slots fit, and the ceiling rose by about a tenth. 256 tokens per request:

| Slots | Deep tier | Offered rate | Done | Latency p50 | TTFT p50 | Completion tok/s | Answered |
|---|---|---|---|---|---|---|---|
| 1 | running | 0.5/s | 29 | 2.4 s | 0.49 s | 97 | 22 of 29 |
| 2 | running, model 16 % on the processor | 0.5/s | 20 | 31.2 s | 23.4 s | 40 | 15 of 20 |
| 2 | stopped | 0.5/s | 29 | 2.5 s | 0.35 s | 99 | 18 of 29 |
| 2 | stopped | 1/s | 56 | 16.7 s | 13.9 s | 129 | 41 of 56 |
| 2 | stopped | 2/s | 71 | 38.8 s | 35.0 s | 131 | 53 of 71 |

**The challenger**, Google's 26-billion-parameter mixture-of-experts model (3.8 billion
active) at Q4_K_M, needed a newer engine to pull at all. On the same two slots with the
deep tier stopped it did not fit either: a fifth of it on the processor. Its numbers on
that footing, 256 tokens per request:

| Offered rate | Done | Latency p50 | TTFT p50 | Completion tok/s | Answered |
|---|---|---|---|---|---|
| 0.5/s | 29 | 8.8 s | 4.6 s | 106 | 0 of 29 |
| 1/s | 40 | 31.4 s | 27.1 s | 102 | 1 of 40 |
| 2/s | 59 | 54.0 s | 49.9 s | 112 | 2 of 59 |

It reasons even longer than the incumbent: with 256 tokens it answered almost nothing.
The comparison is not settled by this table, because neither model was measured alone on
one slot at the same budget; it is settled enough to keep the incumbent for the demo.

Method notes, so the tables can be reread: latency is the whole stream from the request
to the done chunk; TTFT is the first chunk of any kind, reasoning included; completion
tokens per second is the engine's own count summed over the window divided by the
window, so it is the whole engine's output, all callers together; "answered" counts
generations whose visible answer was not empty.

## What surprised us

**The first limit was the platform, not the model.** At four a second, 92 of 239
requests failed and the ones that succeeded sat at a p50 of 5.2 seconds, a number too
round to be the model's. It was the internal load balancer's retry policy, shared with
every other gRPC service: five seconds per try, and a retry on it. A generation slower
than five seconds was reset and started again, paid for twice, and failed after the
retries ran out. The fix is a policy of its own for the llm route: retry only a dial that
never reached the service, and never on a clock. The same load after the change has no
failures.

**What the fix revealed was the queue.** With nothing cutting the streams short, four a
second offered to an engine that finishes about two a second produced a queue that grew
for the whole minute: a p50 of 13.7 seconds and a first token at 13.3 seconds, though
each generation, once running, still took its 0.7 seconds. The engine runs a couple of
requests at a time; its parallelism is a setting left at its default. Above roughly two
generations a second, a visitor waits in line, and the honest demo shows the line.

**A 48-token answer is not a short answer for a model that reasons.** The model thinks
before it speaks, and the engine streams that thinking first. With a 48-token budget,
the thinking used the budget: only 4 of 59 generations at one a second contained any
answer at all. Before we measured this, the service dropped the reasoning chunks on the
floor, so a generation arrived as a single done chunk with no text and no first token
to time. Now the chunks are marked as reasoning and passed on, the request can turn
reasoning off for engines that allow it, and the operation counts who answered. At 256
tokens, three in four answered.

**At 48 tokens the card and the processor were indistinguishable.** The same 0.69
seconds, the same 0.32 to the first token, the same 47 and 95 tokens a second at one and
two a second, and a ceiling of about 110 a second on both. A request that short is
dominated by everything but decoding: the prompt, the engine's own scheduling, the
stream. We do not fully trust the processor's decode rate implied by those numbers (it
is above what its memory bandwidth should allow for this model), and the runs cannot be
repeated now that the card is back; the honest statement is that the instrument, at 48
tokens, could not tell the two apart. At 256 tokens the card's rate per stream is
about 100 tokens a second and its ceiling about 117 a second, all callers together; the
processor was not measured at that length.

**The instrument hit the budget it was measuring.** The first deep-tier run at 256
tokens was refused outright: the load operation's caller had spent its daily budget of
200,000 tokens on the fast-tier runs. The service did exactly what it promises a person.
The platform's own instruments are now exempt from the limit and still recorded, and the
fact that a day of measurement is a person's budget for a few weeks is worth knowing.

**Load order decides who gets the card.** The deep tier's server takes about three
gigabytes for its attention and cache when it starts; the fast model takes twelve to
fourteen. Started first, the deep tier leaves the fast model short and a sixth of it
lands on the processor; started second, into what the fast model left, both fit and the
fast model stays whole. The order is now part of how the machine is brought up. The
engine upgrade that the challenger required also changed the fast model's footprint
from fourteen gigabytes to twelve and a half, which is what made the pair fit with room.

**"Reasoning off" is a request, not an order.** The service can ask an engine not to
reason. The fast tier's model reasons regardless of the switch, and the deep tier's
server put the reasoning inside the answer text until it was told the format to separate
it with. Both now arrive as reasoning chunks the page folds away; the demo's switch says
what it can and cannot do.

**The deep tier costs twelve to twenty tokens a second per stream, and it slows as the
answer grows.** Forty-eight tokens took 3.2 seconds with the first token at 0.77
seconds, about twenty a second after the first; an average of 165 tokens took 13.9
seconds, about twelve a second, with the same first token. The experts are read from
DDR4-2133 memory with the attention on the card, and a longer sequence means more
attention per token. Both are above the ten a second the RFC's bandwidth hypothesis
predicted, though the hypothesis was for the whole stack and this is one stream at a
time; ten of eleven long generations answered, against nine of twenty-three short ones.

## What we would do differently

Measure the budget question before choosing it: the first runs should have asked for
enough tokens to get past the reasoning, or turned it off. Read the proxy's route policy
before pointing a stream at it. Set the engine's parallelism on purpose. Exempt the
instrument before pointing it at a budget.

## Next

Both candidates alone on one slot at the same budget, to settle the challenger. The deep
tier under two callers at once. The processor at 256 tokens, with the card disabled on
purpose, to settle what the 48-token runs could not. Then the studies that replace the
engine, one layer at a time.

## Glossary

- **TTFT**: time to first token, from the request to the first chunk of the stream.
- **Open-loop load**: requests sent on a schedule regardless of how earlier ones fare,
  so latency and queues are measured rather than hidden by back-pressure.
- **Reasoning tokens**: tokens a model produces while thinking before its answer; they
  spend the same budget and are streamed first.
- **Per-try timeout**: a proxy's clock on one attempt of a request, after which it resets
  the attempt and may retry it.
- **Mixture of experts**: a model whose layers hold many expert blocks of which only a
  few run for each token, so the active parameters, and the memory read per token, are
  a fraction of the total.
