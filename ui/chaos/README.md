# ui/chaos

The chaos admin UI: a Next.js static export served by `chaos serve` at `/chaos`. Full
documentation in [docs/chaos/ui.md](../../docs/chaos/ui.md); the API it talks to in
[docs/chaos/api.md](../../docs/chaos/api.md).

```sh
mise run chaos:serve   # API on :7700 (terminal 1)
mise run ui:dev        # dev server on :3001 (terminal 2)
mise run ui:check      # prettier, eslint, tsc
mise run ui:build      # static export into out/, picked up by chaos serve and the image
```

Without mise: `pnpm install`, `pnpm dev`, `pnpm build`. Node 24 and pnpm 12 are pinned
in the repo's `mise.toml`.
