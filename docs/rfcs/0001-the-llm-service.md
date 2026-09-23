---
title: The llm service, an L2 over two engines
status: decided
date: 2026-09-23
public: false
summary: A service of this platform that serves open-weight models from one workstation, split into the engines that run the weights and the layer that owns the contract, the budget and the record; two models from day one, one that fits the graphics card and one that runs from memory.
---

## Problem

I want a model of my own on this site: one that answers questions about this platform,
later about my own history, and eventually interviews me about it, served from the
machine under my desk rather than rented by the token. Two things make that harder than
"run a model server and put a chat box on a page".

The first is honesty. A page that says "my own LLM" over a hosted API is a lie, and a
page that says it over an unmodified download is a stretch. What I can truthfully own on
one workstation is the layer around the weights: how requests are admitted, routed,
bounded, recorded and measured, and later how the model is adapted to my own work. The
service has to be built so that layer is real and the weights are a replaceable part.

The second is the hardware. The machine has a 16 GB graphics card, 247 GiB of system
memory and a 24-core processor. A model that fits the card answers a visitor at a
speed that feels like typing; a model that does not fit the card runs from system memory
at a fraction of that, but with far more capacity. Neither alone is what I want: the
first is the demo, the second is the judge. The service has to hold both without the
contract knowing which one answered.

## Proposal

Borrow the shape from rollups: an **L1** that does the expensive, generic work and an
**L2** that owns everything with a policy in it.

**L1 is whatever runs the weights.** Today that is Ollama for the model on the card
and llama.cpp's server for the model in memory; later it is an engine of our own,
written in Rust on candle, one layer at a time (weight loading, the tokenizer, the
forward pass, the cache, batching, sampling), each layer a study with a number to beat.
An engine holds a model and turns a prompt into a stream of tokens. It knows nothing
about who is asking.

**L2 is the `llm` service**, a service of this platform like every other one, and it
owns:

- the contract: one streaming RPC that takes a conversation and returns chunks, an
  embedding RPC, a models RPC that says what is behind each tier and whether it is
  up, and a budget RPC that tells a caller where they stand;
- identity and budget: the caller is whoever the gateway verified, never a claim in the
  request; each caller has a daily token budget, summed from the record, and a request
  over it is refused with the count and the reset time;
- routing: a request names a **tier**, `fast` or `deep`, and the service picks the
  engine; a request that names none goes to the default;
- bounds and deadlines: how many messages, how long, how many tokens; the tier's timeout
  is one clock from admission that covers the dial and the stream;
- the record: a session a caller may continue, and one row per generation with the
  tier, the engine, the model, the outcome, the engine's own token counts and the time
  to the first token; no prompt and no completion text is stored;
- observability: a span per request, tokens and time-to-first-token as metrics, a probe
  per engine, and checks the chaos tool runs against a live stack.

Two rules keep the split honest. Nothing above the engine table in the service's
configuration, and nothing in its contract, names an engine or its wire format; a new
engine is a module and a line of configuration. And every engine, the test stub
included, passes one conformance suite: the same cases, run once per engine, so an
engine is done when it passes, not when it worked once.

## Alternatives considered

**A hosted API as the first engine.** Fastest path to a good demo, and it would fit the
L1 seat exactly. Rejected for now: every prompt would leave the machine, and the "own"
in "my own model" would start as a plan rather than a fact. The seat stays open; a
hosted engine can be added later without touching the contract.

**One model, not two.** Simpler to operate. Rejected: a model that fits 16 GB is a
demo, not a judge; a model that needs 60 GB is a judge, not a demo. Routing by tier is
cheap and it is the only way both stories run on one machine.

**Serving directly from a model server with a thin page in front.** No Rust, no
service, a week saved. Rejected because the interesting part is the layer, not the
page: budgets, records, deadlines, fault injection and measurement are what the studies
will be about, and a thin page has none of them. It is also what every other product in
this space is.

**Fine-tuning first.** The eventual goal is a model that knows my work. Rejected as a
starting point: retrieval over the repositories is cheaper, inspectable (every answer
names its sources) and stays current with the code. Fine-tuning comes when a measured
evaluation shows a gap retrieval cannot close, and the comparison itself is a study.

## Decision

Build the `llm` service as described, with these choices fixed for the first study and
open to the measurements after it.

**The fast tier** runs on the card, served by Ollama. The candidate is a 20 billion
parameter mixture-of-experts model with a few billion active parameters per token, at
4-bit, which leaves the card room for the context; the challenger is Google's 26B
mixture-of-experts release of the same season. The first study measures both on this
card, and the numbers pick the one that stays.

**The deep tier** runs from system memory, served by llama.cpp with the attention and
the cache on the card and the expert weights in RAM. The model is OpenAI's 117 billion
parameter open-weight release, about 5 billion active per token, at its native 4-bit.
The reason is bandwidth, not capacity: this machine's memory is eight sticks of
DDR4-2133 in four channels, roughly 68 GB/s in theory, so a model whose *active*
weights read about 3 GB per token stays above ten tokens a second, and one whose active
set is four times that would crawl. A faster memory kit is the single cheapest upgrade
this machine has, and it is noted, not planned.

**The record** lives in the platform's Postgres, in the service's own schema, keyed by
the verified subject. Without the database the service still runs: it records nothing
and enforces no budget, and it says so at start and on the wire, because a demo must
never mistake "unlimited" for "under budget".

**The engines are dialled directly**, like databases, not through the gateway; they are
model servers on the machine with the weights, not services of the platform. Their
addresses are configuration and are not in this document
([REDACTED: where the engines listen]).

**The gate.** The demo page is behind a sign-in, so that asking needs an account and the
budget applies to a person. The mechanism is the platform's usual one
([REDACTED: how the page is gated and what the service reads to know the caller]);
what matters to a reader is that the service never verifies a token itself and never
trusts a claim in a request.

## Publication

This RFC is public once its author has read it on the page. The lab it appears in is
itself admins-only until then; the flip is one deliberate change in two places, listed
in the site's own notes. The first study follows: what a 16 GB card really does, with
the baselines of both tiers on both candidate models, measured by the chaos tool. When
the demo goes live, its page names what it sends and where, before it sends anything.

## Status log

- 2026-09-23: opened, and decided the same day: the split, the two tiers, the candidate
  models, the record and the budget. The service exists (`crates/llm`) with three engines
  and the conformance suite; the studies come next.
