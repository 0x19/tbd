---
title: The sandbox
status: open
date: 2026-09-24
public: true
lab: llm
summary: Running a stranger's Go or Rust program on the platform's own machine and giving back what it printed, with every way it could do harm named first and closed by a layer that does not trust the one before it.
---

## Problem

The workbench's models write code, and the next thing anyone wants is to run it. Running
a program someone else wrote, on the machine that also serves the platform, is the most
dangerous thing the platform could offer: a program can do anything the machine lets it
do, and a model will happily write one that tries. The same need exists elsewhere in the
lab: grading a learner's program on the platform rather than on their own machine was
deferred as "a sandboxing project of its own" (RFC 0008). This is that project.

## What a hostile program could try

Every one of these is a test in the escape suite before anything is offered, and every
defence below names which of them it stops.

1. **Reach a network:** the internet, the machine, or the platform's own services.
2. **Read or write the machine:** its files, its other programs' memory, its devices.
3. **Exhaust the machine:** memory, processors, processes, disk, open files.
4. **Outlive its turn:** run forever, sleep forever, leave a child behind.
5. **See another run:** anything left by an earlier program, anything of a concurrent one.
6. **Attack the compiler:** code that is slow or huge to compile, or that asks the build
   to run something before the program itself does.
7. **Flood the answer:** endless output, or output shaped to break the page that shows it.

## Proposal

**Two layers, the pattern the model service already uses.** The engine is a small daemon
on the machine, the only thing that starts a sandbox. The service in front of it, the
runner, is a platform service like any other: it knows who is asking, how many runs are
in flight, what each caller has left today, and it writes the audit line. A request says
three things (the language, the source, the input) and nothing in it ever becomes an
option of the sandbox.

**One throwaway sandbox per run**, compiling and then running inside the same one, so
no compiled program ever crosses back to the machine. It is a container whose kernel is
gVisor: a kernel written in a memory-safe language that runs in user space and answers
the program's system calls itself, so the program never talks to the machine's kernel
directly.

| Defence | Stops |
|---|---|
| gVisor's own kernel between the program and the machine's | 2, 4, and the kernel attacks behind them |
| No network at all: none requested, and the runtime configured to refuse one even if asked | 1 |
| The image read-only; the only writable place a small in-memory scratch space, gone with the sandbox | 2, 5 |
| An unprivileged user, every capability dropped, no way to gain one | 2 |
| Limits on memory, processors, processes and file size | 3 |
| A deadline for compiling and one for running, enforced from outside by removing the sandbox | 4, 6 |
| Standard library only: no packages fetched, no Cargo (so no build scripts or macros from elsewhere), no C | 6, 1 |
| Output capped per stream; the page renders it as text, never as markup | 7 |
| The runner: a caller required, a bounded number at once, a daily allowance per caller | 3 at the platform's scale |
| One audit line per run: who, which language, how big, how it ended, never the code or its output | the record SOC 2-style controls ask for |

[REDACTED: the exact limits, the daemon's address and how the runner proves itself to it]

**Who may run.** Admins while the lab is private (RFC 0006); a signed-in visitor with a
daily allowance when it is published. Agents over MCP do not get the tool: running code
on behalf of an agent is a separate decision, made later and on purpose.

## Alternatives considered

**Firecracker microVMs.** Each run in its own virtual machine on the processor's
virtualisation: a stronger wall than gVisor's, and what large hosted sandboxes use. More
to build and keep (a kernel, a root filesystem per language, the jailer that confines
the VM monitor itself, which had a file-overwrite flaw of its own early in 2026). Not
first; a study compares the two on this machine, and Firecracker replaces gVisor if the
cost is small.

**Namespaces and seccomp alone** (bubblewrap, a plain container). Much cheaper, and the
program talks to the machine's real kernel, so one kernel bug is a way out. Rejected as
the only wall; it is what gVisor adds a second one to.

**WebAssembly.** Both languages compile to it, and a WebAssembly runtime is a strong,
small sandbox. Rejected for now: the program the page runs would not be the program the
learner would ship, the compiler itself still has to run somewhere, and Go's support is
the newest of its targets.

## Decision

Open. Fixed so far: two layers, the engine on the machine and the runner in the
platform; one gVisor sandbox per run for compiling and running; no network; the
standard library only; every limit enforced from outside the sandbox; admins only while
the lab is private; no MCP tool. Still to decide: Firecracker after its study; a vetted,
offline set of packages; a longer-lived shape for programs that serve a port (the
hosted grader of RFC 0008), which is a different sandbox and a later RFC.

## Publication

The workbench runs Go and Rust code blocks and shows what they printed. A study, "What
it takes to run a stranger's code", reports the escape suite's results, how long a run
takes cold, and what a run costs the machine.

## Status log

- 2026-09-24: opened. The kernel is installed on the machine: a sandbox sees gVisor, not
  the machine's kernel, and has no network even when one is asked for.
- 2026-09-24: the engine runs. The daemon compiles and runs a Go or Rust program in one
  throwaway sandbox in one to two seconds cold, and the escape suite contains every
  hostile program it has (study 0003). A sandbox that reaches its memory or process
  limit is ended whole, and the daemon says so in words.
- 2026-09-24: the runner is the platform's way in: a verified caller with the role, the
  bounds, a daily allowance, a bounded queue, then the sandbox; one audit line per run
  without its code, and a test that fails if the code ever reaches the log.
- 2026-09-24: the workbench runs Go and Rust code blocks through it, as an admin; the
  page's "what this page sends" says the code is run once and not kept.
