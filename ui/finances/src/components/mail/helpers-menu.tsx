"use client";

// The helpers as a menu beside the body: every `{{Name}}` with what it
// becomes right now, grouped; a click inserts it where the caret is. The
// invoice group shows only when an invoice is being sent.
import { Braces } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { useT } from "@/lib/i18n";
import { HELPER_GROUPS, helpers, type TemplateContext } from "@/lib/mail-template";

export function HelpersMenu({ ctx, onInsert }: { ctx: TemplateContext; onInsert: (token: string) => void }) {
  const t = useT();
  const values = helpers(ctx);
  return (
    <Popover>
      <PopoverTrigger asChild>
        <Button size="sm" variant="ghost" className="h-7 gap-1 px-2 text-xs">
          <Braces className="size-3.5" /> {t("mail.insert_helper")}
        </Button>
      </PopoverTrigger>
      <PopoverContent align="end" className="w-96 p-0">
        <p className="text-muted-foreground border-b px-3 py-2 text-xs">{t("mail.helpers_hint")}</p>
        <div className="max-h-80 overflow-y-auto py-1">
          {HELPER_GROUPS.filter((g) => g.id !== "invoice" || ctx.invoice).map((g) => (
            <div key={g.id} className="py-1">
              <p className="text-muted-foreground px-3 py-1 text-[10px] font-medium tracking-wide uppercase">
                {t(`mail.helper_group.${g.id}`)}
              </p>
              {g.names.map((name) => {
                const v = values[name] ?? "";
                const shown = name === "Summary" ? (v.split("\n")[0] ?? "") : v;
                return (
                  <button
                    key={name}
                    type="button"
                    className="hover:bg-muted/60 grid w-full grid-cols-[9rem_1fr] items-baseline gap-2 px-3 py-1.5 text-left text-xs"
                    onClick={() => onInsert(`{{${name}}}`)}
                  >
                    <span className="font-mono">{`{{${name}}}`}</span>
                    <span className="text-muted-foreground truncate">{shown || "—"}</span>
                  </button>
                );
              })}
            </div>
          ))}
        </div>
      </PopoverContent>
    </Popover>
  );
}
