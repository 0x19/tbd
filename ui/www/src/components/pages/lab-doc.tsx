"use client";

import Link from "next/link";

import { Frame } from "@/components/kit";
import { StatusStamp } from "@/components/lab/status-stamp";
import { useT } from "@/lib/i18n";
import type { LabDoc } from "@/lib/lab";

/**
 * One RFC or study: the eyebrow with its number, stamp and date, the title,
 * the summary, the links to what it supersedes or measures, and the prose as
 * the generator rendered it (English in both languages; the chrome translates).
 * `[REDACTED: ...]` marks are black bars with the reason on hover.
 */
export function LabDocContent({ doc }: { doc: LabDoc }) {
  const t = useT();
  const label = doc.kind === "rfc" ? t("lab.rfc") : t("lab.study");
  const related: { key: string; slug: string; href: string }[] = [];
  if (doc.supersedes)
    related.push({ key: "lab.supersedes", slug: doc.supersedes, href: `/lab/rfc/${doc.supersedes}/` });
  if (doc.supersededBy) {
    related.push({ key: "lab.superseded_by", slug: doc.supersededBy, href: `/lab/rfc/${doc.supersededBy}/` });
  }
  if (doc.rfc) related.push({ key: "lab.measures", slug: doc.rfc, href: `/lab/rfc/${doc.rfc}/` });
  return (
    <>
      <Frame className="pt-20 pb-8 sm:pt-28">
        <p className="text-muted-foreground flex flex-wrap items-center gap-3 font-mono text-[11px] tracking-[0.18em] uppercase">
          <span>
            {label} {doc.number}
          </span>
          <StatusStamp status={doc.status} />
          <span className="tabular-nums">{doc.date}</span>
        </p>
        <h1 className="mt-4 max-w-3xl text-4xl font-semibold tracking-[-0.035em] text-balance sm:text-5xl">
          {doc.title}
        </h1>
        <p className="text-muted-foreground mt-6 max-w-2xl text-lg text-pretty">{doc.summary}</p>
        {doc.headline ? (
          <p className="mt-6 flex items-baseline gap-3">
            <span className="font-mono text-3xl tabular-nums">{doc.headline}</span>
            {doc.headlineNote ? (
              <span className="text-muted-foreground text-sm">{doc.headlineNote}</span>
            ) : null}
          </p>
        ) : null}
        {related.length ? (
          <p className="text-muted-foreground mt-6 flex flex-wrap gap-x-6 gap-y-1 text-sm">
            {related.map((r) => (
              <span key={r.key + r.slug}>
                {t(r.key)}{" "}
                <Link href={r.href} className="text-foreground underline underline-offset-4">
                  {r.slug}
                </Link>
              </span>
            ))}
          </p>
        ) : null}
      </Frame>
      <Frame className="pb-24 sm:pb-32">
        <article
          className="lab-prose max-w-3xl border-t pt-10"
          dangerouslySetInnerHTML={{ __html: doc.html }}
        />
        <p className="mt-16">
          <Link
            href="/lab/"
            className="text-muted-foreground hover:text-foreground font-mono text-[11px] tracking-[0.14em] uppercase"
          >
            ← {t("lab.back")}
          </Link>
        </p>
      </Frame>
    </>
  );
}
