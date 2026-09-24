#!/usr/bin/env bash
# Installs gVisor's runsc on the host and registers it with Docker as the
# runtime `runsc`, for the sandbox (docs/sandbox/README.md, RFC 0010).
#
# - A pinned release, verified against its published SHA-512 before anything is
#   installed.
# - The runtime is registered with `--network=none` in its own arguments, so a
#   runsc container has no network even if a caller asked Docker for one: the
#   sandbox's recipe asks for none too, and either alone is enough.
# - Docker is reloaded, not restarted: a runtime is a reloadable setting, and a
#   restart would stop every running container (the cluster, the model engines).
#   The script checks the running containers are the same before and after.
#
# Idempotent: run it again to reinstall the same release or move to a new one.
set -euo pipefail

VERSION="${GVISOR_VERSION:-20260921.0}"
ARCH="$(uname -m)"
BASE="https://storage.googleapis.com/gvisor/releases/release/${VERSION}/${ARCH}"
DAEMON_JSON=/etc/docker/daemon.json

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
cd "$work"

echo "gvisor ${VERSION} (${ARCH}): downloading"
curl -fsSLO "${BASE}/gvisor.tar.bz2"
curl -fsSLO "${BASE}/gvisor.tar.bz2.sha512"
sha512sum -c gvisor.tar.bz2.sha512
tar -xjf gvisor.tar.bz2

# runsc, its containerd shim, and the helpers it runs beside it (the sentry and
# friends), which it looks for in `gvisor-bin/` next to itself.
for bin in runsc containerd-shim-runsc-v1; do
  [ -f "$bin" ] || { echo "missing $bin in the release" >&2; exit 1; }
  sudo install -o root -g root -m 0755 "$bin" "/usr/local/bin/$bin"
done
[ -d gvisor-bin ] || { echo "missing gvisor-bin/ in the release" >&2; exit 1; }
sudo install -d -o root -g root -m 0755 /usr/local/bin/gvisor-bin
for helper in gvisor-bin/*; do
  sudo install -o root -g root -m 0755 "$helper" "/usr/local/bin/$helper"
done
/usr/local/bin/runsc --version | head -1

before="$(docker ps -q | sort)"

# Merge the runtime into daemon.json, keeping everything else (the GPU runtime).
sudo cp -a "$DAEMON_JSON" "$DAEMON_JSON.bak.$(date +%Y%m%d%H%M%S)" 2>/dev/null || true
sudo python3 - "$DAEMON_JSON" <<'PY'
import json, os, sys
path = sys.argv[1]
cfg = json.load(open(path)) if os.path.exists(path) else {}
cfg.setdefault("runtimes", {})["runsc"] = {
    "path": "/usr/local/bin/runsc",
    "runtimeArgs": ["--network=none"],
}
tmp = path + ".tmp"
with open(tmp, "w") as f:
    json.dump(cfg, f, indent=4)
os.replace(tmp, path)
PY

sudo systemctl reload docker
for _ in $(seq 1 30); do
  docker info --format '{{json .Runtimes}}' | grep -q '"runsc"' && break
  sleep 1
done
docker info --format '{{json .Runtimes}}' | grep -q '"runsc"' || { echo "docker did not pick up runsc" >&2; exit 1; }

after="$(docker ps -q | sort)"
if [ "$before" != "$after" ]; then
  echo "the running containers changed across the reload" >&2
  exit 1
fi
echo "runsc registered; the $(echo "$after" | grep -c .) running containers are the same ones"
