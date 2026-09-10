// Zod schemas at the API boundary. They mirror docs/chaos/api.md one to one;
// a field added to a Rust type in crates/chaos/src/api is added here.
import { z } from "zod";

export const RequestCounts = z.object({
  total: z.number(),
  failed: z.number(),
});

export const ErrorKind = z.enum(["unavailable", "internal", "overloaded", "timeout"]);
export type ErrorKind = z.infer<typeof ErrorKind>;

export type Behavior =
  | { type: "healthy" }
  | { type: "slow"; latency: string; jitter?: string }
  | { type: "hang" }
  | { type: "error"; kind: ErrorKind; rate?: number; message?: string }
  | { type: "delayed_failure"; healthy_for: string; then: Behavior };

export const Behavior: z.ZodType<Behavior> = z.lazy(() =>
  z.discriminatedUnion("type", [
    z.object({ type: z.literal("healthy") }),
    z.object({
      type: z.literal("slow"),
      latency: z.string(),
      jitter: z.string().optional(),
    }),
    z.object({ type: z.literal("hang") }),
    z.object({
      type: z.literal("error"),
      kind: ErrorKind,
      rate: z.number().optional(),
      message: z.string().optional(),
    }),
    z.object({
      type: z.literal("delayed_failure"),
      healthy_for: z.string(),
      then: Behavior,
    }),
  ]),
);

export const InstanceInfo = z.object({
  name: z.string(),
  kind: z.string(),
  addr: z.string(),
  running: z.boolean(),
  depends_on: z.array(z.string()),
  behavior: Behavior.nullable(),
  requests: RequestCounts.nullable(),
});
export type InstanceInfo = z.infer<typeof InstanceInfo>;

export const Latency = z.object({
  p50_ms: z.number(),
  p90_ms: z.number(),
  p99_ms: z.number(),
  max_ms: z.number(),
  mean_ms: z.number(),
});

export const OpSnapshot = z.object({
  total: z.number(),
  failed: z.number(),
  latency: Latency,
});

export const LoadSnapshot = z.object({
  elapsed_s: z.number(),
  requests_total: z.number(),
  requests_success: z.number(),
  requests_failed: z.number(),
  error_rate: z.number(),
  throughput_rps: z.number(),
  latency: Latency,
  per_target: z.record(z.string(), RequestCounts),
  per_op: z.record(z.string(), OpSnapshot),
  errors: z.record(z.string(), z.number()),
});
export type LoadSnapshot = z.infer<typeof LoadSnapshot>;

export const EventOutcome = z.object({
  at_s: z.number(),
  action: z.string(),
  error: z.string().nullable(),
});
export type EventOutcome = z.infer<typeof EventOutcome>;

export const AssertionResult = z.object({
  name: z.string(),
  passed: z.boolean(),
  expected: z.string(),
  actual: z.string(),
});

export const ScenarioResult = z.object({
  name: z.string(),
  file: z.string().nullable(),
  passed: z.boolean(),
  skipped: z.boolean(),
  duration_s: z.number(),
  load: LoadSnapshot.nullable(),
  services: z.record(z.string(), RequestCounts),
  events: z.array(EventOutcome),
  assertions: z.array(AssertionResult),
  error: z.string().nullable(),
});
export type ScenarioResult = z.infer<typeof ScenarioResult>;

export const CheckResult = z.object({
  name: z.string(),
  surface: z.string(),
  passed: z.boolean(),
  latency_ms: z.number(),
  detail: z.string(),
});
export type CheckResult = z.infer<typeof CheckResult>;

export const ValidateReport = z.object({
  checks: z.array(CheckResult),
  passed: z.number(),
  failed: z.number(),
});
export type ValidateReport = z.infer<typeof ValidateReport>;

export const RunKind = z.enum(["scenario", "load", "validate"]);
export type RunKind = z.infer<typeof RunKind>;
export const RunStatus = z.enum(["running", "passed", "failed", "error", "cancelled", "completed"]);
export type RunStatus = z.infer<typeof RunStatus>;

export const RunSummary = z.object({
  id: z.string(),
  kind: RunKind,
  name: z.string(),
  scenario_id: z.string().nullable(),
  status: RunStatus,
  started_at: z.string(),
  finished_at: z.string().nullable(),
  duration_s: z.number(),
  requests_total: z.number().nullable(),
  error_rate: z.number().nullable(),
  p99_ms: z.number().nullable(),
  passed: z.tuple([z.number(), z.number()]).nullable(),
  error: z.string().nullable(),
});
export type RunSummary = z.infer<typeof RunSummary>;

export const RunRecord = RunSummary.omit({
  requests_total: true,
  error_rate: true,
  p99_ms: true,
  passed: true,
}).extend({
  scenario: ScenarioResult.nullable(),
  load: LoadSnapshot.nullable(),
  validate: ValidateReport.nullable(),
  samples: z.array(LoadSnapshot),
  events: z.array(EventOutcome),
  request: z.unknown().nullable(),
});
export type RunRecord = z.infer<typeof RunRecord>;

export const RunFeed = z.discriminatedUnion("type", [
  z.object({ type: z.literal("started"), run: RunSummary }),
  z.object({ type: z.literal("phase"), id: z.string(), name: z.string() }),
  z.object({
    type: z.literal("load"),
    id: z.string(),
    snapshot: LoadSnapshot,
  }),
  z.object({
    type: z.literal("timeline"),
    id: z.string(),
    event: EventOutcome,
  }),
  z.object({ type: z.literal("finished"), run: RunRecord }),
]);
export type RunFeed = z.infer<typeof RunFeed>;

export const GlobalEvent = z.discriminatedUnion("type", [
  z.object({ type: z.literal("run_started"), run: RunSummary }),
  z.object({ type: z.literal("run_finished"), run: RunSummary }),
  z.object({
    type: z.literal("stack_changed"),
    instances: z.array(InstanceInfo),
  }),
]);
export type GlobalEvent = z.infer<typeof GlobalEvent>;

export const Links = z.object({
  grafana: z.string(),
  victorialogs: z.string(),
  pyroscope: z.string(),
  envoy_admin: z.string(),
});

export const ChaosConfig = z.object({
  serve: z.object({
    listen: z.string(),
    base_path: z.string(),
    ui_dir: z.string(),
    ui_path: z.string(),
    start_stack: z.boolean(),
  }),
  paths: z.object({
    topology: z.string(),
    scenarios: z.string(),
    results: z.string(),
  }),
  targets: z.object({ protocol: z.string(), engine: z.string() }),
  validate: z.object({ timeout: z.string() }),
  links: Links,
});
export type ChaosConfig = z.infer<typeof ChaosConfig>;

export const Overview = z.object({
  version: z.string(),
  env: z.string(),
  config_files: z.array(z.string()),
  config: ChaosConfig,
  stack: z.array(InstanceInfo).nullable(),
  active_run: RunSummary.nullable(),
  recent_runs: z.array(RunSummary),
  last_validate: RunSummary.nullable(),
  scenarios: z.number(),
  runs: z.number(),
});
export type Overview = z.infer<typeof Overview>;

export const ScenarioEntry = z.object({
  id: z.string(),
  file: z.string(),
  name: z.string().nullable(),
  description: z.string(),
  skip: z.boolean(),
  ok: z.boolean(),
  error: z.string().nullable(),
});
export type ScenarioEntry = z.infer<typeof ScenarioEntry>;

export const ScenarioDetail = ScenarioEntry.extend({
  text: z.string(),
  parsed: z.unknown().nullable(),
  last_run: RunSummary.nullable(),
});
export type ScenarioDetail = z.infer<typeof ScenarioDetail>;

export const CheckReply = z.object({
  ok: z.boolean(),
  name: z.string().nullable(),
  error: z.string().nullable(),
  parsed: z.unknown().nullable(),
});
export type CheckReply = z.infer<typeof CheckReply>;

export const OpKind = z.enum(["rest_evaluate", "graphql_evaluate", "ws_echo", "grpc_ping"]);
export type OpKind = z.infer<typeof OpKind>;

/** The `[load]` table of a scenario, as JSON. */
export type LoadConfig = {
  rate: number;
  duration: string;
  warmup?: string;
  timeout?: string;
  max_in_flight?: number;
  pattern?: { type: "constant" } | { type: "ramp"; start_rate: number; end_rate: number };
  operations: { op: OpKind; weight: number }[];
};

export type LoadRequest = {
  name?: string;
  targets?: { name: string; http_url: string }[];
  load: LoadConfig;
};
