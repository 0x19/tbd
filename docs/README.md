# Documentation

| Read this when | Page |
|---|---|
| You are new to the repo | [README.md](../README.md), then [ARCHITECTURE.md](../ARCHITECTURE.md) |
| You want to run, load-test or fault-test the stack | [chaos/README.md](chaos/README.md) |
| A validate check, a scenario, a campaign or the admin UI failed and you want to know what it means | [chaos/runbook.md](chaos/runbook.md) |
| You need every `chaos` flag, output field and exit code | [chaos/commands.md](chaos/commands.md) |
| You are writing or debugging a scenario | [chaos/scenarios.md](chaos/scenarios.md) |
| You are writing a stress campaign or reading a finding | [chaos/stress.md](chaos/stress.md) |
| You are building or scripting against the admin UI (`chaos serve`) | [chaos/api.md](chaos/api.md) |
| You are using or changing the admin UI itself (`ui/chaos`) | [chaos/ui.md](chaos/ui.md) |
| You want to change what chaos does per environment (`configs/chaos/`) | [chaos/config.md](chaos/config.md) |
| You want to know how chaos works inside | [chaos/architecture.md](chaos/architecture.md) |
| You are adding a service, operation, action, assertion or check | [chaos/extending.md](chaos/extending.md) |
| You are adding a service | [tbd/README.md](tbd/README.md), the scaffolding CLI |
| You want to know what the ledger service is today | [ledger/README.md](ledger/README.md) |
| You want to know what the protocol service is today | [protocol/README.md](protocol/README.md) |
| You are working on the `protocol` service | [../crates/protocol/CLAUDE.md](../crates/protocol/CLAUDE.md) |
| You are working on the `ledger` service | [../crates/ledger/CLAUDE.md](../crates/ledger/CLAUDE.md) |
| You are working on the `humans` service | [../crates/humans/CLAUDE.md](../crates/humans/CLAUDE.md) |
| A check failed in CI, or you are changing CI | [ci.md](ci.md) |
| You want metrics, traces, logs, dashboards, or to run the local cluster | [observability.md](observability/README.md) |
| You want to know how sign-in, tokens and API access work | [auth/README.md](auth/README.md) |
| You are changing how traffic is routed | [../devops/envoy/README.md](../devops/envoy/README.md) |
| You want the local cluster reachable from the internet | [local-cluster.md](local-cluster.md#reaching-it-from-the-internet) and [../devops/edge/README.md](../devops/edge/README.md) |
| You are deploying | [../devops/README.md](../devops/README.md) |
| You want the earlier product thinking | [design/](design/README.md), idea material only |

Every crate and the `devops/` and `scenarios/` directories also carry a `CLAUDE.md` with
the non-obvious facts about that directory: boundaries, invariants, gotchas. They are
written for AI assistants but they are the shortest accurate orientation for humans too.

Every page listed here, plus the root `README.md` and `ARCHITECTURE.md`, the devops READMEs
and the crate notes, is also served by the chaos admin UI under **Knowledge base**, bundled
at build time ([chaos/ui.md](chaos/ui.md)); relative links between pages keep working there.
