---
title: The platform over MCP
status: open
date: 2026-09-24
public: true
summary: Every public RPC of the platform offered as a tool over the Model Context Protocol, as one more transport of the gateway beside REST, server-sent events, GraphQL and the multiplexed socket, so an agent can use the platform the way a page does.
---

## Problem

The platform is built to be used by people through pages and by programs through its
API. The programs that most want to use it now are agents: a coding assistant that
should be able to ask the local models, read the budget, start a measurement and read
its result without anyone writing glue for each of those. The Model Context Protocol is
how those agents discover and call tools, and a platform that is not reachable over it
is invisible to them.

## Proposal

**One more transport, not one more service.** The gateway already turns every annotated
RPC of every service into REST, server-sent events and calls on the multiplexed socket,
from the same descriptors. MCP becomes a fourth rendering of the same registry:

- every public RPC is a tool, named after its service and method (`llm_generate`,
  `llm_list_models`, `finance_list_transactions`), described by its comment in the
  contract, with an input schema generated from its request message by the same rules
  the published OpenAPI document uses, so the two never disagree;
- a call is the RPC, invoked as the caller: the gateway reads who is asking exactly as
  for every other transport, and a service answers an agent exactly as it answers a page;
- a streaming RPC is collected to its end, under a cap, and returned whole; for a
  generation the answer's text is joined and the model's reasoning kept apart;
- two resources: the list of tools and the OpenAPI document.

It is served over streamable HTTP on the API host, stateless, so either of the gateway's
replicas can answer any request.

**The gate is the platform's.** The API host already requires a verified bearer token
issued for the platform's API on every route; MCP adds no path around it
([REDACTED: how the token is minted and checked]). An agent is a caller like any other:
it has a subject, a budget and rights, and the record says what it did.

## Alternatives considered

**A separate MCP service with hand-written tools.** More control over each tool's shape.
Rejected: every new RPC would need a tool written twice, and the two would drift. The
registry is the single source; a tool that needs a friendlier shape gets it by changing
the RPC.

**Only the model service.** Smaller. Rejected: the point is that an agent can use the
platform, and the platform is more than its models.

**Local standard-input transport.** Simplest for one machine. Rejected as the main
transport: the agents that should use this are not always on the same machine, and the
platform's gate already works over HTTP.

## Decision

Open. Fixed so far: MCP as a transport of the gateway over the same registry and schema
rules; tools named by service and method; calls as the caller; streams collected under a
cap; streamable HTTP, stateless, on the API host behind the existing gate. Still to
decide: whether some RPCs are withheld from agents by default, and how an agent is told
its budget before it spends it.

## Publication

The protocol's contract page documents the tool names, the cap and how an agent is
connected. No study; the transport either answers every tool the registry lists or it
does not, and the tests say which.

## Status log

- 2026-09-24: opened.
- 2026-09-24: live on the API host. Every public RPC is a tool; the arguments' schemas
  and the tools' descriptions come from the contract and its comments; a streaming tool
  answers once with what it collected. Resources are not offered yet: the tool list is
  itself the catalogue.
