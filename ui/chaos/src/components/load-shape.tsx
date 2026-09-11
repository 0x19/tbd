"use client";

import { Info } from "lucide-react";

import { useChaos } from "@/app/providers";
import { InputGroup, InputGroupAddon, InputGroupInput, InputGroupText } from "@/components/ui/input-group";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { opTargetKind } from "@/lib/api/schema";
import { kindOf, withCapability } from "@/lib/kinds";
import {
  activeKinds,
  type LoadShape,
  OPS,
  stackInstances,
  targetOf,
  type TargetSource,
  weightOf,
} from "@/lib/load";

type Props = {
  shape: LoadShape;
  onChange: (shape: LoadShape) => void;
};

/**
 * The load form: shape (rate or ramp, window, concurrency), the operation mix
 * grouped by the kind each operation targets, and per kind where those
 * instances come from: the serve stack, the deployed stack behind Envoy
 * (`[targets]` in the config), or a URL. The load page and the schedule
 * dialog render the same fields; `toLoadRequest` turns the result into a run.
 */
export function LoadShapeFields({ shape, onChange }: Props) {
  const { overview, kinds } = useChaos();
  const set = (patch: Partial<LoadShape>) => onChange({ ...shape, ...patch });
  const totalWeight = OPS.reduce((n, op) => n + weightOf(shape, op), 0);
  // Operations grouped by target kind, registry order; a kind the registry
  // does not list (an old server) comes last.
  const groups = [
    ...withCapability(kinds, "load_target").map((k) => k.name),
    ...OPS.map(opTargetKind).filter((k) => !kinds.some((x) => x.name === k)),
  ].filter((kind, i, all) => all.indexOf(kind) === i && OPS.some((op) => opTargetKind(op) === kind));
  const active = activeKinds(shape);

  return (
    <div className="grid gap-6">
      <section>
        <h2 className="mb-3 text-base font-semibold">Shape</h2>
        <div className="grid gap-3 rounded-xl border p-4">
          <ShapeField
            label="Pattern"
            help="Constant holds the rate; ramp interpolates from start to end over the duration."
          >
            <select
              className="bg-background h-8 w-full rounded-lg border px-2 text-sm"
              value={shape.ramp ? "ramp" : "constant"}
              onChange={(e) => set({ ramp: e.target.value === "ramp" })}
            >
              <option value="constant">Constant</option>
              <option value="ramp">Ramp</option>
            </select>
          </ShapeField>
          {shape.ramp ? (
            <div className="grid grid-cols-2 gap-3">
              <ShapeField label="Start rate" help="Requests per second at t = 0.">
                <Rate value={shape.startRate} onChange={(startRate) => set({ startRate })} />
              </ShapeField>
              <ShapeField label="End rate" help="Requests per second at the end.">
                <Rate value={shape.endRate} onChange={(endRate) => set({ endRate })} />
              </ShapeField>
            </div>
          ) : (
            <ShapeField
              label="Rate"
              help="Requests per second across every target, open loop: latency does not slow the pacer."
            >
              <Rate value={shape.rate} onChange={(rate) => set({ rate })} />
            </ShapeField>
          )}
          <div className="grid grid-cols-2 gap-3">
            <ShapeField label="Duration" help="Measured window, excluding warmup.">
              <InputGroup>
                <InputGroupInput
                  value={shape.duration}
                  onChange={(e) => set({ duration: e.target.value })}
                  placeholder="10s"
                />
              </InputGroup>
            </ShapeField>
            <ShapeField label="Warmup" help="Same load first; its numbers are discarded.">
              <InputGroup>
                <InputGroupInput
                  value={shape.warmup}
                  onChange={(e) => set({ warmup: e.target.value })}
                  placeholder="500ms"
                />
              </InputGroup>
            </ShapeField>
            <ShapeField label="Timeout" help="Per request; a timeout counts as a failure.">
              <InputGroup>
                <InputGroupInput
                  value={shape.timeout}
                  onChange={(e) => set({ timeout: e.target.value })}
                  placeholder="5s"
                />
              </InputGroup>
            </ShapeField>
            <ShapeField label="Max in flight" help="Concurrency cap; the pacer stalls when reached.">
              <InputGroup>
                <InputGroupInput
                  value={shape.maxInFlight}
                  onChange={(e) => set({ maxInFlight: e.target.value })}
                  inputMode="numeric"
                />
              </InputGroup>
            </ShapeField>
          </div>
        </div>
      </section>

      <section>
        <h2 className="mb-3 text-base font-semibold">Operations</h2>
        <div className="grid gap-4 rounded-xl border p-4">
          {groups.map((kind) => (
            <div key={kind} className="grid gap-3">
              <div className="text-muted-foreground text-xs font-medium tracking-wide uppercase">
                {kindOf(kinds, kind).label} operations
              </div>
              {OPS.filter((op) => opTargetKind(op) === kind).map((op) => (
                <ShapeField key={op} label={op} help="Relative weight in the mix; zero leaves it out.">
                  <InputGroup>
                    <InputGroupInput
                      value={shape.weights[op] ?? ""}
                      onChange={(e) => set({ weights: { ...shape.weights, [op]: e.target.value } })}
                      inputMode="numeric"
                      placeholder="0"
                    />
                    <InputGroupAddon align="inline-end">
                      <InputGroupText>
                        {totalWeight ? `${Math.round((weightOf(shape, op) / totalWeight) * 100)}%` : "–"}
                      </InputGroupText>
                    </InputGroupAddon>
                  </InputGroup>
                </ShapeField>
              ))}
            </div>
          ))}
        </div>
      </section>

      <section>
        <h2 className="mb-3 text-base font-semibold">Targets</h2>
        <div className="grid gap-3 rounded-xl border p-4">
          {active.length ? null : (
            <div className="text-muted-foreground text-sm">
              Give an operation a weight to pick its target.
            </div>
          )}
          {active.map((kind) => {
            const k = kindOf(kinds, kind);
            const t = targetOf(shape, kind);
            const running = stackInstances(overview, kind).map((i) => i.name);
            const deployed = overview?.config.targets[kind];
            const choose = (patch: Partial<typeof t>) =>
              set({ targets: { ...shape.targets, [kind]: { ...t, ...patch } } });
            return (
              <div key={kind} className="grid gap-2">
                <ShapeField
                  label={`${k.label} target`}
                  help={`Where the ${k.label.toLowerCase()} operations go: the instances this serve started, the deployed stack through Envoy, or any ${k.surface === "grpc" ? "gRPC" : "base"} URL.`}
                >
                  <select
                    aria-label={`${k.label} target`}
                    className="bg-background h-8 w-full rounded-lg border px-2 text-sm"
                    value={t.source}
                    onChange={(e) => choose({ source: e.target.value as TargetSource })}
                  >
                    <option value="stack">
                      Serve stack ({running.join(", ") || `no ${k.label.toLowerCase()} running`})
                    </option>
                    <option value="deployed" disabled={!deployed}>
                      Deployed stack ({deployed ?? "no target configured"})
                    </option>
                    <option value="url">A URL</option>
                  </select>
                </ShapeField>
                {t.source === "url" ? (
                  <InputGroup>
                    <InputGroupInput
                      aria-label={`${k.label} URL`}
                      value={t.url}
                      onChange={(e) => choose({ url: e.target.value })}
                      placeholder={deployed ?? "http://localhost:18080"}
                    />
                  </InputGroup>
                ) : null}
              </div>
            );
          })}
        </div>
      </section>
    </div>
  );
}

function Rate({ value, onChange }: { value: string; onChange: (v: string) => void }) {
  return (
    <InputGroup>
      <InputGroupInput value={value} onChange={(e) => onChange(e.target.value)} inputMode="numeric" />
      <InputGroupAddon align="inline-end">
        <InputGroupText>req/s</InputGroupText>
      </InputGroupAddon>
    </InputGroup>
  );
}

export function ShapeField({
  label,
  help,
  children,
}: {
  label: string;
  help?: string;
  children: React.ReactNode;
}) {
  return (
    <label className="grid gap-1.5">
      <span className="flex items-center gap-1 text-sm font-medium">
        {label}
        {help ? <Hint text={help} /> : null}
      </span>
      {children}
    </label>
  );
}

export function Hint({ text }: { text: string }) {
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <span className="inline-flex">
          <Info className="text-muted-foreground size-3.5" />
        </span>
      </TooltipTrigger>
      <TooltipContent>{text}</TooltipContent>
    </Tooltip>
  );
}
