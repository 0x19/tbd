---
title: The llm service, an L2 over two engines
status: decided
date: 2026-09-23
public: true
summary: The model-serving core of a general-purpose AI development platform on one workstation, split into the engines that run the weights and the layer that owns the contract, the admission, the budget and the record; two models from day one, one that fits the graphics card and one that runs from memory.
---

## Problem

I want an AI development platform of my own: models served from the machine under my
desk rather than rented by the token, as fast and as stable as that machine can make
them, with the memory, the tools and the measurement that let it do real work, and
reachable by other agents as well as by people. This document decides its core, the
service that serves the models. The pillars around it each have a document of their
own: memory and tools (0002), a shield at the edge (0003), the workbench where it is
used and watched (0004), the platform over MCP (0005), and when it all goes public
(0006). Two things make the core harder than "run a model server and put a chat box on
a page".

The first is honesty. A page that says "my own LLM" over a hosted API is a lie, and a
page that says it over an unmodified download is a stretch. What I can truthfully own on
one workstation is the layer around the weights: how requests are admitted, routed,
bounded, recorded and measured, and later how the model is adapted to my own work. The
service has to be built so that layer is real and the weights are a replaceable part.

The second is the hardware. The machine has a 16 GB graphics card, 247 GiB of system
memory and a 24-core processor. A model that fits the card answers a visitor at a
speed that feels like typing; a model that does not fit the card runs from system memory
at a fraction of that, but with far more capacity. Neither alone is what I want: the
first is the one that answers, the second is the one that thinks harder. The service
has to hold both without the contract knowing which one answered, and it has to stay
standing when more people ask than the card can serve.

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
  embedding RPC for the tiers whose engine declares one (a chat model does not embed,
  whatever it would answer if asked), a models RPC that says what is behind each tier,
  whether it is up and which build and weights it is, and a budget RPC that tells a
  caller where they stand;
- identity and budget: the caller is whoever the gateway verified, never a claim in the
  request; each caller has a daily token budget, summed from the record, and a request
  over it is refused with the count and the reset time;
- routing: a request names a **tier**, `fast` or `deep`, and the service picks the
  engine; a request that names none goes to the default;
- admission, bounds and deadlines: how many generations each tier runs at once, how many
  may wait and for how long, and a fast, named refusal beyond that; how many messages,
  how long, how many tokens; the tier's timeout is one clock from admission that covers
  the dial and the stream;
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

**One model, not two.** Simpler to operate. Rejected: a model that fits 16 GB answers
fast but thinks shallow; a model that needs 60 GB thinks harder but answers slowly. Routing by tier is
cheap and it is the only way both stories run on one machine.

**Serving directly from a model server with a thin page in front.** No Rust, no
service, a week saved. Rejected because the interesting part is the layer, not the
page: budgets, records, deadlines, fault injection and measurement are what the studies
will be about, and a thin page has none of them. It is also what every other product in
this space is.

**Fine-tuning first.** Rejected as a starting point: memory and recall (RFC 0002) are
cheaper, inspectable (every answer names its sources) and stay current with what they
remember. Fine-tuning comes when a measured evaluation shows a gap recall cannot close,
and the comparison itself is a study.

**Letting the queue grow.** An engine that is busy can simply be asked to wait, and the
engines do queue. Rejected: study 0001 measured what that means, a first token after
forty seconds for everyone once the offered rate passes what the card can serve. A
caller is better served by a fast, named refusal it can retry than by a wait it cannot
see the end of.

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
The reason is bandwidth, not capacity, and it is a hypothesis to test rather than a
number to quote: this machine's memory is eight sticks of DDR4-2133 in four channels,
roughly 68 GB/s in theory, a ceiling consistent with double-digit tokens a second for a
model whose *active* weights read about 3 GB per token, and with a crawl for one whose
active set is four times that. Decode from system memory is not only a memory read:
the processor's arithmetic, the transfers to the card, expert dispatch, cache behaviour
and the engine's own overhead all sit between the ceiling and the number. The study
measures how close the stack gets. A faster memory kit is the single cheapest upgrade
this machine has, and it is noted, not planned.

**Admission.** Each tier runs a bounded number of generations at once and lets a
bounded number wait for a bounded time; beyond either, the request is refused at once,
saying which tier was busy and how long it waited. The numbers are configuration, set
from the studies (the card holds one fast context beside the deep tier's cache, so the
fast tier starts at one or two in flight), and the service exports how many are in
flight, how long they waited and how many were refused.

**One engine for both tiers, once measured.** Both tiers stay behind the same contract,
so the fast tier's engine can move to the one the deep tier uses, with continuous
batching and a small draft model for speculative decoding where one exists for the
model. It moves when a study shows it at least as fast and at least as stable on this
card; until then the fast tier stays where it is.

**The record** lives in the platform's Postgres, in the service's own schema, keyed by
the verified subject. Without the database the service still runs: it records nothing
and enforces no budget, and it says so at start and on the wire, because a demo must
never mistake "unlimited" for "under budget".

**Reproducibility.** The same model name over another quantisation, another chat
template or another engine build is another model, and a benchmark that cannot say
which one it ran against cannot be rerun six months later. So every generation's row
names the engine's build and the revision of the weights (a digest, a file name) as the
service read them from the engine, and the models RPC shows the pair per tier; an
engine that does not say is recorded as unknown, never guessed. The prompt template's
version joins that pair once the service owns one.

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

This RFC is public in the lab, and the lab is behind a sign-in until RFC 0006 says
otherwise. Study 0001 measured the first decision (what a 16 GB card really does);
study 0002 measures the second (admission and one engine for both tiers). The workbench
(RFC 0004) is where the service is used and watched, and its page names what it sends
and where, before it sends anything.

## Status log

- 2026-09-23: opened, and decided the same day: the split, the two tiers, the candidate
  models, the record and the budget. The service exists (`crates/llm`) with three engines
  and the conformance suite; the studies come next.
- 2026-09-24: after review. The deep tier's throughput is stated as a hypothesis with
  the ceiling and what sits under it, not as a number; the record names the engine
  build and the model revision per generation; embedding is a capability a tier
  declares, not something every engine is assumed to do.
- 2026-09-24: both tiers live on the workstation. The card's driver had been missing for
  the running kernel, so the fast tier first answered from the processor at a crawl;
  with the card back it generates at about 126 tokens a second. The deep tier, the
  117B model with its experts in memory and attention on the card, generates at about
  19 tokens a second, above the ten this document predicted. Both models fill the
  card to within a gigabyte when loaded together; the first study measures the split.
- 2026-09-24: the charter widens. This service is now the core of a general-purpose AI
  development platform rather than a single demo, with a document per pillar (0002 to
  0006). Admission control joins the decision: bounded generations in flight and in
  waiting per tier, and a fast, named refusal beyond them, after study 0001 measured
  what an unbounded queue costs. The fast tier's move to the deep tier's engine is
  decided on a condition: when a study shows it at least as fast and as stable.
- 2026-09-24: admission control is live. Each tier runs as many generations at once as
  its engine really runs in parallel (one on each tier today), keeps a short line
  behind them, and refuses beyond it at once with a reason; a wait that runs out is
  refused too. The model listing says, per tier, how many are running and waiting.
