"use client";

import Link from "next/link";
import type { ReactNode } from "react";

import { Frame, Reveal, SectionHead } from "@/components/kit";
import { DocList, latest, Timeline } from "@/components/pages/lab";
import { Button } from "@/components/ui/button";
import { labs } from "@/data/labs";
import { rfcs, studies } from "@/generated/lab/index";
import { useLang, useT } from "@/lib/i18n";

/**
 * One lab's own page: what it is, its live view when it runs (the `live`
 * slot, filled by the arena's feed), the ways into it, its RFCs and studies,
 * and its timeline from their status logs. `app/lab/<id>/page.tsx` carries the
 * metadata and picks the lab.
 */
export function LabHomeContent({ id, live }: { id: string; live?: ReactNode }) {
  const t = useT();
  const { lang } = useLang();
  const l = labs.find((x) => x.id === id);
  if (!l) return null;
  const mine = <T extends { lab: string }>(xs: T[]) => xs.filter((e) => e.lab === id);
  const docs = [...mine(rfcs), ...mine(studies)];
  let n = 0;
  const next = () => String(++n).padStart(2, "0");
  return (
    <>
      <Frame className="pt-20 pb-10 sm:pt-28">
        <p className="mb-6">
          <Link
            href="/lab/"
            className="text-muted-foreground hover:text-foreground font-mono text-[11px] tracking-[0.14em] uppercase"
          >
            ← {t("lab.back")}
          </Link>
        </p>
        <p className="text-muted-foreground font-mono text-[11px] tracking-[0.18em] uppercase">
          {t("lab.eyebrow")} · {l.id}
        </p>
        <h1 className="mt-4 max-w-3xl text-4xl font-semibold tracking-[-0.035em] text-balance sm:text-6xl">
          {l.name[lang]}
        </h1>
        <p className="text-muted-foreground mt-6 max-w-2xl text-lg text-pretty">{l.what[lang]}</p>
        {l.workbench ? (
          <p className="mt-8 flex flex-wrap gap-3">
            <Button asChild>
              <Link href={l.workbench}>{t("lab.home.workbench")}</Link>
            </Button>
            <Button variant="ghost" asChild>
              <a href="#documents">{t("lab.home.documents")}</a>
            </Button>
          </p>
        ) : null}
      </Frame>

      {l.live ? (
        <Frame className="pb-16">
          <SectionHead n={next()} label={t("lab.home.live.label")} lead={t("lab.home.live.lead")} />
          <Reveal className="mt-10">
            {live ?? <p className="text-muted-foreground text-sm">{t("lab.home.live.none")}</p>}
          </Reveal>
        </Frame>
      ) : null}

      <Frame className="pb-16">
        <SectionHead n={next()} label={t("lab.home.surfaces.label")} lead={t("lab.home.surfaces.lead")} />
        <Reveal className="mt-10">
          <dl className="divide-y border-y">
            {l.surfaces.map((s) => (
              <div key={s} className="grid gap-1 py-4 sm:grid-cols-[9rem_minmax(0,1fr)] sm:gap-6">
                <dt className="font-mono text-sm">{s}</dt>
                <dd className="text-muted-foreground text-sm text-pretty">
                  {t(`lab.surface.${s.toLowerCase()}`)}
                </dd>
              </div>
            ))}
          </dl>
        </Reveal>
      </Frame>

      <Frame className="pb-16">
        <div id="documents" className="scroll-mt-24">
          <SectionHead n={next()} label={t("lab.home.documents")} lead={t("lab.home.documents.lead")} />
        </div>
        <Reveal>
          <DocList label={t("lab.rfcs.label")} entries={mine(rfcs)} empty={t("lab.empty.rfcs")} />
          <DocList label={t("lab.studies.label")} entries={mine(studies)} empty={t("lab.empty.studies")} />
        </Reveal>
      </Frame>

      <Frame className="pb-24 sm:pb-32">
        <SectionHead n={next()} label={t("lab.home.timeline.label")} lead={t("lab.home.timeline.lead")} />
        <Reveal className="mt-10">
          <Timeline lines={latest(docs)} />
        </Reveal>
      </Frame>
    </>
  );
}
