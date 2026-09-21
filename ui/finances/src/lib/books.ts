// The books page's pure helpers: the RRiF classes, the opening CSV a person
// hands over, and integer sums. Money never becomes a float here; the wire
// gives minor-unit strings and they are added as BigInt.
import type { TrialBalanceRow } from "@/lib/api/schema";
import { decimalToMinor } from "@/lib/receipts";

/** The classes a service company's books use, in statement order. */
export const CLASSES = [0, 1, 2, 3, 4, 7, 8, 9] as const;

export type OpeningRow = { account_code: string; debit_minor: string; credit_minor: string };

export type ParsedOpening = {
  rows: OpeningRow[];
  /** Lines the parser could not read as `account, [name,] debit, credit`. */
  bad: string[];
  debit: bigint;
  credit: bigint;
};

function splitCsv(line: string, sep: string): string[] {
  const out: string[] = [];
  let cur = "";
  let quoted = false;
  for (let i = 0; i < line.length; i++) {
    const ch = line[i];
    if (ch === '"') {
      if (quoted && line[i + 1] === '"') {
        cur += '"';
        i++;
      } else quoted = !quoted;
    } else if (ch === sep && !quoted) {
      out.push(cur);
      cur = "";
    } else cur += ch;
  }
  out.push(cur);
  return out.map((s) => s.trim());
}

/** Read an opening trial balance from CSV text: `account,debit,credit` or
 *  `account,name,debit,credit`, with either `,` or `;` as the separator and
 *  either `.` or `,` as the decimal mark; a header line is skipped when its
 *  first cell is not a code. Amounts are signed, as the accountant's column
 *  is. */
export function parseOpening(text: string): ParsedOpening {
  const rows: OpeningRow[] = [];
  const bad: string[] = [];
  let debit = 0n;
  let credit = 0n;
  const lines = text.split(/\r?\n/).filter((l) => l.trim() !== "");
  const sep = lines[0]?.includes(";") ? ";" : ",";
  for (const [i, line] of lines.entries()) {
    const cells = splitCsv(line, sep);
    const code = cells[0] ?? "";
    if (!/^\d{3,6}$/.test(code)) {
      if (i === 0) continue; // the header
      bad.push(line);
      continue;
    }
    const amounts = cells.slice(1).filter((c) => /^-?[\d.,\s]+$/.test(c) && /\d/.test(c));
    const [d, c] = amounts.slice(-2);
    const debitMinor = d === undefined ? "" : decimalToMinor(d);
    const creditMinor = c === undefined ? "" : decimalToMinor(c);
    if (amounts.length < 2 || debitMinor === "" || creditMinor === "") {
      bad.push(line);
      continue;
    }
    rows.push({ account_code: code, debit_minor: debitMinor, credit_minor: creditMinor });
    debit += BigInt(debitMinor);
    credit += BigInt(creditMinor);
  }
  return { rows, bad, debit, credit };
}

/** The rows of one class, in the order they came (code order). */
export function rowsOfClass(rows: TrialBalanceRow[], klass: number): TrialBalanceRow[] {
  return rows.filter((r) => r.class === klass);
}

/** `1` … `12` as `01` … `12`. */
export function mm(month: number): string {
  return String(month).padStart(2, "0");
}
