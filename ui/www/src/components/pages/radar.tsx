"use client";

import { useEffect, useId, useState } from "react";

import { Eyebrow, Frame, Reveal } from "@/components/kit";
import { TabList } from "@/components/tabs";
import { useLang, useT } from "@/lib/i18n";

/**
 * The Radar (Quiet Pager): the latest weekly digest per language, in the
 * reader's language when there is one, from the radar service on the same
 * origin (`GET /v1/radar/digests`, public, rate-limited at Envoy). Every issue is
 * marked as written by a language model; a digest the llm's stub engine wrote
 * (`stub`) is never shown. The text is the model's, so it is rendered as React
 * elements by `Prose` (paragraphs, bullets, links with http(s) targets only),
 * never as HTML. Field names are the proto's (the gateway keeps them).
 * `app/radar/page.tsx` carries the metadata.
 */

type Digest = {
  id: string;
  week: string;
  language: "go" | "rust";
  lang: "en" | "hr";
  changed: string;
  why: string;
  drill: string;
  script: string;
  item_count: number;
  model: string;
  ai_written: boolean;
  stub: boolean;
};

type State = { kind: "loading" } | { kind: "failed" } | { kind: "ready"; digests: Digest[] };

export function RadarContent() {
  const t = useT();
  const { lang } = useLang();
  const id = useId();
  const [state, setState] = useState<State>({ kind: "loading" });
  const [language, setLanguage] = useState<"go" | "rust">("go");
  const [week, setWeek] = useState<string | null>(null);

  useEffect(() => {
    let live = true;
    fetch("/v1/radar/digests?limit=40", { headers: { accept: "application/json" } })
      .then(async (res) => {
        if (!res.ok) throw new Error(String(res.status));
        const body = (await res.json()) as { digests?: Digest[] };
        if (live) setState({ kind: "ready", digests: (body.digests ?? []).filter((d) => !d.stub) });
      })
      .catch(() => {
        if (live) setState({ kind: "failed" });
      });
    return () => {
      live = false;
    };
  }, []);

  const digests = state.kind === "ready" ? state.digests : [];
  const ofLanguage = digests.filter((d) => d.language === language);
  const weeks = [...new Set(ofLanguage.map((d) => d.week))];
  const shownWeek = week && weeks.includes(week) ? week : weeks[0];
  // The reader's language when that week has it, else English.
  const current =
    ofLanguage.find((d) => d.week === shownWeek && d.lang === lang) ??
    ofLanguage.find((d) => d.week === shownWeek && d.lang === "en");

  const tabs = (["go", "rust"] as const).map((key) => ({
    key,
    label: key === "go" ? "Go" : "Rust",
    n: new Set(digests.filter((d) => d.language === key).map((d) => d.week)).size,
  }));

  return (
    <>
      <Frame className="pt-20 pb-12 sm:pt-28">
        <Eyebrow>{t("radar.eyebrow")}</Eyebrow>
        <h1 className="mt-6 max-w-3xl text-4xl font-semibold tracking-[-0.035em] text-balance sm:text-6xl">
          {t("radar.title")}
        </h1>
        <p className="text-muted-foreground mt-6 max-w-xl text-lg text-pretty">{t("radar.lead")}</p>
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
              <article className="grid gap-10 py-10 lg:grid-cols-[minmax(0,1fr)_16rem]">
                <div>
                  <p className="text-muted-foreground flex flex-wrap items-center gap-x-4 gap-y-1 font-mono text-[11px] tracking-[0.18em] uppercase">
                    <span className="text-foreground">{current.week}</span>
                    <span className="rounded-sm border px-1.5 py-0.5">{t("radar.ai")}</span>
                    <span>{t("radar.from", { n: current.item_count })}</span>
                  </p>
                  <Section title={t("radar.changed")} text={current.changed} />
                  <Section title={t("radar.why")} text={current.why} />
                  <Section title={t("radar.drill")} text={current.drill} />
                  <details className="mt-10 border-t pt-6">
                    <summary className="cursor-pointer font-medium tracking-tight">
                      {t("radar.script")}
                    </summary>
                    <p className="text-muted-foreground mt-4 text-pretty whitespace-pre-line">
                      {current.script}
                    </p>
                  </details>
                  <p className="text-muted-foreground/70 mt-8 font-mono text-[11px]">{current.model}</p>
                </div>
                {weeks.length > 1 ? (
                  <nav aria-label={t("radar.archive")}>
                    <p className="text-muted-foreground font-mono text-[11px] tracking-[0.18em] uppercase">
                      {t("radar.archive")}
                    </p>
                    <ul className="mt-3 divide-y border-y">
                      {weeks.map((w) => (
                        <li key={w}>
                          <button
                            type="button"
                            onClick={() => setWeek(w)}
                            aria-current={w === shownWeek ? "true" : undefined}
                            className={`w-full py-2.5 text-left font-mono text-sm tabular-nums transition-colors ${
                              w === shownWeek
                                ? "text-foreground"
                                : "text-muted-foreground hover:text-foreground"
                            }`}
                          >
                            {w}
                          </button>
                        </li>
                      ))}
                    </ul>
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

function Section({ title, text }: { title: string; text: string }) {
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
 * `[text](url)` links whose target is http or https, and `**bold**` dropped to
 * plain text. Everything else is text. No HTML is ever produced from it.
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
  const link = /\[([^\]]+)\]\((https?:\/\/[^\s)]+)\)|(https?:\/\/[^\s)]+)/g;
  let last = 0;
  for (const m of text.matchAll(link)) {
    const at = m.index ?? 0;
    parts.push(text.slice(last, at).replace(/\*\*/g, ""));
    const href = m[2] ?? m[3] ?? "";
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
