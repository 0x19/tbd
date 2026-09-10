# devops

Deployment for the two binaries, `engine` and `protocol`. Read `devops/README.md` first;
this file is the non-obvious part.

- `docker/Dockerfile` is one multi-stage build for both binaries, selected with
  `--build-arg BIN=engine|protocol` and `--build-arg PORT=`. cargo-chef caches
  dependencies; the runtime image is distroless with no shell, so there are no
  in-container health commands. Health is the orchestrator's job: gRPC probe on the
  engine, `/healthz` and `/readyz` on the protocol.
- The build copies only `Cargo.toml`, `Cargo.lock`, `crates/` and `proto/`
  (`.dockerignore` is an allowlist). A new top-level directory the build needs must be
  added there.
- Protos compile inside the image without `protoc` (protox). Do not add protoc to the
  builder.
- `k8s/` is kustomize: `base/` plus `overlays/dev` and `overlays/prod`. Image names are
  `ghcr.io/ORG/tbd-*`; `ORG` is a placeholder to replace, along with `newTag` in the
  prod overlay. Probes and security context (non-root, read-only rootfs, no caps) are
  in `base/`.
- `ansible/` runs from its own directory (`ansible.cfg` is there); the mise tasks set
  `dir` accordingly. `bootstrap.yml` once per host, `deploy.yml` per release with
  `-e image_tag=...`; it refuses `REPLACE_ME`. Secrets go in `group_vars/vault.yml`
  (ansible-vault), never in `all.yml`.
- Env vars the services read: `ENGINE_LISTEN_ADDR`, `PROTOCOL_LISTEN_ADDR`,
  `PROTOCOL_ENGINE_URL`, `RUST_LOG`, `LOG_FORMAT`. A new one must be added to
  `k8s/base/configmap.yaml`, `compose.yaml`, `.env.example` and the ansible compose
  template.

CI builds both images on every push and pushes on `main`; `release.yml` pushes on `v*`
tags. See `docs/ci.md`.
