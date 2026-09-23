"use client";

import Link from "next/link";

import { Eyebrow, Frame } from "@/components/kit";
import { company } from "@/data/site";
import { useLang, useT } from "@/lib/i18n";
import { messages } from "@/lib/i18n/messages";

/** The terms in the reader's language; `app/terms/page.tsx` carries the metadata. */
export function TermsContent() {
  const t = useT();
  const { lang } = useLang();
  const vars = { company: company.legalName, oib: company.oib };

  // The sections are numbered keys read in order until one is missing, so a
  // section added to the dictionary needs no change here. English decides how
  // many there are; a Croatian gap falls back to English inside `t`.
  const sections: { title: string; body: string[] }[] = [];
  for (let n = 1; `terms.s${n}.title` in messages.en; n++) {
    const body: string[] = [];
    for (let m = 1; `terms.s${n}.p${m}` in messages.en; m++) body.push(t(`terms.s${n}.p${m}`, vars));
    sections.push({ title: t(`terms.s${n}.title`), body });
  }

  return (
    <>
      <Frame className="pt-20 pb-10 sm:pt-28">
        <Eyebrow>{t("terms.eyebrow")}</Eyebrow>
        <h1 className="mt-6 max-w-3xl text-4xl font-semibold tracking-[-0.035em] text-balance sm:text-6xl">
          {t("terms.title")}
        </h1>
        <p className="text-muted-foreground mt-6 max-w-xl text-lg text-pretty">
          {t("terms.lead_a")}{" "}
          <Link className="text-foreground underline-offset-4 hover:underline" href="/legal/">
            {t("terms.lead_link")}
          </Link>{" "}
          {t("terms.lead_b")}
        </p>
      </Frame>

      <Frame className="pb-24 sm:pb-32">
        <dl className="max-w-3xl" lang={lang}>
          {sections.map((s, i) => (
            <div
              key={s.title}
              className="grid gap-3 border-t py-8 sm:grid-cols-[3rem_minmax(0,1fr)] sm:gap-6"
            >
              <dt className="text-muted-foreground/50 font-mono text-[11px] tabular-nums">
                {String(i + 1).padStart(2, "0")}
              </dt>
              <dd>
                <h2 className="text-lg font-medium tracking-tight">{s.title}</h2>
                <div className="text-muted-foreground mt-3 space-y-3 text-sm text-pretty">
                  {s.body.map((p) => (
                    <p key={p.slice(0, 24)}>{p}</p>
                  ))}
                </div>
              </dd>
            </div>
          ))}
        </dl>
      </Frame>
    </>
  );
}
