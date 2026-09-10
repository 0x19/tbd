"use client";

import { Plus } from "lucide-react";

import { useChaos } from "@/app/providers";
import { Markdown, outline } from "@/components/markdown";
import { Button } from "@/components/ui/button";
import { kindsDoc, scenariosDoc } from "@/generated/docs";
import { stackSnippets } from "@/lib/kinds";

/** Blocks the editor can insert; each is a complete, checking piece of a scenario. The
 *  Stack group is generated from the registered kinds (`stackSnippets`). */
export const SNIPPETS: { group: string; items: { title: string; toml: string }[] }[] = [
  {
    group: "Load",
    items: [
      {
        title: "Constant load",
        toml: `[load]
rate = 100
duration = "3s"
warmup = "500ms"
timeout = "5s"
`,
      },
      {
        title: "Ramp",
        toml: `[load.pattern]
type = "ramp"
start_rate = 10
end_rate = 200
`,
      },
      {
        title: "Operation mix",
        toml: `[[load.operations]]
op = "rest_evaluate"
weight = 4

[[load.operations]]
op = "graphql_evaluate"
weight = 2

[[load.operations]]
op = "ws_echo"
weight = 2

[[load.operations]]
op = "grpc_ping"
weight = 1
`,
      },
    ],
  },
  {
    group: "Timeline",
    items: [
      {
        title: "Inject errors",
        toml: `[[timeline]]
at = "1s"
action = "set_behavior"
service = "engine-1"
behavior = { type = "error", kind = "unavailable", rate = 0.5 }
`,
      },
      {
        title: "Add latency",
        toml: `[[timeline]]
at = "1s"
action = "set_behavior"
service = "engine-1"
behavior = { type = "slow", latency = "50ms", jitter = "20ms" }
`,
      },
      {
        title: "Hang",
        toml: `[[timeline]]
at = "1s"
action = "set_behavior"
service = "engine-1"
behavior = { type = "hang" }
`,
      },
      {
        title: "Fail after a while",
        toml: `[[timeline]]
at = "500ms"
action = "set_behavior"
service = "engine-1"
[timeline.behavior]
type = "delayed_failure"
healthy_for = "1s"
[timeline.behavior.then]
type = "error"
kind = "internal"
`,
      },
      {
        title: "Heal",
        toml: `[[timeline]]
at = "2s"
action = "set_behavior"
service = "engine-1"
behavior = { type = "healthy" }
`,
      },
      {
        title: "Stop, then start",
        toml: `[[timeline]]
at = "1s"
action = "stop"
service = "engine-1"

[[timeline]]
at = "2s"
action = "start"
service = "engine-1"
`,
      },
      {
        title: "Log marker",
        toml: `[[timeline]]
at = "1500ms"
action = "log"
message = "halfway"
`,
      },
    ],
  },
  {
    group: "Assertions",
    items: [
      {
        title: "Window bounds",
        toml: `[assertions]
max_error_rate = 0.01
max_p50_ms = 50
max_p99_ms = 200
min_requests = 250
min_throughput = 90
`,
      },
      {
        title: "Per service",
        toml: `[assertions.services.engine-1]
min_requests = 200
max_failed = 0

[assertions.services.protocol-1]
min_requests = 100
`,
      },
    ],
  },
];

/** The scenario file reference (docs/chaos/scenarios.md, bundled) with insertable snippets. */
export function ScenarioReference({ onInsert }: { onInsert?: (toml: string) => void }) {
  const { kinds } = useChaos();
  const toc = [...outline(scenariosDoc), ...outline(kindsDoc)];
  const snippets = [{ group: "Stack", items: stackSnippets(kinds) }, ...SNIPPETS];
  return (
    <div className="grid min-w-0 [grid-template-columns:minmax(0,1fr)] gap-6">
      {onInsert ? (
        <section>
          <h3 className="mb-2 text-sm font-semibold">Insert a block</h3>
          <p className="text-muted-foreground mb-3 text-xs">
            Each block goes in at the cursor. Rename services to match your stack.
          </p>
          <div className="grid gap-3">
            {snippets.map((g) => (
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

      <Markdown text={scenariosDoc} />
      <Markdown text={kindsDoc} />
    </div>
  );
}
