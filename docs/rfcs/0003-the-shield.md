---
title: The shield
status: open
date: 2026-09-24
public: true
lab: llm
summary: An eBPF program in front of the edge that counts and limits traffic per source before the kernel builds a socket for it, written in Rust, measured under a storm the chaos tool raises inside the machine, and honest about what a hook on every packet costs.
---

## Problem

Everything the platform serves comes in through one edge: a proxy that terminates
TLS, a gateway that verifies who is asking, and the services behind them. Each of those
limits abuse in its own way, and each of them does it after the kernel has already
accepted the packet, built the connection and handed it to a process. A flood that never
intends to complete a request still costs a socket, a handshake and a place in a queue
at every layer before anything says no.

The kernel can say no earlier. eBPF lets a small, verified program run at the network
driver (XDP), before the kernel allocates anything for the packet, and decide there:
pass, drop, or count. That is where volumetric abuse is cheapest to refuse. It is also
where a mistake is most expensive, because the program runs for every packet the machine
receives, wanted or not.

## Proposal

**The program.** An XDP program in Rust, built with aya, attached to the interface the
edge listens on. For every packet it keeps, in kernel maps:

- per source address, a token bucket (a rate and a burst) and a count of what passed and
  what was dropped, in an LRU map so a spoofed flood cannot fill it and stay;
- per verdict and per protocol, per-CPU counters, so counting never contends;
- an allow list that is never limited (the machine's own and the operator's addresses)
  and a deny list that is always dropped.

The rate, the burst and the lists are written from user space; the program only reads
them. Anything it does not recognise, it passes: the shield limits, it does not parse.

**The service.** A `shield` service of this platform loads the program, owns its
maps, exports the counters as metrics beside every other service's, and answers a small
contract: the current counters, the top talkers, the limits, and a change to the limits
from an operator. It is the only thing on the machine that touches the program.

**The storm.** A chaos scenario raises a storm against the shield inside the machine: a
network namespace joined to the host by a virtual pair, the program attached to the
host's end, and a generator in the namespace sending at rates well past the limits from
a few sources and from many. Nothing crosses a real network. The scenario asserts that
the edge behind the shield stays within its latency budget while the storm runs, and
that the counters say what was dropped and why.

**The honest half.** A hook that runs for every packet is not free, and whether it pays
depends on what it replaces. So the study measures the program's own cost: throughput
and latency of legitimate traffic through the edge with no program, with the program
passing everything, and with the program limiting, at the same offered load. A second
probe, a traffic-control or kernel-probe program that times requests through the edge
per flow, is measured the same way, and the study says plainly where a hook earns its
place and where it would only slow the machine down.

## Alternatives considered

**Rate limiting in the proxy alone.** The proxy already limits per source. Kept, and
not replaced: it limits requests, the shield limits packets, and a request-level limit
cannot stop what never becomes a request.

**nftables rules.** Simpler, no program to verify, and good at static lists. Rejected
as the main mechanism because per-source buckets with a burst are awkward to express
and to observe there; the study uses nftables as a baseline to beat.

**eBPF in C with libbpf.** The reference toolchain and the widest set of examples.
Rejected in favour of Rust and aya so that the program, its loader and the service share
one language and one build, which is the platform's rule everywhere else.

## Decision

Open. Fixed so far: XDP in Rust with aya, per-source token buckets in an LRU map,
per-CPU counters, lists written from user space, pass on anything unrecognised; a
`shield` service that alone owns the program; the storm raised inside the machine in a
network namespace; the program's own cost measured and published. Still to decide: the
default rate and burst (from the study), whether the shield sits on the physical
interface or only on the edge's
([REDACTED: which interface, and the addresses on the allow list]), and a later step:
routing at the same hook, a layer-4 balancer in front of the services, as its own study.

## Publication

The workbench (RFC 0004) shows the shield's counters live, and a signed-in visitor can
start the storm scenario and watch the drops rise and the edge's latency hold. Study 0003
publishes the storm and the cost.

## Status log

- 2026-09-24: opened.
