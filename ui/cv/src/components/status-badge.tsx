"use client";

import { Badge } from "@/components/ui/badge";
import { useT } from "@/lib/i18n";

/** The state of a request as a badge, in the reader's language. */
export function StatusBadge({ status }: { status: string }) {
  const t = useT();
  const variant =
    status === "approved"
      ? "default"
      : status === "requested"
        ? "secondary"
        : status === "none"
          ? "outline"
          : "destructive";
  return (
    <Badge variant={variant} className="capitalize">
      {t(`status.${status}`)}
    </Badge>
  );
}
