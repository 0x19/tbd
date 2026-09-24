"use client";

// Pick receipts of the sender's party to attach: a search over the ledger,
// a click toggles. The service fetches the bytes at send time.
import { FileText, Search } from "lucide-react";
import { useState } from "react";

import { Badge } from "@/components/ui/badge";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { api } from "@/lib/api/client";
import { useFetch } from "@/lib/api/hooks";
import { day, money } from "@/lib/format";
import { useT } from "@/lib/i18n";
import { cn } from "@/lib/utils";

import type { Attached } from "./draft";

export function AttachDialog({
  open,
  partyId,
  chosen,
  onClose,
  onPick,
}: {
  open: boolean;
  partyId: string;
  chosen: Attached[];
  onClose: () => void;
  onPick: (d: Attached) => void;
}) {
  const t = useT();
  const [q, setQ] = useState("");
  const found = useFetch(
    () => (open && partyId ? api.documents({ party_ids: [partyId], q, limit: 40 }) : Promise.resolve(null)),
    0,
    [open, q, partyId],
  );
  const docs = found.data?.documents ?? [];
  return (
    <Dialog open={open} onOpenChange={(o) => !o && onClose()}>
      <DialogContent className="sm:max-w-xl">
        <DialogHeader>
          <DialogTitle>{t("mail.attach")}</DialogTitle>
          <DialogDescription>{t("mail.attach_search")}</DialogDescription>
        </DialogHeader>
        <div className="relative">
          <Search className="text-muted-foreground absolute top-1/2 left-2.5 size-4 -translate-y-1/2" />
          <Input value={q} onChange={(e) => setQ(e.target.value)} className="pl-8" autoFocus />
        </div>
        <div className="-mx-1 max-h-96 space-y-1 overflow-y-auto px-1">
          {docs.map((d) => {
            const on = chosen.some((x) => x.id === d.id);
            return (
              <button
                key={d.id}
                type="button"
                className={cn(
                  "hover:bg-muted/50 flex w-full items-center gap-3 rounded-md border px-3 py-2 text-left text-sm",
                  on && "border-primary bg-primary/5",
                )}
                onClick={() => onPick({ id: d.id, filename: d.filename, vendor: d.vendor })}
              >
                <FileText className="text-muted-foreground size-4 shrink-0" />
                <span className="min-w-0 flex-1">
                  <span className="block truncate font-medium">{d.vendor || d.filename}</span>
                  <span className="text-muted-foreground block truncate text-xs">
                    {d.doc_date ? day(d.doc_date) : "—"} · {d.filename}
                  </span>
                </span>
                <span className="shrink-0 font-mono text-xs tabular-nums">
                  {d.total_minor && d.total_minor !== "0" ? money(d.total_minor, d.currency || "EUR") : ""}
                </span>
                {on ? <Badge variant="secondary">✓</Badge> : null}
              </button>
            );
          })}
          {found.data && docs.length === 0 ? (
            <p className="text-muted-foreground py-8 text-center text-sm">{t("mail.attach_none_found")}</p>
          ) : null}
        </div>
        <p className="text-muted-foreground text-xs">{t("mail.attach_chosen", { n: chosen.length })}</p>
      </DialogContent>
    </Dialog>
  );
}
