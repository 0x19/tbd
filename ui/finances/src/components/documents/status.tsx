"use client";

// The receipt's state as a chip, and a field's provenance as a small word
// beside its value. The same words the sheet uses, so the list and the
// detail never disagree about what a guess is.
import { AlertTriangle, CheckCircle2, CircleDashed, HelpCircle, Loader2, UserCheck } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { useT } from "@/lib/i18n";
import { foundKey, type ReceiptStatus, sure } from "@/lib/receipts";
import { cn } from "@/lib/utils";

const LOOK: Record<ReceiptStatus, { icon: typeof CheckCircle2; className: string }> = {
  reading: { icon: Loader2, className: "text-muted-foreground" },
  unreadable: { icon: AlertTriangle, className: "text-destructive bg-destructive/10" },
  missing: { icon: CircleDashed, className: "text-amber-700 bg-amber-500/12 dark:text-amber-300" },
  guessed: { icon: HelpCircle, className: "text-amber-700 bg-amber-500/12 dark:text-amber-300" },
  read: { icon: CheckCircle2, className: "text-emerald-700 bg-emerald-600/12 dark:text-emerald-300" },
  corrected: { icon: UserCheck, className: "text-sky-700 bg-sky-600/12 dark:text-sky-300" },
};

export function ReceiptStatusChip({ status, className }: { status: ReceiptStatus; className?: string }) {
  const t = useT();
  const { icon: Icon, className: look } = LOOK[status];
  return (
    <Badge
      variant="outline"
      className={cn("gap-1 border-transparent text-[10px] font-medium", look, className)}
    >
      <Icon className={cn("size-3", status === "reading" && "animate-spin")} />
      {t(`documents.status.${status}`)}
    </Badge>
  );
}

/** "read", "guessed", "set by you" beside a value; the sure ones stay quiet
 *  unless asked to show, so the eye lands on what is not. */
export function Provenance({ by, always = false }: { by: string | undefined; always?: boolean }) {
  const t = useT();
  if (!always && sure(by)) return null;
  return (
    <span
      className={cn(
        "ml-1.5 rounded px-1 py-px text-[10px] whitespace-nowrap",
        sure(by) ? "text-muted-foreground bg-muted" : "bg-amber-500/12 text-amber-700 dark:text-amber-300",
      )}
    >
      {t(foundKey(by))}
    </span>
  );
}
