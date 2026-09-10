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
import { api } from "@/lib/api/client";
import { describe } from "@/lib/api/hooks";
import type { InstanceInfo } from "@/lib/api/schema";

type Props = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  instances: InstanceInfo[];
  onAdded: (next: InstanceInfo[]) => void;
};

/** Add a brand-new engine or protocol to the running stack. */
export function AddInstanceDialog({ open, onOpenChange, instances, onAdded }: Props) {
  const engines = instances.filter((i) => i.kind === "engine" && i.running);
  const [kind, setKind] = useState<"engine" | "protocol">("protocol");
  const [name, setName] = useState("");
  const [engine, setEngine] = useState(engines[0]?.name ?? "");
  const [heartbeat, setHeartbeat] = useState("1s");
  const [busy, setBusy] = useState(false);

  const valid = kind === "engine" || engine.length > 0;

  const add = async () => {
    setBusy(true);
    try {
      const next = await api.stackAdd({
        kind,
        ...(name.trim() ? { name: name.trim() } : {}),
        ...(kind === "protocol" ? { engine } : { heartbeat: heartbeat.trim() || "1s" }),
      });
      onAdded(next);
      toast.success(`added ${kind}`);
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
            Starts now on a free port. A protocol forwards to one engine; load with no explicit targets
            spreads over every running protocol.
          </DialogDescription>
        </DialogHeader>
        <div className="grid gap-4">
          <div className="grid gap-4 sm:grid-cols-2">
            <Field label="Kind">
              <Select value={kind} onValueChange={(v) => setKind(v as "engine" | "protocol")}>
                <SelectTrigger aria-label="Kind">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="protocol">Protocol</SelectItem>
                  <SelectItem value="engine">Engine</SelectItem>
                </SelectContent>
              </Select>
            </Field>
            <Field label="Name (optional)">
              <Input
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder={`${kind}-${instances.filter((i) => i.kind === kind).length + 1}`}
                className="font-mono"
              />
            </Field>
          </div>
          {kind === "protocol" ? (
            <Field label="Forwards to">
              <Select value={engine} onValueChange={setEngine}>
                <SelectTrigger aria-label="Forwards to">
                  <SelectValue placeholder="Pick a running engine" />
                </SelectTrigger>
                <SelectContent>
                  {engines.map((e) => (
                    <SelectItem key={e.name} value={e.name}>
                      {e.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </Field>
          ) : (
            <Field label="Heartbeat">
              <Input value={heartbeat} onChange={(e) => setHeartbeat(e.target.value)} placeholder="1s" />
            </Field>
          )}
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
