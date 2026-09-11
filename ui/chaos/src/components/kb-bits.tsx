"use client";

import {
  Activity,
  Boxes,
  FlaskConical,
  Lightbulb,
  type LucideIcon,
  Package,
  Rocket,
  Server,
} from "lucide-react";
import Link from "next/link";

import { Badge } from "@/components/ui/badge";
import { ago } from "@/lib/format";
import { docHref, type KbDoc, readMinutes } from "@/lib/kb";

/** One icon per category, the kit's icon tile style. */
export const CATEGORY_ICONS: Record<string, LucideIcon> = {
  start: Rocket,
  chaos: FlaskConical,
  observability: Activity,
  services: Boxes,
  deployment: Server,
  crates: Package,
  design: Lightbulb,
};

export function CategoryIcon({ id, className }: { id: string; className?: string }) {
  const Icon = CATEGORY_ICONS[id] ?? Package;
  return <Icon className={className ?? "size-4"} />;
}

/** The dot-separated chip row under a title: source path, length, freshness, flags. */
export function DocMeta({ doc, advisory }: { doc: KbDoc; advisory: boolean }) {
  return (
    <div className="text-muted-foreground flex flex-wrap items-center gap-x-2 gap-y-1 text-xs">
      <span className="text-foreground font-mono">{doc.path}</span>
      <span className="text-muted-foreground/60">·</span>
      <span>{readMinutes(doc.words)} min read</span>
      {doc.updated ? (
        <>
          <span className="text-muted-foreground/60">·</span>
          <span title={doc.updated}>updated {ago(doc.updated)}</span>
        </>
      ) : null}
      {doc.generated ? (
        <Badge variant="outline" className="text-[10px]" title="Written by mise run chaos:docs from the code">
          generated
        </Badge>
      ) : null}
      {advisory ? (
        <Badge variant="outline" className="text-[10px]" title="Earlier idea material, not a spec">
          idea material
        </Badge>
      ) : null}
    </div>
  );
}

/** A document as a list row: title, summary, meta. */
export function DocRow({ doc, hash = "" }: { doc: KbDoc; hash?: string }) {
  return (
    <Link
      href={docHref(doc.id, hash)}
      className="hover:bg-muted/40 block rounded-lg px-3 py-3 transition-colors"
    >
      <div className="flex flex-wrap items-baseline justify-between gap-2">
        <span className="font-medium">{doc.title}</span>
        <span className="text-muted-foreground font-mono text-[11px]">{doc.path}</span>
      </div>
      {doc.summary ? (
        <p className="text-muted-foreground mt-1 line-clamp-2 text-sm leading-6">{doc.summary}</p>
      ) : null}
    </Link>
  );
}
