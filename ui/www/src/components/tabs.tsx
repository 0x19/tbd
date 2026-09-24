"use client";

import { useRef } from "react";

import { cn } from "@/lib/utils";

/**
 * A row of tabs over a hairline, each with its count: the one tab bar the
 * site uses (the playgrounds, the work index). It follows the WAI-ARIA tabs
 * pattern: arrows move between tabs, Home and End jump to the ends, and only
 * the selected tab is in the tab order. The panel it controls is the caller's,
 * with `id={`${id}-panel`}` and `aria-labelledby={`${id}-tab-${value}`}`.
 */
export function TabList<K extends string>({
  id,
  label,
  tabs,
  value,
  onChange,
}: {
  id: string;
  label: string;
  tabs: { key: K; label: string; n: number }[];
  value: K;
  onChange: (key: K) => void;
}) {
  const refs = useRef<(HTMLButtonElement | null)[]>([]);
  const onKey = (e: React.KeyboardEvent, i: number) => {
    const last = tabs.length - 1;
    const next =
      e.key === "ArrowRight"
        ? i === last
          ? 0
          : i + 1
        : e.key === "ArrowLeft"
          ? i === 0
            ? last
            : i - 1
          : e.key === "Home"
            ? 0
            : e.key === "End"
              ? last
              : null;
    if (next === null) return;
    e.preventDefault();
    onChange(tabs[next]!.key);
    refs.current[next]?.focus();
  };
  return (
    <div role="tablist" aria-label={label} className="flex gap-x-7 overflow-x-auto border-b">
      {tabs.map((x, i) => {
        const on = x.key === value;
        return (
          <button
            key={x.key}
            ref={(el) => {
              refs.current[i] = el;
            }}
            role="tab"
            id={`${id}-tab-${x.key}`}
            aria-selected={on}
            aria-controls={`${id}-panel`}
            tabIndex={on ? 0 : -1}
            onClick={() => onChange(x.key)}
            onKeyDown={(e) => onKey(e, i)}
            className={cn(
              "relative -mb-px flex shrink-0 items-baseline gap-1.5 border-b py-3 font-mono text-[11px] tracking-[0.18em] whitespace-nowrap uppercase transition-colors",
              on
                ? "border-foreground text-foreground"
                : "text-muted-foreground hover:text-foreground border-transparent",
            )}
          >
            {x.label}
            <span className={cn("tabular-nums", on ? "text-foreground/50" : "text-muted-foreground/50")}>
              {x.n}
            </span>
          </button>
        );
      })}
    </div>
  );
}
