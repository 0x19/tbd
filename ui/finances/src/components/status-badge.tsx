import { Badge } from "@/components/ui/badge";
import { useT } from "@/lib/i18n";
import { cn } from "@/lib/utils";

const GOOD =
  "border-transparent bg-emerald-600/12 text-emerald-700 dark:bg-emerald-500/15 dark:text-emerald-300";
const BAD = "border-transparent bg-destructive/12 text-destructive";
const WARN = "border-transparent bg-amber-500/15 text-amber-700 dark:text-amber-300";
const MUTED = "border-transparent bg-muted text-muted-foreground";
const SOLID = "border-transparent bg-foreground text-background";

/** A sync outcome or a connection status as a chip, the way the kit does them. */
const STYLE: Record<string, string> = {
  ok: GOOD,
  linked: GOOD,
  authorized: GOOD,
  running: WARN,
  syncing: WARN,
  partial: WARN,
  interrupted: MUTED,
  booked: GOOD,
  declared: SOLID,
  inferred: MUTED,
  pending: WARN,
  rate_limited: WARN,
  backing_off: WARN,
  transport: WARN,
  expired: BAD,
  revoked: BAD,
  failed: BAD,
  consent_invalid: BAD,
  error: BAD,
};

export function StatusBadge({ status, className }: { status: string; className?: string }) {
  const t = useT();
  const s = status || "never";
  const word = t(`status.${s}`);
  return (
    <Badge variant="outline" className={cn("gap-1.5", STYLE[s] ?? MUTED, className)}>
      {word === `status.${s}` ? s.replace(/_/g, " ") : word}
    </Badge>
  );
}

/** Green / amber / grey dot with a word, like "● Healthy" in the kit lists. */
export function Dot({ tone, label }: { tone: "good" | "warn" | "bad" | "off"; label: string }) {
  const color =
    tone === "good"
      ? "bg-emerald-500"
      : tone === "warn"
        ? "bg-amber-500"
        : tone === "bad"
          ? "bg-destructive"
          : "bg-muted-foreground/40";
  return (
    <span className="inline-flex items-center gap-1.5 text-sm">
      <span className={cn("size-1.5 rounded-full", color)} />
      {label}
    </span>
  );
}
