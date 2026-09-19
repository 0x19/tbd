"use client";

import { useEffect, useRef, useState } from "react";

import { cn } from "@/lib/utils";

/** The page's horizontal frame: one measure, one set of gutters, everywhere. */
export function Frame({ children, className }: { children: React.ReactNode; className?: string }) {
  return <div className={cn("mx-auto w-full max-w-6xl px-6 sm:px-8", className)}>{children}</div>;
}

/**
 * A numbered section head: the rule, the number and the label on one line, the
 * statement under it. The pattern repeats down the page so a reader always
 * knows where they are.
 */
export function SectionHead({
  n,
  label,
  title,
  lead,
  className,
}: {
  n: string;
  label: string;
  title?: React.ReactNode;
  lead?: string;
  className?: string;
}) {
  return (
    <div className={cn("border-t pt-5", className)}>
      <div className="text-muted-foreground flex items-baseline gap-3 font-mono text-[11px] tracking-[0.18em] uppercase">
        <span className="text-foreground/40">{n}</span>
        <span>{label}</span>
      </div>
      {title ? (
        <h2 className="mt-7 max-w-3xl text-3xl font-semibold tracking-[-0.02em] text-balance sm:text-4xl">
          {title}
        </h2>
      ) : null}
      {lead ? <p className="text-muted-foreground mt-4 max-w-xl text-pretty">{lead}</p> : null}
    </div>
  );
}

/** Mono, uppercase, wide: the small label that sits above everything else. */
export function Eyebrow({ children, className }: { children: React.ReactNode; className?: string }) {
  return (
    <p className={cn("text-muted-foreground font-mono text-[11px] tracking-[0.18em] uppercase", className)}>
      {children}
    </p>
  );
}

/** Fades its children up once, the first time they come into view. */
export function Reveal({
  children,
  delay = 0,
  className,
}: {
  children: React.ReactNode;
  delay?: number;
  className?: string;
}) {
  const ref = useRef<HTMLDivElement>(null);
  const [shown, setShown] = useState(false);

  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    // No observer, no problem: show it and move on.
    if (typeof IntersectionObserver === "undefined") {
      setShown(true);
      return;
    }
    const io = new IntersectionObserver(
      ([entry]) => {
        if (entry?.isIntersecting) {
          setShown(true);
          io.disconnect();
        }
      },
      { rootMargin: "0px 0px -12% 0px" },
    );
    io.observe(el);
    return () => io.disconnect();
  }, []);

  return (
    <div
      ref={ref}
      className={cn("reveal", className)}
      data-shown={shown}
      style={delay ? { transitionDelay: `${delay}ms` } : undefined}
    >
      {children}
    </div>
  );
}

/** An endless, unhurried line of words; paused for anyone who asks for less motion. */
export function Marquee({ items }: { items: readonly string[] }) {
  const line = [...items, ...items];
  return (
    <div
      className="relative overflow-hidden py-5"
      style={{
        maskImage: "linear-gradient(to right, transparent, black 8%, black 92%, transparent)",
        WebkitMaskImage: "linear-gradient(to right, transparent, black 8%, black 92%, transparent)",
      }}
      aria-hidden
    >
      <div className="marquee-track flex w-max items-center gap-10">
        {line.map((item, i) => (
          <span
            key={`${item}-${i}`}
            className="text-muted-foreground/70 font-mono text-xs tracking-[0.16em] whitespace-nowrap uppercase"
          >
            {item}
          </span>
        ))}
      </div>
    </div>
  );
}

/** A small monospace tag: a language, a piece of the stack. */
export function Tag({ children }: { children: React.ReactNode }) {
  return (
    <span className="ring-border text-muted-foreground rounded-md px-1.5 py-0.5 font-mono text-[11px] ring-1 ring-inset">
      {children}
    </span>
  );
}

/**
 * One row of the work index: a number, the name, what it does, and an arrow
 * that arrives on hover. The whole row is the target.
 */
export function IndexRow({
  n,
  name,
  meta,
  year,
  children,
  href,
}: {
  n: string;
  name: string;
  meta?: string;
  year?: string;
  children: React.ReactNode;
  href: string;
}) {
  const external = href.startsWith("http");
  return (
    <a
      href={href}
      {...(external ? { target: "_blank", rel: "noreferrer" } : {})}
      className="group hover:bg-muted/40 -mx-4 flex flex-col gap-2 border-b px-4 py-6 transition-colors sm:flex-row sm:items-baseline sm:gap-8"
    >
      <span className="text-muted-foreground/60 w-8 shrink-0 font-mono text-[11px] tabular-nums">{n}</span>
      <span className="flex w-full shrink-0 items-baseline gap-2 sm:w-56">
        <span className="font-mono text-base font-medium tracking-tight">{name}</span>
        {meta ? <span className="text-muted-foreground text-[11px]">{meta}</span> : null}
      </span>
      <span className="text-muted-foreground group-hover:text-foreground flex-1 text-sm text-pretty transition-colors">
        {children}
      </span>
      {year ? (
        <span className="text-muted-foreground/60 shrink-0 font-mono text-[11px] tabular-nums">{year}</span>
      ) : null}
      <span className="text-muted-foreground hidden shrink-0 translate-x-0 transition-transform group-hover:translate-x-1 sm:block">
        ↗
      </span>
    </a>
  );
}
