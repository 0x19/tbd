# Profiling

Continuous CPU profiles of the running services, viewed as flame graphs over time and
linked from traces. Stored in Pyroscope; two ways to collect them, and they coexist.

## In-process (always available)

Every Rust service profiles itself with `pprof-rs` when `PYROSCOPE_SERVER_ADDRESS` is
set: 100 Hz SIGPROF sampling, uploaded every ten seconds, tagged `service_name` and
`version`. Off when the variable is unset; a failure to reach Pyroscope logs a warning
and the service runs on. Code: `crates/common/src/profiling.rs`, started from each
binary's `main` after telemetry.

This is what the local cluster uses, and it gives exact Rust frames anywhere the
service runs, including compose and bare processes on the host.

## eBPF (real nodes)

`devops/k8s/observability/ebpf/` deploys Grafana Alloy as one privileged pod per node
with the `pyroscope.ebpf` component. It samples every process in the `tbd` and
`observability` namespaces at 97 Hz with no code changes, so Envoy and the observability
components are profiled too. Labels: `service_name` = container name, `namespace`,
`pod`, `container`, `node`.

```sh
kubectl apply -k devops/k8s/observability/ebpf
```

It is not part of the main observability kustomization because it cannot work on k3d or
kind: there the "node" is a container, `hostPID` yields that container's PID namespace
(PID 1 is `docker-init`, some fifty processes instead of a thousand), and the profiler's
kernel self-check never sees its own PID. Real machines or VMs as nodes are fine, and
the manifest is ready for them. The nodes also need tracefs; the local cluster mounts it
anyway so the manifest is testable the moment nodes are real.

## Symbols

Release binaries keep symbols and line tables (`[profile.release]` in `Cargo.toml`) and
are built with frame pointers (`.cargo/config.toml`), so both profilers show
`tbd_engine::…` and `tbd_protocol::…` frames rather than addresses. Cost: tens of
megabytes per image, about one percent of CPU.

## Viewing profiles

| Where | What |
|---|---|
| Grafana → Drilldown → Profiles, `/a/grafana-pyroscope-app/explore` | services ranked by CPU, flame graph per service, diff between two time ranges, top functions |
| Pyroscope's UI, http://localhost:4040 (`192.168.178.21:4040` from the LAN) | the same data in Pyroscope's own explorer |
| From a trace span | "Profiles for this span" opens the CPU flame graph of that service around the span's time |

Profile type: `process_cpu:cpu:nanoseconds:cpu:nanoseconds` from both collectors.
Memory profiles are not collected yet; the in-process agent can add them with a
jemalloc-based backend when there is a reason to.

## Reading a flame graph

Width is time on CPU. Look for wide frames under your own functions
(`tbd_engine::…`, `tbd_protocol::…`) rather than under tokio's scheduler or the
allocator; those are the cost of your code. A wide `memcpy` or serialisation frame under
a handler is the usual first find. Compare "before" and "after" ranges with the diff
view when a deploy moved a latency percentile.

## Configuration

| Setting | Where | Default | Notes |
|---|---|---|---|
| in-process on/off | `PYROSCOPE_SERVER_ADDRESS` | unset locally on the host; set in the cluster ConfigMap | one variable, no rebuild |
| in-process rate | `SAMPLE_RATE_HZ` in `profiling.rs` | 100 Hz | uploads every 10 s |
| eBPF rate | `ebpf/alloy-profiles.yaml`, `sample_rate` | 97 Hz | prime, to avoid aliasing with periodic work |
| eBPF interval | `collect_interval` | 15 s | one profile per target per interval |
| eBPF namespaces | the `keep` relabel rule | `tbd`, `observability` | widen to profile more |
| retention | Pyroscope defaults, filesystem backend on a 50 GiB PVC | | object storage in production |

## Cost

About 1 % CPU on profiled processes from frame pointers, the Alloy pod's own sampling
overhead, and tens of megabytes per image for symbols. Nothing at request latency.

Memory is the real price of the in-process agent, and it is paid per process. Measured
on 2026-09-11 with the release `ledger` binary on the memory store, idle:

| | resident after 5 s | after 15 s and steady |
|---|---|---|
| `PYROSCOPE_SERVER_ADDRESS` unset | 9 MB | 9 MB |
| set (in-process `pprof-rs`, 100 Hz) | 221 MB | 287 MB |

The agent symbolises its samples from the binary's line tables, and that mapping is what
the 280 MB is; every service in the cluster idles between 170 and 290 MB for the same
reason. The pod limits (`devops/k8s/base/*/deployment.yaml`, 512Mi) are set with that
in, because the first load run against a 256Mi ledger ended in `OOMKilled`. On real
nodes the eBPF profiler covers every container without any of it, so unsetting
`PYROSCOPE_SERVER_ADDRESS` there gives the memory back; in k3d, where eBPF cannot run,
the in-process agent is the only source of profiles and the memory is the fee.

## Troubleshooting

| Symptom | Look at |
|---|---|
| no services in Drilldown | in the cluster: `PYROSCOPE_SERVER_ADDRESS` in the ConfigMap and `in-process profiling on` in the service log; on real nodes with eBPF: `kubectl -n observability logs ds/alloy-profiles` |
| service logs `in-process profiling disabled ... create profiler error` | `pprof-rs` needs a writable temp directory; the pods mount an `emptyDir` at `/tmp` for exactly this (root filesystem stays read-only) |
| profiles exist but are nearly empty | the process is idle; profiles only contain CPU time actually spent. `mise run local:load` drives about 700 req/s through Envoy and yields several CPU-seconds per service per minute. |
| eBPF says `system analysis request was not handled` | the nodes are containers (k3d, kind); use in-process profiling there |
| flame graph is all `[unknown]` frames for a service | the binary is stripped or built without frame pointers; check the release profile and rebuild with `mise run local:restart` |
| a service is missing | its namespace is not in the `keep` rule, or the pod is not on a node with a running Alloy |
| trace → profile shows nothing | the span's `service.name` must equal the container name; the link uses `service_name` |
