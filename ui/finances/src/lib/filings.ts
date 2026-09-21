// What the filings page knows about a form's keys: which are money, which
// are counts or rates, the order they are shown in, and the label a key has.
// Pure, so the list, the strip and the sheet agree.
import type { Filing } from "@/lib/api/schema";
import { money } from "@/lib/format";
import { decimalToMinor } from "@/lib/receipts";

/** The headline columns per form, in display order (mirrors form.rs). */
export const HEADLINE: Record<string, string[]> = {
  pd: ["1", "2", "3", "26", "35", "38", "43", "44", "55", "56", "57", "59"],
  pdv: ["000", "104", "105", "200.Vrijednost", "200.Porez", "300.Vrijednost", "300.Porez", "400", "500"],
  pdv_s: ["IsporukeUkupno.I1", "IsporukeUkupno.I2"],
  zp: ["IsporukeUkupno.I1", "IsporukeUkupno.I2", "IsporukeUkupno.I3", "IsporukeUkupno.I4"],
  joppd: [
    "A.BrojOsoba",
    "A.BrojRedaka",
    "A.PredujamPoreza.P1",
    "A.PredujamPoreza.P2",
    "A.Doprinosi.GeneracijskaSolidarnost.P1",
    "A.Doprinosi.KapitaliziranaStednja.P1",
    "A.Doprinosi.ZdravstvenoOsiguranje.P1",
  ],
  pd_ipo: [
    "Podaci.Podaci1.Sveukupno.S1",
    "Podaci.Podaci2.Sveukupno.S1",
    "Podaci.Podaci3.Sveukupno.S1",
    "Podaci.Podaci4.Sveukupno.S1",
  ],
  tz: ["01", "02", "03", "04", "05", "06", "07"],
};

/** The forms in the order the page lists them. */
export const FORMS = ["pd", "pdv", "pdv_s", "zp", "joppd", "pd_ipo", "tz"];

/** Keys whose value is not an amount: a rate, a count, a year, a code. */
function plain(form: string, key: string): boolean {
  if (form === "pd") return key === "43" || key === "111" || key === "112" || /^Godina\d\d$/.test(key);
  if (form === "tz") return key === "02";
  if (form === "joppd")
    return /^A\.(BrojOsoba|BrojRedaka|DatumIzvjesca|OznakaIzvjesca|VrstaIzvjesca)$/.test(key);
  return false;
}

/** A value as the page shows it: money for an amount, the text otherwise. */
export function figure(form: string, key: string, value: string): string {
  if (!value) return "";
  if (!/^-?\d+(\.\d{1,2})?$/.test(value.trim())) return value;
  if (plain(form, key)) return value;
  const minor = decimalToMinor(value);
  return minor ? money(minor, "EUR") : value;
}

/** The label a key has in the dictionary, or nothing. */
export function labelOf(t: (k: string) => string, form: string, key: string): string {
  const k = `filings.${form}.${key}`;
  const s = t(k);
  return s === k ? "" : s;
}

/** Keys in a stable, numeric-aware order: "1", "2", "10", "26", then names. */
export function keyOrder(keys: string[]): string[] {
  const num = (k: string) => {
    const m = /^(\d+)(?:\.(.*))?$/.exec(k);
    return m ? Number(m[1]) : Number.POSITIVE_INFINITY;
  };
  return [...keys].sort((a, b) => {
    const na = num(a);
    const nb = num(b);
    if (na !== nb) return na - nb;
    return a.localeCompare(b, undefined, { numeric: true });
  });
}

/** A sum over one headline key across filings, in minor units; only rows
 *  that carry the key count, so a missing figure never reads as zero. */
export function sumOf(filings: Filing[], key: string): { minor: bigint; n: number } {
  let minor = 0n;
  let n = 0;
  for (const f of filings) {
    const v = f.headline[key];
    if (v === undefined) continue;
    const m = decimalToMinor(v);
    if (!m) continue;
    minor += BigInt(m);
    n += 1;
  }
  return { minor, n };
}
