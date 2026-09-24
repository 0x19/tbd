"use client";

import { useCallback, useEffect, useId, useState } from "react";

import { Eyebrow, Frame, Reveal } from "@/components/kit";
import { TabList } from "@/components/tabs";
import { issues } from "@/generated/radar/index";
import { useLang, useT } from "@/lib/i18n";
import { useMe } from "@/lib/me";
import {
  type ImpactKey,
  isArchive,
  issueLabel,
  type RadarChange as Change,
  type RadarDigest as Digest,
} from "@/lib/radar";
import { cn } from "@/lib/utils";

/**
 * The Radar (Quiet Pager): the week's digest per language, in the reader's
 * language when there is one, from the radar service on the same origin
 * (`GET /v1/radar/digests`, open and rate-limited at Envoy; with
 * `include_drafts=true`, which the page sends only for an admin, through the
 * lab's gate instead). A digest is a draft until
 * the owner publishes it: an admin sees drafts, stamped, with Publish and
 * Unpublish; everyone else sees published issues only (the service decides).
 * A digest the llm's stub engine wrote is never shown. The model's text is
 * rendered as React elements (`Prose`: paragraphs, bullets, http(s) links),
 * never as HTML. The published issues are rendered at build time
 * (`tool/radar-data.ts`) so the HTML carries them for search engines and link
 * previews; each week also has its own page, `/radar/2026-w39/`. Field names
 * are the proto's (the gateway keeps them). `app/radar/page.tsx` and
 * `app/radar/[week]/page.tsx` have the metadata.
 */

type State = { kind: "loading" } | { kind: "failed" } | { kind: "ready"; digests: Digest[] };

const IMPACTS: { key: ImpactKey; label: string }[] = [
  { key: "IMPACT_BREAKING", label: "radar.impact.breaking" },
  { key: "IMPACT_WORTH_KNOWING", label: "radar.impact.worth" },
  { key: "IMPACT_NICE_TO_KNOW", label: "radar.impact.nice" },
];

/** Monday and Sunday of an ISO week label ("2026-W39"). */
export function isoWeekRange(week: string): [Date, Date] | null {
  const m = /^(\d{4})-W(\d{2})$/.exec(week);
  if (!m) return null;
  const year = Number(m[1]);
  const n = Number(m[2]);
  // The ISO week containing 4 January is week 1.
  const jan4 = new Date(Date.UTC(year, 0, 4));
  const monday1 = new Date(jan4);
  monday1.setUTCDate(jan4.getUTCDate() - ((jan4.getUTCDay() + 6) % 7));
  const monday = new Date(monday1);
  monday.setUTCDate(monday1.getUTCDate() + (n - 1) * 7);
  const sunday = new Date(monday);
  sunday.setUTCDate(monday.getUTCDate() + 6);
  return [monday, sunday];
}

function weekLabel(
  week: string,
  lang: string,
  t: (k: string, v?: Record<string, string | number>) => string,
) {
  const n = Number(week.split("-W")[1] ?? 0);
  const range = isoWeekRange(week);
  if (!range) return week;
  const fmt = new Intl.DateTimeFormat(lang === "hr" ? "hr-HR" : "en-US", {
    day: "numeric",
    month: "short",
    year: "numeric",
    timeZone: "UTC",
  });
  return `${t("radar.week", { n })} · ${fmt.formatRange(range[0], range[1])}`;
}

function readParam(name: string): string | null {
  if (typeof window === "undefined") return null;
  return new URLSearchParams(window.location.search).get(name);
}

/** Every published digest the build rendered, newest week first. */
const built: Digest[] = issues.flatMap((i) => i.digests);

/** The build's digests with a fetch laid over them: a fetched digest replaces
 *  the built one with its id, and the rest of the build stays, since the fetch
 *  asks only for the newest few and the archive is in the build. Newest first. */
function merged(fetched: Digest[]): Digest[] {
  const byId = new Map(built.map((d) => [d.id, d]));
  for (const d of fetched) byId.set(d.id, d);
  return [...byId.values()].sort((a, b) => b.week.localeCompare(a.week));
}

/** The issue number of a week, when it is published. */
const numberOf = (week: string) => issues.find((i) => i.week === week)?.number;

export function RadarContent({ fixedWeek }: { fixedWeek?: string } = {}) {
  const t = useT();
  const { lang } = useLang();
  const id = useId();
  const me = useMe();
  const admin = me?.role === "admin";
  // The build's issues first, so the HTML carries them; the browser then asks
  // the service for anything newer (and an admin for drafts).
  const [state, setState] = useState<State>(
    built.length ? { kind: "ready", digests: built } : { kind: "loading" },
  );
  const [language, setLanguage] = useState<"go" | "rust">("go");
  const [week, setWeek] = useState<string | null>(fixedWeek ?? null);
  const [impact, setImpact] = useState<ImpactKey | "all">("all");
  const [busy, setBusy] = useState(false);

  // Drafts only for an admin: that read goes through the lab's gate at Envoy,
  // which a visitor's request must never hit.
  const load = useCallback(async (drafts: boolean) => {
    try {
      const res = await fetch(`/v1/radar/digests?limit=80${drafts ? "&include_drafts=true" : ""}`, {
        cache: "no-store",
        headers: { accept: "application/json" },
      });
      if (!res.ok) throw new Error(String(res.status));
      const body = (await res.json()) as { digests?: Digest[] };
      setState({ kind: "ready", digests: merged((body.digests ?? []).filter((d) => !d.stub)) });
    } catch {
      // The build's issues still read; only an empty build is a failure.
      setState(built.length ? { kind: "ready", digests: built } : { kind: "failed" });
    }
  }, []);

  useEffect(() => {
    const l = readParam("language");
    if (l === "go" || l === "rust") setLanguage(l);
    if (!fixedWeek) setWeek(readParam("week"));
    void load(false);
  }, [load, fixedWeek]);

  // Reload once the visitor is known to be an admin, so drafts appear.
  useEffect(() => {
    if (admin) void load(true);
  }, [admin, load]);

  const remember = (next: { language?: string; week?: string | null }) => {
    const q = new URLSearchParams(window.location.search);
    if (next.language) q.set("language", next.language);
    if (next.week !== undefined) {
      if (next.week) q.set("week", next.week);
      else q.delete("week");
    }
    const s = q.toString();
    window.history.replaceState(null, "", s ? `?${s}` : window.location.pathname);
  };

  const digests = state.kind === "ready" ? state.digests : [];
  const ofLanguage = digests.filter((d) => d.language === language);
  const weeks = [...new Set(ofLanguage.map((d) => d.week))];
  // The archive list, by year, newest first.
  const years = [...new Set(weeks.map((w) => w.slice(0, 4)))];
  const shownWeek = week && weeks.includes(week) ? week : weeks[0];
  const current =
    ofLanguage.find((d) => d.week === shownWeek && d.lang === lang) ??
    ofLanguage.find((d) => d.week === shownWeek && d.lang === "en") ??
    ofLanguage.find((d) => d.week === shownWeek);
  const changes = current?.changes ?? [];
  const shownChanges = impact === "all" ? changes : changes.filter((c) => c.impact === impact);

  const tabs = (["go", "rust"] as const).map((key) => ({
    key,
    label: key === "go" ? "Go" : "Rust",
    n: new Set(digests.filter((d) => d.language === key).map((d) => d.week)).size,
  }));

  const publish = async (d: Digest, value: boolean) => {
    setBusy(true);
    try {
      await fetch(`/v1/radar/digests/${d.id}/publish`, {
        method: "POST",
        headers: { "content-type": "application/json", accept: "application/json" },
        body: JSON.stringify({ publish: value }),
      });
      await load(true);
    } finally {
      setBusy(false);
    }
  };

  return (
    <>
      <Frame className="pt-20 pb-12 sm:pt-28">
        <Eyebrow>{t("radar.eyebrow")}</Eyebrow>
        <h1 className="mt-6 max-w-3xl text-4xl font-semibold tracking-[-0.035em] text-balance sm:text-6xl">
          {t("radar.title")}
        </h1>
        <p className="text-muted-foreground mt-6 max-w-xl text-lg text-pretty">{t("radar.lead")}</p>
        <p className="text-muted-foreground/70 mt-4 font-mono text-[11px] tracking-[0.14em] uppercase">
          {t("radar.fineprint")}
        </p>
      </Frame>

      <Frame className="pb-24 sm:pb-32">
        <Reveal>
          <TabList
            id={id}
            label={t("radar.langs")}
            tabs={tabs}
            value={language}
            onChange={(k) => {
              setLanguage(k);
              setWeek(null);
              setImpact("all");
              remember({ language: k, week: null });
            }}
          />
          <div
            role="tabpanel"
            id={`${id}-panel`}
            aria-labelledby={`${id}-tab-${language}`}
            className="border-b"
          >
            {state.kind === "loading" ? (
              <p className="text-muted-foreground py-12 text-sm">{t("radar.loading")}</p>
            ) : state.kind === "failed" ? (
              <p className="text-muted-foreground py-12 text-sm">{t("radar.failed")}</p>
            ) : !current ? (
              <p className="text-muted-foreground py-12 text-sm text-pretty">{t("radar.empty")}</p>
            ) : (
              <article className="grid gap-10 py-10 lg:grid-cols-[minmax(0,1fr)_17rem]">
                <div>
                  <p className="text-muted-foreground flex flex-wrap items-center gap-x-4 gap-y-1 font-mono text-[11px] tracking-[0.18em] uppercase">
                    <span className="text-foreground">
                      {isArchive(current)
                        ? `${t("radar.archive_label")} · `
                        : numberOf(current.week)
                          ? `Radar ${issueLabel(numberOf(current.week) ?? 0)} · `
                          : ""}
                      {weekLabel(current.week, lang, t)}
                    </span>
                    <span>{t("radar.from", { n: current.item_count })}</span>
                  </p>

                  {current.status === "draft" ? (
                    <div className="mt-5 flex flex-wrap items-center gap-3 rounded-md border border-dashed px-4 py-3">
                      <span className="font-mono text-[11px] tracking-[0.14em] uppercase">
                        {t("radar.draft")}
                      </span>
                      {admin ? (
                        <button
                          type="button"
                          disabled={busy}
                          onClick={() => void publish(current, true)}
                          className="bg-foreground text-background ml-auto rounded-md px-3 py-1.5 text-sm font-medium disabled:opacity-50"
                        >
                          {busy ? t("radar.working") : t("radar.publish")}
                        </button>
                      ) : null}
                    </div>
                  ) : admin ? (
                    <div className="mt-5 flex justify-end">
                      <button
                        type="button"
                        disabled={busy}
                        onClick={() => void publish(current, false)}
                        className="text-muted-foreground hover:text-foreground rounded-md border px-3 py-1.5 text-sm disabled:opacity-50"
                      >
                        {busy ? t("radar.working") : t("radar.unpublish")}
                      </button>
                    </div>
                  ) : null}

                  {changes.length ? (
                    <>
                      {current.summary ? (
                        <p className="mt-8 max-w-2xl text-lg text-pretty">{current.summary}</p>
                      ) : null}
                      <div role="group" aria-label={t("radar.impacts")} className="mt-8 flex flex-wrap gap-2">
                        {[{ key: "all" as const, label: "radar.impact.all" }, ...IMPACTS].map((f) => {
                          const n =
                            f.key === "all"
                              ? changes.length
                              : changes.filter((c) => c.impact === f.key).length;
                          if (f.key !== "all" && n === 0) return null;
                          const on = impact === f.key;
                          return (
                            <button
                              key={f.key}
                              type="button"
                              aria-pressed={on}
                              onClick={() => setImpact(f.key)}
                              className={cn(
                                "rounded-full border px-3 py-1 font-mono text-[11px] tracking-[0.12em] uppercase transition-colors",
                                on
                                  ? "border-foreground text-foreground"
                                  : "text-muted-foreground hover:text-foreground",
                              )}
                            >
                              {t(f.label)} <span className="tabular-nums opacity-60">{n}</span>
                            </button>
                          );
                        })}
                      </div>
                      <div className="mt-6 divide-y border-y">
                        {shownChanges.map((c, i) => (
                          <ChangeCard key={`${c.url}-${i}`} c={c} />
                        ))}
                      </div>
                    </>
                  ) : (
                    <>
                      <Section title={t("radar.changed")} text={current.changed} />
                      <Section title={t("radar.why")} text={current.why} />
                    </>
                  )}

                  <Section title={t("radar.drill")} text={current.drill} />
                  <details className="mt-10 border-t pt-6">
                    <summary className="cursor-pointer font-medium tracking-tight">
                      {t("radar.script")}
                    </summary>
                    <p className="text-muted-foreground mt-4 text-pretty whitespace-pre-line">
                      {current.script}
                    </p>
                  </details>
                  <p className="text-muted-foreground/70 mt-8 font-mono text-[11px]">
                    {t("radar.generated", { model: current.model })} ·{" "}
                    {isArchive(current)
                      ? t("radar.archive_note")
                      : current.status === "draft"
                        ? t("radar.unreviewed")
                        : t("radar.reviewed")}
                  </p>
                </div>

                {weeks.length > 0 ? (
                  <nav aria-label={t("radar.archive")}>
                    <p className="text-muted-foreground font-mono text-[11px] tracking-[0.18em] uppercase">
                      {t("radar.archive")}
                    </p>
                    {years.map((year, yi) => {
                      const ofYear = weeks.filter((w) => w.startsWith(year));
                      return (
                        <details key={year} open={yi === 0} className="mt-3 border-t pt-2">
                          <summary className="text-foreground cursor-pointer py-1 text-sm font-medium">
                            {year}{" "}
                            <span className="text-muted-foreground font-mono text-[11px]">
                              {ofYear.length}
                            </span>
                          </summary>
                          <ul className="mt-1 divide-y border-y">
                            {ofYear.map((w) => {
                              const d =
                                ofLanguage.find((x) => x.week === w && x.lang === lang) ??
                                ofLanguage.find((x) => x.week === w);
                              const breaking = (d?.changes ?? []).filter(
                                (c) => c.impact === "IMPACT_BREAKING",
                              ).length;
                              const draft = ofLanguage.some((x) => x.week === w && x.status === "draft");
                              return (
                                <li key={w}>
                                  <a
                                    href={`/radar/${w.toLowerCase()}/`}
                                    onClick={(e) => {
                                      // Stay on the page when the week is loaded here; the link
                                      // is the week's own page for crawlers and new tabs.
                                      if (e.metaKey || e.ctrlKey || e.shiftKey || e.button !== 0) return;
                                      e.preventDefault();
                                      setWeek(w);
                                      setImpact("all");
                                      remember({ week: w });
                                    }}
                                    aria-current={w === shownWeek ? "true" : undefined}
                                    className={cn(
                                      "flex w-full flex-col gap-0.5 py-2.5 text-left transition-colors",
                                      w === shownWeek
                                        ? "text-foreground"
                                        : "text-muted-foreground hover:text-foreground",
                                    )}
                                  >
                                    <span className="text-sm">
                                      {numberOf(w)
                                        ? `${issueLabel(numberOf(w) ?? 0)} · `
                                        : d && isArchive(d)
                                          ? `${t("radar.archive_label")} · `
                                          : ""}
                                      {weekLabel(w, lang, t)}
                                    </span>
                                    <span className="flex gap-3 font-mono text-[11px]">
                                      {breaking ? (
                                        <span className="text-destructive">
                                          {t("radar.breaking_n", { n: breaking })}
                                        </span>
                                      ) : null}
                                      {draft ? <span>{t("radar.draft")}</span> : null}
                                    </span>
                                  </a>
                                </li>
                              );
                            })}
                          </ul>
                        </details>
                      );
                    })}
                  </nav>
                ) : null}
              </article>
            )}
          </div>
        </Reveal>
      </Frame>
    </>
  );
}

function ChangeCard({ c }: { c: Change }) {
  const t = useT();
  const label = IMPACTS.find((i) => i.key === c.impact)?.label;
  return (
    <div className="py-6">
      <p className="flex flex-wrap items-center gap-3 font-mono text-[11px] tracking-[0.14em] uppercase">
        {label ? (
          <span
            className={cn(
              "rounded-sm border px-1.5 py-0.5",
              c.impact === "IMPACT_BREAKING"
                ? "border-destructive text-destructive"
                : c.impact === "IMPACT_WORTH_KNOWING"
                  ? "text-foreground"
                  : "text-muted-foreground",
            )}
          >
            {t(label)}
          </span>
        ) : null}
        <span className="text-muted-foreground">{c.area}</span>
      </p>
      <h3 className="mt-3 text-lg font-medium tracking-tight">
        <SafeLink href={c.url}>{c.title}</SafeLink>
      </h3>
      <dl className="text-muted-foreground mt-3 grid gap-2 text-sm sm:grid-cols-[9rem_minmax(0,1fr)] sm:gap-x-6">
        <dt className="text-foreground/80 font-medium">{t("radar.what")}</dt>
        <dd className="text-pretty">
          <Inline text={c.what} />
        </dd>
        <dt className="text-foreground/80 font-medium">{t("radar.production")}</dt>
        <dd className="text-pretty">
          <Inline text={c.production_impact} />
        </dd>
        <dt className="text-foreground/80 font-medium">{t("radar.try")}</dt>
        <dd className="text-pretty">
          <Inline text={c.try_it} />
        </dd>
      </dl>
    </div>
  );
}

/** A link only when the target is http(s); otherwise plain text. */
function SafeLink({ href, children }: { href: string; children: React.ReactNode }) {
  if (!/^https?:\/\//.test(href)) return <>{children}</>;
  return (
    <a href={href} target="_blank" rel="noreferrer noopener" className="underline-offset-4 hover:underline">
      {children}
    </a>
  );
}

function Section({ title, text }: { title: string; text: string }) {
  if (!text) return null;
  return (
    <section className="mt-10">
      <h2 className="text-xl font-medium tracking-tight">{title}</h2>
      <div className="text-muted-foreground mt-4 space-y-3 text-pretty">
        <Prose text={text} />
      </div>
    </section>
  );
}

/**
 * The model's Markdown, safely: blank-line paragraphs, `- ` or `* ` bullets,
 * `[text](url)`, `<url>` and bare links whose target is http or https, and
 * `**bold**` dropped to plain text. Everything else is text. No HTML is ever
 * produced from it.
 */
function Prose({ text }: { text: string }) {
  const blocks = text.trim().split(/\n\s*\n/);
  return (
    <>
      {blocks.map((block, n) => {
        const lines = block
          .split("\n")
          .map((l) => l.trim())
          .filter(Boolean);
        const bullets = lines.every((l) => /^[-*] /.test(l));
        if (bullets) {
          return (
            <ul key={n} className="list-disc space-y-2 pl-5">
              {lines.map((l, i) => (
                <li key={i}>
                  <Inline text={l.replace(/^[-*] /, "")} />
                </li>
              ))}
            </ul>
          );
        }
        return (
          <p key={n}>
            <Inline text={lines.join(" ")} />
          </p>
        );
      })}
    </>
  );
}

function Inline({ text }: { text: string }) {
  const parts: React.ReactNode[] = [];
  // [text](url), <url>, or a bare url; http(s) only.
  const link = /\[([^\]]+)\]\((https?:\/\/[^\s)]+)\)|<(https?:\/\/[^\s>]+)>|(https?:\/\/[^\s)>]+)/g;
  let last = 0;
  for (const m of text.matchAll(link)) {
    const at = m.index ?? 0;
    parts.push(text.slice(last, at).replace(/\*\*/g, ""));
    const href = m[2] ?? m[3] ?? m[4] ?? "";
    parts.push(
      <a
        key={at}
        href={href}
        target="_blank"
        rel="noreferrer noopener"
        className="text-foreground underline underline-offset-4"
      >
        {(m[1] ?? href).replace(/\*\*/g, "")}
      </a>,
    );
    last = at + m[0].length;
  }
  parts.push(text.slice(last).replace(/\*\*/g, ""));
  return <>{parts}</>;
}
