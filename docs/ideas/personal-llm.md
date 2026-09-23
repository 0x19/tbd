# A personal LLM stack, built low-level

> Status: idea, 2026-09-23. Not built. The lab behind [claim-confidence.md](claim-confidence.md),
> and a set of engineering case studies for the site once there are numbers.

## The goal

A model that knows the owner's work: the code, the design documents, the RFCs, the CV and
how they fit together, so it can answer "why is this system built like this" the way the
owner would, and eventually interview the owner about it. Built from open-weight models,
served from this machine, and built low enough that the serving stack itself is the
learning: inference serving, batching, KV caching, embeddings, vector search, an eval
pipeline and tracing, all wired into the observability this repository already has.

## What it may learn from

- This repository: the code, `ARCHITECTURE.md`, the crate notes, `docs/`, the plans and
  the design notes.
- The owner's own repositories, public and private, under their own accounts and the
  products they built themselves.
- The public CV (`ui/www/src/data/site.ts`) and the writing on the site.

**Never a client's repository, current or former.** No source, no transcripts, no
tickets. The public site already says where the owner worked and what they did there in
one paragraph; that paragraph is the whole of what the model may know about it.

## Phases

1. **Model lab.** Get the GPU working (see below), pick two or three open-weight models
   that fit the card, and measure them on this machine: tokens per second at batch one and
   at batch eight, time to first token, memory at each quantisation. Serve them behind a
   gRPC service in this workspace so every later phase has one stable API and a trace per
   request. Baselines from existing runtimes first (Ollama is installed; llama.cpp and
   vLLM are the other two to compare), the owner's own serving path second.
2. **Understand my code.** Parse the allowed repositories with tree-sitter per language
   into symbols, call and dependency graphs, and chunks that respect scope. Read the commit
   history for how each design moved. Embed the chunks and the documents; store them next
   to the existing Postgres (`pgvector`) unless the benchmarks say a dedicated engine is
   needed. Questions the eval set must answer: "where is the retry budget for the engine
   LB decided", "why is the ledger on ClickHouse and Postgres both", with the file and
   the reasoning.
3. **Understand me.** A skill graph built from what the code and the documents show the
   owner actually did: languages, systems, the decisions and their consequences. This is
   the Candidate Engineering Model from the product idea, built once for one person.
4. **The interviewer.** An adaptive, adversarial Staff-engineer interviewer over phases 2
   and 3, with speech to text and text to speech so a session is spoken. Every question
   cites the evidence it was built from; every judgment is logged for the eval set.
5. **Fine-tuning.** Only once the RAG baseline has been measured. LoRA or SFT on the
   owner's writing and decisions, evaluated against the same set, to see what retrieval
   cannot give (voice, habits, the shape of an answer) and what it can.
6. **Product.** If phases 1 to 5 hold up on the owner, generalise to one authorised
   GitHub account per candidate: [claim-confidence.md](claim-confidence.md).

## RAG or fine-tuning

Start with retrieval. It is cheaper, it is inspectable (every answer names its sources),
and it keeps the model's knowledge of the code current with the code. Fine-tune when the
eval set shows a gap retrieval cannot close. Publish the comparison either way; that
comparison is one of the case studies.

## The stack, deliberately low-level

| Layer | First baseline | Then build |
|---|---|---|
| Inference | Ollama (installed), llama.cpp, vLLM | a serving path of our own with continuous batching and a KV cache we can measure |
| Embeddings | an open embedding model behind the same API | chunking that follows the AST, not the line count |
| Search | `pgvector` in the existing Postgres | hybrid: vectors plus the symbol graph |
| Evals | a fixed question set with graded answers, run in CI like the chaos checks | regression on every model or prompt change |
| Tracing | the OTel, Tempo and VictoriaMetrics stack already here | one trace from question to retrieved chunks to tokens |

If any of this becomes a service it goes in through `mise run tbd -- new service <name>`
and follows every convention in the root `CLAUDE.md`: config in `configs/`, a span and a
`RequestTimer` per request, metric names registered, no business logic in the protocol.

## Case studies this would produce

- Can an LLM tell whether an engineer understands their own code?
- Teaching an LLM twenty years of one engineer's work.
- RAG versus fine-tuning for source-code understanding, measured.
- Building an adversarial Staff-engineer interviewer.
- Serving open-weight models from one workstation: what a 16 GB card really does.

## Hardware at hand, surveyed 2026-09-23

| Part | What is here |
|---|---|
| CPU | AMD Ryzen Threadripper 3960X, 24 cores, 48 threads, AVX2 (no AVX-512) |
| RAM | 247 GiB, 8 GiB swap |
| GPU | NVIDIA GeForce RTX 4070 Ti SUPER, 16 GB VRAM; driver 575-open installed, module missing for the running kernel |
| CUDA | toolkit 12.9 at `/usr/local/cuda`; Docker has the `nvidia` runtime; the k3d nodes advertise no GPU |
| Disks | `/mnt/raid0`: 4x 4 TB NVMe RAID 0, 14 TB free, about 1.4 GB/s write and 2.8 GB/s read; `/mnt/development`: 355 GB free; `/`: 81 GB free |
| Network | 1 Gbit uplink; a 7 GB model file takes about a minute to pull |
| Runtimes | Ollama 0.24.0 as a service with `qwen3-vl:8b` pulled; Python 3.13 without torch or uv; Rust 1.98; Node 24 |

**The GPU was dark until 2026-09-24.** The kernel had been upgraded to 6.14.0-37 in May
2026 and no prebuilt NVIDIA module followed for the 575 series, so Ollama ran on the
CPU. The fix that worked, with no reboot: the 580 open driver from NVIDIA's own
repository through DKMS, which builds the module for whatever kernel runs. A native
llama.cpp build fails on this Ubuntu (CUDA 12.9 headers against glibc 2.41); the
project's CUDA container serves the deep tier instead.

**What 16 GB means.** Weights at 4-bit quantisation take roughly half a gigabyte per
billion parameters, so models up to about 14B fit on the card with room for the KV cache;
30B-class models spill into system RAM and run at CPU speed. QLoRA fine-tuning of a 7B
to 8B model fits. Everything larger is a benchmark of the CPU and the 247 GiB of RAM, not
of the GPU. These are rules of thumb to be replaced by phase 1 measurements.
