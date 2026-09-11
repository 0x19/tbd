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
  /** The store's fault behaviour, for kinds with `store_fault`. */
  store_behavior: Behavior.nullish(),
  requests: RequestCounts.nullable(),
  added: z.boolean(),
});
export type InstanceInfo = z.infer<typeof InstanceInfo>;

/** One key of a kind's `[stack.<plural>.X]` table, as `GET /overview` describes it. */
export const KindField = z.object({
  name: z.string(),
  label: z.string(),
  kind: z.enum(["text", "duration", "instance_of"]),
  of_kind: z.string().nullable(),
  required: z.boolean(),
  default: z.string().nullable(),
});
export type KindField = z.infer<typeof KindField>;

/** A registered service kind (docs/chaos/kinds.md). */
export const KindDescriptor = z.object({
  name: z.string(),
  label: z.string(),
  plural: z.string(),
  surface: z.string(),
  fault: z.boolean(),
  /** The service has a store chaos can fail (`set_store_behavior`). */
  store_fault: z.boolean().default(false),
  counters: z.boolean(),
  load_target: z.boolean(),
  addable: z.boolean(),
  target: z.boolean(),
  dependency_kind: z.string().nullable(),
  fields: z.array(KindField),
});
export type KindDescriptor = z.infer<typeof KindDescriptor>;

/** One validate check of the catalogue. */
export const CheckInfo = z.object({
  name: z.string(),
  surface: z.string(),
  kind: z.string(),
});
export type CheckInfo = z.infer<typeof CheckInfo>;

/** Body of `POST /stack`: a kind, an optional name, and the kind's own fields. */
export type AddInstance = { kind: string; name?: string } & Record<string, string | undefined>;

/** Body of `POST /validate`: a URL per kind with a target, and the timeout. Null means the config's. */
export type ValidateRequest = Record<string, string | null | undefined>;

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

export const RunKind = z.enum(["scenario", "load", "validate", "stress"]);
export type RunKind = z.infer<typeof RunKind>;
export const RunStatus = z.enum(["running", "passed", "failed", "error", "cancelled", "completed"]);
export type RunStatus = z.infer<typeof RunStatus>;

export const RunSummary = z.object({
  id: z.string(),
  kind: RunKind,
  name: z.string(),
  scenario_id: z.string().nullable(),
  schedule_id: z.string().nullable(),
  status: RunStatus,
  started_at: z.string(),
  finished_at: z.string().nullable(),
  duration_s: z.number(),
  requests_total: z.number().nullable(),
  error_rate: z.number().nullable(),
  throughput_rps: z.number().nullable(),
  p50_ms: z.number().nullable(),
  p90_ms: z.number().nullable(),
  p99_ms: z.number().nullable(),
  passed: z.tuple([z.number(), z.number()]).nullable(),
  error: z.string().nullable(),
  /** The service kinds the run exercised (stack, load targets or validated kinds). */
  services: z.array(z.string()).default([]),
  /** The campaign a stress run came from (`stress/<id>.toml`). */
  campaign_id: z.string().nullish(),
  /** Findings so far, for stress runs. */
  findings: z.number().nullish(),
});
export type RunSummary = z.infer<typeof RunSummary>;

// ---- stress campaigns and findings (docs/chaos/stress.md, api.md) ----

export const CheckCount = z.object({ passed: z.number(), violated: z.number() });
export type CheckCount = z.infer<typeof CheckCount>;

/** One `stress` SSE frame: the checks a second. */
export const StressSnapshot = z.object({
  elapsed_s: z.number(),
  phase: z.string(),
  ops_total: z.number(),
  ops_failed: z.number(),
  tolerated: z.number(),
  redriven: z.number(),
  checks: z.record(z.string(), CheckCount),
  findings: z.number(),
  subjects: z.number(),
  workers: z.record(z.string(), z.number()),
});
export type StressSnapshot = z.infer<typeof StressSnapshot>;

export const WorkerClass = z.enum(["owner", "contention", "fuzz"]);
export type WorkerClass = z.infer<typeof WorkerClass>;

export const FindingSummary = z.object({
  id: z.string(),
  invariant: z.string(),
  signature: z.string(),
  message: z.string(),
  subject: z.string(),
  worker: WorkerClass,
  campaign: z.string(),
  run_id: z.string().nullish(),
  target: z.string(),
  found_at: z.string(),
  trace_len: z.number(),
  shrunk: z.boolean(),
});
export type FindingSummary = z.infer<typeof FindingSummary>;

export const ReplayOutcome = z.object({
  at: z.string(),
  target: z.string(),
  reproduced: z.boolean(),
  message: z.string().nullish(),
  steps_run: z.number(),
});
export type ReplayOutcome = z.infer<typeof ReplayOutcome>;

/** Why a call in a trace failed: a gRPC status, a broken connection or a timeout. */
export const CallError = z.discriminatedUnion("kind", [
  z.object({ kind: z.literal("status"), code: z.string(), message: z.string() }),
  z.object({ kind: z.literal("transport") }).passthrough(),
  z.object({ kind: z.literal("timeout") }),
]);
export type CallError = z.infer<typeof CallError>;

/** One step of a finding's trace; the request is symbolic (`op` plus its fields). */
export const Step = z.object({
  index: z.number(),
  request: z.object({ op: z.string() }).passthrough(),
  response: z.unknown().optional(),
  error: CallError.optional(),
  at_ms: z.number(),
  tolerated: z.boolean().default(false),
});
export type Step = z.infer<typeof Step>;

export const Finding = FindingSummary.omit({ trace_len: true }).extend({
  expected: z.unknown(),
  actual: z.unknown(),
  store: z.string().nullish(),
  trace: z.array(Step),
  original_len: z.number(),
  shrink_note: z.string().nullish(),
  replays: z.array(ReplayOutcome).default([]),
});
export type Finding = z.infer<typeof Finding>;

/** Findings that share a signature: the same rule broken the same way. */
export const FindingGroup = z.object({
  invariant: z.string(),
  signature: z.string(),
  count: z.number(),
  first: z.string(),
  last: z.string(),
  runs: z.array(z.string()),
  campaigns: z.array(z.string()),
  sample: FindingSummary,
});
export type FindingGroup = z.infer<typeof FindingGroup>;

/** A stress run's result, on `RunRecord.stress`. */
export const StressResult = z.object({
  name: z.string(),
  passed: z.boolean(),
  skipped: z.boolean(),
  store: z.string().nullish(),
  targets: z.array(z.string()),
  load: LoadSnapshot.nullish(),
  checks: z.record(z.string(), CheckCount),
  tolerated: z.number().default(0),
  redriven: z.number().default(0),
  findings: z.array(FindingSummary),
  stopped_early: z.boolean().default(false),
  error: z.string().nullish(),
});
export type StressResult = z.infer<typeof StressResult>;

/** A ledger target for a campaign or a replay: `http_url` carries the gRPC URL. */
export type StressTarget = { name: string; http_url: string; kind?: string };
export type StressRequest = { campaign: string; targets?: StressTarget[]; name?: string };
export type ReplayRequest = { finding: string; targets?: StressTarget[]; attempts?: number };

export const RunRecord = RunSummary.omit({
  requests_total: true,
  error_rate: true,
  throughput_rps: true,
  p50_ms: true,
  p90_ms: true,
  p99_ms: true,
  passed: true,
}).extend({
  scenario: ScenarioResult.nullable(),
  load: LoadSnapshot.nullable(),
  validate: ValidateReport.nullable(),
  samples: z.array(LoadSnapshot),
  events: z.array(EventOutcome),
  request: z.unknown().nullable(),
  stress: StressResult.nullish(),
  replay: ReplayOutcome.nullish(),
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
  z.object({ type: z.literal("stress"), id: z.string(), snapshot: StressSnapshot }),
  z.object({ type: z.literal("finding"), id: z.string(), finding: FindingSummary }),
  z.object({ type: z.literal("finished"), run: RunRecord }),
]);
export type RunFeed = z.infer<typeof RunFeed>;

/** A queued or scheduled unit of work, the API's `Job` shape. */
export type Job =
  | { scenario: string }
  | "all_scenarios"
  | { load: LoadRequest }
  | { validate: ValidateRequest }
  | { stress: StressRequest }
  | { replay: ReplayRequest };

export const Job: z.ZodType<Job> = z.union([
  z.literal("all_scenarios"),
  z.object({ scenario: z.string() }),
  z.object({ load: z.custom<LoadRequest>((v) => typeof v === "object" && v !== null && "load" in v) }),
  // The API writes absent keys as null; the keys are the kinds with a target plus `timeout`.
  z.object({ validate: z.record(z.string(), z.string().nullish()) }),
  z.object({
    stress: z.custom<StressRequest>((v) => typeof v === "object" && v !== null && "campaign" in v),
  }),
  z.object({ replay: z.custom<ReplayRequest>((v) => typeof v === "object" && v !== null && "finding" in v) }),
]);

export const QueuedRun = z.object({
  id: z.string(),
  job: Job,
  kind: RunKind,
  name: z.string(),
  scenario_id: z.string().nullable(),
  schedule_id: z.string().nullable(),
  queued_at: z.string(),
});
export type QueuedRun = z.infer<typeof QueuedRun>;

export const Schedule = z.object({
  id: z.string(),
  name: z.string(),
  cron: z.string(),
  job: Job,
  enabled: z.boolean(),
  notify: z.enum(["failures", "always", "off"]),
  created_at: z.string(),
  updated_at: z.string(),
  next_at: z.string().nullable(),
  last_fired_at: z.string().nullable(),
  last_skipped_at: z.string().nullable(),
  fired: z.number(),
  skipped: z.number(),
});
export type Schedule = z.infer<typeof Schedule>;

export const NotifyMode = z.enum(["failures", "always", "off"]);
export type NotifyMode = z.infer<typeof NotifyMode>;

export type ScheduleSpec = { name: string; cron: string; job: Job; enabled: boolean; notify: NotifyMode };

export const Me = z.object({
  user: z.object({ sub: z.string(), email: z.string(), name: z.string(), role: z.string() }).nullable(),
  signout: z.string(),
  signout_all: z.string(),
});
export type Me = z.infer<typeof Me>;

export const NotifyState = z.object({
  enabled: z.boolean(),
  channel: z.string(),
  on: z.array(RunStatus),
  kinds: z.array(RunKind),
  env: z.string(),
});
export type NotifyState = z.infer<typeof NotifyState>;

export const GlobalEvent = z.discriminatedUnion("type", [
  z.object({ type: z.literal("run_started"), run: RunSummary }),
  z.object({ type: z.literal("run_finished"), run: RunSummary }),
  z.object({
    type: z.literal("stack_changed"),
    instances: z.array(InstanceInfo),
  }),
  z.object({ type: z.literal("queue_changed"), queue: z.array(QueuedRun) }),
  z.object({ type: z.literal("schedules_changed"), schedules: z.array(Schedule) }),
]);
export type GlobalEvent = z.infer<typeof GlobalEvent>;

export const Links = z.object({
  domain: z.string(),
  grafana: z.string(),
  victorialogs: z.string(),
  metrics: z.string(),
  pyroscope: z.string(),
  envoy_admin: z.string(),
  chaos: z.string(),
  auth: z.string(),
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
    schedules: z.string(),
    campaigns: z.string(),
    findings: z.string(),
  }),
  // One URL per kind with a validate target, resolved (defaults filled in).
  targets: z.record(z.string(), z.string()),
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
  kinds: z.array(KindDescriptor),
  validate: z.object({ checks: z.array(CheckInfo) }),
  active_run: RunSummary.nullable(),
  queue: z.array(QueuedRun),
  recent_runs: z.array(RunSummary),
  last_validate: RunSummary.nullable(),
  scenarios: z.number(),
  campaigns: z.number(),
  findings: z.number(),
  finding_signatures: z.number(),
  runs: z.number(),
  schedules: z.number(),
  schedules_enabled: z.number(),
  next_schedule: Schedule.nullable(),
  notify: NotifyState,
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

/** A campaign file as listed (`GET /stress`). */
export const CampaignEntry = ScenarioEntry.extend({ has_stack: z.boolean() });
export type CampaignEntry = z.infer<typeof CampaignEntry>;

export const CampaignDetail = CampaignEntry.extend({
  text: z.string(),
  parsed: z.unknown().nullable(),
  last_run: RunSummary.nullable(),
});
export type CampaignDetail = z.infer<typeof CampaignDetail>;

export const CheckReply = z.object({
  ok: z.boolean(),
  name: z.string().nullable(),
  error: z.string().nullable(),
  parsed: z.unknown().nullable(),
});
export type CheckReply = z.infer<typeof CheckReply>;

export const OpKind = z.enum([
  "rest_evaluate",
  "graphql_evaluate",
  "ws_echo",
  "grpc_ping",
  "ledger_append",
  "ledger_current",
  "ledger_history",
  "ledger_retract",
  "ledger_lifecycle",
  "ledger_erase_cycle",
  "ledger_fuzz",
]);
/** The kind of instance an operation targets (mirrors `OpKind::target_kind`). */
export const opTargetKind = (op: OpKind): string => (op.startsWith("ledger_") ? "ledger" : "protocol");
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
  seed?: number;
  subjects?: number;
};

export type LoadRequest = {
  name?: string;
  targets?: { name: string; http_url: string; kind?: string }[];
  load: LoadConfig;
};
