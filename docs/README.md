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
| You are working on the `finance` service | [../crates/finance/CLAUDE.md](../crates/finance/CLAUDE.md) |
| A linked Gmail mailbox keeps expiring, or you are setting the Google clients up | [finance/gmail.md](finance/gmail.md) |
| You want the company's incoming e-invoices in the receipts, or are linking the intermediary | [finance/mojeracun.md](finance/mojeracun.md) (Moj-eRačun), [finance/eracuni.md](finance/eracuni.md) (e-računi) |
| You want the company's ePorezna filings (PD, PDV, JOPPD, …) in the service, or are uploading one | [finance/filings.md](finance/filings.md) |
| You want the books: the chart of accounts, periods, the opening balances and the trial balance | [finance/books.md](finance/books.md) |
| You are setting up, or changing, the full CV behind sign-in and approval on `cv.<domain>` | [cv/README.md](cv/README.md), [../crates/cv/CLAUDE.md](../crates/cv/CLAUDE.md) |
| You are working on the `playground` service | [../crates/playground/CLAUDE.md](../crates/playground/CLAUDE.md) |
| You are changing the public playground or its rules | [playground/README.md](playground/README.md) |
| You want the mobile architecture: a bearer client, PKCE, the theme from the web kit | [mobile/README.md](mobile/README.md) |
| You are building the mobile app or a package under `/mobile` | [../mobile/CLAUDE.md](../mobile/CLAUDE.md), then [../mobile/README.md](../mobile/README.md) for devices and environments |
| You are working on the `cv` service | [../crates/cv/CLAUDE.md](../crates/cv/CLAUDE.md) |
| You want to run, call or deploy the `llm` service: the engines behind it, the tiers, the budget, the record | [llm/README.md](llm/README.md) |
| You are working on the `llm` service | [../crates/llm/CLAUDE.md](../crates/llm/CLAUDE.md) |
| You are working on the `arena` service | [../crates/arena/CLAUDE.md](../crates/arena/CLAUDE.md) |
| A check failed in CI, or you are changing CI | [ci.md](ci.md) |
| You want metrics, traces, logs, dashboards, or to run the local cluster | [observability.md](observability/README.md) |
| You want to know how sign-in, tokens and API access work | [auth/README.md](auth/README.md) |
| You are changing how traffic is routed | [../devops/envoy/README.md](../devops/envoy/README.md) |
| You want the local cluster reachable from the internet | [local-cluster.md](local-cluster.md#reaching-it-from-the-internet) and [../devops/edge/README.md](../devops/edge/README.md) |
| You are deploying | [../devops/README.md](../devops/README.md) |
| You want the earlier product thinking | [design/](design/README.md), idea material only |
| You want the fun ideas we might build next, such as the interview simulator and the personal LLM stack | [ideas/](ideas/README.md), idea material only |
| You are writing an RFC or a study for the site's lab, or its page failed the redaction check | [rfcs/README.md](rfcs/README.md), [studies/README.md](studies/README.md) |

Every crate and the `devops/` and `scenarios/` directories also carry a `CLAUDE.md` with
the non-obvious facts about that directory: boundaries, invariants, gotchas. They are
written for AI assistants but they are the shortest accurate orientation for humans too.

Every page listed here, plus the root `README.md` and `ARCHITECTURE.md`, the devops READMEs
and the crate notes, is also served by the chaos admin UI under **Knowledge base**, bundled
at build time ([chaos/ui.md](chaos/ui.md)); relative links between pages keep working there.
