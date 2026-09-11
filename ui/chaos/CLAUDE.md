@AGENTS.md

# ui/chaos

The chaos admin UI, built on the shadcnblocks Admin Kit (v2.3.0, the paid ZIP; not in
git, the user keeps it at `/mnt/development/0x19/admin-kit/`). Read `docs/chaos/ui.md`
first; `docs/chaos/api.md` is the contract the pages consume.

- What is the kit's, verbatim or lightly adapted: `src/components/ui/*` (shadcn on
  Radix, `asChild` composition, never `render=`), `src/components/layout/*` (sidebar,
  nav-group, team-switcher, header, sub-header, notifications, theme controls),
  `theme-*.tsx` plus `src/lib/theme-preset*.ts` (the theme preset picker in the header),
  `search-provider.tsx`, `src/app/globals.css`, ESLint (simple-import-sort, React
  Compiler rules relaxed like the kit) and Prettier (tailwind plugin). Regenerate `ui/`
  files from the kit or `pnpm dlx shadcn@latest add`, do not hand-edit.
- What is ours: pages under `src/app/`, `src/components/{kit,charts,status-badge,
runs-table,behavior-dialog,instances-table,field,queue-panel,schedule-dialog,load-shape,
toml-editor,markdown,scenario-reference,campaign-reference,kb-bits,stress-target}.tsx`,
  `src/lib/{api,format,runs,jobs,load,kb,toml-highlight}`,
  `src/app/editor.css` (TOML token colours and the `.doc` styles; globals.css stays the
  kit's),
  `src/data/{site,sidebar-data}`, `src/app/providers.tsx` (chaos overview context +
  the kit's search state), `command-menu.tsx`, `header-notifications.tsx` (finished
  runs from the live feed).
- Static export (`output: "export"`, no `basePath`, `trailingSlash: true`): the kit's
  `(admin)/layout.tsx` read a cookie on the server; ours mounts the shell client-side in
  `src/app/layout.tsx` with the sidebar open by default. Nothing may need a Node runtime.
- The API base is same-origin in a build and `http://127.0.0.1:7700/api/chaos/v1` under
  `next dev` (`NEXT_PUBLIC_CHAOS_API` overrides). Do not hardcode hosts in pages.
- `src/lib/api/schema.ts` mirrors `crates/chaos/src/api` one type at a time through Zod.
  Change both, and `docs/chaos/api.md`, in the same commit.
- Fonts: Inter (`--font-sans`) and Geist Mono (`--font-mono`) via next/font on `<html>`,
  which is what `globals.css` maps to Tailwind. Charts use the kit's `--chart-*` blues.
- Kit primitives dropped because their packages were pruned: drawer (vaul), form
  (react-hook-form), calendar (react-day-picker), map (maplibre), button-group and item
  (@radix-ui/react-slot). Add the package back before restoring one.
- Detail pages are `/x/view/?id=` with `useSearchParams` under `Suspense`; a dynamic
  segment cannot be exported.
- `pnpm dev` is on 3001 because Grafana owns 3000 locally.
- `src/generated/docs.ts` is written by `scripts/gen-docs.mjs` before dev, build, lint and
  typecheck (`pnpm gen`); it is gitignored. It carries the knowledge base (`kbDocs`,
  `kbCategories`: every page under `docs/`, the root README and ARCHITECTURE, the devops
  READMEs, the crate `CLAUDE.md`s, with title, summary, headings, links and the last
  commit) and the three named exports the editors insert from. Categories are the table
  at the top of the script; a new docs directory needs a row there or it is not bundled.
  Edit the markdown, never the generated file. `src/lib/kb.ts` orders categories, resolves
  relative links to `/kb/view/?doc=<path>` and searches sections; heading ids are
  deduplicated identically in the script and in `markdown.tsx`. The editor is CodeMirror 6 (`toml-editor.tsx`, the legacy
  TOML mode, tokens styled by class); `markdown.tsx` renders the doc with the same
  highlighter for `toml` fences.
- Checks: `mise run ui:check` (prettier, eslint, tsc) is part of `mise run ci`; `mise run
ui:build` must pass before the chaos image is built (`local:build` does both).
- `e2e/smoke.mjs` (`mise run ui:e2e`) is the browser check against a running cluster;
  extend it when a page gains a flow. The theme control is a menu: click, then "Dark".
- `chaos serve` serves `out/` at the root of its host (`crates/chaos/src/api/ui.rs`).
