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
- The OTel agent config is the only place that knows our JSON log shape
  (`span.trace_id` lifted to `trace_id`). Changing log fields in `tbd-common` may need
  a change there.
- Validate before applying: `kustomize build <dir> > /dev/null`, `mise run envoy:validate`,
  and YAML parse; there is no CI job that applies these to a cluster yet.
