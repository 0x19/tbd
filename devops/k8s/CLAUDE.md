# devops/k8s

Read `README.md` here first. Non-obvious facts:

- Apply order matters: `observability` before an app overlay, because `tbd-env` points
  `OTEL_EXPORTER_OTLP_ENDPOINT` at `otel-collector.observability.svc`.
- `engine` and `protocol` Services are headless on purpose (`clusterIP: None`). Envoy
  does the balancing. Giving them a ClusterIP silently moves balancing to kube-proxy.
- Never change ports on the in-cluster `envoy` Service; `PROTOCOL_ENGINE_URL` is
  `http://envoy:50051`. Host-facing ports go on `overlays/local/envoy-lb.yaml` and must
  match the `-p` flags in `mise.toml` `local:up` and `ansible/playbooks/local.yml`.
- `local-services.yaml` under `observability/` is applied with `-f`, not part of the
  kustomization, so other environments never get `LoadBalancer` Services.
- `observability/kustomization.yaml` pulls `../../grafana`; the dashboard ConfigMaps
  are generated there with hash suffixes and the Grafana Deployment's volume names are
  rewritten by kustomize. Do not hardcode a hashed name anywhere.
- Tempo 3 monolithic config: no `ingest`, no `block_builder`, no
  `metrics_generator.traces_storage`. A wrong key crash-loops the pod; the log names it.
  `local-blocks` is not a processor in 3.x either (Tempo logs it as unknown); TraceQL
  metrics for Traces Drilldown work without it.
- Grafana's anonymous role is Editor in `observability/grafana.yaml` for local use only.
- The OTel agent config is the only place that knows our JSON log shape
  (`span.trace_id` lifted to `trace_id`). Changing log fields in `tbd-common` may need
  a change there.
- Validate before applying: `kustomize build <dir> > /dev/null`, `mise run envoy:validate`,
  and YAML parse; there is no CI job that applies these to a cluster yet.
- `observability/ebpf/` (Alloy eBPF profiler) is a separate kustomization for real
  nodes. It cannot work on k3d or kind (node PID namespace is a container's), so never
  add it to the main observability kustomization. Locally, services profile themselves
  (`PYROSCOPE_SERVER_ADDRESS` in `base/configmap.yaml`).
- Profiles are only readable if release binaries keep symbols and frame pointers
  (`Cargo.toml` `[profile.release]`, `.cargo/config.toml` rustflags, and the Dockerfile
  copies `.cargo/`). A stripped build shows `[unknown]` frames.
- Service pods mount an `emptyDir` at `/tmp`: the in-process profiler creates temp
  files and the root filesystem is read-only. Removing it silently disables profiling.
