"use client";

// The controls every listening playground shares: the status line, the
// input level (so a silent page is visibly silent rather than broken), the
// microphone picker once the browser will name them, and start and stop.

import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

import type { Mic } from "./use-mic";

export function MicStatus({ mic, hearing }: { mic: Mic; hearing?: string | null }) {
  return (
    <span className="text-muted-foreground flex items-center gap-2 font-mono text-[11px] uppercase">
      <span
        className={cn(
          "size-2 rounded-full",
          mic.status === "listening" ? "bg-emerald-500" : "bg-muted-foreground/40",
          mic.status === "listening" && !hearing && "animate-pulse",
        )}
      />
      {mic.status === "idle" && "not listening"}
      {mic.status === "starting" && "asking for the microphone"}
      {mic.status === "listening" && (hearing ?? "listening")}
      {mic.status === "blocked" && "microphone blocked"}
      {mic.status === "unsupported" && "no microphone here"}
    </span>
  );
}

export function MicPanel({ mic, children }: { mic: Mic; children?: React.ReactNode }) {
  const quiet = mic.level < 0.0015;
  const silent = mic.level < 0.0005;
  return (
    <>
      {mic.status === "listening" ? (
        <div className="mt-6">
          <div className="text-muted-foreground flex items-center justify-between font-mono text-[11px] tracking-[0.18em] uppercase">
            <span>Input level</span>
            <span>{silent ? "nothing reaching the page" : quiet ? "very quiet" : "hearing sound"}</span>
          </div>
          <div className="bg-muted mt-2 h-1.5 w-full overflow-hidden rounded-full">
            <div
              className={cn(
                "h-full rounded-full transition-[width] duration-75",
                quiet ? "bg-muted-foreground/50" : "bg-emerald-500",
              )}
              style={{ width: `${Math.min(100, Math.sqrt(mic.level) * 220)}%` }}
            />
          </div>
          {mic.inputs.length > 1 ? (
            <label className="text-muted-foreground mt-3 flex items-center gap-2 text-sm">
              <span className="shrink-0">Microphone</span>
              <select
                className="bg-background min-w-0 flex-1 rounded-md border px-2 py-1 text-sm"
                value={mic.inputId}
                onChange={(e) => void mic.start(e.target.value)}
              >
                {mic.inputs.map((d) => (
                  <option key={d.id} value={d.id}>
                    {d.label}
                  </option>
                ))}
              </select>
            </label>
          ) : null}
          {silent ? (
            <p className="text-muted-foreground mt-2 text-sm">
              The browser is listening but no sound arrives. Try another microphone above, or check the system
              input level and that nothing else holds the microphone.
            </p>
          ) : null}
        </div>
      ) : null}

      <div className="mt-8 flex flex-wrap items-center gap-3">
        {mic.status === "listening" || mic.status === "starting" ? (
          <Button variant="outline" onClick={mic.stop}>
            Stop listening
          </Button>
        ) : (
          <Button onClick={() => void mic.start()}>Start listening</Button>
        )}
        {children}
        {mic.status === "blocked" ? (
          <p className="text-muted-foreground text-sm">
            The browser refused the microphone. Allow it for this site and press start again.
          </p>
        ) : null}
        {mic.status === "unsupported" ? (
          <p className="text-muted-foreground text-sm">
            This browser has no microphone access, or the page is not on https.
          </p>
        ) : null}
      </div>
    </>
  );
}
