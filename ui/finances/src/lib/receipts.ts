// What a receipt's fields say about themselves, as the page needs it:
// how sure each one is, whether the receipt needs a person, and the
// conversions between the wire's minor units and what a person types. Pure,
// so the ledger, the sheet and the numbers at the top agree.
import type { Document } from "@/lib/api/schema";

/** A field the reader is sure about: read from a label, or set by a person. */
export function sure(by: string | undefined): boolean {
  return by === "label" || by === "declared";
}

export type ReceiptStatus = "reading" | "unreadable" | "missing" | "guessed" | "read" | "corrected";

/** One word for the receipt as a whole, worst first: still being read; the
 *  reader failed; vendor or amount not found; found but guessed; read from
 *  the text; or corrected by a person, which outranks everything. */
export function statusOf(d: Document): ReceiptStatus {
  if (d.declared) return "corrected";
  if (!d.extracted_at) return "reading";
  if (d.found_by.error) return "unreadable";
  if (!d.vendor || !d.total_minor) return "missing";
  if (!sure(d.found_by.vendor) || !sure(d.found_by.amount)) return "guessed";
  return "read";
}

/** Whether a person should open it before the accountant asks. */
export function needsLook(d: Document): boolean {
  const s = statusOf(d);
  return s === "missing" || s === "unreadable" || s === "guessed";
}

/** An amount worth adding up: read or declared, never a guess. */
export function reliableAmount(d: Document): boolean {
  return Boolean(d.total_minor) && (d.declared || sure(d.found_by.amount));
}

/** Sums per currency of the receipts whose amount is reliable, and how many
 *  were left out because theirs is a guess or missing. */
export function totals(rows: Document[]): { sums: [string, bigint][]; guessed: number; missing: number } {
  const by = new Map<string, bigint>();
  let guessed = 0;
  let missing = 0;
  for (const d of rows) {
    if (!d.total_minor) {
      missing++;
      continue;
    }
    if (!reliableAmount(d)) {
      guessed++;
      continue;
    }
    const c = d.currency || "EUR";
    by.set(c, (by.get(c) ?? 0n) + BigInt(d.total_minor));
  }
  return { sums: [...by.entries()], guessed, missing };
}

export function minorToDecimal(minor: string): string {
  if (!minor) return "";
  const neg = minor.startsWith("-");
  const digits = (neg ? minor.slice(1) : minor).padStart(3, "0");
  return `${neg ? "-" : ""}${digits.slice(0, -2)}.${digits.slice(-2)}`;
}

export function decimalToMinor(s: string): string {
  const t = s.trim().replace(/\s/g, "");
  if (!t) return "";
  // Either separator may be the decimal one; the last one wins, as in the reader.
  const lastDot = t.lastIndexOf(".");
  const lastComma = t.lastIndexOf(",");
  const at = Math.max(lastDot, lastComma);
  const whole = (at >= 0 ? t.slice(0, at) : t).replace(/[^\d-]/g, "");
  const frac = (at >= 0 ? t.slice(at + 1) : "").replace(/\D/g, "").padEnd(2, "0").slice(0, 2);
  if (!/^-?\d*$/.test(whole)) return "";
  return `${whole || "0"}${frac}`.replace(/^(-?)0+(?=\d)/, "$1");
}

/** How a field was found, as the key of a short word for the page. */
export function foundKey(by: string | undefined): string {
  switch (by) {
    case "label":
    case "sender":
    case "first":
    case "received":
    case "payment":
    case "text":
    case "mailbox":
    case "declared":
      return `documents.found.${by}`;
    default:
      return "documents.found.none";
  }
}

/** The month's first and last day, for the server's inclusive bounds. */
export function monthBounds(ym: string): { from: string; to: string } {
  const [y, m] = ym.split("-").map(Number);
  const last = new Date(Date.UTC(y!, m!, 0)).getUTCDate();
  return { from: `${ym}-01`, to: `${ym}-${String(last).padStart(2, "0")}` };
}

/** Bytes as a person reads them. */
export function fileSize(bytes: string | number): string {
  const n = Number(bytes);
  if (!Number.isFinite(n) || n <= 0) return "";
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(0)} KB`;
  return `${(n / (1024 * 1024)).toFixed(1)} MB`;
}
