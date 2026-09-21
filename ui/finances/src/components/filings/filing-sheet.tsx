"use client";

// One filing: what the form says about itself, every value it carries with
// a label where the page knows one, one table per kind of repeated row, and
// the two actions: read the XML again, download it.
import { Download, RefreshCw } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { toast } from "sonner";

import { FormBadge } from "@/components/filings/form-badge";
import { DetailList } from "@/components/kit";
import { Button } from "@/components/ui/button";
import { Sheet, SheetContent, SheetDescription, SheetHeader, SheetTitle } from "@/components/ui/sheet";
import { Skeleton } from "@/components/ui/skeleton";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { api, blobUrl } from "@/lib/api/client";
import { describe } from "@/lib/api/hooks";
import type { Filing } from "@/lib/api/schema";
import { figure, keyOrder, labelOf } from "@/lib/filings";
import { day, when } from "@/lib/format";
import { useT } from "@/lib/i18n";

type Row = Record<string, unknown>;

export function FilingSheet({
  id,
  onClose,
  onChanged,
}: {
  id: string | null;
  onClose: () => void;
  onChanged: () => void;
}) {
  const t = useT();
  const [filing, setFiling] = useState<Filing | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState("");

  useEffect(() => {
    if (!id) {
      setFiling(null);
      setError(null);
      return;
    }
    let cancelled = false;
    setFiling(null);
    setError(null);
    api
      .filing(id)
      .then((r) => {
        if (!cancelled && r.filing) setFiling(r.filing);
      })
      .catch((e: unknown) => {
        if (!cancelled) setError(describe(e));
      });
    return () => {
      cancelled = true;
    };
  }, [id]);

  const values = useMemo(() => {
    if (!filing) return [];
    return keyOrder(Object.keys(filing.values)).map((k) => ({ k, v: filing.values[k] ?? "" }));
  }, [filing]);

  const rowGroups = useMemo(() => {
    if (!filing?.rows_json) return [];
    let rows: Row[] = [];
    try {
      const parsed: unknown = JSON.parse(filing.rows_json);
      if (Array.isArray(parsed)) rows = parsed as Row[];
    } catch {
      rows = [];
    }
    const groups = new Map<string, Row[]>();
    for (const r of rows) {
      const kind = typeof r._kind === "string" ? r._kind : "";
      groups.set(kind, [...(groups.get(kind) ?? []), r]);
    }
    return Array.from(groups.entries()).map(([kind, list]) => {
      const columns = keyOrder(
        Array.from(new Set(list.flatMap((r) => Object.keys(r).filter((k) => k !== "_kind")))),
      );
      return { kind, list, columns };
    });
  }, [filing]);

  const readAgain = async () => {
    if (!filing) return;
    setBusy("read");
    try {
      await api.extractDocument(filing.id);
      const r = await api.filing(filing.id);
      if (r.filing) setFiling(r.filing);
      toast.success(t("filings.read_again_done"));
      onChanged();
    } catch (e) {
      toast.error(describe(e));
    } finally {
      setBusy("");
    }
  };

  const download = async () => {
    if (!filing) return;
    setBusy("download");
    try {
      const r = await api.document(filing.id);
      const url = blobUrl(r.bytes, "text/xml");
      const a = document.createElement("a");
      a.href = url;
      a.download = filing.filename || `${filing.form}.xml`;
      a.click();
      setTimeout(() => URL.revokeObjectURL(url), 10_000);
    } catch (e) {
      toast.error(describe(e));
    } finally {
      setBusy("");
    }
  };

  const cell = (v: unknown): string => {
    if (v == null) return "";
    if (typeof v === "string") return v;
    return JSON.stringify(v);
  };

  return (
    <Sheet open={!!id} onOpenChange={(o) => !o && onClose()}>
      <SheetContent className="flex w-full flex-col gap-4 overflow-y-auto sm:max-w-3xl">
        <SheetHeader className="pr-8">
          <SheetTitle className="flex flex-wrap items-center gap-2">
            {filing ? (
              <>
                <FormBadge form={filing.form} />
                <span>{t(`filings.form_long.${filing.form}`)}</span>
              </>
            ) : (
              t("filings.title")
            )}
          </SheetTitle>
          <SheetDescription>
            {filing
              ? `${filing.period_from}${filing.period_to && filing.period_to !== filing.period_from ? ` – ${filing.period_to}` : ""} · ${filing.obveznik} · OIB ${filing.oib}`
              : ""}
          </SheetDescription>
        </SheetHeader>

        {error ? <p className="text-destructive text-sm">{error}</p> : null}
        {!filing && !error ? <Skeleton className="h-64 w-full" /> : null}

        {filing ? (
          <>
            {filing.error ? (
              <p className="border-destructive/40 bg-destructive/10 text-destructive rounded-md border px-3 py-2 text-sm">
                {t("filings.error", { error: filing.error })}
              </p>
            ) : null}

            <DetailList
              rows={[
                { k: t("filings.schema"), v: <span className="font-mono">{filing.schema}</span> },
                { k: t("filings.period"), v: `${filing.period_from} – ${filing.period_to}` },
                { k: t("filings.obveznik"), v: `${filing.obveznik} (${filing.oib})` },
                { k: t("filings.prepared"), v: filing.prepared_at ? when(filing.prepared_at) : "—" },
                { k: t("filings.author"), v: filing.author || "—" },
                ...(filing.report_mark ? [{ k: t("filings.report_mark"), v: filing.report_mark }] : []),
                { k: t("filings.file"), v: filing.filename },
                { k: t("documents.reader"), v: `${filing.parser_version} · ${day(filing.parsed_at)}` },
              ]}
            />

            <div className="flex flex-wrap gap-2">
              <Button size="sm" variant="outline" disabled={!!busy} onClick={() => void readAgain()}>
                <RefreshCw className="size-3" /> {t("filings.read_again")}
              </Button>
              <Button size="sm" variant="outline" disabled={!!busy} onClick={() => void download()}>
                <Download className="size-3" /> {t("filings.download")}
              </Button>
            </div>

            <section>
              <h3 className="mb-2 text-sm font-medium">{t("filings.values")}</h3>
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead className="w-48">#</TableHead>
                    <TableHead></TableHead>
                    <TableHead className="text-right">EUR</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {values.map(({ k, v }) => {
                    const label = labelOf(t, filing.form, k);
                    return (
                      <TableRow key={k}>
                        <TableCell className="font-mono text-xs">{k}</TableCell>
                        <TableCell className="text-muted-foreground">{label}</TableCell>
                        <TableCell className="text-right tabular-nums">{figure(filing.form, k, v)}</TableCell>
                      </TableRow>
                    );
                  })}
                </TableBody>
              </Table>
            </section>

            {rowGroups.map((g) => (
              <section key={g.kind}>
                <h3 className="mb-2 text-sm font-medium">
                  {t("filings.rows")} · <span className="font-mono">{g.kind}</span>
                </h3>
                <div className="overflow-x-auto">
                  <Table>
                    <TableHeader>
                      <TableRow>
                        {g.columns.map((c) => (
                          <TableHead key={c} className="font-mono text-xs">
                            {c}
                          </TableHead>
                        ))}
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {g.list.map((r, i) => (
                        <TableRow key={i}>
                          {g.columns.map((c) => (
                            <TableCell key={c} className="tabular-nums">
                              {cell(r[c])}
                            </TableCell>
                          ))}
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                </div>
              </section>
            ))}
          </>
        ) : null}
      </SheetContent>
    </Sheet>
  );
}
