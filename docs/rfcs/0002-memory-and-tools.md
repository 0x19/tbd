---
title: Memory and tools
status: open
date: 2026-09-24
public: true
lab: llm
summary: What turns a model that answers into a platform that works - a memory it recalls from with its sources named, tools it may call under the caller's budget, and an agent loop that is traced step by step.
---

## Problem

The service in RFC 0001 answers one conversation at a time and forgets it: nothing is
stored but who asked and what it cost. That is right for a demo and wrong for anything
that has to do work across days. An assistant worth using remembers what it was told,
recalls it when it matters and says where it came from; and it can act, not only talk:
read a file, query a service of this platform, start a measurement, and look at the
result before it answers.

Both are easy to do badly. Memory that is a pile of transcripts is a privacy problem
and a retrieval problem at once; tools that the caller declares freely are a way to make
the platform call anything; an agent loop without a budget and a trace is a way to spend
a day's tokens on a question nobody can reconstruct afterwards.

## Proposal

**Memory is a service of its own**, not a table inside the model service. It keeps three
kinds of entry per subject, each with the text, where it came from and when:

- **facts**: short statements the subject or an agent chose to keep ("the ledger's
  facts are append-only"), with the source they were taken from;
- **episodes**: what happened in a session, summarised when the session ends, never the
  raw transcript;
- **preferences**: how the subject wants to be answered.

Each entry is embedded by a tier whose engine declares an embedding model (RFC 0001's
capability rule) and stored beside its vector in the platform's Postgres. Recall is a
query: the conversation's last turn, embedded, the nearest entries of that subject above
a threshold, and a hard cap on how many and how many tokens. What was recalled goes into
the prompt as a block the model is told to cite, and comes back to the caller beside the
answer, so every answer that leaned on memory says which entries it leaned on. A subject
can list, correct and forget their entries; forgetting is deletion, not a flag.

**Tools are the platform's, not the caller's.** The contract gains tool calling: the
model may ask to call one of the tools the service offers for the tier and the caller,
with arguments that must validate against the tool's schema. The first tools are the
platform's own RPCs, the same set RFC 0005 exposes over MCP, filtered to the caller's
rights; a tool runs as the caller, never as the service. A caller cannot hand the model a
new tool in the request.

**The agent loop lives in the service** and is bounded: a number of steps, a number of
tokens (counted against the same daily budget), a wall-clock limit, and a trace per step
(what the model asked, what the tool returned, how long each took) recorded beside the
generation row without the text of either. The caller sees every step as it happens, as
chunks of their own kind in the same stream.

## Alternatives considered

**Keep the transcripts and search them.** The most recall for the least design, and the
most exposure: every question ever asked, verbatim, in a database. Rejected; the record
stays free of text, and memory keeps what was chosen to be kept.

**A vector database beside Postgres.** Rejected for now: the platform already runs
Postgres, the entries are counted in thousands per subject, not billions, and one store
is one thing to back up, migrate and reason about. A study says when that stops being
true.

**A multi-model database in its place (SurrealDB).** One engine for documents, graph
edges, vectors and full-text search, written in Rust, embeddable, and sold for exactly
this: agent memory as a knowledge graph with vector recall. Rejected for now, for four
reasons. It would be a second stateful system to run, back up and test beside a Postgres
that already has vector indexes and serves the rest of the platform. Its ready-made memory
layer is the part this RFC means to build and understand, not buy. Its third major
version is months old and its index layouts have changed between minor releases. Its
licence restricts offering it as a service until it becomes Apache 2.0 in 2030. It
would win if memory became a dense graph whose main question is several hops deep; the
memory study measures how deep recall actually goes, and this is revisited if the answer
is "often more than one".

**Caller-declared tools, as the model APIs offer them.** Rejected: a platform that runs
whatever tool a request describes runs whatever the request wants. The tools are the
platform's, listed and schema-checked.

## Decision

Open. Fixed so far: memory as its own service with the three kinds, no transcripts,
recall with sources returned to the caller, forgetting as deletion; tools as the
platform's RPCs run as the caller; the loop bounded by steps, tokens and time and traced
without text. Still to decide: the embedding model and its tier, the summarisation of an
episode (which tier, and when), the recall threshold, and whether memory is shared
across subjects ever (the default is never).

## Publication

The workbench (RFC 0004) shows recalled entries beside each answer and every tool call
as a card in the transcript. Study 0004 measures recall: how often the right entry comes
back, how often a wrong one does, and what it costs per turn.

## Status log

- 2026-09-24: opened.
- 2026-09-24: a multi-model database considered in place of Postgres; rejected for now, with the condition that would reopen it.
- 2026-09-24: agents (RFC 0011) are the first users of tools and memory once they exist; until then an agent is grounded by a brief and the page it is on.
