"use client";

import { useState } from "react";
import { toast } from "sonner";

import { Field } from "@/components/field";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import { api } from "@/lib/api/client";
import { describe } from "@/lib/api/hooks";
import type { Job, ScenarioEntry, Schedule } from "@/lib/api/schema";
import { CRON_PRESETS, type JobKind, jobKind, loadJob } from "@/lib/jobs";

export type SchedulePreset = { kind: JobKind; scenario?: string };

type Props = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** Existing schedule to edit; otherwise a new one. */
  schedule?: Schedule | null;
  /** For a new schedule: what to start with (from "Schedule…" on a scenario). */
  preset?: SchedulePreset | null;
  scenarios: ScenarioEntry[];
  onSaved: (s: Schedule) => void;
};

const CUSTOM = "custom";

function initialJob(schedule: Schedule | null | undefined, preset: SchedulePreset | null | undefined) {
  if (schedule) {
    const kind = jobKind(schedule.job);
    const j = schedule.job;
    return {
      kind,
      scenario: kind === "scenario" && typeof j === "object" && "scenario" in j ? j.scenario : "",
      rate: kind === "load" && typeof j === "object" && "load" in j ? String(j.load.load.rate) : "200",
      duration: kind === "load" && typeof j === "object" && "load" in j ? j.load.load.duration : "10s",
    };
  }
  return {
    kind: preset?.kind ?? "all_scenarios",
    scenario: preset?.scenario ?? "",
    rate: "200",
    duration: "10s",
  };
}

/** Create or edit one schedule: a name, what to run, how often, on or off. */
export function ScheduleDialog({ open, onOpenChange, schedule, preset, scenarios, onSaved }: Props) {
  const init = initialJob(schedule, preset);
  const [name, setName] = useState(schedule?.name ?? defaultName(init.kind, init.scenario));
  const [kind, setKind] = useState<JobKind>(init.kind);
  const [scenario, setScenario] = useState(init.scenario);
  const [rate, setRate] = useState(init.rate);
  const [duration, setDuration] = useState(init.duration);
  const presetOf = CRON_PRESETS.find((p) => p.cron === (schedule?.cron ?? CRON_PRESETS[1]!.cron));
  const [presetCron, setPresetCron] = useState(presetOf ? presetOf.cron : CUSTOM);
  const [cron, setCron] = useState(schedule?.cron ?? CRON_PRESETS[1]!.cron);
  const [enabled, setEnabled] = useState(schedule?.enabled ?? true);
  const [busy, setBusy] = useState(false);

  const ready = scenarios.filter((s) => s.ok && !s.skip);

  const job = (): Job => {
    if (kind === "scenario") return { scenario };
    if (kind === "all_scenarios") return "all_scenarios";
    if (kind === "load") return { load: loadJob(name, Number(rate) || 0, duration) };
    return { validate: {} };
  };

  const valid =
    name.trim().length > 0 &&
    cron.trim().length > 0 &&
    (kind !== "scenario" || scenario.length > 0) &&
    (kind !== "load" || (Number(rate) > 0 && duration.trim().length > 0));

  const save = async () => {
    setBusy(true);
    try {
      const spec = { name: name.trim(), cron: cron.trim(), job: job(), enabled };
      const saved = schedule ? await api.scheduleUpdate(schedule.id, spec) : await api.scheduleCreate(spec);
      toast.success(schedule ? `saved ${saved.name}` : `scheduled ${saved.name}`);
      onSaved(saved);
      onOpenChange(false);
    } catch (e) {
      toast.error(describe(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>{schedule ? "Edit schedule" : "New schedule"}</DialogTitle>
          <DialogDescription>
            Queues its job on a cron in UTC. A schedule due while its last job is still queued or running is
            skipped, not stacked.
          </DialogDescription>
        </DialogHeader>

        <div className="grid gap-4">
          <Field label="Name">
            <Input value={name} onChange={(e) => setName(e.target.value)} placeholder="nightly sweep" />
          </Field>

          <div className="grid gap-4 sm:grid-cols-2">
            <Field label="Runs">
              <Select
                value={kind}
                onValueChange={(v) => {
                  const k = v as JobKind;
                  setKind(k);
                  if (!schedule && (name === "" || name === defaultName(kind, scenario)))
                    setName(defaultName(k, scenario));
                }}
              >
                <SelectTrigger aria-label="Runs">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all_scenarios">All scenarios</SelectItem>
                  <SelectItem value="scenario">One scenario</SelectItem>
                  <SelectItem value="load">Load</SelectItem>
                  <SelectItem value="validate">Validate</SelectItem>
                </SelectContent>
              </Select>
            </Field>
            {kind === "scenario" ? (
              <Field label="Scenario">
                <Select
                  value={scenario}
                  onValueChange={(v) => {
                    setScenario(v);
                    if (!schedule && (name === "" || name === defaultName(kind, scenario)))
                      setName(defaultName(kind, v));
                  }}
                >
                  <SelectTrigger aria-label="Scenario">
                    <SelectValue placeholder="Pick one" />
                  </SelectTrigger>
                  <SelectContent>
                    {ready.map((s) => (
                      <SelectItem key={s.id} value={s.id}>
                        {s.name ?? s.id}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </Field>
            ) : kind === "load" ? (
              <div className="grid grid-cols-2 gap-2">
                <Field label="Rate (req/s)">
                  <Input value={rate} onChange={(e) => setRate(e.target.value)} inputMode="numeric" />
                </Field>
                <Field label="Duration">
                  <Input value={duration} onChange={(e) => setDuration(e.target.value)} placeholder="10s" />
                </Field>
              </div>
            ) : (
              <div className="text-muted-foreground self-end pb-2 text-xs">
                {kind === "all_scenarios"
                  ? `${ready.length} ready scenario${ready.length === 1 ? "" : "s"}, one after another.`
                  : "Every surface of the configured targets."}
              </div>
            )}
          </div>

          <div className="grid gap-4 sm:grid-cols-2">
            <Field label="How often">
              <Select
                value={presetCron}
                onValueChange={(v) => {
                  setPresetCron(v);
                  if (v !== CUSTOM) setCron(v);
                }}
              >
                <SelectTrigger aria-label="How often">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {CRON_PRESETS.map((p) => (
                    <SelectItem key={p.cron} value={p.cron}>
                      {p.label}
                    </SelectItem>
                  ))}
                  <SelectItem value={CUSTOM}>Custom cron</SelectItem>
                </SelectContent>
              </Select>
            </Field>
            <Field label="Cron (UTC)">
              <Input
                value={cron}
                onChange={(e) => {
                  setCron(e.target.value);
                  setPresetCron(
                    CRON_PRESETS.some((p) => p.cron === e.target.value) ? e.target.value : CUSTOM,
                  );
                }}
                className="font-mono"
                placeholder="*/15 * * * *"
              />
            </Field>
          </div>

          <label className="flex items-center gap-3 text-sm">
            <Switch checked={enabled} onCheckedChange={setEnabled} aria-label="Enabled" />
            {enabled ? "Enabled" : "Paused: kept, never fires"}
          </label>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button onClick={save} disabled={!valid || busy}>
            {schedule ? "Save" : "Create schedule"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function defaultName(kind: JobKind, scenario: string): string {
  if (kind === "scenario") return scenario ? `${scenario} on a schedule` : "";
  if (kind === "all_scenarios") return "all scenarios";
  if (kind === "load") return "scheduled load";
  return "scheduled validate";
}
