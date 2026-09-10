export function ms(v: number | null | undefined, digits = 1): string {
  if (v === null || v === undefined) return "–";
  if (v >= 1000) return `${(v / 1000).toFixed(2)} s`;
  return `${v.toFixed(digits)} ms`;
}

export function pct(v: number | null | undefined, digits = 2): string {
  if (v === null || v === undefined) return "–";
  return `${(v * 100).toFixed(digits)}%`;
}

export function num(v: number | null | undefined): string {
  if (v === null || v === undefined) return "–";
  return new Intl.NumberFormat().format(Math.round(v));
}

export function seconds(v: number | null | undefined): string {
  if (v === null || v === undefined) return "–";
  if (v < 60) return `${v.toFixed(1)} s`;
  const m = Math.floor(v / 60);
  return `${m} min ${(v - m * 60).toFixed(0)} s`;
}

export function ago(iso: string | null | undefined): string {
  if (!iso) return "–";
  const t = new Date(iso).getTime();
  if (Number.isNaN(t)) return iso;
  const d = Math.max(0, Date.now() - t) / 1000;
  if (d < 5) return "just now";
  if (d < 60) return `${Math.round(d)} s ago`;
  if (d < 3600) return `${Math.round(d / 60)} min ago`;
  if (d < 86400) return `${Math.round(d / 3600)} h ago`;
  return new Date(iso).toLocaleString();
}

export function when(iso: string | null | undefined): string {
  if (!iso) return "–";
  const d = new Date(iso);
  return Number.isNaN(d.getTime()) ? iso : d.toLocaleString();
}
