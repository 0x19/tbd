# ui/finances

The finance UI: a Next.js static export served at `finance.<domain>` behind Envoy's
browser login, talking to `tbd.finance.v1.FinanceService` through the protocol's REST
surface. Documentation in `CLAUDE.md`; the API in `docs/protocol/README.md` and
`proto/tbd/finance/v1/finance.proto`.

```sh
mise run ui:finances:dev     # dev server on :3004 (NEXT_PUBLIC_FINANCE_TOKEN from `mise run auth:token`)
mise run ui:finances:check   # prettier, eslint, tsc
mise run ui:finances:build   # static export into out/, baked into the image
```
