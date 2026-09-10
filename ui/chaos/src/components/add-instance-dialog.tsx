"use client";

import { useState } from "react";
import { toast } from "sonner";

import { useChaos } from "@/app/providers";
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
import { api } from "@/lib/api/client";
import { describe } from "@/lib/api/hooks";
import type { InstanceInfo, KindDescriptor } from "@/lib/api/schema";
import { fieldInitial, listKinds, withCapability } from "@/lib/kinds";

type Props = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  instances: InstanceInfo[];
  onAdded: (next: InstanceInfo[]) => void;
};

/** The kind the dialog opens on: the first addable one with a dependency (a protocol), else the first addable. */
function defaultKind(addable: KindDescriptor[]): KindDescriptor | undefined {
  return addable.find((k) => k.dependency_kind) ?? addable[0];
}

/** Initial values of a kind's fields: defaults, or the first running instance a dependency can point at. */
function initialValues(kind: KindDescriptor | undefined, instances: InstanceInfo[]): Record<string, string> {
  const out: Record<string, string> = {};
  for (const f of kind?.fields ?? []) {
    out[f.name] = fieldInitial(
      f,
      instances.filter((i) => i.kind === f.of_kind && i.running).map((i) => i.name),
    );
  }
  return out;
}

/** Add a new instance of any addable kind to the running stack; the form comes from the kind's fields. */
export function AddInstanceDialog({ open, onOpenChange, instances, onAdded }: Props) {
  const { kinds } = useChaos();
  const addable = withCapability(kinds, "addable");
  const first = defaultKind(addable);
  const [kindName, setKindName] = useState(first?.name ?? "");
  const [name, setName] = useState("");
  const [values, setValues] = useState<Record<string, string>>(() => initialValues(first, instances));
  const [busy, setBusy] = useState(false);
  const kind = addable.find((k) => k.name === kindName) ?? first;

  const pickKind = (next: string) => {
    setKindName(next);
    setValues(
      initialValues(
        addable.find((k) => k.name === next),
        instances,
      ),
    );
  };

  const valid = !!kind && kind.fields.every((f) => !f.required || (values[f.name] ?? "").trim().length > 0);
  const loadTargets = listKinds(withCapability(kinds, "load_target"));

  const add = async () => {
    if (!kind) return;
    setBusy(true);
    try {
      const next = await api.stackAdd({
        kind: kind.name,
        ...(name.trim() ? { name: name.trim() } : {}),
        ...Object.fromEntries(
          Object.entries(values)
            .map(([k, v]) => [k, v.trim()])
            .filter(([, v]) => v.length > 0),
        ),
      });
      onAdded(next);
      toast.success(`added ${kind.name}`);
      onOpenChange(false);
    } catch (e) {
      toast.error(describe(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Add an instance</DialogTitle>
          <DialogDescription>
            Starts now on a free port. A dependency must name a running instance; load with no explicit
            targets spreads over every running{" "}
            {loadTargets === "none" ? "load target" : loadTargets.replace(/s$/, "")}.
          </DialogDescription>
        </DialogHeader>
        <div className="grid gap-4">
          <div className="grid gap-4 sm:grid-cols-2">
            <Field label="Kind">
              <Select value={kind?.name ?? ""} onValueChange={pickKind}>
                <SelectTrigger aria-label="Kind">
                  <SelectValue placeholder="Pick a kind" />
                </SelectTrigger>
                <SelectContent>
                  {addable.map((k) => (
                    <SelectItem key={k.name} value={k.name} data-testid={`kind-${k.name}`}>
                      {k.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </Field>
            <Field label="Name (optional)">
              <Input
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder={
                  kind ? `${kind.name}-${instances.filter((i) => i.kind === kind.name).length + 1}` : ""
                }
                className="font-mono"
              />
            </Field>
          </div>
          {kind?.fields.map((f) =>
            f.kind === "instance_of" ? (
              <Field key={f.name} label={`${f.label} (${f.required ? "required" : "optional"})`}>
                <Select
                  value={values[f.name] ?? ""}
                  onValueChange={(v) => setValues({ ...values, [f.name]: v })}
                >
                  <SelectTrigger aria-label={f.label}>
                    <SelectValue placeholder={`Pick a running ${f.of_kind ?? "instance"}`} />
                  </SelectTrigger>
                  <SelectContent>
                    {instances
                      .filter((i) => i.kind === f.of_kind && i.running)
                      .map((i) => (
                        <SelectItem key={i.name} value={i.name}>
                          {i.name}
                        </SelectItem>
                      ))}
                  </SelectContent>
                </Select>
              </Field>
            ) : (
              <Field key={f.name} label={f.label}>
                <Input
                  value={values[f.name] ?? ""}
                  onChange={(e) => setValues({ ...values, [f.name]: e.target.value })}
                  placeholder={f.default ?? (f.kind === "duration" ? "1s" : "")}
                />
              </Field>
            ),
          )}
          {kind && kind.fields.length === 0 ? (
            <p className="text-muted-foreground text-sm">
              A {kind.name} takes no settings; faults can be injected once it runs.
            </p>
          ) : null}
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button onClick={add} disabled={!valid || busy}>
            Add and start
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
