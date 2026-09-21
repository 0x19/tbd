"use client";

// The opening trial balance a person hands the books: a CSV the accountant
// printed, parsed here into signed minor units, shown with both sums and the
// difference before anything is sent, then one ImportOpeningBalances call.
// The service refuses what does not balance or names a code the chart lacks;
// its sentence is the toast.
import { FileUp } from "lucide-react";
import { useMemo, useState } from "react";
import { toast } from "sonner";

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
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { api } from "@/lib/api/client";
import { describe } from "@/lib/api/hooks";
import { parseOpening } from "@/lib/books";
import { money } from "@/lib/format";
import { useT } from "@/lib/i18n";

export function ImportOpening({
  partyId,
  year,
  hasOpening,
  onImported,
  disabled,
}: {
  partyId: string;
  year: number;
  /** Whether the year already has an opening, so the dialog can say it replaces it. */
  hasOpening: boolean;
  onImported: () => void;
  disabled?: boolean;
}) {
  const t = useT();
  const [open, setOpen] = useState(false);
  const [text, setText] = useState("");
  const [asOf, setAsOf] = useState(`${year}-01-01`);
  const [source, setSource] = useState("filed");
  const [busy, setBusy] = useState(false);
  const parsed = useMemo(() => parseOpening(text), [text]);
  const diff = parsed.debit - parsed.credit;

  const reset = () => {
    setText("");
    setAsOf(`${year}-01-01`);
    setSource("filed");
  };

  const submit = async () => {
    setBusy(true);
    try {
      const r = await api.importOpeningBalances({
        party_id: partyId,
        fiscal_year: year,
        as_of: asOf,
        source,
        rows: parsed.rows,
      });
      toast.success(
        t("books.import.done", {
          year,
          n: r.accounts,
          debit: money(r.total_debit_minor),
        }),
      );
      setOpen(false);
      reset();
      onImported();
    } catch (e) {
      toast.error(t("books.import.failed", { reason: describe(e) }));
    } finally {
      setBusy(false);
    }
  };

  return (
    <Dialog
      open={open}
      onOpenChange={(o) => {
        setOpen(o);
        if (!o) reset();
      }}
    >
      <Button size="sm" disabled={disabled || !partyId} onClick={() => setOpen(true)}>
        <FileUp className="size-3" /> {t("books.import")}
      </Button>
      <DialogContent className="max-w-xl">
        <DialogHeader>
          <DialogTitle>{t("books.import.title")}</DialogTitle>
          <DialogDescription>{t("books.import.desc")}</DialogDescription>
        </DialogHeader>
        <div className="grid gap-4">
          <div className="grid gap-1.5">
            <Label htmlFor="opening-file">{t("books.import.file")}</Label>
            <Input
              id="opening-file"
              type="file"
              accept=".csv,text/csv,text/plain"
              onChange={(e) => {
                const f = e.target.files?.[0];
                if (!f) return;
                void f.text().then(setText);
              }}
            />
            <p className="text-muted-foreground text-xs">{t("books.import.file_hint")}</p>
          </div>
          <div className="grid gap-4 sm:grid-cols-2">
            <div className="grid gap-1.5">
              <Label htmlFor="opening-as-of">{t("books.import.as_of")}</Label>
              <Input id="opening-as-of" type="date" value={asOf} onChange={(e) => setAsOf(e.target.value)} />
              <p className="text-muted-foreground text-xs">{t("books.import.as_of_hint")}</p>
            </div>
            <div className="grid gap-1.5">
              <Label>{t("books.import.source")}</Label>
              <Select value={source} onValueChange={setSource}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="filed">{t("books.import.source.filed")}</SelectItem>
                  <SelectItem value="imported">{t("books.import.source.imported")}</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>
          {parsed.rows.length > 0 ? (
            <div className="rounded-md border p-3 text-sm">
              <div className="tabular-nums">
                {t("books.import.preview", {
                  n: parsed.rows.length,
                  debit: money(parsed.debit.toString()),
                  credit: money(parsed.credit.toString()),
                })}
              </div>
              {diff !== 0n ? (
                <div className="mt-1 text-red-600 dark:text-red-400">
                  {t("books.import.off", { diff: money(diff.toString(), "EUR", { sign: true }) })}
                </div>
              ) : null}
              {hasOpening ? (
                <div className="text-muted-foreground mt-1">{t("books.import.replace", { year })}</div>
              ) : null}
            </div>
          ) : null}
          {parsed.bad.length > 0 ? (
            <div className="rounded-md border border-amber-300 p-3 text-sm dark:border-amber-700">
              <div>{t("books.import.bad", { n: parsed.bad.length })}</div>
              <pre className="text-muted-foreground mt-1 max-h-24 overflow-auto text-xs">
                {parsed.bad.slice(0, 8).join("\n")}
              </pre>
            </div>
          ) : null}
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => setOpen(false)} disabled={busy}>
            {t("common.cancel")}
          </Button>
          <Button onClick={() => void submit()} disabled={busy || parsed.rows.length === 0 || !asOf}>
            {t("books.import.go", { n: parsed.rows.length })}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
