# devops/ansible

- Run playbooks from this directory; `ansible.cfg` sets `inventory`, `roles_path` and
  the output format. The mise tasks set `dir` accordingly.
- `ansible-core` is installed by mise from PyPI via uv (`pipx:ansible-core`);
  collections come from `requirements.yml`. The `yaml` stdout callback is not available
  with core alone; `ansible.cfg` uses `default` with `result_format = yaml`.
- `playbooks/local.yml` targets `localhost` and shells out to `k3d`, `docker` and
  `kubectl`, so it needs mise's tools on `PATH` (`mise run ansible:local` or
  `mise exec -- ansible-playbook ...`). Its k3s image, ports and storage path must match
  `mise.toml` `local:up`.
- `deploy.yml` refuses `image_tag=REPLACE_ME` and renders Envoy in front of the services
  from `roles/tbd_app/templates/compose.yaml.j2`; env vars added to the services must be
  added to that template.
- Secrets only in `group_vars/vault.yml` (ansible-vault); never in `all.yml`.
