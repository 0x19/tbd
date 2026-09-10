# ledger

The append-only facts store the humans plane is built on
([docs/design/humans/000](../design/humans/000-facts-ledger.md)). Today it is a
**stub**: the service exists, runs in the cluster behind Envoy, is exercised by the
chaos tool, and answers one RPC that says so on the wire. The facts API on Postgres is
the next plan; encryption ([005](../design/humans/005-encryption.md)) the one after.

| Piece | Where |
|---|---|
| Crate, scaffolded by `tbd new service ledger` | `crates/ledger` ([CLAUDE.md](../../crates/ledger/CLAUDE.md)) |
| Contract | `proto/tbd/ledger/v1/ledger.proto`: `LedgerService.Ping` |
| Config layers | `configs/ledger/{base,local,dev,production}.toml`; `ledger config` prints the merged result |
| Deployment | `devops/k8s/base/ledger`, port 50052, metrics 9464; reached through Envoy's internal listener (`http://envoy:50051`, matched by service name); no edge route |
| Chaos | `[stack.ledgers.X]` in topologies and scenarios; `chaos validate --ledger`; the `grpc_ledger_ping` check |

## The contract today

```proto
rpc Ping(PingRequest) returns (PingResponse);
message PingResponse { string message = 1; string version = 2; bool stub = 3; }
```

`stub` is `true` and stays so until a real RPC lands. A caller may rely on that field
the way it relies on `stub` in the engine's scores: a placeholder never looks like a
result.

```sh
mise run run:ledger                                    # on 127.0.0.1:50052 with configs/ledger local
grpcurl -plaintext -d '{"message":"hi"}' localhost:50052 tbd.ledger.v1.LedgerService/Ping
# through the cluster's internal listener (reflection there is the engine's, so pass the proto)
grpcurl -plaintext -import-path proto -proto tbd/ledger/v1/ledger.proto \
  -d '{"message":"hi"}' localhost:15051 tbd.ledger.v1.LedgerService/Ping
```

## What comes next

The facts ledger of the design: `Fact` with provenance, `append`, `current`,
`history`, `retract`, `erase`, on a dedicated Postgres with sqlx migrations; then the
`humans` service on top, scaffolded the same way.
