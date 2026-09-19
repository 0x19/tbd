// The accountant's month as text and as a bundle, built from the
// reconciliation rows in whatever language the page is in *now*. Pure
// functions of the rows and the translator: the reconciliation page renders
// them for the download, and the composer renders them again on every
// language change, so the README, the CSVs, the file names inside the zip
// and the mail's summary always agree with the screen.

import type { Reason, ReconciliationRow } from "@/lib/api/schema";
import { dateOnly, money, monthLong } from "@/lib/format";
import type { useT } from "@/lib/i18n";
import type { BundlePlan } from "@/lib/mail-template";
import { safeName } from "@/lib/zip";

export type T = ReturnType<typeof useT>;

/** One company's month: the rows the service returned, as they came. */
export type MonthData = {
  /** The company's display name. */
  company: string;
  /** YYYY-MM. */
  month: string;
  rows: ReconciliationRow[];
};

const ORDER: Record<string, number> = { receipt: 0, eracun: 2, none: 3, personal: 4, income: 5, internal: 6 };

function sortKey(r: ReconciliationRow): number {
  if (r.need === "receipt") return r.status === "missing" ? 0 : 1;
  return ORDER[r.need] ?? 9;
}

/** Missing receipts first, then covered, then the rest; by date within. */
export function sortRows(rows: ReconciliationRow[]): ReconciliationRow[] {
  return [...rows].sort(
    (a, b) => sortKey(a) - sortKey(b) || a.transaction.booking_date.localeCompare(b.transaction.booking_date),
  );
}

export function csv(rows: string[][]): string {
  const cell = (v: string) => (/[",\n;]/.test(v) ? `"${v.replace(/"/g, '""')}"` : v);
  return rows.map((r) => r.map(cell).join(";")).join("\r\n") + "\r\n";
}

export function amountOf(r: ReconciliationRow): string {
  return money(r.transaction.amount_minor, r.transaction.currency);
}

/** A server reason, in the page's language: the code with its arguments. */
export function sayWhy(t: T, prefix: "need" | "why", r: Reason | null | undefined, fallback: string): string {
  if (!r) return fallback;
  const vars: Record<string, string> = { ...r.args };
  if (r.args.amount_minor) vars.amount = money(r.args.amount_minor, r.args.currency || "EUR");
  const out = t(`${prefix}.${r.code}`, vars);
  return out === `${prefix}.${r.code}` ? fallback : out;
}

export function needLabel(t: T, need: string): string {
  return t(`accountant.need.${need}`);
}

const missingOf = (rows: ReconciliationRow[]) =>
  rows.filter((r) => r.need === "receipt" && r.status === "missing");
const coveredOf = (rows: ReconciliationRow[]) =>
  rows.filter((r) => r.need === "receipt" && r.status === "covered");

/** The README, and the mail's `{{Summary}}`: what is attached, what is
 *  still missing, what the accountant receives on their own. */
export function summaryText(t: T, m: MonthData): string {
  const rows = sortRows(m.rows);
  const covered = coveredOf(rows);
  const missing = missingOf(rows);
  const eracun = rows.filter((r) => r.need === "eracun");
  const line = (r: ReconciliationRow) =>
    `  ${dateOnly(r.transaction.booking_date)}  ${r.transaction.counterparty_name}  ${amountOf(r)}`;
  return [
    `${m.company} · ${monthLong(m.month)}`,
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
}

/** The bundle as files: the text ones with their content, the receipts by
 *  document with the name each takes. The download and the mail share it,
 *  so what the accountant gets is the same either way. */
export function plan(t: T, m: MonthData): BundlePlan {
  const rows = sortRows(m.rows);
  const missing = missingOf(rows);
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
  for (const r of rows) {
    const files: string[] = [];
    if (r.need === "receipt") {
      for (const d of r.documents) {
        const ext = /\.(jpe?g|png)$/i.exec(d.filename);
        const suffix = ext ? ext[1]!.toLowerCase().replace("jpeg", "jpg") : "pdf";
        let name = `${folder}/${r.transaction.booking_date}_${safeName(d.vendor || r.transaction.counterparty_name)}_${safeName(money(d.total_minor || r.transaction.amount_minor, d.currency || r.transaction.currency))}.${suffix}`;
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
    filename: `${safeName(m.company.toLowerCase())}-${m.month}-${t("accountant.bundle.zip_suffix")}.zip`,
    files: [
      { name: t("accountant.bundle.readme_file"), text: summaryText(t, m) + "\n" },
      { name: t("accountant.bundle.summary_file"), text: "﻿" + csv(summary) },
      { name: t("accountant.bundle.missing_file"), text: "﻿" + csv(missingRows) },
    ],
    receipts,
  };
}
