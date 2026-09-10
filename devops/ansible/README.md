# Ansible

Two playbooks, run from this directory.

| Playbook | When | Command |
|---|---|---|
| `playbooks/bootstrap.yml` | once per new host | `ansible-playbook -i inventory/dev.yml playbooks/bootstrap.yml` |
| `playbooks/deploy.yml` | every release | `ansible-playbook -i inventory/prod.yml playbooks/deploy.yml -e image_tag=v1.2.3 --ask-vault-pass` |

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

## Variables

Any variable in `group_vars/all.yml` or `roles/tbd_app/defaults/main.yml` can be
overridden per inventory group, per host, or on the command line with `-e`.
`image_tag` is required for deploy and the playbook refuses `REPLACE_ME`.
