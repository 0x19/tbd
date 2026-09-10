// Helpers around the API's `Job` shape: build one from form values and
// describe one for lists.
import type { Job, LoadRequest, ScenarioEntry } from "@/lib/api/schema";

export type JobKind = "scenario" | "all_scenarios" | "load" | "validate";

export function jobKind(job: Job): JobKind {
  if (job === "all_scenarios") return "all_scenarios";
  if ("scenario" in job) return "scenario";
  if ("load" in job) return "load";
  return "validate";
}

/** One line for lists: "Scenario baseline", "All scenarios", "Load 200 req/s for 10s". */
export function describeJob(job: Job, scenarios?: ScenarioEntry[]): string {
  if (job === "all_scenarios") {
    const ready = scenarios?.filter((s) => s.ok && !s.skip).length;
    return ready === undefined ? "All scenarios" : `All scenarios (${ready})`;
  }
  if ("scenario" in job) {
    const name = scenarios?.find((s) => s.id === job.scenario)?.name;
    return `Scenario ${name ?? job.scenario}`;
  }
  if ("load" in job) {
    const l = job.load.load;
    const shape =
      l.pattern?.type === "ramp" ? `${l.pattern.start_rate}→${l.pattern.end_rate} req/s` : `${l.rate} req/s`;
    return `Load ${shape} for ${l.duration}`;
  }
  return "Validate";
}

/** The load page's default mix, for schedules that only ask for rate and duration. */
export function loadJob(name: string, rate: number, duration: string): LoadRequest {
  return {
    name: name || undefined,
    targets: [],
    load: {
      rate,
      duration,
      warmup: "500ms",
      timeout: "5s",
      max_in_flight: 256,
      operations: [
        { op: "rest_evaluate", weight: 4 },
        { op: "graphql_evaluate", weight: 2 },
        { op: "ws_echo", weight: 2 },
        { op: "grpc_ping", weight: 1 },
      ],
    },
  };
}

/** Cron presets offered in the schedule dialog; anything else is "custom". */
export const CRON_PRESETS: { label: string; cron: string }[] = [
  { label: "Every 5 minutes", cron: "*/5 * * * *" },
  { label: "Every 15 minutes", cron: "*/15 * * * *" },
  { label: "Every hour", cron: "0 * * * *" },
  { label: "Every 6 hours", cron: "0 */6 * * *" },
  { label: "Daily at 03:00 UTC", cron: "0 3 * * *" },
  { label: "Weekdays at 07:00 UTC", cron: "0 7 * * 1-5" },
];

export function presetLabel(cron: string): string {
  return CRON_PRESETS.find((p) => p.cron === cron)?.label ?? cron;
}
