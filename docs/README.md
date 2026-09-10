# Documentation

| Read this when | Page |
|---|---|
| You are new to the repo | [README.md](../README.md), then [ARCHITECTURE.md](../ARCHITECTURE.md) |
| You want to run, load-test or fault-test the stack | [chaos/README.md](chaos/README.md) |
| You need every `chaos` flag, output field and exit code | [chaos/commands.md](chaos/commands.md) |
| You are writing or debugging a scenario | [chaos/scenarios.md](chaos/scenarios.md) |
| You want to know how chaos works inside | [chaos/architecture.md](chaos/architecture.md) |
| You are adding a service, operation, action, assertion or check | [chaos/extending.md](chaos/extending.md) |
| A check failed in CI, or you are changing CI | [ci.md](ci.md) |
| You are deploying | [../devops/README.md](../devops/README.md) |
| You want the earlier product thinking | [design/](design/README.md), idea material only |

Every crate and the `devops/` and `scenarios/` directories also carry a `CLAUDE.md` with
the non-obvious facts about that directory: boundaries, invariants, gotchas. They are
written for AI assistants but they are the shortest accurate orientation for humans too.
