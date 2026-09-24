# The sandbox

Runs one Go or Rust program, one file, standard library only, in one throwaway gVisor
container, and says what it printed (RFC 0010). Two layers, like the llm service:

- **`sandboxd`** (L1, `crates/sandboxd`): a daemon on the host, the only thing that starts
  a sandbox. `POST /run` with `{language, source, stdin}` and a bearer token; the
  answer is `{id, language, outcome, compile, run, total_ms}`, each step
  `{exit_code, stdout, stderr, truncated, wall_ms, killed}`.
- **The runner** (L2, a platform service): who may run, how many at once, how many a
  day, the audit line; it calls `sandboxd` as the llm service calls an engine.

## The runner

`RunnerService` (`proto/tbd/runner/v1/runner.proto`, `crates/runner`): `Run` at
`POST /v1/runner/run`, `ListLanguages` at `GET /v1/runner/languages`, over every surface
the gateway offers. The MCP allowlist does not include it: running code for an agent is
a later decision (RFC 0010). A run, in order:

1. a verified caller, always (`UNAUTHENTICATED` without one), with `[access]
   require_role` (`admin` while the lab is private; `PERMISSION_DENIED` otherwise);
2. the bounds (`[limits]`; `INVALID_ARGUMENT` naming the field; nothing is spent);
3. the caller's daily allowance (`[budget] runs_per_day`, in memory, so a restart
   resets it; `RESOURCE_EXHAUSTED` with the reset time);
4. a slot (`[admission]`: 4 at once, 8 waiting, 10 s; `RESOURCE_EXHAUSTED` "busy: ..."),
   matching `sandboxd`'s own limit so the daemon's refusal is never reached;
5. the engine: `sandboxd` over HTTP with the token (`RUNNER_SANDBOX_URL`,
   `RUNNER_SANDBOX_TOKEN` from the `runner-sandbox` Secret), or `stub` (tests and the
   chaos tool; runs nothing, `stub: true`; refused in production). A daemon that refuses
   the token is `UNAVAILABLE` to the caller, never more.

One `audit` line per run: subject, caller kind, language, source bytes, outcome, time;
a test fails if the source ever reaches the log. Metrics `tbd_runner_runs_total`,
`tbd_runner_duration_seconds`, `tbd_runner_in_flight`. On the site's host,
`/v1/runner/` has the lab's gate (`docs/auth/README.md`).

## One run in the sandbox

1. `docker run --runtime runsc` with the recipe's fixed restrictions
   (`crates/sandboxd/src/recipe.rs`): no network (and the runtime is registered with
   `--network=none`, so none even if asked), no IPC, a read-only image, a scratch space
   in memory at `/work` (the only writable place, `[limits] scratch`), a 16 MB `/tmp`
   that cannot execute, user 65534, every capability dropped, `no-new-privileges`,
   memory with swap, processors and processes capped, a file-size cap, no core dumps,
   no container logs. Its only process sleeps; a container the daemon lost still ends.
2. The source written into `/work` on standard input (`docker exec -i … cat`).
3. The compiler (`go build` offline with `CGO_ENABLED=0` and a read-only cache of the
   standard library built into the image; `rustc` directly, never Cargo), with
   `[limits] compile_timeout`.
4. The program, with its input on standard input and `[limits] run_timeout`.
5. The container removed, whatever happened (also when the caller went away).

Nothing from a request becomes an option or a word of a command: the language picks a
row of a fixed table, the source and the input only travel on standard input.

`killed` says why a step was stopped from outside: `timeout` (its deadline; the daemon
removes the container), `output` (a stream passed `[limits] max_output_bytes`; cut and
removed), `memory` (killed by the kernel in the sandbox), `file_size` (a file past the
cap), `limit` (the whole sandbox was ended at its memory or process limit; the step's
stderr says so in words, not in the runtime's own terms). `outcome` is `ok`, `exit`
(non-zero), `compile_error` or `killed`.

## Running it

| Task | Does |
|---|---|
| `mise run sandbox:install` | gVisor from its pinned release (SHA-512 checked) into `/usr/local/bin`, registered in `/etc/docker/daemon.json` as `runsc`; Docker is **reloaded**, never restarted, and the task fails if a running container changed |
| `mise run sandbox:images` | `tbd-sandbox-go:dev`, `tbd-sandbox-rust:dev` from `devops/docker/sandbox-*.Dockerfile`, pinned by digest, built locally, never pushed |
| `mise run sandbox:secrets` | the token: a root-only systemd credential (`/etc/credstore/sandboxd.token`) and the `runner-sandbox` Secret in `tbd`, the same value; `ROTATE=1` makes a new one |
| `mise run sandbox:deploy` | builds `sandboxd`, installs it with `configs/sandboxd` into `/etc/tbd/sandboxd` and the unit `devops/sandbox/sandboxd.service`, starts it |
| `mise run sandbox:test` | every test on real sandboxes, the escape suite included |
| `mise run sandbox:escape` | the escape suite alone, one line per hostile program: study 0003's table |

The unit runs `sandboxd` with a new user each start (`DynamicUser`) in the `docker`
group, no capabilities, a read-only system, and systemd's firewall letting in only this
machine and the local cluster's networks (`172.16.0.0/12`); it listens on
`0.0.0.0:7788` behind that; its metrics on `127.0.0.1:9490`. Pods reach it as `host.k3d.internal:7788`. The token is the
second check, compared in constant time; a token shorter than 32 bytes refuses to
start.

**The one deviation.** The platform keeps secrets only in Kubernetes Secrets; the host
side of this token is a root-only systemd credential, because the daemon is not in the
cluster. `sandbox:secrets` writes both from one value.

## Bounds (`configs/sandboxd/base.toml`)

64 KiB of source, 64 KiB of input, 64 KiB of output per stream, 20 s to compile, 5 s to
run, 4 runs at once (a fifth is `429` at once: queueing is the runner's job), 512 MB of
memory with swap, one processor, 128 processes, a 256 MB scratch space, 64 MB files.

## The escape suite (`crates/sandboxd/tests/it/escape.rs`)

Every case is a test that fails if the program gets out: no network (the internet, the
host, the cluster, loopback, name resolution), the host invisible (own hostname, a
handful of processes, read-only outside `/work`, no Docker socket), privileged calls
refused (`mount`, `unshare`, `ptrace`, `bpf`, `setuid`), a fork bomb and memory past the
limit ended with the next run unaffected, the disk cap, spinning and sleeping ended at
the deadline, endless output cut, nothing left between runs, `go:generate` and cgo
running nothing, a compile-time loop ending, and no container left behind.

Metrics: `tbd_sandbox_runs_total`, `tbd_sandbox_duration_seconds`,
`tbd_sandbox_in_flight` (docs/observability/metrics.md). Each run is one `audit` line:
id, language, outcome, time; never the source or the output.
