---
title: The workbench and the arena
status: open
date: 2026-09-24
public: true
lab: llm
summary: The page where the platform is used and watched - a keyboard-first workbench for conversations with the models, tools and memory, beside a live view of what happens behind it, and a small set of runs a visitor may start to see the platform under load and fault.
---

## Problem

The demo page shows that the model answers. It does not show what makes the platform
worth building: the tiers deciding who answers, the queue filling and refusing, the
budget counting, memory being recalled, a tool being called, the chaos tool breaking
something on purpose and the platform holding. A visitor sees a text box and a reply,
and everything interesting happens where they cannot see it.

A page that is going to be linked from the front of the site also has to be good in its
own right: something a developer would want to keep open, not a form.

## Proposal

**The workbench** is a full-height, keyboard-first page in the manner of the terminal
coding agents: a transcript that streams, a prompt at the bottom, and around them only
what earns its place.

- The transcript: each turn streams as it is generated; the model's reasoning folds
  away above its answer; a tool call is a card with its arguments, its result and its
  time; recalled memory is listed under the answer that used it, each entry with its
  source.
- The prompt: multi-line, enter to send, a slash menu for the things a keyboard user
  wants without leaving it (tier, reasoning, clear, a new session, the runs below), and
  a status line under it: tier, model, tokens in and out, time to the first token,
  tokens a second, budget left today.
- Sessions: a list to the side, kept in the browser; nothing about them leaves it unless
  the visitor turns memory on for them (RFC 0002).

**The arena** is the "behind the scenes" panel beside the workbench, live:

- the tiers: in flight, waiting, refused, tokens a second, first-token latency, each as
  a small moving series;
- the current chaos run, if any: its phase, its offered and achieved rate, its errors,
  and its own tokens and latency, as the chaos tool reports them each second;
- the shield (RFC 0003): packets passed and dropped per second, top sources by count.

It is fed over the platform's multiplexed socket by an `arena` service that subscribes
to the chaos tool's run feed, the model service's meters and the shield's counters, and
republishes one compact stream. A visitor never talks to the chaos tool directly.

**Preset runs.** A signed-in visitor may start one of a short, fixed list of runs from
the workbench: a burst at the fast tier, a slow deep-tier question under load, the model
engine killed mid-stream, the storm at the shield. One run at a time for the whole site,
a cool-down per visitor, each preset bounded in rate and length so no visitor can do
more than watch the platform work hard for a minute. The arena shows the run as it
happens and what the platform did about it.

## Alternatives considered

**Grafana on the page.** The dashboards exist and are good. Rejected for the public
page: they expose the whole platform's internals and need an account of their own; the
arena shows a chosen few series, from a stream built for it.

**Polling the services from the page.** Simpler. Rejected: several requests a second per
visitor against the services the page is trying to show under load would be the load.

**Letting visitors write their own load.** More fun. Rejected: a preset is something the
platform is designed to survive; an arbitrary run is something it might not, on a
machine that also serves everything else.

## Decision

Open. Fixed so far: the page layout (transcript, prompt with slash menu and status line,
sessions kept in the browser), the arena as its own service republishing one stream over
the multiplexed socket, preset runs only, one at a time, bounded and cooled down.

The stream's shape is fixed too: one whole snapshot a second, the current one first, so
a viewer that falls behind skips ahead instead of replaying. It carries each model tier
(running, slots, waiting, up, tokens a second, time to the first token at the median and
the 99th percentile, refusals a minute), each way in (REST, server-sent events, the
socket, MCP, gRPC) as the chaos tool last checked it end to end, with the time of that
check and the number of tools an agent is offered, and the chaos tool's current run with
its rates. A figure no source could give is absent, never zero, and the snapshot says
which source is behind and why. Watching needs the lab's role while the lab is private.

Still to decide: the preset list and each one's bounds, and when the page leaves the lab
for the front of the site (RFC 0006).

## Publication

The workbench replaces the demo page in the lab. Study 0005 measures what watching
costs: the arena's own load on the platform, and the latency from an event to the pixel.

## Status log

- 2026-09-24: opened.
- 2026-09-24: the arena runs. One snapshot a second from the model service, the metrics
  store and the chaos tool, over the same gateway as everything else; the ways in are the
  chaos tool's own end-to-end checks, run on a schedule, never inferred.
- 2026-09-24: the workbench replaces the demo page. Sessions kept in the browser, a
  transcript that streams with the reasoning shown while the model thinks, a slash menu
  and a status line, the same turn over server-sent events, the socket or MCP, the
  platform's tools as cards, and the arena beside it; the lab's own page carries the same
  live view across its width. Preset runs are still to come.
- 2026-09-24: the workbench runs code. A Go or Rust block in an answer has a run button,
  and `/run go` or `/run rust` runs code typed into the prompt; the program is compiled
  and run once in the sandbox of RFC 0010 and its output comes back as a card, with why
  it was stopped when it was.
- 2026-09-24: the workbench talks to agents (RFC 0011). A session talks to the bare model
  or to one agent, picked in its header or with `/agent`; with an agent the service's
  tier and bounds apply and the status line says so. The same conversation, with the same
  figures under every answer, is a chat at the bottom of every page for an admin, and it
  hands over to the workbench whole.
- 2026-09-24: sessions are named and agents are first-class in the workbench. A session
  is started for the bare model or for an agent, keeps its name once given one, shows
  whom it talks to, and saves as markdown; the side column lists the agents, and an
  agent can be told which page is being read, to test its grounding.
