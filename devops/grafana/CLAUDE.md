# devops/grafana

- Dashboards are JSON files here and read-only in the UI; edit the file, then
  `mise run grafana:reload`. Keep `uid`, `schemaVersion: 39`, tag `tbd`, and the
  cross-links block found in every dashboard.
- Datasource references must be `{ "type": ..., "uid": "victoriametrics" | "victorialogs" | "tempo" }`.
  Any other UID renders "datasource not found" for everyone.
- Metric names come from `crates/common/src/metrics.rs` `names`; label names are
  exactly `service`, `transport`, `route`, `status`, `kind`, `direction`, `version`.
  Envoy metrics use `envoy_cluster_name` and `envoy_http_conn_manager_prefix`.
- VictoriaLogs panels put the LogsQL in `expr`. Service filter is `service.name:<name>`.
- A new dashboard file must be added to `kustomization.yaml`'s `grafana-dashboards`
  list and to the tables in `README.md` and `docs/observability/dashboards.md`.
- Validate JSON with python before reloading; a broken file makes the provider skip it
  silently.
