"use client";

import Link from "next/link";

import { Frame, IndexRow, Reveal, SectionHead, Tag } from "@/components/kit";
import { StatusStamp } from "@/components/lab/status-stamp";
import { rfcs, studies } from "@/generated/lab/index";
import { useT } from "@/lib/i18n";
import { useSite } from "@/lib/i18n/site";

/**
 * The lab index: what is live, the RFCs, the studies. The prose of every page
 * is English in both languages; the chrome translates. `app/lab/page.tsx`
 * carries the metadata.
 */
export function LabContent() {
  const t = useT();
  const { playgrounds } = useSite();
  const live = playgrounds.filter((p) => p.href?.startsWith("/lab/"));
  return (
    <>
      <Frame className="pt-20 pb-10 sm:pt-28">
        <p className="text-muted-foreground font-mono text-[11px] tracking-[0.18em] uppercase">
          {t("lab.eyebrow")}
        </p>
        <h1 className="mt-4 max-w-3xl text-4xl font-semibold tracking-[-0.035em] text-balance sm:text-6xl">
          {t("lab.title")}
        </h1>
        <p className="text-muted-foreground mt-6 max-w-xl text-lg text-pretty">{t("lab.lead")}</p>
        <p className="text-muted-foreground mt-3 max-w-xl text-sm">{t("lab.english_only")}</p>
      </Frame>

      <Frame className="pb-16">
        <SectionHead n="01" label={t("lab.live.label")} lead={t("lab.live.lead")} />
        <Reveal className="mt-10">
          <div className="grid gap-4 sm:grid-cols-2">
            {live.map((p) => (
              <Link
                key={p.href}
                href={p.href!}
                className="group hover:bg-muted/40 flex flex-col gap-3 rounded-lg border p-6 transition-colors"
              >
                <div className="flex items-center justify-between">
                  <span className="font-mono text-base font-medium tracking-tight">{p.name}</span>
                  <Tag>{p.tag}</Tag>
                </div>
                <p className="text-muted-foreground group-hover:text-foreground text-sm text-pretty transition-colors">
                  {p.what}
                </p>
              </Link>
            ))}
            <Link
              href="/lab/demo/"
              className="group hover:bg-muted/40 flex flex-col gap-3 rounded-lg border border-dashed p-6 transition-colors"
            >
              <div className="flex items-center justify-between">
                <span className="font-mono text-base font-medium tracking-tight">{t("lab.demo.title")}</span>
                <Tag>{t("lab.demo.tag")}</Tag>
              </div>
              <p className="text-muted-foreground group-hover:text-foreground text-sm text-pretty transition-colors">
                {t("lab.demo.card")}
              </p>
            </Link>
          </div>
        </Reveal>
      </Frame>

      <Frame className="pb-16">
        <SectionHead n="02" label={t("lab.rfcs.label")} lead={t("lab.rfcs.lead")} />
        <Reveal className="mt-10">
          {rfcs.length === 0 ? (
            <p className="text-muted-foreground text-sm">{t("lab.empty.rfcs")}</p>
          ) : (
            <div>
              {rfcs.map((r) => (
                <IndexRow key={r.slug} n={r.number} name={r.title} year={r.date} href={r.href}>
                  <span className="flex flex-col gap-2 sm:flex-row sm:items-baseline sm:gap-3">
                    <StatusStamp status={r.status} className="shrink-0" />
                    <span>{r.summary}</span>
                  </span>
                </IndexRow>
              ))}
            </div>
          )}
        </Reveal>
      </Frame>

      <Frame className="pb-24 sm:pb-32">
        <SectionHead n="03" label={t("lab.studies.label")} lead={t("lab.studies.lead")} />
        <Reveal className="mt-10">
          {studies.length === 0 ? (
            <p className="text-muted-foreground text-sm">{t("lab.empty.studies")}</p>
          ) : (
            <div>
              {studies.map((s) => (
                <IndexRow key={s.slug} n={s.number} name={s.title} year={s.date} href={s.href}>
                  <span className="flex flex-col gap-2 sm:flex-row sm:items-baseline sm:gap-3">
                    <StatusStamp status={s.status} className="shrink-0" />
                    {s.headline ? (
                      <span
                        className="text-foreground shrink-0 font-mono text-sm tabular-nums"
                        title={s.headlineNote}
                      >
                        {s.headline}
                      </span>
                    ) : null}
                    <span>{s.summary}</span>
                  </span>
                </IndexRow>
              ))}
            </div>
          )}
        </Reveal>
      </Frame>
    </>
  );
}
