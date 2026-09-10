"use client";

import { useState } from "react";
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
import { Field } from "@/components/field";
import type { Behavior, ErrorKind } from "@/lib/api/schema";

type Kind = Behavior["type"];

const KINDS: { value: Kind; label: string; help: string }[] = [
  { value: "healthy", label: "healthy", help: "Normal operation." },
  {
    value: "slow",
    label: "slow",
    help: "Add latency to every request, plus uniform jitter.",
  },
  {
    value: "hang",
    label: "hang",
    help: "Never answer; clients hit their timeout.",
  },
  {
    value: "error",
    label: "error",
    help: "Fail a fraction of requests with a gRPC status.",
  },
  {
    value: "delayed_failure",
    label: "delayed failure",
    help: "Healthy for a while, then another behaviour.",
  },
];

const ERROR_KINDS: ErrorKind[] = ["unavailable", "internal", "overloaded", "timeout"];

/** The same JSON the timeline uses, built from a small form. */
export function BehaviorDialog({
  open,
  onOpenChange,
  instance,
  current,
  onApply,
}: {
  open: boolean;
  onOpenChange: (o: boolean) => void;
  instance: string;
  current: Behavior | null;
  onApply: (b: Behavior) => Promise<void>;
}) {
  const [kind, setKind] = useState<Kind>(current?.type ?? "healthy");
  const [latency, setLatency] = useState("200ms");
  const [jitter, setJitter] = useState("50ms");
  const [errorKind, setErrorKind] = useState<ErrorKind>("unavailable");
  const [rate, setRate] = useState("0.5");
  const [message, setMessage] = useState("injected from the UI");
  const [healthyFor, setHealthyFor] = useState("5s");
  const [thenKind, setThenKind] = useState<ErrorKind>("unavailable");
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState<string | null>(null);

  const build = (): Behavior => {
    switch (kind) {
      case "healthy":
        return { type: "healthy" };
      case "hang":
        return { type: "hang" };
      case "slow":
        return { type: "slow", latency, jitter };
      case "error":
        return {
          type: "error",
          kind: errorKind,
          rate: Number(rate),
          message,
        };
      case "delayed_failure":
        return {
          type: "delayed_failure",
          healthy_for: healthyFor,
          then: { type: "error", kind: thenKind, rate: 1, message },
        };
    }
  };

  const apply = async () => {
    setBusy(true);
    setErr(null);
    try {
      await onApply(build());
      onOpenChange(false);
    } catch (e) {
      setErr(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  const select = (
    <select
      className="h-8 w-full rounded-md border bg-background px-2 text-sm"
      value={kind}
      onChange={(e) => setKind(e.target.value as Kind)}
    >
      {KINDS.map((k) => (
        <option key={k.value} value={k.value}>
          {k.label}
        </option>
      ))}
    </select>
  );

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Behaviour of {instance}</DialogTitle>
          <DialogDescription>{KINDS.find((k) => k.value === kind)?.help}</DialogDescription>
        </DialogHeader>
        <div className="grid gap-3">
          <Field label="Behaviour">{select}</Field>
          {kind === "slow" ? (
            <div className="grid grid-cols-2 gap-3">
              <Field label="Latency">
                <Input value={latency} onChange={(e) => setLatency(e.target.value)} placeholder="200ms" />
              </Field>
              <Field label="Jitter">
                <Input value={jitter} onChange={(e) => setJitter(e.target.value)} placeholder="50ms" />
              </Field>
            </div>
          ) : null}
          {kind === "error" ? (
            <>
              <div className="grid grid-cols-2 gap-3">
                <Field label="Kind">
                  <KindSelect value={errorKind} onChange={setErrorKind} />
                </Field>
                <Field label="Rate (0..1)">
                  <Input value={rate} onChange={(e) => setRate(e.target.value)} inputMode="decimal" />
                </Field>
              </div>
              <Field label="Message">
                <Input value={message} onChange={(e) => setMessage(e.target.value)} />
              </Field>
            </>
          ) : null}
          {kind === "delayed_failure" ? (
            <div className="grid grid-cols-2 gap-3">
              <Field label="Healthy for">
                <Input value={healthyFor} onChange={(e) => setHealthyFor(e.target.value)} placeholder="5s" />
              </Field>
              <Field label="Then fail with">
                <KindSelect value={thenKind} onChange={setThenKind} />
              </Field>
            </div>
          ) : null}
          <pre className="overflow-x-auto rounded-md bg-muted p-2 text-xs">{JSON.stringify(build())}</pre>
          {err ? <p className="text-sm text-destructive">{err}</p> : null}
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button onClick={apply} disabled={busy}>
            Apply
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function KindSelect({ value, onChange }: { value: ErrorKind; onChange: (k: ErrorKind) => void }) {
  return (
    <select
      className="h-8 w-full rounded-md border bg-background px-2 text-sm"
      value={value}
      onChange={(e) => onChange(e.target.value as ErrorKind)}
    >
      {ERROR_KINDS.map((k) => (
        <option key={k} value={k}>
          {k}
        </option>
      ))}
    </select>
  );
}
