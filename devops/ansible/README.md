# Ansible

Three playbooks, run from this directory. `ansible-core` comes from mise
(`mise run setup`); the collections from `requirements.yml`.

| Playbook | When | Command |
|---|---|---|
| `playbooks/local.yml` | bring up or refresh the local k3d cluster on this machine | `ansible-playbook playbooks/local.yml [-e rebuild=false]` or `mise run ansible:local` |
| `playbooks/bootstrap.yml` | once per new host | `ansible-playbook -i inventory/dev.yml playbooks/bootstrap.yml` |
| `playbooks/deploy.yml` | every release | `ansible-playbook -i inventory/prod.yml playbooks/deploy.yml -e image_tag=v1.2.3 --ask-vault-pass` |

`local.yml` runs against `localhost`: creates the k3d cluster if missing (same image,
ports and storage as `mise run local:up`), builds and imports the images unless
`rebuild=false`, applies the observability stack, the host-facing services and the
`local` overlay, rolls the app onto rebuilt images, waits for every rollout, and prints the
URLs. It is idempotent; re-run it after code changes. The compose-based deploy for real
hosts renders Envoy in front of the services from `roles/tbd_app/templates/compose.yaml.j2`.

Bootstrap installs base packages, a firewall (22 plus `firewall_allow_tcp`), the Docker
engine with the compose plugin, and a non-root `deploy_user`. Deploy logs in to the
registry, pulls both images at `image_tag`, renders `compose.yaml` into `app_dir`
(`/opt/tbd`), runs `docker compose up -d`, and waits for `/healthz`.

## Before first use

1. `ansible-galaxy collection install -r requirements.yml`
2. Replace the example hosts in `inventory/*.yml`.
3. Set `image_org` in `group_vars/all.yml`.
4. `cp group_vars/vault.yml.example group_vars/vault.yml`, fill it in, then
   `ansible-vault encrypt group_vars/vault.yml`.

## Output format

`ansible.cfg` uses the built-in `default` callback with `result_format = yaml`. The
`yaml` callback plugin lives in `community.general`, which `ansible-core` alone does not
ship; do not switch back to it.

## Variables

Any variable in `group_vars/all.yml` or `roles/tbd_app/defaults/main.yml` can be
overridden per inventory group, per host, or on the command line with `-e`.
`image_tag` is required for deploy and the playbook refuses `REPLACE_ME`.
