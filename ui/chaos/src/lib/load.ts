// Ad-hoc load as a form: the shape, the mix and where it goes. Shared by the
// load page and the schedule dialog; translated to and from the API's
// `LoadRequest` (docs/chaos/api.md, "Runs").
import { type KindDescriptor, type LoadRequest, OpKind, opTargetKind, type Overview } from "@/lib/api/schema";
import { kindOf } from "@/lib/kinds";

/** Where the instances of one target kind come from. */
export type TargetSource = "stack" | "deployed" | "url";

export type TargetChoice = { source: TargetSource; url: string };

export type LoadShape = {
  rate: string;
  duration: string;
  warmup: string;
  timeout: string;
  maxInFlight: string;
  ramp: boolean;
  startRate: string;
  endRate: string;
  /** Relative weight per operation, as typed; zero or blank leaves it out. */
  weights: Record<string, string>;
  /** Per target kind (protocol, ledger, …); a kind not listed runs on the serve stack. */
  targets: Record<string, TargetChoice>;
};

export const OPS = OpKind.options;

/** The load page's starting point: the protocol mix on the serve stack. */
export const DEFAULT_SHAPE: LoadShape = {
  rate: "200",
  duration: "10s",
  warmup: "500ms",
  timeout: "5s",
  maxInFlight: "256",
  ramp: false,
  startRate: "10",
  endRate: "500",
  weights: { rest_evaluate: "4", graphql_evaluate: "2", ws_echo: "2", grpc_ping: "1" },
  targets: {},
};

export function secondsOf(s: string): number {
  const m = /^(\d+(?:\.\d+)?)\s*(ms|s|m|h)?$/.exec(s.trim());
  if (!m) return 0;
  const n = Number(m[1]);
  return m[2] === "ms" ? n / 1000 : m[2] === "m" ? n * 60 : m[2] === "h" ? n * 3600 : n;
}

export function weightOf(shape: LoadShape, op: string): number {
  return Math.max(0, Number(shape.weights[op]) || 0);
}

export function weightedOps(shape: LoadShape): OpKind[] {
  return OPS.filter((op) => weightOf(shape, op) > 0);
}

/** The kinds the weighted operations target, in order of first appearance. */
export function activeKinds(shape: LoadShape): string[] {
  const out: string[] = [];
  for (const op of weightedOps(shape)) {
    const kind = opTargetKind(op);
    if (!out.includes(kind)) out.push(kind);
  }
  return out;
}

export function targetOf(shape: LoadShape, kind: string): TargetChoice {
  return shape.targets[kind] ?? { source: "stack", url: "" };
}

/** Running serve-stack instances of `kind`. */
export function stackInstances(overview: Overview | null, kind: string) {
  return (overview?.stack ?? []).filter((i) => i.running && i.kind === kind);
}

/**
 * The request the API takes. Every kind on the serve stack: no `targets`, the
 * server picks its running instances. Otherwise one explicit list: the stack's
 * running instances by address (what the server would pick), the deployed
 * stack's URL from `[targets]`, or the typed URL, each tagged with its kind.
 */
export function toLoadRequest(shape: LoadShape, name: string, overview: Overview | null): LoadRequest {
  const kinds = activeKinds(shape);
  const targets: NonNullable<LoadRequest["targets"]> = [];
  if (kinds.some((k) => targetOf(shape, k).source !== "stack")) {
    for (const kind of kinds) {
      const t = targetOf(shape, kind);
      if (t.source === "url") targets.push({ name: `${kind}-url`, http_url: t.url.trim(), kind });
      else if (t.source === "deployed")
        targets.push({ name: `deployed-${kind}`, http_url: overview?.config.targets[kind] ?? "", kind });
      else
        for (const i of stackInstances(overview, kind))
          targets.push({ name: i.name, http_url: `http://${i.addr}`, kind });
    }
  }
  const maxInFlight = Number(shape.maxInFlight);
  return {
    name: name || undefined,
    targets,
    load: {
      rate: shape.ramp ? Number(shape.endRate) || 0 : Number(shape.rate) || 0,
      duration: shape.duration.trim(),
      warmup: shape.warmup.trim() || undefined,
      timeout: shape.timeout.trim() || undefined,
      max_in_flight: maxInFlight > 0 ? maxInFlight : undefined,
      pattern: shape.ramp
        ? { type: "ramp", start_rate: Number(shape.startRate) || 0, end_rate: Number(shape.endRate) || 0 }
        : undefined,
      operations: weightedOps(shape).map((op) => ({ op, weight: weightOf(shape, op) })),
    },
  };
}

/** A saved request back into the form (editing a schedule). */
export function fromLoadRequest(req: LoadRequest, overview: Overview | null): LoadShape {
  const l = req.load;
  const weights: Record<string, string> = {};
  for (const o of l.operations) weights[o.op] = String(o.weight);
  const targets: Record<string, TargetChoice> = {};
  for (const t of req.targets ?? []) {
    const kind = t.kind ?? "protocol";
    if (targets[kind]) continue;
    const deployed = overview?.config.targets[kind];
    if (deployed && t.http_url === deployed) targets[kind] = { source: "deployed", url: "" };
    else if ((overview?.stack ?? []).some((i) => i.name === t.name && i.kind === kind))
      targets[kind] = { source: "stack", url: "" };
    else targets[kind] = { source: "url", url: t.http_url };
  }
  const ramp = l.pattern?.type === "ramp" ? l.pattern : null;
  return {
    rate: String(l.rate),
    duration: l.duration,
    warmup: l.warmup ?? "",
    timeout: l.timeout ?? "",
    maxInFlight: l.max_in_flight === undefined ? "" : String(l.max_in_flight),
    ramp: ramp !== null,
    startRate: ramp ? String(ramp.start_rate) : DEFAULT_SHAPE.startRate,
    endRate: ramp ? String(ramp.end_rate) : DEFAULT_SHAPE.endRate,
    weights,
    targets,
  };
}

/** Why this shape cannot run yet, or null. */
export function loadProblem(
  shape: LoadShape,
  overview: Overview | null,
  kinds: KindDescriptor[],
): string | null {
  if (shape.ramp) {
    if (!(Number(shape.endRate) > 0)) return "the ramp needs an end rate above zero";
  } else if (!(Number(shape.rate) > 0)) return "the rate must be above zero";
  if (!(secondsOf(shape.duration) > 0)) return "the duration is not a duration (10s, 2m)";
  const active = activeKinds(shape);
  if (!active.length) return "give at least one operation a weight";
  for (const kind of active) {
    const label = kindOf(kinds, kind).label.toLowerCase();
    const t = targetOf(shape, kind);
    if (t.source === "url" && !t.url.trim()) return `${label}: give a URL`;
    if (t.source === "deployed" && !overview?.config.targets[kind])
      return `${label}: no deployed target in the config`;
    if (t.source === "stack" && !stackInstances(overview, kind).length)
      return `${label}: nothing running in the serve stack`;
  }
  return null;
}

/** "200 req/s for 10s: ledger_append 3, ledger_current 4 → deployed ledger" for lists. */
export function describeLoad(req: LoadRequest): string {
  const l = req.load;
  const shape =
    l.pattern?.type === "ramp" ? `${l.pattern.start_rate}→${l.pattern.end_rate} req/s` : `${l.rate} req/s`;
  const mix = l.operations.map((o) => `${o.op} ${o.weight}`).join(", ");
  const where = describeTargets(req.targets ?? []);
  return `${shape} for ${l.duration}: ${mix || "no operations"} → ${where}`;
}

function describeTargets(targets: NonNullable<LoadRequest["targets"]>): string {
  if (!targets.length) return "serve stack";
  const parts: string[] = [];
  const seen = new Set<string>();
  for (const t of targets) {
    const kind = t.kind ?? "protocol";
    if (seen.has(kind)) continue;
    seen.add(kind);
    if (t.name === `deployed-${kind}`) parts.push(`deployed ${kind}`);
    else if (t.name === `${kind}-url`) parts.push(`${kind} at ${t.http_url}`);
    else
      parts.push(
        `${kind} ${targets
          .filter((x) => (x.kind ?? "protocol") === kind)
          .map((x) => x.name)
          .join(", ")}`,
      );
  }
  return parts.join("; ");
}
