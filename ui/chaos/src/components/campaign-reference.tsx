"use client";

import { Plus } from "lucide-react";

import { Markdown, outline } from "@/components/markdown";
import { Button } from "@/components/ui/button";
import { stressDoc } from "@/generated/docs";

/** Blocks the campaign editor can insert; each is a complete, checking table. */
export const CAMPAIGN_SNIPPETS: { group: string; items: { title: string; toml: string }[] }[] = [
  {
    group: "Workload",
    items: [
      {
        title: "Owner workers",
        toml: `[workload.owner]
workers = 4
subjects = 2
pace = "0ms"
`,
      },
      {
        title: "Operation mix",
        toml: `[workload.owner.mix]
append = 6
current = 3
history = 3
history_cut = 2
retract = 1
idempotent_replay = 1
pair_relation = 1
erase_cycle = 0.2
expiring = 0.5
`,
      },
    ],
  },
  {
    group: "Faults",
    items: [
      {
        title: "Tolerate a fault timeline",
        toml: `[faults]
tolerate = ["unavailable", "transport", "timeout"]
settle = "1500ms"
`,
      },
      {
        title: "Inject errors",
        toml: `[[timeline]]
at = "1s"
action = "set_behavior"
service = "ledger-1"
behavior = { type = "error", kind = "unavailable", rate = 0.3 }

[[timeline]]
at = "3s"
action = "set_behavior"
service = "ledger-1"
behavior = { type = "healthy" }
`,
      },
    ],
  },
  {
    group: "Stop",
    items: [
      {
        title: "Stop at the first finding",
        toml: `[stop]
max_findings = 1
shrink = true
shrink_attempts = 200
shrink_timeout = "60s"
`,
      },
      {
        title: "Switch an invariant off",
        toml: `[invariants]
expiry = false
`,
      },
    ],
  },
];

/** The campaign file reference (docs/chaos/stress.md, bundled) with insertable snippets. */
export function CampaignReference({ onInsert }: { onInsert?: (toml: string) => void }) {
  const toc = outline(stressDoc);
  return (
    <div className="grid min-w-0 [grid-template-columns:minmax(0,1fr)] gap-6">
      {onInsert ? (
        <section>
          <h3 className="mb-2 text-sm font-semibold">Insert a block</h3>
          <p className="text-muted-foreground mb-3 text-xs">
            Each block goes in at the cursor. Rename the ledger to match your stack.
          </p>
          <div className="grid gap-3">
            {CAMPAIGN_SNIPPETS.map((g) => (
              <div key={g.group}>
                <div className="text-muted-foreground mb-1 text-[11px] font-medium tracking-wide uppercase">
                  {g.group}
                </div>
                <div className="flex flex-wrap gap-1.5">
                  {g.items.map((s) => (
                    <Button
                      key={s.title}
                      variant="outline"
                      size="sm"
                      className="h-7 text-xs"
                      onClick={() => onInsert(s.toml)}
                    >
                      <Plus className="size-3" /> {s.title}
                    </Button>
                  ))}
                </div>
              </div>
            ))}
          </div>
        </section>
      ) : null}

      <nav className="rounded-lg border p-3 text-xs" aria-label="Contents">
        <div className="text-muted-foreground mb-1 font-medium">On this page</div>
        <ul className="grid gap-0.5">
          {toc.map((h) => (
            <li key={h.id} className={h.level === 3 ? "pl-3" : ""}>
              <a href={`#${h.id}`} className="hover:underline">
                {h.title}
              </a>
            </li>
          ))}
        </ul>
      </nav>

      <Markdown text={stressDoc} />
    </div>
  );
}
