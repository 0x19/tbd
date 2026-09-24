"use client";

import Link from "next/link";
import { useId, useState } from "react";

import { Frame, IndexRow, Reveal, SectionHead } from "@/components/kit";
import { StatusStamp } from "@/components/lab/status-stamp";
import { TabList } from "@/components/tabs";
import { LabLiveStrip } from "@/components/workbench/lab-live";
import { labs } from "@/data/labs";
import { rfcs, studies } from "@/generated/lab/index";
import { useLang, useT } from "@/lib/i18n";
import type { LabEntry } from "@/lib/lab";
import { useDrafts } from "@/lib/lab-drafts";
import { useMe } from "@/lib/me";

/** Every status-log line of every public document, newest first, with where it came from. */
export function latest(entries: LabEntry[], limit?: number) {
  const lines = entries.flatMap((e) =>
    e.log.map((l, i) => ({ ...l, entry: e, key: `${e.kind}-${e.slug}-${i}` })),
  );
  // Newest date first; within a day, the later line of a document first.
  lines.sort((a, b) => b.date.localeCompare(a.date) || b.key.localeCompare(a.key));
  return limit ? lines.slice(0, limit) : lines;
}

/**
 * The lab index: one card per lab (what it is, how many documents, where to
 * go), the latest lines of every status log as a feed of what changed, and
 * every RFC and study with a tab per lab. The prose of every document is
 * English in both languages; the chrome translates. `app/lab/page.tsx`
 * carries the metadata.
 */
export function LabContent() {
  const t = useT();
  const { lang } = useLang();
  const id = useId();
  const [tab, setTab] = useState<string>("all");
  // An admin also sees the drafts, stamped as such (src/lib/lab-drafts.ts).
  const drafts = useDrafts().entries;
  const allRfcs = [...drafts.filter((e) => e.kind === "rfc"), ...rfcs];
  const allStudies = [...drafts.filter((e) => e.kind === "study"), ...studies];
  const all = [...allRfcs, ...allStudies];
  // A lab whose documents are all drafts is itself a draft: its card and tab show
  // only to an admin, so publishing the lab never publishes a lab nobody wrote up.
  const admin = useMe()?.role === "admin";
  const shownLabs = labs.filter((l) => admin || [...rfcs, ...studies].some((e) => e.lab === l.id));
  const tabs = [
    { key: "all", label: t("lab.index.all"), n: all.length },
    ...shownLabs.map((l) => ({ key: l.id, label: l.id, n: all.filter((e) => e.lab === l.id).length })),
  ];
  const inTab = (e: LabEntry) => tab === "all" || e.lab === tab;
  const shownRfcs = allRfcs.filter(inTab);
  const shownStudies = allStudies.filter(inTab);
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
        <SectionHead n="01" label={t("lab.labs.label")} lead={t("lab.labs.lead")} />
        <Reveal className="mt-10">
          <div className="grid gap-4 lg:grid-cols-2">
            {shownLabs.map((l) => {
              const docs = all.filter((e) => e.lab === l.id);
              const last = latest(docs, 1)[0];
              return (
                <article key={l.id} className="flex flex-col rounded-lg border">
                  <Link
                    href={l.href}
                    className="group hover:bg-muted/30 flex flex-1 flex-col gap-4 p-6 transition-colors sm:p-8"
                  >
                    <p className="text-muted-foreground flex items-center gap-3 font-mono text-[11px] tracking-[0.18em] uppercase">
                      <span>{l.id}</span>
                      <span className="tabular-nums">
                        {t("lab.counts", {
                          rfcs: docs.filter((e) => e.kind === "rfc").length,
                          studies: docs.filter((e) => e.kind === "study").length,
                        })}
                      </span>
                      <span aria-hidden className="ml-auto transition-transform group-hover:translate-x-1">
                        →
                      </span>
                    </p>
                    <h2 className="text-2xl font-medium tracking-tight text-balance">{l.name[lang]}</h2>
                    <p className="text-muted-foreground text-sm text-pretty">{l.what[lang]}</p>
                    {l.live ? <LabLiveStrip /> : null}
                    {last ? (
                      <p className="text-muted-foreground/80 mt-auto border-t pt-4 text-sm text-pretty">
                        <span className="text-foreground font-mono text-xs tabular-nums">{last.date}</span>{" "}
                        <span className="lab-inline" dangerouslySetInnerHTML={{ __html: last.html }} />
                      </p>
                    ) : null}
                  </Link>
                  <div className="flex flex-wrap gap-x-6 gap-y-2 border-t px-6 py-4 font-mono text-[11px] tracking-[0.14em] uppercase sm:px-8">
                    <Link href={l.href} className="hover:text-foreground text-muted-foreground">
                      {t("lab.open")}
                    </Link>
                    {l.workbench ? (
                      <Link href={l.workbench} className="hover:text-foreground text-muted-foreground">
                        {t("lab.workbench")}
                      </Link>
                    ) : null}
                  </div>
                </article>
              );
            })}
          </div>
        </Reveal>
      </Frame>

      <Frame className="pb-16">
        <SectionHead n="02" label={t("lab.latest.label")} lead={t("lab.latest.lead")} />
        <Reveal className="mt-10">
          <Timeline lines={latest(all, 10)} />
        </Reveal>
      </Frame>

      <Frame className="pb-24 sm:pb-32">
        <SectionHead n="03" label={t("lab.index.label")} lead={t("lab.index.lead")} />
        <Reveal className="mt-10">
          <TabList id={id} label={t("lab.index.label")} tabs={tabs} value={tab} onChange={setTab} />
          <div role="tabpanel" id={`${id}-panel`} aria-labelledby={`${id}-tab-${tab}`}>
            <DocList label={t("lab.rfcs.label")} entries={shownRfcs} empty={t("lab.empty.rfcs")} />
            <DocList label={t("lab.studies.label")} entries={shownStudies} empty={t("lab.empty.studies")} />
          </div>
        </Reveal>
      </Frame>
    </>
  );
}

/** Status-log lines as a dated list, each linking to its document. */
export function Timeline({ lines }: { lines: ReturnType<typeof latest> }) {
  const t = useT();
  if (!lines.length) return <p className="text-muted-foreground text-sm">{t("lab.latest.empty")}</p>;
  return (
    <ol className="divide-y border-y">
      {lines.map((l) => (
        <li key={l.key} className="grid gap-2 py-4 sm:grid-cols-[7rem_9rem_minmax(0,1fr)] sm:gap-6">
          <span className="font-mono text-xs tabular-nums">{l.date}</span>
          <Link
            href={l.entry.href}
            className="text-muted-foreground hover:text-foreground font-mono text-[11px] tracking-[0.14em] uppercase"
            title={l.entry.title}
          >
            {l.entry.kind === "rfc" ? t("lab.rfc") : t("lab.study")} {l.entry.number}
          </Link>
          <span
            className="lab-inline text-muted-foreground text-sm text-pretty"
            dangerouslySetInnerHTML={{ __html: l.html }}
          />
        </li>
      ))}
    </ol>
  );
}

/** RFCs or studies as index rows with their stamp (and a study's headline). */
export function DocList({ label, entries, empty }: { label: string; entries: LabEntry[]; empty: string }) {
  return (
    <div className="mt-10">
      <p className="text-muted-foreground mb-2 font-mono text-[11px] tracking-[0.18em] uppercase">{label}</p>
      {entries.length === 0 ? (
        <p className="text-muted-foreground text-sm">{empty}</p>
      ) : (
        <div>
          {entries.map((e) => (
            <IndexRow key={e.slug} n={e.number} name={e.title} year={e.date} href={e.href}>
              <span className="flex flex-col gap-2 sm:flex-row sm:items-baseline sm:gap-3">
                {isDraft(e) ? <DraftStamp /> : null}
                <StatusStamp status={e.status} className="shrink-0" />
                {e.headline ? (
                  <span
                    className="text-foreground shrink-0 font-mono text-sm tabular-nums"
                    title={e.headlineNote}
                  >
                    {e.headline}
                  </span>
                ) : null}
                <span>{e.summary}</span>
              </span>
            </IndexRow>
          ))}
        </div>
      )}
    </div>
  );
}

/** Whether an entry is a draft: its link is the admin-only draft page. */
export function isDraft(e: LabEntry): boolean {
  return e.href.startsWith("/lab/draft/");
}

/** The mark a draft carries in every list, next to its status. */
export function DraftStamp() {
  const t = useT();
  return (
    <span className="shrink-0 rounded-sm border border-dashed px-1.5 py-0.5 font-mono text-[10px] tracking-[0.14em] whitespace-nowrap uppercase">
      {t("drafts.stamp")}
    </span>
  );
}
