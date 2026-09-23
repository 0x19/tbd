---
title: What a 16 GB card really does
status: running
date: 2026-09-24
public: false
summary: The fast tier measured with the chaos tool, first on the processor while the card is dark and then on the card; what the numbers say about the model, the engine and the platform in front of it.
rfc: 0001-the-llm-service
---

## The question

How many tokens a second does the fast tier give a visitor, how long until the first
one appears, how many visitors at once before that gets worse, and which part of the
stack sets each limit: the model, the engine that runs it, or the platform in front of
it?

## What we believed

That the card would set the ceiling and everything in front of it would be invisible.
That a 48-token answer is a short answer. That the interesting numbers would be the
engine's.

## What we built

The chaos tool's `llm_generate` operation: one short prompt per request out of four,
streamed to the done chunk, the whole stream timed, and beside that timing the meters
a latency cannot carry: prompt and completion tokens, generations that reached the end,
generations whose answer was not empty, and the time to the first chunk (`ttft`). Load
is open loop on an absolute clock: requests are sent at the configured rate whether or
not earlier ones have finished, so a slow engine shows up as latency and queue, never as
a quietly lower rate. Runs are sixty seconds after a five-second warm-up, against the
deployed service through the platform's internal load balancer, with the model's
engine on the host.

## What we measured

The card was dark for these runs (a kernel and driver mismatch, fixed later in this
study), so the engine ran the fast tier's 20-billion-parameter mixture-of-experts model
on the 24-core processor. Each request asked for at most 48 tokens.

| Offered rate | Generations done | Failed | Latency p50 | Latency p99 | TTFT p50 | Completion tokens/s | Answered |
|---|---|---|---|---|---|---|---|
| 1/s | 59 | 0 | 0.69 s | 0.80 s | 0.32 s | 47 | 4 of 59 |
| 2/s | 119 | 0 | 0.70 s | 0.72 s | 0.33 s | 95 | 6 of 119 |
| 4/s, before the fix | 147 | 92 | 5.2 s | 5.4 s | 4.8 s | 109 | 4 of 147 |
| 4/s, after the fix | 162 | 0 | 13.7 s | 14.9 s | 13.3 s | 105 | 6 of 162 |

Method notes, so the table can be reread: latency is the whole stream from the request
to the done chunk; TTFT is the first chunk of any kind, reasoning included; completion
tokens per second is the engine's own count summed over the window divided by the
window; "answered" counts generations whose visible answer was not empty.

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
second offered to an engine that answers about two a second produced a queue that grew
for the whole minute: a p50 of 13.7 seconds and a first token at 13.3 seconds, though
each generation, once running, still took its 0.7 seconds. The engine ran the requests
one at a time; its parallelism is a setting we had left at its default. The aggregate
was the same either way, about 105 completion tokens a second, which is this processor's
ceiling for this model at this quantisation. Above roughly two generations a second, a
visitor waits in line, and the honest demo shows the line.

**A 48-token answer is not a short answer for a model that reasons.** The model thinks
before it speaks, and the engine streams that thinking first. With a 48-token budget,
the thinking used the budget: only 4 of 59 generations at one a second contained any
answer at all. Before we measured this, the service dropped the reasoning chunks on the
floor, so a generation arrived as a single done chunk with no text and no first token
to time. Now the chunks are marked as reasoning and passed on, the request can turn
reasoning off for engines that allow it, and the operation counts who answered.

**One generation a second on the processor feels like typing.** A 48-token generation
took 0.69 seconds at the median with the first token at 0.32 seconds, and the second
request a second did not slow the first. That is on a processor whose memory is
DDR4-2133. It is the number to beat on the card.

## What we would do differently

Measure the budget question before choosing it: the first runs should have asked for
enough tokens to get past the reasoning, or turned it off. Read the proxy's route
policy before pointing a stream at it. Set the engine's parallelism on purpose.

## Next

The card: the same three rates with the driver in place. The engine's parallelism set
to match the cores. The challenger model on the same table. The deep tier, from memory.

## Glossary

- **TTFT**: time to first token, from the request to the first chunk of the stream.
- **Open-loop load**: requests sent on a schedule regardless of how earlier ones fare,
  so latency and queues are measured rather than hidden by back-pressure.
- **Reasoning tokens**: tokens a model produces while thinking before its answer; they
  spend the same budget and are streamed first.
- **Per-try timeout**: a proxy's clock on one attempt of a request, after which it resets
  the attempt and may retry it.
