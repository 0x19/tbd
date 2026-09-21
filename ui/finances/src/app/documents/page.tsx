"use client";

// Receipts: everything pulled from linked mailboxes or uploaded, as a ledger
// for daily use. The server reads each PDF's text for vendor, date, amount
// and number and says how it found each one; the page never presents a
// guess as a fact: the numbers at the top add up only what was read or
// corrected, a guessed amount is marked as one in the row, and the receipts
// that need a person are one click away. Filters live in the URL so a view
// can be shared; a correction made in the sheet is declared and outlives
// any re-read. Search reaches the text.
import { AlertTriangle, FileText, HelpCircle, Receipt, Search, UserCheck, X } from "lucide-react";
import { useRouter, useSearchParams } from "next/navigation";
import { Suspense, useEffect, useMemo, useRef, useState } from "react";

import { useFinance } from "@/app/providers";
import { ReceiptSheet } from "@/components/documents/receipt-sheet";
import { Provenance, ReceiptStatusChip } from "@/components/documents/status";
import { KpiStrip, PageTitle } from "@/components/kit";
import { ScopeToggle } from "@/components/scope-toggle";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Skeleton } from "@/components/ui/skeleton";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { UploadReceipt } from "@/components/upload-receipt";
import { api } from "@/lib/api/client";
import { useFetch } from "@/lib/api/hooks";
import type { Document } from "@/lib/api/schema";
import { day, money, monthLabel, monthsBefore, thisMonth, when } from "@/lib/format";
import { useT } from "@/lib/i18n";
import { monthBounds, needsLook, type ReceiptStatus, statusOf, sure, totals } from "@/lib/receipts";
import { cn } from "@/lib/utils";

/** The server answers up to this many for the filters; the page slices. */
const FETCH = 200;
const PAGE = 25;

type View = "all" | "look" | "corrected" | "unpriced";
const VIEWS: { id: View; match: (d: Document) => boolean }[] = [
  { id: "all", match: () => true },
  { id: "look", match: needsLook },
  { id: "corrected", match: (d) => d.declared },
  { id: "unpriced", match: (d) => !d.total_minor },
];

export default function DocumentsPage() {
  return (
    <Suspense>
      <Documents />
    </Suspense>
  );
}

function Documents() {
  const t = useT();
  const router = useRouter();
  const params = useSearchParams();
  const { partyIds, partyName, multi } = useFinance();
  const key = partyIds.join(",");

  const q = params.get("q") ?? "";
  const vendor = params.get("vendor") ?? "";
  const month = params.get("month") ?? "";
  const view = (params.get("view") as View) || "all";
  const page = Math.max(1, Number(params.get("page") ?? "1") || 1);
  const [search, setSearch] = useState(q);
  const searchRef = useRef<HTMLInputElement>(null);
  const set = (patch: Record<string, string>) => {
    const next = new URLSearchParams(params.toString());
    for (const [k, v] of Object.entries(patch)) {
      if (v) next.set(k, v);
      else next.delete(k);
    }
    if (!("page" in patch)) next.delete("page");
    const s = next.toString();
    router.replace(s ? `/documents/?${s}` : "/documents/");
  };
  useEffect(() => {
    const h = setTimeout(() => {
      if (search.trim() !== q) set({ q: search.trim() });
    }, 250);
    return () => clearTimeout(h);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [search]);
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const el = e.target as HTMLElement | null;
      if (el && (el.tagName === "INPUT" || el.tagName === "TEXTAREA" || el.isContentEditable)) return;
      if (e.key === "/") {
        e.preventDefault();
        searchRef.current?.focus();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  const months = useMemo(() => Array.from({ length: 24 }, (_, i) => monthsBefore(thisMonth(), i)), []);
  const bounds = month ? monthBounds(month) : {};
  const docs = useFetch(
    () => api.documents({ party_ids: partyIds, q, vendor, ...bounds, limit: FETCH }),
    30_000,
    [key, q, vendor, month],
  );
  const all = useMemo(() => docs.data?.documents ?? [], [docs.data]);
  const counts = useMemo(
    () => Object.fromEntries(VIEWS.map((v) => [v.id, all.filter(v.match).length])) as Record<View, number>,
    [all],
  );
  const rows = useMemo(
    () => all.filter(VIEWS.find((v) => v.id === view)?.match ?? (() => true)),
    [all, view],
  );
  const pages = Math.max(1, Math.ceil(rows.length / PAGE));
  const shown = rows.slice((page - 1) * PAGE, page * PAGE);
  const sums = useMemo(() => totals(all), [all]);
  const truncated = (docs.data?.total ?? 0) > all.length;
  const filtered = Boolean(q || vendor || month || view !== "all");

  const [openId, setOpenId] = useState<string | null>(params.get("open"));
  const open = (id: string | null) => {
    setOpenId(id);
    set({ open: id ?? "", page: String(page) });
  };
  const [uploadParty, setUploadParty] = useState("");
  const vendors = docs.data?.vendors ?? [];

  const mainSum = sums.sums[0];
  const otherSums = sums.sums.slice(1);

  return (
    <>
      <PageTitle title={t("documents.title")} description={t("documents.description")}>
        <ScopeToggle className="md:hidden" />
        <div className="flex items-center gap-2">
          {partyIds.length > 1 ? (
            <Select value={uploadParty} onValueChange={setUploadParty}>
              <SelectTrigger className="w-44">
                <SelectValue placeholder={t("documents.upload_for")} />
              </SelectTrigger>
              <SelectContent>
                {partyIds.map((id) => (
                  <SelectItem key={id} value={id}>
                    {partyName(id)}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          ) : null}
          <UploadReceipt
            partyId={uploadParty || partyIds[0] || ""}
            onUploaded={(doc) => {
              docs.reload();
              open(doc.id);
            }}
          />
        </div>
      </PageTitle>

      {docs.loading && !docs.data ? (
        <Skeleton className="h-28 w-full" />
      ) : (
        <KpiStrip
          items={[
            {
              icon: Receipt,
              label: filtered ? t("documents.kpi.in_view") : t("documents.kpi.all"),
              value: all.length,
              hint: truncated
                ? t("documents.kpi.truncated", { n: docs.data?.total ?? 0 })
                : month
                  ? monthLabel(month)
                  : t("documents.kpi.all_hint"),
            },
            {
              icon: FileText,
              label: t("documents.kpi.sum"),
              value: mainSum ? money(mainSum[1].toString(), mainSum[0]) : money(0, "EUR"),
              hint:
                [
                  otherSums.length ? otherSums.map(([c, m]) => money(m.toString(), c)).join(" + ") : "",
                  sums.guessed || sums.missing
                    ? t("documents.kpi.sum_hint", { g: sums.guessed, m: sums.missing })
                    : "",
                ]
                  .filter(Boolean)
                  .join(" · ") || t("documents.kpi.sum_all_read"),
            },
            {
              icon: AlertTriangle,
              label: t("documents.kpi.look"),
              value: counts.look,
              hint: counts.look ? t("documents.kpi.look_hint") : t("documents.kpi.look_none"),
            },
            {
              icon: UserCheck,
              label: t("documents.kpi.corrected"),
              value: counts.corrected,
              hint: t("documents.kpi.corrected_hint"),
            },
          ]}
        />
      )}

      <Card>
        <CardContent className="space-y-3 p-4">
          <div className="flex flex-wrap items-center gap-2">
            <Tabs value={view} onValueChange={(v) => set({ view: v === "all" ? "" : v })}>
              <TabsList>
                {VIEWS.map((v) => (
                  <TabsTrigger key={v.id} value={v.id} className="gap-1.5">
                    {t(`documents.view.${v.id}`)}
                    <span className="text-muted-foreground font-mono text-[10px] tabular-nums">
                      {counts[v.id]}
                    </span>
                  </TabsTrigger>
                ))}
              </TabsList>
            </Tabs>
            <div className="relative min-w-56 flex-1">
              <Search className="text-muted-foreground pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2" />
              <Input
                ref={searchRef}
                value={search}
                onChange={(e) => setSearch(e.target.value)}
                placeholder={t("documents.search_placeholder")}
                className="pl-8"
              />
            </div>
            <Select value={vendor || "any"} onValueChange={(v) => set({ vendor: v === "any" ? "" : v })}>
              <SelectTrigger className="w-48">
                <SelectValue placeholder={t("common.vendor")} />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="any">{t("documents.all_vendors")}</SelectItem>
                {vendors.map((v) => (
                  <SelectItem key={v.vendor} value={v.vendor}>
                    {v.vendor} · {v.count}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Select value={month || "any"} onValueChange={(v) => set({ month: v === "any" ? "" : v })}>
              <SelectTrigger className="w-40">
                <SelectValue placeholder={t("common.month")} />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="any">{t("common.any_month")}</SelectItem>
                {months.map((m) => (
                  <SelectItem key={m} value={m}>
                    {monthLabel(m)}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            {filtered ? (
              <Button
                variant="ghost"
                size="sm"
                onClick={() => {
                  setSearch("");
                  set({ q: "", vendor: "", month: "", view: "" });
                }}
              >
                <X /> {t("documents.clear")}
              </Button>
            ) : null}
          </div>

          {docs.error ? (
            <p className="text-destructive p-4 text-sm">{docs.error}</p>
          ) : docs.loading && !docs.data ? (
            <Skeleton className="h-64 w-full" />
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="w-24">{t("common.date")}</TableHead>
                  <TableHead>{t("common.vendor")}</TableHead>
                  <TableHead className="hidden md:table-cell">{t("common.number")}</TableHead>
                  <TableHead className="hidden lg:table-cell">{t("documents.source")}</TableHead>
                  <TableHead className="w-28">{t("documents.col.status")}</TableHead>
                  <TableHead className="w-32 text-right">{t("common.amount")}</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {shown.map((d) => (
                  <Row key={d.id} d={d} multi={multi} partyName={partyName} onOpen={() => open(d.id)} />
                ))}
                {docs.data && shown.length === 0 ? (
                  <TableRow>
                    <TableCell colSpan={6} className="text-muted-foreground py-12 text-center text-sm">
                      {filtered ? t("common.nothing_matches") : t("documents.nothing_pulled")}
                    </TableCell>
                  </TableRow>
                ) : null}
              </TableBody>
            </Table>
          )}

          {docs.data ? (
            <div className="text-muted-foreground flex flex-wrap items-center justify-between gap-2 text-xs">
              <span>
                {t("documents.range", {
                  from: rows.length ? (page - 1) * PAGE + 1 : 0,
                  to: Math.min(page * PAGE, rows.length),
                  n: rows.length,
                })}
                {truncated ? ` · ${t("documents.kpi.truncated", { n: docs.data.total })}` : ""}
              </span>
              {pages > 1 ? (
                <span className="flex items-center gap-1">
                  <Button
                    variant="outline"
                    size="sm"
                    disabled={page <= 1}
                    onClick={() => set({ page: String(page - 1) })}
                  >
                    ‹
                  </Button>
                  <span className="px-2 tabular-nums">
                    {page} / {pages}
                  </span>
                  <Button
                    variant="outline"
                    size="sm"
                    disabled={page >= pages}
                    onClick={() => set({ page: String(page + 1) })}
                  >
                    ›
                  </Button>
                </span>
              ) : null}
            </div>
          ) : null}
        </CardContent>
      </Card>

      <ReceiptSheet
        id={openId}
        onClose={() => open(null)}
        onChanged={docs.reload}
        vendors={vendors.map((v) => v.vendor)}
      />
    </>
  );
}

function Row({
  d,
  multi,
  partyName,
  onOpen,
}: {
  d: Document;
  multi: boolean;
  partyName: (id: string) => string;
  onOpen: () => void;
}) {
  const t = useT();
  const s = d.sources[0];
  const status: ReceiptStatus = statusOf(d);
  const amountSure = d.declared || sure(d.found_by.amount);
  return (
    <TableRow className="cursor-pointer" onClick={onOpen}>
      <TableCell className="text-xs whitespace-nowrap tabular-nums">
        {d.doc_date ? (
          <span>
            {day(d.doc_date)}
            <Provenance by={d.found_by.date} />
          </span>
        ) : (
          <span className="text-muted-foreground" title={t("documents.found.received")}>
            {when(s?.received_at ?? d.created_at).slice(0, 11)}
          </span>
        )}
      </TableCell>
      <TableCell className="max-w-64">
        <div className="flex items-center gap-1.5">
          <span className={cn("truncate", d.vendor ? "font-medium" : "text-muted-foreground italic")}>
            {d.vendor || t("documents.unknown_vendor")}
          </span>
          {d.vendor && !d.declared ? <Provenance by={d.found_by.vendor} /> : null}
          {multi ? (
            <Badge variant="secondary" className="shrink-0 text-[10px]">
              {partyName(d.party_id)}
            </Badge>
          ) : null}
        </div>
        <div className="text-muted-foreground max-w-64 truncate text-[11px] lg:hidden">
          {s?.subject || d.filename}
        </div>
      </TableCell>
      <TableCell className="text-muted-foreground hidden font-mono text-xs md:table-cell">
        {d.invoice_no || "—"}
      </TableCell>
      <TableCell className="text-muted-foreground hidden max-w-96 lg:table-cell">
        <span className="block truncate text-xs" title={s ? `${s.sender} · ${s.subject}` : d.filename}>
          {s ? s.subject || d.filename : t("documents.origin_upload_short")}
        </span>
        <span className="block truncate text-[11px]">{s ? s.sender : d.filename}</span>
      </TableCell>
      <TableCell>
        <ReceiptStatusChip status={status} />
      </TableCell>
      <TableCell className="text-right font-mono whitespace-nowrap tabular-nums">
        {d.total_minor ? (
          <span
            className={cn(
              !amountSure && "text-muted-foreground underline decoration-dotted underline-offset-4",
            )}
            title={amountSure ? undefined : t("documents.amount_guessed")}
          >
            {money(d.total_minor, d.currency || "EUR")}
          </span>
        ) : status === "reading" ? (
          <span className="text-muted-foreground text-xs italic">{t("documents.reading")}</span>
        ) : (
          <span
            className="text-muted-foreground inline-flex items-center gap-1 text-xs"
            title={t("documents.amount_missing")}
          >
            <HelpCircle className="size-3" /> —
          </span>
        )}
      </TableCell>
    </TableRow>
  );
}
