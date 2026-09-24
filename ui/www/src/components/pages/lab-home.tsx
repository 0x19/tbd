"use client";

import Link from "next/link";

import { Frame, Reveal, SectionHead } from "@/components/kit";
import { DocList, latest, Timeline } from "@/components/pages/lab";
import { Button } from "@/components/ui/button";
import { LabLive } from "@/components/workbench/lab-live";
import { labs } from "@/data/labs";
import { rfcs, studies } from "@/generated/lab/index";
import { useLang, useT } from "@/lib/i18n";
import { useDrafts } from "@/lib/lab-drafts";

/**
 * One lab's own page: what it is, its live view when it runs (the arena's
 * feed, for a lab with `live`), the ways into it, its RFCs and studies (an
 * admin also sees its drafts, stamped), and its timeline from their status
 * logs. `app/lab/<id>/page.tsx` carries the metadata and picks the lab.
 */
export function LabHomeContent({ id }: { id: string }) {
  const t = useT();
  const { lang } = useLang();
  // A way in without a sentence in the dictionary shows its name alone.
  const has = (key: string) => t(key) !== key;
  const drafts = useDrafts(id).entries;
  const l = labs.find((x) => x.id === id);
  if (!l) return null;
  const mine = <T extends { lab: string }>(xs: T[]) => xs.filter((e) => e.lab === id);
  // An admin sees this lab's drafts beside the published documents, stamped.
  const labRfcs = [...mine(rfcs), ...drafts.filter((e) => e.kind === "rfc")].sort((a, b) =>
    b.number.localeCompare(a.number),
  );
  const labStudies = [...mine(studies), ...drafts.filter((e) => e.kind === "study")].sort(
    (a, b) => b.date.localeCompare(a.date) || b.number.localeCompare(a.number),
  );
  const docs = [...labRfcs, ...labStudies];
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
            <LabLive />
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
                  {has(`lab.surface.${s.toLowerCase()}`) ? t(`lab.surface.${s.toLowerCase()}`) : null}
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
          <DocList label={t("lab.rfcs.label")} entries={labRfcs} empty={t("lab.empty.rfcs")} />
          <DocList label={t("lab.studies.label")} entries={labStudies} empty={t("lab.empty.studies")} />
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
