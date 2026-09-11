// Helpers around the API's `Job` shape: build one from form values and
// describe one for lists.
import type { Job, LoadRequest, Overview, ScenarioEntry } from "@/lib/api/schema";
import { describeLoad, type LoadShape, toLoadRequest } from "@/lib/load";

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
  if ("load" in job) return `Load ${describeLoad(job.load)}`;
  return "Validate";
}

/** A schedule's load job from the same form the load page uses. */
export function loadJob(name: string, shape: LoadShape, overview: Overview | null): LoadRequest {
  return toLoadRequest(shape, name, overview);
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
