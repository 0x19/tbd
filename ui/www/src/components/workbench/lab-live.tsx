"use client";

import { LivePanel } from "@/components/workbench/live-panel";
import { fig, useArena } from "@/lib/arena";
import { useT } from "@/lib/i18n";
import { useMeState } from "@/lib/me";

/**
 * The arena on a lab's own page, full width. While the lab is private the
 * arena answers only an admin, so anyone else is told so instead of shown a
 * panel that never fills.
 */
export function LabLive() {
  const t = useT();
  const { me, known } = useMeState();
  const admin = me?.role === "admin";
  const arena = useArena(admin);
  if (!known) return null;
  if (!admin) return <p className="text-muted-foreground text-sm">{t("wb.live.admins")}</p>;
  return <LivePanel arena={arena} wide />;
}

/** One line of the arena for a lab's card: each tier's state, queue and speed, and the sandbox's. */
export function LabLiveStrip() {
  const { me } = useMeState();
  const arena = useArena(me?.role === "admin");
  const s = arena.snapshot;
  if (!s) return null;
  return (
    <p className="text-muted-foreground flex flex-wrap gap-x-4 gap-y-1 font-mono text-[11px] tabular-nums">
      {s.tiers.map((tier) => (
        <span key={tier.tier} className="flex items-center gap-1.5">
          <span
            className={
              tier.up ? "size-1.5 rounded-full bg-emerald-500" : "bg-destructive size-1.5 rounded-full"
            }
          />
          {tier.tier} {tier.in_flight}/{tier.max_in_flight}
          {tier.waiting ? ` +${tier.waiting}` : ""} · {fig(tier.tokens_per_second, 1, " tok/s")}
        </span>
      ))}
      {s.runner ? (
        <span className="flex items-center gap-1.5">
          <span
            className={
              s.runner.up ? "size-1.5 rounded-full bg-emerald-500" : "bg-destructive size-1.5 rounded-full"
            }
          />
          sandbox {s.runner.in_flight ?? 0}/{s.runner.max_in_flight ?? 0} ·{" "}
          {fig(s.runner.runs_per_minute, 1, " runs/min")}
        </span>
      ) : null}
    </p>
  );
}
