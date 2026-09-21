"use client";

// Filings: the company's ePorezna forms, one card per form, each row a
// period with the form's headline figures as filed. Filters live in the
// URL. The strip adds up only what the listed forms carry; nothing here is
// computed from the books, it is what was filed, read from the XML.
import { AlertTriangle, FileCheck2, Landmark, Receipt, Users } from "lucide-react";
import { useRouter, useSearchParams } from "next/navigation";
import { Suspense, useMemo, useState } from "react";

import { useFinance } from "@/app/providers";
import { FilingSheet } from "@/components/filings/filing-sheet";
import { FormBadge } from "@/components/filings/form-badge";
import { KpiStrip, PageTitle } from "@/components/kit";
import { ScopeToggle } from "@/components/scope-toggle";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Skeleton } from "@/components/ui/skeleton";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { UploadFiling } from "@/components/upload-filing";
import { api } from "@/lib/api/client";
import { useFetch } from "@/lib/api/hooks";
import type { Filing } from "@/lib/api/schema";
import { figure, FORMS, HEADLINE, labelOf, sumOf } from "@/lib/filings";
import { day, money } from "@/lib/format";
import { useT } from "@/lib/i18n";

export default function FilingsPage() {
  return (
    <Suspense>
      <Filings />
    </Suspense>
  );
}

function Filings() {
  const t = useT();
  const router = useRouter();
  const params = useSearchParams();
  const { partyIds, partyName } = useFinance();
  const key = partyIds.join(",");
  const form = params.get("form") ?? "";
  const year = Number(params.get("year") ?? "0") || 0;
  const open = params.get("open") ?? "";
  const [uploadParty, setUploadParty] = useState("");

  const set = (patch: Record<string, string>) => {
    const next = new URLSearchParams(params.toString());
    for (const [k, v] of Object.entries(patch)) {
      if (v) next.set(k, v);
      else next.delete(k);
    }
    const s = next.toString();
    router.replace(s ? `/filings/?${s}` : "/filings/");
  };

  const list = useFetch(() => api.filings({ party_ids: partyIds, form, year }), 30_000, [key, form, year]);
  const filings = useMemo(() => list.data?.filings ?? [], [list.data]);
  const years = list.data?.years ?? [];
  const byForm = useMemo(() => {
    const m = new Map<string, Filing[]>();
    for (const f of filings) m.set(f.form, [...(m.get(f.form) ?? []), f]);
    return FORMS.filter((x) => m.has(x)).map((x) => ({ form: x, rows: m.get(x) ?? [] }));
  }, [filings]);

  const pdTax = sumOf(filings, "44");
  const pdvDue = sumOf(filings, "400");
  const gross = sumOf(filings, "A.PredujamPoreza.P1");
  const hint = year ? t("filings.kpi.hint_year", { year: String(year) }) : t("filings.kpi.hint_all");
  const errors = filings.filter((f) => f.error).length;

  return (
    <>
      <PageTitle title={t("filings.title")} description={t("filings.description")}>
        <ScopeToggle className="md:hidden" />
        <div className="flex items-center gap-2">
          {partyIds.length > 1 ? (
            <Select value={uploadParty} onValueChange={setUploadParty}>
              <SelectTrigger className="w-44">
                <SelectValue placeholder={t("filings.upload_for")} />
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
          <UploadFiling
            partyId={uploadParty || partyIds[0] || ""}
            onUploaded={(docs) => {
              list.reload();
              const last = docs[docs.length - 1];
              if (last) set({ open: last.id });
            }}
          />
        </div>
      </PageTitle>

      {list.loading && !list.data ? (
        <Skeleton className="h-28 w-full" />
      ) : (
        <KpiStrip
          items={[
            {
              icon: FileCheck2,
              label: t("filings.kpi.in_view"),
              value: filings.length,
              hint: errors ? t("filings.error", { error: String(errors) }) : hint,
            },
            {
              icon: Landmark,
              label: t("filings.kpi.pd_tax"),
              value: pdTax.n ? money(pdTax.minor.toString(), "EUR") : "—",
              hint,
            },
            {
              icon: Receipt,
              label: t("filings.kpi.pdv_due"),
              value: pdvDue.n ? money(pdvDue.minor.toString(), "EUR") : "—",
              hint: pdvDue.n ? `${pdvDue.n} × PDV` : hint,
            },
            {
              icon: Users,
              label: t("filings.kpi.joppd_gross"),
              value: gross.n ? money(gross.minor.toString(), "EUR") : "—",
              hint: gross.n ? `${gross.n} × JOPPD` : hint,
            },
          ]}
        />
      )}

      <div className="flex flex-wrap items-center gap-2">
        <Select value={form || "all"} onValueChange={(v) => set({ form: v === "all" ? "" : v })}>
          <SelectTrigger className="w-40">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">{t("filings.all_forms")}</SelectItem>
            {FORMS.map((f) => (
              <SelectItem key={f} value={f}>
                {t(`filings.form.${f}`)} · {t(`filings.form_long.${f}`)}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <Select
          value={year ? String(year) : "all"}
          onValueChange={(v) => set({ year: v === "all" ? "" : v })}
        >
          <SelectTrigger className="w-32">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">{t("filings.all_years")}</SelectItem>
            {years.map((y) => (
              <SelectItem key={y} value={String(y)}>
                {y}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <span className="text-muted-foreground text-sm">
          {t(filings.length === 1 ? "filings.count_one" : "filings.count_other", {
            n: String(filings.length),
          })}
        </span>
      </div>

      {!list.loading && filings.length === 0 ? (
        <Card>
          <CardContent className="text-muted-foreground py-10 text-center text-sm">
            {t("filings.nothing")}
          </CardContent>
        </Card>
      ) : null}

      {byForm.map(({ form: f, rows }) => {
        const columns = HEADLINE[f] ?? [];
        return (
          <Card key={f}>
            <CardHeader className="flex flex-row items-center gap-2">
              <FormBadge form={f} />
              <CardTitle className="text-base">{t(`filings.form_long.${f}`)}</CardTitle>
            </CardHeader>
            <CardContent className="overflow-x-auto">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>{t("filings.period")}</TableHead>
                    {f === "joppd" ? <TableHead>{t("filings.report_mark")}</TableHead> : null}
                    {columns.map((c) => (
                      <TableHead key={c} className="text-right" title={c}>
                        {labelOf(t, f, c) || c}
                      </TableHead>
                    ))}
                    <TableHead>{t("filings.prepared")}</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {rows.map((r) => (
                    <TableRow key={r.id} className="cursor-pointer" onClick={() => set({ open: r.id })}>
                      <TableCell className="whitespace-nowrap">
                        {r.period_from}
                        {r.period_to && r.period_to !== r.period_from ? ` – ${r.period_to}` : ""}
                        {r.error ? (
                          <Badge variant="destructive" className="ml-2">
                            <AlertTriangle className="size-3" />
                          </Badge>
                        ) : null}
                      </TableCell>
                      {f === "joppd" ? <TableCell className="font-mono">{r.report_mark}</TableCell> : null}
                      {columns.map((c) => (
                        <TableCell key={c} className="text-right tabular-nums">
                          {r.headline[c] !== undefined ? figure(f, c, r.headline[c]) : ""}
                        </TableCell>
                      ))}
                      <TableCell className="text-muted-foreground whitespace-nowrap">
                        {r.prepared_at ? day(r.prepared_at) : "—"}
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </CardContent>
          </Card>
        );
      })}

      <FilingSheet id={open || null} onClose={() => set({ open: "" })} onChanged={() => list.reload()} />
    </>
  );
}
