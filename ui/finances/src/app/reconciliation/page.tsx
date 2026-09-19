"use client";

// The accountant's month: every transaction of the company sorted into what
// they need from us -- nothing (a domestic supplier's e-invoice reaches them,
// tax and salary carry their own paperwork), or a receipt we owe -- and for
// those, which pulled receipt covers it. Rules decide by default; a policy
// on a counterparty or a link made by hand overrides them. The bundle is
// built here, in the browser, in the page's language: the covered receipts,
// a summary, and the list of what is still missing.
import {
  CheckCircle2,
  CircleDashed,
  Download,
  FileCheck,
  Link2,
  Mail,
  Search,
  Unlink,
  XCircle,
} from "lucide-react";
import { useRouter } from "next/navigation";
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
import { UploadReceipt } from "@/components/upload-receipt";
import { api, ApiError } from "@/lib/api/client";
import { describe, useFetch } from "@/lib/api/hooks";
import type { LinkedDocument, Reason, ReconciliationRow } from "@/lib/api/schema";
import { dateOnly, money, monthLong, monthsBefore, thisMonth } from "@/lib/format";
import { useT } from "@/lib/i18n";
import { stashPrefill } from "@/lib/mail-template";
import { safeName, zip, type ZipEntry } from "@/lib/zip";

type T = ReturnType<typeof useT>;

const NEED_TONE: Record<string, string> = {
  receipt: "",
  eracun: "border-transparent bg-sky-500/15 text-sky-700 dark:text-sky-300",
  none: "border-transparent bg-muted text-muted-foreground",
  personal: "border-transparent bg-violet-500/15 text-violet-700 dark:text-violet-300",
  income: "border-transparent bg-emerald-600/12 text-emerald-700 dark:text-emerald-300",
  internal: "border-transparent bg-muted text-muted-foreground",
};

const POLICIES = ["auto", "eracun", "receipt", "none", "personal"] as const;

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

/** A server reason, in the page's language: the code with its arguments. */
function sayWhy(t: T, prefix: "need" | "why", r: Reason | null | undefined, fallback: string): string {
  if (!r) return fallback;
  const vars: Record<string, string> = { ...r.args };
  if (r.args.amount_minor) vars.amount = money(r.args.amount_minor, r.args.currency || "EUR");
  const out = t(`${prefix}.${r.code}`, vars);
  return out === `${prefix}.${r.code}` ? fallback : out;
}

function sayWhys(t: T, d: LinkedDocument): string {
  if (!d.why.length) return d.reason;
  return d.why.map((w) => sayWhy(t, "why", w, w.code)).join(" · ");
}

function needLabel(t: T, need: string): string {
  return t(`accountant.need.${need}`);
}

export default function AccountantPage() {
  const t = useT();
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
    () =>
      chosen ? api.reconciliation(chosen, month) : Promise.reject(new Error(t("accountant.no_company"))),
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
      const v = BigInt(r.transaction.amount_minor);
      by.set(c, (by.get(c) ?? 0n) + (v < 0n ? -v : v));
    }
    return [...by.entries()].map(([c, m]) => money(m.toString(), c)).join(" + ");
  }, [missing]);

  const line = (r: ReconciliationRow) =>
    `  ${dateOnly(r.transaction.booking_date)}  ${r.transaction.counterparty_name}  ${amountOf(r)}`;
  const summaryText = () =>
    [
      `${chosenName} · ${monthLong(month)}`,
      "",
      t("accountant.bundle.readme_attached", { n: covered.length }),
      ...covered.map(
        (r) =>
          `${line(r)}  → ${r.documents.map((d) => `${d.vendor} ${d.invoice_no || d.filename}`).join(", ")}`,
      ),
      "",
      t("accountant.bundle.readme_missing", { n: missing.length }),
      ...missing.map(
        (r) =>
          `${line(r)}${r.original_amount_minor ? ` (${money(r.original_amount_minor, r.original_currency)})` : ""}`,
      ),
      "",
      t("accountant.bundle.readme_eracun", { n: eracun.length }),
      ...eracun.map(line),
    ].join("\n");

  /** The bundle as files: the text ones with their content, the receipts by
   *  document with the name each takes. The download and the mail share it,
   *  so what the accountant gets is the same either way. */
  const plan = () => {
    const folder = t("accountant.bundle.folder");
    const used = new Set<string>();
    const receipts: { document_id: string; name: string }[] = [];
    const h = (k: string) => t(`accountant.csv.${k}`);
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
      ].map(h),
    ];
    for (const r of sorted) {
      const files: string[] = [];
      if (r.need === "receipt") {
        for (const d of r.documents) {
          const m = /\.(jpe?g|png)$/i.exec(d.filename);
          const ext = m ? m[1]!.toLowerCase().replace("jpeg", "jpg") : "pdf";
          let name = `${folder}/${r.transaction.booking_date}_${safeName(d.vendor || r.transaction.counterparty_name)}_${safeName(money(d.total_minor || r.transaction.amount_minor, d.currency || r.transaction.currency))}.${ext}`;
          let n = 2;
          while (used.has(name)) name = name.replace(/(\.[a-z]+)$/, `_${n++}$1`);
          used.add(name);
          receipts.push({ document_id: d.document_id, name });
          files.push(name.slice(folder.length + 1));
        }
      }
      summary.push([
        dateOnly(r.transaction.booking_date),
        r.transaction.counterparty_name,
        amountOf(r),
        r.transaction.currency,
        r.original_amount_minor ? money(r.original_amount_minor, r.original_currency) : "",
        needLabel(t, r.need),
        r.status ? t(`accountant.status.${r.status}`) : "",
        files.join(" | "),
        r.documents
          .map((d) => d.invoice_no)
          .filter(Boolean)
          .join(" | "),
        sayWhy(t, "need", r.need_why, r.need_reason),
      ]);
    }
    const missingRows = [
      ["date", "counterparty", "amount", "original", "reason"].map(h),
      ...missing.map((r) => [
        dateOnly(r.transaction.booking_date),
        r.transaction.counterparty_name,
        amountOf(r),
        r.original_amount_minor ? money(r.original_amount_minor, r.original_currency) : "",
        sayWhy(t, "need", r.need_why, r.need_reason),
      ]),
    ];
    return {
      filename: `${safeName(chosenName.toLowerCase())}-${month}-${t("accountant.bundle.zip_suffix")}.zip`,
      files: [
        { name: t("accountant.bundle.readme_file"), text: summaryText() + "\n" },
        { name: t("accountant.bundle.summary_file"), text: "\uFEFF" + csv(summary) },
        { name: t("accountant.bundle.missing_file"), text: "\uFEFF" + csv(missingRows) },
      ],
      receipts,
    };
  };

  const router = useRouter();
  const sendByMail = () => {
    stashPrefill({ party_id: chosen, month, summary: summaryText(), bundle: plan() });
    router.push("/mail/");
  };

  const [bundling, setBundling] = useState(false);
  const bundle = async () => {
    setBundling(true);
    try {
      const p = plan();
      const enc = new TextEncoder();
      const entries: ZipEntry[] = p.files.map((f) => ({ name: f.name, bytes: enc.encode(f.text) }));
      for (const r of p.receipts) {
        const got = await api.document(r.document_id);
        entries.push({ name: r.name, bytes: bytesOf(got.bytes) });
      }
      const blob = zip(entries);
      const a = window.document.createElement("a");
      a.href = URL.createObjectURL(blob);
      a.download = p.filename;
      a.click();
      setTimeout(() => URL.revokeObjectURL(a.href), 60_000);
      toast.success(t("accountant.bundled", { covered: covered.length, missing: missing.length }));
    } catch (e) {
      toast.error(describe(e));
    } finally {
      setBundling(false);
    }
  };

  const s = data.data?.summary;
  return (
    <>
      <PageTitle title={t("accountant.title")} description={t("accountant.description")}>
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
        <p className="text-muted-foreground text-sm">{t("accountant.no_company")}</p>
      ) : null}

      {s ? (
        <KpiStrip
          items={[
            {
              icon: XCircle,
              label: t("accountant.kpi.missing"),
              value: String(s.receipt_missing),
              hint: missingSum || t("accountant.kpi.missing_none"),
            },
            {
              icon: FileCheck,
              label: t("accountant.kpi.covered"),
              value: String(s.receipt_covered),
              hint: t("accountant.kpi.covered_hint"),
            },
            {
              icon: CheckCircle2,
              label: t("accountant.kpi.eracun"),
              value: String(s.eracun),
              hint: t("accountant.kpi.eracun_hint"),
            },
            {
              icon: CircleDashed,
              label: t("accountant.kpi.nothing"),
              value: String(s.none + s.personal + s.internal),
              hint: t("accountant.kpi.nothing_hint", {
                none: s.none,
                personal: s.personal,
                internal: s.internal,
              }),
            },
          ]}
        />
      ) : null}

      <div className="flex flex-wrap items-center gap-2">
        {(["all", "missing", "receipt", "eracun", "other"] as const).map((k) => (
          <Button key={k} size="sm" variant={only === k ? "secondary" : "ghost"} onClick={() => setOnly(k)}>
            {t(`accountant.filter.${k}`)}
          </Button>
        ))}
        <div className="flex-1" />
        <Button
          size="sm"
          variant="outline"
          onClick={() => {
            void navigator.clipboard
              .writeText(summaryText())
              .then(() => toast.success(t("accountant.copied")));
          }}
          disabled={!data.data}
        >
          {t("accountant.copy_summary")}
        </Button>
        <Button size="sm" variant="outline" onClick={() => void bundle()} disabled={bundling || !data.data}>
          <Download /> {bundling ? t("accountant.bundling") : t("accountant.download_bundle")}
        </Button>
        <Button size="sm" onClick={sendByMail} disabled={!data.data}>
          <Mail /> {t("accountant.send_mail")}
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
                  <TableHead className="w-24">{t("accountant.col.date")}</TableHead>
                  <TableHead>{t("accountant.col.counterparty")}</TableHead>
                  <TableHead className="text-right">{t("accountant.col.amount")}</TableHead>
                  <TableHead>{t("accountant.col.need")}</TableHead>
                  <TableHead>{t("accountant.col.receipt")}</TableHead>
                  <TableHead className="w-44">{t("accountant.col.rule")}</TableHead>
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
                      {t("accountant.nothing_for", { month: monthLong(month) })}
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
  const t = useT();
  return (
    <span
      className="inline-flex items-center gap-1 rounded-md border px-1.5 py-0.5 text-[11px]"
      title={`${sayWhys(t, d) || d.source}${d.confidence ? ` · ${d.confidence}` : ""} · ${d.filename}`}
    >
      {d.source === "declared" ? <Link2 className="size-3" /> : <FileCheck className="size-3" />}
      {d.vendor || d.filename} · {d.total_minor ? money(d.total_minor, d.currency || "EUR") : "?"}
      {d.doc_date ? <span className="text-muted-foreground">· {d.doc_date.slice(5)}</span> : null}
      {onRemove ? (
        <button
          type="button"
          onClick={onRemove}
          className="text-muted-foreground hover:text-destructive ml-0.5"
          aria-label={t("accountant.unlink")}
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
  const t = useT();
  const tx = r.transaction;
  const [busy, setBusy] = useState(false);
  const [finding, setFinding] = useState(false);
  const [mismatch, setMismatch] = useState<{ documentId: string; why: string } | null>(null);
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
  // The safeguard: the service refuses a receipt whose reading disagrees
  // with the charge; the person sees what was read and may attach anyway.
  const linkChecked = (documentId: string, force = false) =>
    act(async () => {
      try {
        const row = (await api.linkDocument(tx.id, documentId, force)).row;
        const reasons = row?.documents.find((d) => d.document_id === documentId)?.why ?? [];
        const code = reasons.map((w) => w.code);
        toast.success(
          code.includes("checked")
            ? t("accountant.checked")
            : code.includes("unread")
              ? t("accountant.unread")
              : t("accountant.linked"),
        );
        return row ?? undefined;
      } catch (e) {
        if (e instanceof ApiError && e.code === "failed_precondition") {
          setMismatch({ documentId, why: e.message });
          return undefined;
        }
        throw e;
      }
    });
  const missing = r.need === "receipt" && r.status === "missing";
  const policy = r.policy_id ? r.need : "auto";
  const docLabel = (d: LinkedDocument) =>
    `${d.vendor || d.filename} · ${d.total_minor ? money(d.total_minor, d.currency || "EUR") : "?"}`;
  return (
    <TableRow className={missing ? "bg-destructive/5" : undefined}>
      <TableCell className="text-xs whitespace-nowrap tabular-nums">{dateOnly(tx.booking_date)}</TableCell>
      <TableCell className="max-w-72">
        <div className="truncate font-medium">{tx.counterparty_name || "—"}</div>
        <div className="text-muted-foreground max-w-72 truncate text-[11px]" title={tx.remittance}>
          {tx.remittance.replace(/^HR\d\d \| /, "")}
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
              {missing ? t("accountant.receipt_missing") : t("accountant.receipt_attached")}
            </Badge>
          ) : (
            <Badge variant="outline" className={`text-[10px] ${NEED_TONE[r.need] ?? ""}`}>
              {needLabel(t, r.need)}
            </Badge>
          )}
          <span className="text-muted-foreground text-[11px]">
            {sayWhy(t, "need", r.need_why, r.need_reason)}
          </span>
        </div>
      </TableCell>
      <TableCell>
        <div className="flex flex-wrap items-center gap-1">
          {r.documents.map((d) => (
            <DocChip
              key={d.document_id}
              d={d}
              onRemove={() =>
                void act(
                  async () => (await api.unlinkDocument(tx.id, d.document_id)).row,
                  t("accountant.unlinked"),
                )
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
                  title={`${sayWhys(t, d)} · ${d.confidence}`}
                  onClick={() => void linkChecked(d.document_id)}
                >
                  {t("accountant.use", { doc: docLabel(d) })}
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
              <Search className="size-3" /> {t("accountant.find")}
            </Button>
          ) : null}
          {missing ? (
            <UploadReceipt
              partyId={party}
              variant="ghost"
              className="h-6 px-1.5 text-[11px]"
              disabled={busy}
              label={t("accountant.attach")}
              onUploaded={(doc) => void linkChecked(doc.id)}
            />
          ) : null}
        </div>
        <FindDialog
          open={finding}
          onClose={() => setFinding(false)}
          party={party}
          initial={tx.counterparty_name.split(/[*\s]/)[0] ?? ""}
          onPick={(id) => {
            setFinding(false);
            void linkChecked(id);
          }}
        />
        <Dialog open={mismatch !== null} onOpenChange={(o) => !o && setMismatch(null)}>
          <DialogContent className="sm:max-w-md">
            <DialogHeader>
              <DialogTitle>{t("accountant.mismatch.title")}</DialogTitle>
              <DialogDescription>{mismatch?.why}</DialogDescription>
            </DialogHeader>
            <div className="flex justify-end gap-2">
              <Button variant="outline" size="sm" onClick={() => setMismatch(null)}>
                {t("common.cancel")}
              </Button>
              <Button
                size="sm"
                variant="destructive"
                disabled={busy}
                onClick={() => {
                  const id = mismatch?.documentId;
                  setMismatch(null);
                  if (id) void linkChecked(id, true);
                }}
              >
                {t("accountant.mismatch.anyway")}
              </Button>
            </div>
          </DialogContent>
        </Dialog>
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
                      match: tx.counterparty_name,
                      exact: true,
                      policy: v,
                      note: "",
                    });
                  }
                  onPolicy();
                },
                v === "auto"
                  ? t("accountant.back_to_rules")
                  : t("accountant.policy_set", {
                      counterparty: tx.counterparty_name,
                      policy: t(`accountant.policy.${v}`),
                    }),
              )
            }
          >
            <SelectTrigger className="h-7 text-[11px]" disabled={busy}>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {POLICIES.map((k) => (
                <SelectItem key={k} value={k} className="text-xs">
                  {t(`accountant.policy.${k}`)}
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
  const t = useT();
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
          <DialogTitle>{t("accountant.find.title")}</DialogTitle>
          <DialogDescription>{t("accountant.find.description")}</DialogDescription>
        </DialogHeader>
        <Input
          value={q}
          onChange={(e) => setQ(e.target.value)}
          placeholder={t("accountant.find.placeholder")}
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
                  · {d.doc_date ? dateOnly(d.doc_date) : t("accountant.find.no_date")} ·{" "}
                  {d.invoice_no || d.filename}
                </span>
              </span>
              <span className="font-mono whitespace-nowrap tabular-nums">
                {d.total_minor ? money(d.total_minor, d.currency || "EUR") : "—"}
              </span>
            </button>
          ))}
          {found.data && found.data.documents.length === 0 ? (
            <p className="text-muted-foreground p-2 text-xs">{t("common.nothing_matches")}</p>
          ) : null}
        </div>
      </DialogContent>
    </Dialog>
  );
}
