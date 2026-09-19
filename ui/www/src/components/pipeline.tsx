import { pipeline, rails } from "@/data/site";

/**
 * The request path as a row of stages with the two rails that run under all of
 * them. Hairline grid rather than a canvas: it is a drawing, not an app, and it
 * has to read the same in both themes and at phone width.
 */
export function Pipeline() {
  return (
    <div className="border-y">
      <div className="bg-border/70 grid gap-px sm:grid-cols-4">
        {pipeline.map((s, i) => (
          <div key={s.stage} className="bg-background relative px-6 py-6">
            {/* The direction of travel, between the cards on a wide screen. */}
            {i > 0 ? (
              <span
                aria-hidden
                className="text-muted-foreground bg-background absolute top-7 -left-[11px] hidden size-[22px] place-items-center rounded-full border text-[11px] sm:grid"
              >
                →
              </span>
            ) : null}
            <h3 className="font-medium tracking-tight">{s.stage}</h3>
            <p className="text-muted-foreground mt-1 font-mono text-[11px]">{s.name}</p>
            <dl className="mt-4 space-y-1.5">
              {s.rows.map((r) => (
                <div key={r.k} className="flex items-baseline justify-between gap-3">
                  <dt className="text-muted-foreground/70 shrink-0 text-[11px]">{r.k}</dt>
                  <dd className="text-right font-mono text-[11px] text-pretty">{r.v}</dd>
                </div>
              ))}
            </dl>
          </div>
        ))}
      </div>
      {rails.map((r) => (
        <div
          key={r.label}
          className="flex flex-col gap-1 border-t px-6 py-4 sm:flex-row sm:items-baseline sm:gap-6"
        >
          <span className="text-muted-foreground/70 shrink-0 font-mono text-[11px] tracking-[0.18em] uppercase">
            {r.label}
          </span>
          <span className="text-sm text-pretty">{r.value}</span>
        </div>
      ))}
    </div>
  );
}
