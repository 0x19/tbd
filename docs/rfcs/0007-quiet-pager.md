---
title: Quiet Pager, an InOrbit lab
status: open
date: 2026-09-24
public: false
lab: quietpager
summary: A build-in-public product for Go and Rust engineers — fault-graded katas from production systems and a weekly Radar of what changed in both languages, told by an AI avatar — built on the platform, shown on the site, with its tools open source.
---

## Problem

Staying current on two languages that move every six weeks, and practising the parts of
systems work that only show up under failure, have no good home. The syntax is well served
(Exercism, Rustlings, Go by Example); the behaviour is not. Nobody grades a learner's rate
limiter or write-ahead log the way production grades it: under latency, resets, restarts
and a performance budget. The one grader that injects faults, Gossip Glomers, is free and
narrow; CodeCrafters, the "build your own X" leader, paused new challenges in May 2026 and
is looking for a buyer after failing to reach startup-scale revenue.

At the same time the owner wants to build in public with AI the way JavaScript Mastery
does, but in his own field, without being on camera, and in a way that feeds the October
2026 contract search rather than competing with it.

## Proposal

One brand, **Quiet Pager**, endorsed by InOrbit ("an InOrbit lab"), with four layers:

1. **The grader** (RFC 0008): fault-graded katas from production systems, each specified
   once and implemented in Go and in Rust side by side. Graded on the learner's machine by
   an open-source CLI, `qp`, that launches their program, puts a fault proxy in front of it,
   runs paced load and a correctness checker, and restarts it where the level says.
2. **The Radar**: a `radar` service on the platform that reads the official Go and Rust
   sources (release notes, blogs, accepted proposals, merged RFCs, This Week in Rust, the
   Clippy changelog) and writes a weekly digest with the llm service: what changed, why it
   matters, a ten-minute drill, and a 60-second script for the avatar, in English and
   Croatian. Shown at `/radar/`, every issue marked AI-written.
3. **The show**: each kata's reference implementation is built by Claude while the grader
   catches its mistakes; screen recordings narrated by the owner's cloned voice, a short
   avatar intro, and a same-day write-up. The honest record of AI agents building
   distributed infrastructure, failures included.
4. **The subscription** (RFC 0009): free kata 1 and the Radar; paid levels, the live agent
   and the archive, billed by Stripe through InOrbit d.o.o.

**Where things live.**

| Piece | Home | Open? |
|---|---|---|
| `qp` (proxy, grader, submit, later agent) | `github.com/quietpager/qp`, Rust, Apache-2.0 | open |
| Kata specs, levels, reference Go and Rust | `github.com/quietpager/katas`, text CC-BY-SA, code Apache-2.0 | open |
| `radar` service | tbd, `crates/radar` | closed |
| `studio` service (catalogue, reports, progress, entitlements) | tbd, `crates/studio` | closed |
| Pages | inorbit.hr: `/radar/`, `/katas/`, this lab at `/lab/quietpager/` | public pages, draft RFCs |

The open repos stand alone: the pacer, histogram and report format are small and
re-implemented in `qp`, not imported from tbd, so a stranger can build and trust them.

**Placement.** The brand lives on inorbit.hr as a section with its own voice; the personal
pages keep their no-sales voice, and anything with a price sits only inside the Quiet Pager
section. quietpager.com, .dev and .ai (registered 2026-09-24) redirect to that section until
there is a reason to move out; moving out is a redirect change, not a rebuild.

**The avatar.** HeyGen (a private one-time training recording, never published) and an
ElevenLabs voice clone, the only verified good Croatian voice. Labelled on every video as
the EU AI Act (Article 50, in force since 2 August 2026) and the platforms require. Never
used for live coding, opinion, community replies, or anything a client sees as the owner.
Budget ceiling 100 USD a month; the stack costs about 53.

## Alternatives considered

**A separate site and company from day one.** Clean for an exit, costly now: InOrbit
issues the invoices either way, the platform would be called over the network, and traffic
would be split away from the owner's availability line. Rejected for now; the domains are
held so the move stays cheap.

**A generic "learn Rust and Go" course.** A crowded field with million-subscriber
incumbents. Rejected: the fault-graded angle is the one thing the owner's existing tools
make possible and others cannot copy quickly.

**Grading on the platform.** Running strangers' Rust and Go on the cluster needs Firecracker
or gVisor isolation, a security project of its own. Rejected for version one; the grade
runs on the learner's machine and the platform only receives the report, shown as a claim.

**Venture scale.** CodeCrafters' pause is the evidence that this market pays a bootstrapped
product, not a venture one. Planned as a bootstrapped product.

## Decision

Build the Radar and the grader in parallel (tracks A and B), then studio and accounts
(track C) once the report format is real, then subscriptions (track D) once RFC 0009's open
decisions are made. Everything open source starts in its own repo in the `quietpager`
organisation. This RFC stays a draft, readable by admins only, until the brand launches.

## Publication

Nothing changes on the public site until launch. Before then: the lab card and this RFC
are visible to admins only, and `/radar/` goes live when its first digest is good enough to
read. At launch: RFCs 0007 and 0008 are made public after a redaction pass, `/katas/`
opens, and quietpager.com's redirect goes live.

## Status log

- 2026-09-24: opened. Name, domains and the endorsed-brand placement decided; plan approved
  for tracks A (Radar) and B (grader) in parallel.
- 2026-09-24: the Radar is live on the site with its first issue (week 39). Its first
  batch after review adds a review gate (drafts until published), one block per change
  with a fixed impact category and a link among the week's items, a dated archive, and
  copy that leads with value.
