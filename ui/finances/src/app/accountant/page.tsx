"use client";

// The accountant's month: every transaction of the company sorted into what
// they need from us -- nothing (a domestic supplier's e-invoice reaches them,
// tax and salary carry their own paperwork), or a receipt we owe -- and for
// those, which pulled receipt covers it. Rules decide by default; a policy
// on a counterparty or a link made by hand overrides them. The bundle is
// built here, in the browser: the covered receipts, a summary, and the list
// of what is still missing.
import {
  CheckCircle2,
  CircleDashed,
  Download,
  FileCheck,
  Link2,
  Search,
  Unlink,
  XCircle,
} from "lucide-react";
import { useCallback, useEffect, useMemo, useState } from "react";
import { toast } from "sonner";

import { useFinance } from "@/app/providers";
import { KpiStrip, PageTitle } from "@/components/kit";
import { MonthStepper } from "@/components/month-stepper";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Skeleton } from "@/components/ui/skeleton";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { api } from "@/lib/api/client";
import { describe, useFetch } from "@/lib/api/hooks";
import type { LinkedDocument, ReconciliationRow } from "@/lib/api/schema";
import { money, monthLabel, monthsBefore, thisMonth } from "@/lib/format";
import { safeName, zip } from "@/lib/zip";

const NEEDS: Record<string, { label: string; tone: string }> = {
  receipt: { label: "receipt", tone: "" },
  eracun: { label: "eRačun", tone: "border-transparent bg-sky-500/15 text-sky-700 dark:text-sky-300" },
  none: { label: "nothing needed", tone: "border-transparent bg-muted text-muted-foreground" },
  personal: {
    label: "personal",
    tone: "border-transparent bg-violet-500/15 text-violet-700 dark:text-violet-300",
  },
  income: {
    label: "income",
    tone: "border-transparent bg-emerald-600/12 text-emerald-700 dark:text-emerald-300",
  },
  internal: { label: "own transfer", tone: "border-transparent bg-muted text-muted-foreground" },
};

const POLICY_CHOICES: [string, string][] = [
  ["auto", "Decide by the rules"],
  ["eracun", "eRačun: the supplier delivers it"],
  ["receipt", "Receipt needed from us"],
  ["none", "Nothing needed"],
  ["personal", "Personal, not a business cost"],
];

const ORDER: Record<string, number> = { receipt: 0, eracun: 2, personal: 3, none: 4, internal: 5, income: 6 };

function sortKey(r: ReconciliationRow): number {
  if (r.need === "receipt") return r.status === "missing" ? 0 : 1;
  return ORDER[r.need] ?? 9;
}

function bytesOf(base64: string): Uint8Array {
  const s = atob(base64);
  const out = new Uint8Array(s.length);
  for (let i = 0; i < s.length; i++) out[i] = s.charCodeAt(i);
  return out;
}

function csv(rows: string[][]): string {
  const cell = (v: string) => (/[",\n;]/.test(v) ? `"${v.replace(/"/g, '""')}"` : v);
  return rows.map((r) => r.map(cell).join(";")).join("\r\n") + "\r\n";
}

function amountOf(r: ReconciliationRow): string {
  return money(r.transaction.amount_minor, r.transaction.currency);
}

export default function AccountantPage() {
  const { parties } = useFinance();
  const orgs = useMemo(() => parties.filter((p) => p.kind === "org"), [parties]);
  const [party, setParty] = useState("");
  const chosen = party || orgs[0]?.id || "";
  const chosenName = orgs.find((o) => o.id === chosen)?.display_name ?? "";
  const months = useMemo(() => {
    const now = thisMonth();
    return Array.from({ length: 18 }, (_, i) => monthsBefore(now, 17 - i));
  }, []);
  const [month, setMonth] = useState(() => monthsBefore(thisMonth(), 1));
  const [only, setOnly] = useState<"all" | "missing" | "receipt" | "eracun" | "other">("all");
  const data = useFetch(
    () => (chosen ? api.reconciliation(chosen, month) : Promise.reject(new Error("no company in scope"))),
    0,
    [chosen, month],
  );
  const [rows, setRows] = useState<ReconciliationRow[]>([]);
  useEffect(() => {
    if (data.data) setRows(data.data.rows);
  }, [data.data]);
  const replace = useCallback((row: ReconciliationRow) => {
    setRows((prev) => prev.map((r) => (r.transaction.id === row.transaction.id ? row : r)));
  }, []);

  const sorted = useMemo(
    () =>
      [...rows].sort(
        (a, b) =>
          sortKey(a) - sortKey(b) || a.transaction.booking_date.localeCompare(b.transaction.booking_date),
      ),
    [rows],
  );
  const shown = sorted.filter((r) => {
    if (only === "all") return true;
    if (only === "missing") return r.need === "receipt" && r.status === "missing";
    if (only === "receipt") return r.need === "receipt";
    if (only === "eracun") return r.need === "eracun";
    return !["receipt", "eracun"].includes(r.need);
  });
  const missing = rows.filter((r) => r.need === "receipt" && r.status === "missing");
  const covered = rows.filter((r) => r.need === "receipt" && r.status === "covered");
  const eracun = rows.filter((r) => r.need === "eracun");
  const missingSum = useMemo(() => {
    const by = new Map<string, bigint>();
    for (const r of missing) {
      const c = r.transaction.currency;
      by.set(
        c,
        (by.get(c) ?? 0n) +
          (BigInt(r.transaction.amount_minor) < 0n
            ? -BigInt(r.transaction.amount_minor)
            : BigInt(r.transaction.amount_minor)),
      );
    }
    return [...by.entries()].map(([c, m]) => money(m.toString(), c)).join(" + ");
  }, [missing]);

  const summaryText = () => {
    const lines = [
      `${chosenName} · ${monthLabel(month)}`,
      "",
      `Receipts attached (${covered.length}):`,
      ...covered.map(
        (r) =>
          `  ${r.transaction.booking_date}  ${r.transaction.counterparty_name}  ${amountOf(r)}  → ${r.documents.map((d) => `${d.vendor} ${d.invoice_no || d.filename}`).join(", ")}`,
      ),
      "",
      `Still missing a receipt (${missing.length}):`,
      ...missing.map(
        (r) =>
          `  ${r.transaction.booking_date}  ${r.transaction.counterparty_name}  ${amountOf(r)}${r.original_amount_minor ? ` (${money(r.original_amount_minor, r.original_currency)})` : ""}`,
      ),
      "",
      `Domestic suppliers, e-invoiced to you directly (${eracun.length}):`,
      ...eracun.map(
        (r) => `  ${r.transaction.booking_date}  ${r.transaction.counterparty_name}  ${amountOf(r)}`,
      ),
    ];
    return lines.join("\n");
  };

  const [bundling, setBundling] = useState(false);
  const bundle = async () => {
    setBundling(true);
    try {
      const entries: { name: string; bytes: Uint8Array }[] = [];
      const used = new Set<string>();
      const summary: string[][] = [
        [
          "date",
          "counterparty",
          "amount",
          "currency",
          "original",
          "need",
          "status",
          "receipt",
          "invoice_no",
          "reason",
        ],
      ];
      for (const r of sorted) {
        const files: string[] = [];
        if (r.need === "receipt") {
          for (const d of r.documents) {
            const got = await api.document(d.document_id);
            let name = `receipts/${r.transaction.booking_date}_${safeName(d.vendor || r.transaction.counterparty_name)}_${safeName(money(d.total_minor || r.transaction.amount_minor, d.currency || r.transaction.currency))}.pdf`;
            let n = 2;
            while (used.has(name)) name = name.replace(/(\.pdf)$/, `_${n++}$1`);
            used.add(name);
            entries.push({ name, bytes: bytesOf(got.bytes) });
            files.push(name.replace("receipts/", ""));
          }
        }
        summary.push([
          r.transaction.booking_date,
          r.transaction.counterparty_name,
          amountOf(r),
          r.transaction.currency,
          r.original_amount_minor ? money(r.original_amount_minor, r.original_currency) : "",
          NEEDS[r.need]?.label ?? r.need,
          r.status,
          files.join(" | "),
          r.documents
            .map((d) => d.invoice_no)
            .filter(Boolean)
            .join(" | "),
          r.need_reason,
        ]);
      }
      const enc = new TextEncoder();
      entries.unshift(
        { name: "README.txt", bytes: enc.encode(summaryText() + "\n") },
        { name: "summary.csv", bytes: enc.encode("﻿" + csv(summary)) },
        {
          name: "missing.csv",
          bytes: enc.encode(
            "﻿" +
              csv([
                ["date", "counterparty", "amount", "original", "reason"],
                ...missing.map((r) => [
                  r.transaction.booking_date,
                  r.transaction.counterparty_name,
                  amountOf(r),
                  r.original_amount_minor ? money(r.original_amount_minor, r.original_currency) : "",
                  r.need_reason,
                ]),
              ]),
          ),
        },
      );
      const blob = zip(entries);
      const a = window.document.createElement("a");
      a.href = URL.createObjectURL(blob);
      a.download = `${safeName(chosenName.toLowerCase())}-${month}-accountant.zip`;
      a.click();
      setTimeout(() => URL.revokeObjectURL(a.href), 60_000);
      toast.success(`Bundle: ${covered.length} receipts, ${missing.length} still missing.`);
    } catch (e) {
      toast.error(describe(e));
    } finally {
      setBundling(false);
    }
  };

  const s = data.data?.summary;
  return (
    <>
      <PageTitle
        title="Accountant"
        description="What the accountant needs from us for each transaction, and which receipt covers it. Domestic suppliers e-invoice them directly; foreign ones we owe a receipt for."
      >
        <div className="flex flex-wrap items-center gap-2">
          {orgs.length > 1 ? (
            <Select value={chosen} onValueChange={setParty}>
              <SelectTrigger className="w-44">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {orgs.map((p) => (
                  <SelectItem key={p.id} value={p.id}>
                    {p.display_name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          ) : null}
          <MonthStepper months={months} value={month} onChange={setMonth} />
        </div>
      </PageTitle>

      {orgs.length === 0 ? (
        <p className="text-muted-foreground text-sm">
          No company in your scope. This page reads the company account only, never a personal one.
        </p>
      ) : null}

      {s ? (
        <KpiStrip
          items={[
            {
              icon: XCircle,
              label: "Missing a receipt",
              value: String(s.receipt_missing),
              hint: missingSum || "nothing outstanding",
            },
            {
              icon: FileCheck,
              label: "Receipts to attach",
              value: String(s.receipt_covered),
              hint: "matched or linked by hand",
            },
            {
              icon: CheckCircle2,
              label: "eRačun, delivered by supplier",
              value: String(s.eracun),
              hint: "domestic, nothing to send",
            },
            {
              icon: CircleDashed,
              label: "Nothing needed",
              value: String(s.none + s.personal + s.internal),
              hint: `${s.none} tax, salary, fees · ${s.personal} personal · ${s.internal} own transfers`,
            },
          ]}
        />
      ) : null}

      <div className="flex flex-wrap items-center gap-2">
        {(
          [
            ["all", "All"],
            ["missing", "Missing"],
            ["receipt", "Receipts"],
            ["eracun", "eRačun"],
            ["other", "Other"],
          ] as const
        ).map(([k, label]) => (
          <Button key={k} size="sm" variant={only === k ? "secondary" : "ghost"} onClick={() => setOnly(k)}>
            {label}
          </Button>
        ))}
        <div className="flex-1" />
        <Button
          size="sm"
          variant="outline"
          onClick={() => {
            void navigator.clipboard
              .writeText(summaryText())
              .then(() => toast.success("Summary copied. Paste it into the mail."));
          }}
          disabled={!data.data}
        >
          Copy summary for the mail
        </Button>
        <Button size="sm" onClick={() => void bundle()} disabled={bundling || !data.data}>
          <Download /> {bundling ? "Bundling…" : "Download bundle"}
        </Button>
      </div>

      <Card>
        <CardContent className="p-0">
          {data.error ? (
            <p className="text-destructive p-4 text-sm">{data.error}</p>
          ) : data.loading && !data.data ? (
            <Skeleton className="m-4 h-64" />
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="w-24">Date</TableHead>
                  <TableHead>Counterparty</TableHead>
                  <TableHead className="text-right">Amount</TableHead>
                  <TableHead>Accountant needs</TableHead>
                  <TableHead>Receipt</TableHead>
                  <TableHead className="w-44">Rule</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {shown.map((r) => (
                  <Row
                    key={r.transaction.id}
                    r={r}
                    party={chosen}
                    onChanged={replace}
                    onPolicy={data.reload}
                  />
                ))}
                {data.data && shown.length === 0 ? (
                  <TableRow>
                    <TableCell colSpan={6} className="text-muted-foreground py-10 text-center text-sm">
                      Nothing here for {monthLabel(month)}.
                    </TableCell>
                  </TableRow>
                ) : null}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>
    </>
  );
}

function DocChip({ d, onRemove }: { d: LinkedDocument; onRemove?: () => void }) {
  return (
    <span
      className="inline-flex items-center gap-1 rounded-md border px-1.5 py-0.5 text-[11px]"
      title={`${d.reason || d.source}${d.confidence ? ` · ${d.confidence}` : ""} · ${d.filename}`}
    >
      {d.source === "declared" ? <Link2 className="size-3" /> : <FileCheck className="size-3" />}
      {d.vendor || d.filename} · {d.total_minor ? money(d.total_minor, d.currency || "EUR") : "?"}
      {d.doc_date ? <span className="text-muted-foreground">· {d.doc_date.slice(5)}</span> : null}
      {onRemove ? (
        <button
          type="button"
          onClick={onRemove}
          className="text-muted-foreground hover:text-destructive ml-0.5"
          aria-label="Unlink"
        >
          <Unlink className="size-3" />
        </button>
      ) : null}
    </span>
  );
}

function Row({
  r,
  party,
  onChanged,
  onPolicy,
}: {
  r: ReconciliationRow;
  party: string;
  onChanged: (row: ReconciliationRow) => void;
  onPolicy: () => void;
}) {
  const t = r.transaction;
  const [busy, setBusy] = useState(false);
  const [finding, setFinding] = useState(false);
  const act = async (f: () => Promise<ReconciliationRow | void>, done?: string) => {
    setBusy(true);
    try {
      const row = await f();
      if (row) onChanged(row);
      if (done) toast.success(done);
    } catch (e) {
      toast.error(describe(e));
    } finally {
      setBusy(false);
    }
  };
  const need = NEEDS[r.need] ?? { label: r.need, tone: "" };
  const missing = r.need === "receipt" && r.status === "missing";
  const policy = r.policy_id ? r.need : "auto";
  return (
    <TableRow className={missing ? "bg-destructive/5" : undefined}>
      <TableCell className="text-xs whitespace-nowrap tabular-nums">{t.booking_date}</TableCell>
      <TableCell className="max-w-72">
        <div className="truncate font-medium">{t.counterparty_name || "—"}</div>
        <div className="text-muted-foreground max-w-72 truncate text-[11px]" title={t.remittance}>
          {t.remittance.replace(/^HR\d\d \| /, "")}
        </div>
      </TableCell>
      <TableCell className="text-right font-mono whitespace-nowrap tabular-nums">
        {amountOf(r)}
        {r.original_amount_minor ? (
          <div className="text-muted-foreground text-[11px]">
            {money(r.original_amount_minor, r.original_currency)}
          </div>
        ) : null}
      </TableCell>
      <TableCell>
        <div className="flex flex-wrap items-center gap-1.5">
          {r.need === "receipt" ? (
            <Badge
              variant="outline"
              className={
                missing
                  ? "bg-destructive/12 text-destructive border-transparent text-[10px]"
                  : "border-transparent bg-emerald-600/12 text-[10px] text-emerald-700 dark:text-emerald-300"
              }
            >
              {missing ? "receipt missing" : "receipt attached"}
            </Badge>
          ) : (
            <Badge variant="outline" className={`text-[10px] ${need.tone}`}>
              {need.label}
            </Badge>
          )}
          <span className="text-muted-foreground text-[11px]">{r.need_reason}</span>
        </div>
      </TableCell>
      <TableCell>
        <div className="flex flex-wrap items-center gap-1">
          {r.documents.map((d) => (
            <DocChip
              key={d.document_id}
              d={d}
              onRemove={() =>
                void act(async () => (await api.unlinkDocument(t.id, d.document_id)).row, "Unlinked.")
              }
            />
          ))}
          {missing
            ? r.suggestions.map((d) => (
                <Button
                  key={d.document_id}
                  size="sm"
                  variant="outline"
                  className="h-6 px-1.5 text-[11px]"
                  disabled={busy}
                  title={`${d.reason} · ${d.confidence}`}
                  onClick={() =>
                    void act(async () => (await api.linkDocument(t.id, d.document_id)).row, "Linked.")
                  }
                >
                  Use {d.vendor || d.filename} ·{" "}
                  {d.total_minor ? money(d.total_minor, d.currency || "EUR") : "?"}
                </Button>
              ))
            : null}
          {r.need === "receipt" ? (
            <Button
              size="sm"
              variant="ghost"
              className="h-6 px-1.5 text-[11px]"
              disabled={busy}
              onClick={() => setFinding(true)}
            >
              <Search className="size-3" /> Find
            </Button>
          ) : null}
        </div>
        <FindDialog
          open={finding}
          onClose={() => setFinding(false)}
          party={party}
          initial={t.counterparty_name.split(/[*\s]/)[0] ?? ""}
          onPick={(id) => {
            setFinding(false);
            void act(async () => (await api.linkDocument(t.id, id)).row, "Linked.");
          }}
        />
      </TableCell>
      <TableCell>
        {r.need === "income" || r.need === "internal" ? (
          <span className="text-muted-foreground text-[11px]">—</span>
        ) : (
          <Select
            value={policy}
            onValueChange={(v) =>
              void act(
                async () => {
                  if (v === "auto") {
                    if (r.policy_id) await api.deletePolicy(r.policy_id);
                  } else {
                    await api.setPolicy({
                      party_id: party,
                      match: t.counterparty_name,
                      exact: true,
                      policy: v,
                      note: "",
                    });
                  }
                  onPolicy();
                },
                v === "auto"
                  ? "Back to the rules."
                  : `${t.counterparty_name}: ${POLICY_CHOICES.find(([k]) => k === v)?.[1]}`,
              )
            }
          >
            <SelectTrigger className="h-7 text-[11px]" disabled={busy}>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {POLICY_CHOICES.map(([k, label]) => (
                <SelectItem key={k} value={k} className="text-xs">
                  {label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        )}
      </TableCell>
    </TableRow>
  );
}

function FindDialog({
  open,
  onClose,
  party,
  initial,
  onPick,
}: {
  open: boolean;
  onClose: () => void;
  party: string;
  initial: string;
  onPick: (documentId: string) => void;
}) {
  const [q, setQ] = useState(initial);
  useEffect(() => {
    if (open) setQ(initial);
  }, [open, initial]);
  const found = useFetch(
    () => (open ? api.documents({ party_ids: [party], q, limit: 20 }) : Promise.resolve(null)),
    0,
    [open, q, party],
  );
  return (
    <Dialog open={open} onOpenChange={(o) => !o && onClose()}>
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>Find the receipt</DialogTitle>
          <DialogDescription>
            Search the pulled receipts by vendor, number, subject or text, and pick the one that paid this.
          </DialogDescription>
        </DialogHeader>
        <Input
          value={q}
          onChange={(e) => setQ(e.target.value)}
          placeholder="vendor, invoice number, anything in the PDF"
          autoFocus
        />
        <div className="max-h-80 space-y-1 overflow-y-auto">
          {(found.data?.documents ?? []).map((d) => (
            <button
              key={d.id}
              type="button"
              className="hover:bg-muted/50 flex w-full items-center justify-between gap-2 rounded-md border px-2 py-1.5 text-left text-xs"
              onClick={() => onPick(d.id)}
            >
              <span className="truncate">
                <span className="font-medium">{d.vendor || d.filename}</span>
                <span className="text-muted-foreground">
                  {" "}
                  · {d.doc_date || "no date"} · {d.invoice_no || d.filename}
                </span>
              </span>
              <span className="font-mono whitespace-nowrap tabular-nums">
                {d.total_minor ? money(d.total_minor, d.currency || "EUR") : "—"}
              </span>
            </button>
          ))}
          {found.data && found.data.documents.length === 0 ? (
            <p className="text-muted-foreground p-2 text-xs">Nothing matches.</p>
          ) : null}
        </div>
      </DialogContent>
    </Dialog>
  );
}
