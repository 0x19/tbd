# playground — the public sandbox

One shared stack, permanently under load, that anyone on the internet may try to
break. It is the first thing on [inorbit.hr/playgrounds](https://inorbit.hr/playgrounds/),
and it exists to show two things at once: that the fault injection in
[chaos](../chaos/README.md) is real, and that the [protocol](../protocol/README.md)
serves the same RPC over REST, server-sent events and a WebSocket without any of
it being written twice.

```
browser ──▶ Caddy (apex) ──▶ Envoy `www` vhost ──▶ protocol ──▶ playground
                              /v1/playground/*                      │
                              public, per-IP limited                └─ tbd-lab
                                                                       protocol ─▶ engine LB ─┬─ engine-1
                                                                       ledger                 └─ engine-2
                                                                       + the load generator
```

## The game

A visitor sees the sandbox as it is, including whatever somebody else did to it
seconds ago. There is **one world and one budget**, shared: a move costs tokens
from the same pot for everyone, and a token comes back every few seconds.

| Move | Cost | What it does | Lasts | Alone |
|---|---|---|---|---|
| Add latency | 1 | an engine goes `slow`, 400 ms ± 80 | 30 s | ejected as a slow outlier; p99 peaked at 52 ms |
| Fail requests | 2 | an engine answers `error{unavailable}` to a third | 30 s | ejected within ~1.5 s; 0.86% failed |
| Stall the store | 2 | the ledger's store fails 1.5% of operations | 20 s | nothing routes around it; 0.52% failed |
| Kill a service | 3 | stops an engine; it restarts itself | 20 s | failed over; 0.00% failed |

The objective has **two clauses**, and breaking either breaks it: a rolling
success rate (99% over 30 s), and a p99 that stays under 250 ms, taken as the
mean of each second's p99 across the window so one slow second is not a
breach. Push either past its line and the world is **breached**; the breach is
timed from the last moment everything was healed, and the fastest ones are kept
on a leaderboard that survives a restart.

**One fault is survivable on purpose.** The last column is measured, not
estimated: each move on its own, against the shipped configuration, the worst
point of a 30 s window. All four stay under the 1% budget. Breaching means
stacking moves faster than they expire and faster than the budget refills —
kill one engine and fail the other, say, and stall the store while the balancer
has nowhere left to send anything. `crates/playground/tests/it/game.rs` holds
that line: it boots the real sandbox and fails if a single move ever breaches
again, or if breaking both engines ever stops breaching.

Every fault expires on its own, so the world heals without anyone tending it and
the next visitor starts from a clean sandbox. That is the whole reset story. The
page shows it happening: a breached world with nothing injected reads `healing`
with the seconds until the last bad second leaves the window, rather than a
`breached` that looks stuck.

## Why one fault is not enough

The sandbox has what a deployed environment has and an in-process lab usually
does not: a load balancer. `tbd_lab::enginelb` stands where Envoy stands in
production, in front of the two engines, and does the three things that make a
replica worth having:

- **Spreads requests, not connections.** gRPC multiplexes every call onto one
  HTTP/2 connection, so a layer-4 proxy would pin the protocol to one engine
  for its lifetime. This forwards whole requests, round-robin.
- **Health checks** take a stopped engine out within 250 ms and put it back
  when it answers again. That is `kill`.
- **Outlier ejection** takes out an engine that *keeps answering* but starts
  failing or crawling — which no health check catches, because a lying backend
  passes every one it is asked. Six requests of evidence over a five-second
  window, and it is out for fifteen seconds, doubling on every consecutive
  ejection so a fault that outlives one cooldown does not cost a fresh blip
  each time it lapses. That is `errors` and `latency`.

The ledger is deliberately **not** balanced. A second ledger with no database
behind it keeps its own facts in memory, so round-robin would append to one and
read from the other; the sandbox measured this at 1.7% failures with nothing
injected at all. One ledger is the honest version, and the better lesson:
stateless replicas are cheap and stateful ones are not. `stall store` is rated
to sit under the budget on its own precisely because nothing can absorb it.

## What a caller cannot do

The service takes **no identity**, so it is written as if every caller is
hostile. The wire carries a move, optionally an instance name, and optionally a
display name — and nothing else:

- **No durations, rates or addresses.** The parameters of each fault are fixed
  in `crates/playground/src/faults.rs`; there is nothing to clamp at runtime
  because nothing variable arrives.
- **`hang` and `delayed_failure` are unreachable.** The first parks a request on
  `pending()` for ever, the second nests behaviours without bound. A unit test
  fails if either becomes injectable.
- **The load generator never sees a caller-supplied URL.** Its targets come from
  `kind::load_targets(&stack, CORE)` — the stack's own instances — which is why
  the playground cannot be used to send traffic at anything else.
- **`kill` will not take the last one down.** With one instance left the move is
  refused, because a stopped sandbox is not a game.
- **Names are one tidy line**: trimmed, control characters dropped, whitespace
  collapsed, capped at 24 characters, and `anonymous` when empty.

Envoy adds the other half: the route carries the repository's first **per-IP**
bucket (`remote_address` as a rate-limit descriptor), 30 requests per 10 s per
address bursting to 60.

## The surfaces

Four RPCs in `proto/tbd/playground/v1/playground.proto`, and the gateway derives
the rest from the annotations:

| RPC | REST | Also |
|---|---|---|
| `GetWorld` | `GET /v1/playground/world` | `/v1/ws` |
| `Watch` (server-streaming) | `GET /v1/playground/events` → SSE | `/v1/ws` |
| `InjectFault` | `POST /v1/playground/faults` | `/v1/ws` |
| `Scores` | `GET /v1/playground/scores` | `/v1/ws` |

A server-streaming RPC **must** end in `/events` — the transcoder enforces it
both ways, and Envoy gives exactly those paths no timeout.

The page offers all three transports as a toggle, with what each is costing. The
numbers are not the same measurement and say so: a pushed frame is timed from
the previous frame, a poll from the request that fetched it.

Every number on the page describes **the last second**, not the run so far. The
load generator's histogram is drained once a second (`Metrics::drain`), because
percentiles cannot be subtracted and a phase-cumulative p99 would lag a full
minute behind the world it claims to describe.

## Running it

```sh
mise run run:playground        # the sandbox starts with it, on PLAYGROUND_LISTEN_ADDR
grpcurl -plaintext -d '{}' 127.0.0.1:50055 tbd.playground.v1.PlaygroundService/GetWorld
grpcurl -plaintext -d '{"move":"MOVE_LATENCY","actor":"me"}' 127.0.0.1:50055 \
  tbd.playground.v1.PlaygroundService/InjectFault
```

The binary runs the game; `serve_with` — what the chaos kind and the integration
tests use — starts **no** sandbox, because a chaos tool that launched the
playground would otherwise spawn four more services inside it.

`[game]` and `[sandbox]` in `configs/playground/base.toml` hold both clauses of
the objective, the budget, the traffic rate and where the scoreboard lives. The
instances themselves are fixed in `crates/playground/src/sandbox.rs`: two
engines behind the balancer, one ledger because it is the only kind with a store
to stall and the one thing that cannot be replicated for free, and a protocol
for the REST operations to drive, pointed at the balancer rather than an engine.
The balancer's policy — probe interval, ejection threshold, cooldown — lives
beside them, with the measurements that set it.

## Why a separate service

`chaos serve` is HTTP-only, so the gateway — which discovers backends from proto
descriptors — cannot reach it at all. More to the point, its API writes files by
path, accepts arbitrary load targets and takes unbounded fault parameters. None
of that belongs one route from the internet. This service exposes four verbs
instead of forty routes, and every one of them is clamped.
