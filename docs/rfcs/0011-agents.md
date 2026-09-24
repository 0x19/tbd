---
title: Agents
status: open
date: 2026-09-24
public: true
lab: llm
summary: A named persona the model service speaks as, with its own instructions, bounds and knowledge, grounded in the page the visitor is reading; the first one talks about this site, from a chat at the bottom of every page.
---

## Problem

The workbench talks to a bare model: it answers anything, knows nothing about where it
is, and has no voice of its own. A visitor reading a page about the platform should be
able to ask about that page, and about the rest of the site, and get an answer that is
grounded in what the site actually says, without the page having to paste the site into
the question. And the platform will want more than one such voice: one that knows the
site, later one that reviews code people run, one that knows a lab.

## Proposal

**An agent is configuration, not code.** One file per agent in the model service's
configuration: its name, a one-line persona a visitor sees, its instructions, the tier
it runs on and its temperature, how long its answers may be, who may use it, and the
knowledge it answers from. Adding an agent is a reviewed file, not a release.

**The service speaks as the agent.** A request that names an agent sends only the
visitor's turns; the service puts the agent's instructions and knowledge in front of
them, in that order, every time. A request that names an agent and also sends
instructions of its own is refused: a visitor cannot replace what an agent is told. The
agent's bounds apply where the request is silent, and the caller's own budget,
admission and record apply as they always do.

**Grounded in the page, not in everything.** The site's public documents are about
twenty thousand tokens and the fast tier's cache is shared by four conversations, so an
agent cannot carry the whole site in every turn. It carries a **brief** instead, a few
thousand tokens: every page with its path and one line, who the site is about and what
the work is, every public RFC and study with its status, summary and newest status
line. And the page the visitor is reading, when the request says which one: its full
text, capped. A question about the page in front of the reader is answered from that
page; a question about the rest is answered from the brief, with a link.

**The knowledge is generated from the public site.** The same build that renders the
lab's documents writes the knowledge file, from the same sources and through the same
redaction check; drafts are never read. A withheld fact stays withheld: the agent sees
the redaction's reason, not the fact. The build fails if the committed file is stale.

**The first agent, the site guide.** It talks about the site: who it is about, the work,
the lab and its documents, the playgrounds. It speaks about the site's owner in the
third person and never as its owner, answers only from its knowledge and says when that does
not cover a question, links the pages it draws on, answers in the visitor's language,
and sends anything personal to the contact page. It has no tools: it can only talk
(tools and memory are RFC 0002's).

**Where it lives.** A chat at the bottom of every page, with the same figures as the
workbench under every answer (tier, model, time to the first token, tokens a second,
tokens in and out, the model's reasoning folded above), and on the home page's live lab
section. The workbench can talk to any agent the caller may use.

## Alternatives considered

**The whole site in every prompt.** The simplest grounding, and too big: every turn
would fill most of a shared cache and wait on its prefill. Rejected.

**Retrieval by embeddings.** Find the few passages that match a question and send only
those. Better at scale, and the right answer once the site outgrows a brief; it needs
the embedding tier and the memory service of RFC 0002. Later.

**An agent per page.** A voice for each section. Rejected: the page is context, not a
persona; one guide that knows which page it is on does the same with one file.

## Decision

Open. Fixed so far: agents as configuration in the model service; the instructions and
knowledge put in front by the service and never by the caller; the brief plus the
current page as grounding; knowledge generated from public documents through the
redaction check; the site guide as the first agent, without tools; admins only while
the lab is private.

**Before visitors meet it** (added to RFC 0006's conditions): an allowance for callers
who are not signed in, a limit per address at the edge, and the privacy page saying
what the chat sends and keeps.

## Publication

The chat at the bottom of every page and the home page's live lab section, for admins
until publication; the workbench's agent picker.

## Status log

- 2026-09-24: opened.
- 2026-09-24: the model service speaks as agents. `ListAgents` shows who each is and
  whether the caller may use it; a request that names one sends only its turns; the
  service composes the rest and refuses instructions from the caller; the record names
  the agent. The site guide's knowledge is generated and checked on every build.
- 2026-09-24: the site guide is on the site, for admins. A chat at the bottom of every
  page but the workbench sends the page's path with the question, shows the workbench's
  figures under every answer, keeps the conversation in the browser and hands it to the
  workbench whole; the home page shows the lab running, the guide and the workbench. A
  link in an answer is kept when it is a page of this site, and opens in the same tab.
