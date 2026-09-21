"use client";

import { Badge } from "@/components/ui/badge";
import { useT } from "@/lib/i18n";
import { cn } from "@/lib/utils";

/** The form's own short name as a badge; the long name on hover. */
export function FormBadge({ form, className }: { form: string; className?: string }) {
  const t = useT();
  return (
    <Badge variant="outline" className={cn("font-mono", className)} title={t(`filings.form_long.${form}`)}>
      {t(`filings.form.${form}`)}
    </Badge>
  );
}
