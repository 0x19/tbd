"use client";

import { useT } from "@/lib/i18n";
import { useSite } from "@/lib/i18n/site";
import { cn } from "@/lib/utils";

/**
 * The stack as a drawing, on the page's spine: the four layers top to bottom
 * with the request's path running through each one, and the two rails as
 * vertical hairlines crossing every layer before each ends at its own line
 * of text. Hairlines and knock-out nodes rather than a canvas: it is a
 * drawing, not an app, and it has to read the same in both themes and at
 * phone width. The box starts at the spine (`-ml-3 sm:-ml-4`, the same
 * offset as the section ports in `home.tsx`) so the path is the spine.
 */
export function Pipeline() {
  const t = useT();
  const { pipeline, rails } = useSite();
  const cols = "grid-cols-[minmax(0,1fr)_2.75rem_2.75rem] sm:grid-cols-[11rem_minmax(0,1fr)_4.5rem_4.5rem]";
  return (
    <div className="-ml-3 border-y sm:-ml-4">
      {/* the packet arrives */}
      <div className={cn("relative grid items-center py-3 pl-6 sm:pl-8", cols)}>
        <Dot />
        <span className="text-muted-foreground font-mono text-[11px] tracking-[0.18em] uppercase">
          {t("home.method.in")}
        </span>
      </div>

      {pipeline.map((s) => (
        <div key={s.stage} className={cn("relative grid border-t pl-6 sm:pl-8", cols)}>
          <Node className="top-9" />
          <div className="py-6">
            <h3 className="font-medium tracking-tight">{s.stage}</h3>
            <p className="text-muted-foreground mt-1 font-mono text-[11px]">{s.name}</p>
          </div>
          <dl className="col-start-1 row-start-2 grid gap-1.5 pb-6 sm:col-start-auto sm:row-start-auto sm:grid-cols-3 sm:gap-4 sm:py-6 sm:pl-6">
            {s.rows.map((r) => (
              <div key={r.k} className="flex items-baseline justify-between gap-3 sm:block">
                <dt className="text-muted-foreground/70 shrink-0 text-[11px]">{r.k}</dt>
                <dd className="text-right font-mono text-[11px] text-pretty sm:mt-1 sm:text-left">{r.v}</dd>
              </div>
            ))}
          </dl>
          <Rail node className="row-span-2 sm:row-span-1" />
          <Rail node className="row-span-2 sm:row-span-1" />
        </div>
      ))}

      {/* the query leaves */}
      <div className={cn("relative grid items-center border-t py-3 pl-6 sm:pl-8", cols)}>
        <Dot />
        <span className="text-muted-foreground font-mono text-[11px] tracking-[0.18em] uppercase">
          {t("home.method.out")}
        </span>
        <Rail />
        <Rail />
      </div>

      {/* each rail ends at its own line */}
      {rails.map((r, i) => (
        <div key={r.label} className={cn("relative grid border-t pl-6 sm:pl-8", cols)}>
          <div className="flex flex-col gap-1 py-4 sm:col-span-2 sm:flex-row sm:items-baseline sm:justify-end sm:gap-6 sm:text-right">
            <span className="text-muted-foreground/70 shrink-0 font-mono text-[11px] tracking-[0.18em] uppercase">
              {r.label}
            </span>
            <span className="text-sm text-pretty">{r.value}</span>
          </div>
          {i === 0 ? (
            <>
              <Rail end />
              <Rail />
            </>
          ) : (
            <>
              <span />
              <Rail end />
            </>
          )}
        </div>
      ))}
    </div>
  );
}

/** A knock-out node on the spine, at the left edge of its row. */
function Node({ className, filled }: { className?: string; filled?: boolean }) {
  return (
    <span
      aria-hidden
      className={cn(
        "absolute -left-[4.5px] size-[9px] rounded-full border",
        filled ? "border-foreground/60 bg-foreground/60" : "border-foreground/40 bg-background",
        className,
      )}
    />
  );
}

/** A small filled dot on the spine: where something enters or leaves. */
function Dot() {
  return (
    <span
      aria-hidden
      className="bg-foreground/60 absolute top-1/2 -left-[2.5px] size-[5px] -translate-y-1/2 rounded-full"
    />
  );
}

/**
 * One cell of a rail: the vertical hairline through the row, a node where it
 * crosses a layer, or its end where it lands on its own line.
 */
function Rail({ node, end, className }: { node?: boolean; end?: boolean; className?: string }) {
  return (
    <span aria-hidden className={cn("relative", className)}>
      <span className={cn("bg-border absolute top-0 left-1/2 w-px", end ? "h-1/2" : "bottom-0")} />
      {node ? (
        <span className="border-foreground/40 bg-background absolute top-9 left-1/2 size-[9px] -translate-x-1/2 rounded-full border" />
      ) : null}
      {end ? (
        <span className="bg-foreground/60 absolute top-1/2 left-1/2 size-[5px] -translate-x-1/2 -translate-y-1/2 rounded-full" />
      ) : null}
    </span>
  );
}
