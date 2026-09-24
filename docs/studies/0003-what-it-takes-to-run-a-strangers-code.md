---
title: What it takes to run a stranger's code
status: measured
date: 2026-09-24
public: true
lab: llm
summary: A hostile program for every way out we could name, run in the sandbox of RFC 0010, and what stopped each one; how long a run takes cold, and what surprised us about how a sandbox dies.
rfc: 0010-the-sandbox
headline: 16 of 16
headline_note: hostile programs contained, each by the defence meant to stop it
---

## The question

Can the platform run a program anyone wrote, in Go or Rust, on the machine that also
serves the models and the site, and give back what it printed without the program
reaching anything else? What does each defence cost in time, and does each one actually
stop what it is meant to stop, or does something behind it do the work?

## What we believed

That the network would be the hard part and the compiler the easy one. That a program
that ate its memory would be killed by the kernel inside the sandbox and say so. That a
fork bomb would hit the process limit and fail politely. That compiling Go in a
read-only sandbox would be a matter of seconds, not tens of them.

## What we built

The sandbox of RFC 0010: one throwaway container per run whose kernel is gVisor, with
no network, a read-only image, a small scratch space in memory, no privileges, and caps
on memory, processors, processes, file size and output, every one enforced from outside.
The daemon that starts it takes only a language, a source and an input.

Then the escape suite: one test per way out, each a real program run in a real sandbox
on the machine, asserting it was contained and printing what stopped it and how long it
took. The suite runs with `mise run sandbox:escape`, and any change to the sandbox's
recipe has to pass it again.

## What we measured

| Hostile program | What stopped it | Time |
|---|---|---|
| Connect to the internet, the machine, the platform's services, loopback; resolve a name | no network interface but loopback, and none reachable: every attempt refused, no name resolved | 1.7 s |
| Read the machine: its hostname, its processes, its container runtime | its own hostname, a handful of its own processes, no runtime socket | 1.6 s |
| Write outside the scratch space: the system, the toolchain, the build cache | read-only everywhere else | (same run) |
| Mount, make a namespace, trace, load an eBPF program, become root | each refused | 1.7 s |
| Fork bomb | the process limit, then the deadline; the next run fine | 6.1 s |
| Allocate past the memory limit | the whole sandbox ended at its limit | 1.6 s |
| Fill the disk | the file-size cap | 1.1 s |
| Spin forever | the run deadline | 5.9 s |
| Sleep forever | the run deadline | 5.9 s |
| Print forever, markup included | the output cap, then stopped | 1.1 s |
| Leave a file for the next run | nothing survives a sandbox | 1.0 s |
| A build that asks for a code generator | the build runs no generator | 1.6 s |
| A program that asks for C | refused at compile time | 0.6 s |
| A compile-time loop | the compiler's own limit, before the deadline | 1.8 s |
| Anything left behind on the machine | no container left after the suite | 1.1 s |
| The wrong key | refused before anything starts | 0.01 s |

A cold run, container start to container removal, five of each through the deployed
daemon:

| Language | Compile, median | Run, median | Whole run, median | Whole run, worst |
|---|---|---|---|---|
| Go | 0.97 s | 52 ms | 1.5 s | 2.9 s |
| Rust | 0.43 s | 54 ms | 0.93 s | 0.97 s |

Four runs at once take about 1.3 seconds each; a fifth is refused at once, and waiting
for a turn is the runner's job, not the sandbox's.

[REDACTED: the limits themselves and where the daemon listens]

## What surprised us

**The first cold Go build took twenty-four seconds, and then failed.** Go compiles its
standard library into a cache on first use; in a fresh read-only sandbox with one
processor that is most of the time, and its temporary files overflowed the small
temporary space. Building the standard library once into the image, as a cache the
sandbox reads but cannot write, took the build to under a second.

**A sandbox dies whole.** We expected a program that ate its memory to be killed inside
the sandbox and exit with the kernel's signal. Under gVisor the memory limit is the
sandbox's, not the program's: past it the whole sandbox is ended from outside, and the
runtime's client reports that in its own terms, with its own identifiers. The fork bomb
did the same once the process limit held it. Both are what the defence is for; the
daemon now names them ("the sandbox reached one of its limits") instead of passing on
the runtime's message.

**The file-size cap is a signal, not an error.** A program writing past it is killed by
the kernel's file-size signal before it sees a write fail; the daemon names that too.

**Registering the runtime needed no restart.** Adding a container runtime is a setting
the container engine reloads; the cluster and the model engines ran through it. That
was the difference between a routine change and an outage.

## What we would do differently

Write the escape suite before the daemon: two of its first answers were right for the
wrong reason, and only the suite said so. Measure a cold build in the real sandbox before
choosing the image, not after.

## Next

Firecracker against gVisor on this machine: the same suite, the same timings, and the
cost of a virtual machine per run. A warm pool only if the runner's users feel the cold
start. A vetted, offline set of packages. The longer-lived shape for a program that
serves a port, which a hosted grader would need.

## Glossary

- **gVisor**: a kernel written in a memory-safe language that runs in user space and
  answers a sandboxed program's system calls itself, so the program never talks to the
  machine's kernel.
- **Escape suite**: hostile programs, one per way out, each a test that fails if the
  program gets out.
- **Cold run**: a run in a sandbox created for it and removed after, with nothing kept
  warm.
- **Fork bomb**: a program that starts copies of itself until something stops it.
