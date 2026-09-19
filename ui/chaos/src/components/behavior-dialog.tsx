"use client";

import { useState } from "react";

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

/** Where the fault bites: the request adapter, or the service's own store. */
export type Surface = "service" | "store";

const SURFACES: { value: Surface; label: string; help: string }[] = [
  {
    value: "service",
    label: "the service",
    help: "The request adapter refuses the call before anything runs.",
  },
  {
    value: "store",
    label: "its store",
    help: "The store fails as a database does: a read before it runs, a write after it committed, so the caller loses the acknowledgement of something that stands.",
  },
];

/** The same JSON the timeline uses, built from a small form. */
export function BehaviorDialog({
  open,
  onOpenChange,
  instance,
  current,
  currentStore,
  storeFault = false,
  onApply,
}: {
  open: boolean;
  onOpenChange: (o: boolean) => void;
  instance: string;
  current: Behavior | null;
  /** The store's behaviour now, for services that have one. */
  currentStore?: Behavior | null;
  /** The kind has a store chaos can fail (`set_store_behavior`). */
  storeFault?: boolean;
  onApply: (b: Behavior, surface: Surface) => Promise<void>;
}) {
  const [surface, setSurface] = useState<Surface>("service");
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
      await onApply(build(), surface);
      onOpenChange(false);
    } catch (e) {
      setErr(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  const select = (
    <select
      aria-label="Behaviour"
      className="bg-background h-8 w-full rounded-md border px-2 text-sm"
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
          <DialogDescription>
            {storeFault ? `${SURFACES.find((s) => s.value === surface)?.help} ` : ""}
            {KINDS.find((k) => k.value === kind)?.help}
          </DialogDescription>
        </DialogHeader>
        <div className="grid gap-3">
          {storeFault ? (
            <Field label="Fault">
              <select
                aria-label="What to fault"
                className="bg-background h-8 w-full rounded-md border px-2 text-sm"
                value={surface}
                onChange={(e) => {
                  const next = e.target.value as Surface;
                  setSurface(next);
                  setKind((next === "store" ? currentStore?.type : current?.type) ?? "healthy");
                }}
              >
                {SURFACES.map((s) => (
                  <option key={s.value} value={s.value}>
                    {s.label}
                  </option>
                ))}
              </select>
            </Field>
          ) : null}
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
          <pre className="bg-muted overflow-x-auto rounded-md p-2 text-xs">{JSON.stringify(build())}</pre>
          {err ? <p className="text-destructive text-sm">{err}</p> : null}
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
      className="bg-background h-8 w-full rounded-md border px-2 text-sm"
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
