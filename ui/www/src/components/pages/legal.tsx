"use client";

import Link from "next/link";

import { Eyebrow, Frame } from "@/components/kit";
import { company, orDash } from "@/data/site";
import { useT } from "@/lib/i18n";

/** The imprint and the privacy notice in the reader's language; `app/legal/page.tsx` carries the metadata. */
export function LegalContent() {
  const t = useT();
  const rows = [
    { k: t("legal.rows.name"), v: company.legalName },
    { k: t("legal.rows.person"), v: company.person },
    { k: t("legal.rows.founded"), v: company.founded ? String(company.founded) : "—" },
    { k: t("legal.rows.address"), v: orDash(company.address) },
    { k: t("legal.rows.oib"), v: company.oib },
    { k: t("legal.rows.mbs"), v: orDash(company.registration) },
    { k: t("legal.rows.email"), v: company.email },
  ];

  return (
    <>
      <Frame className="pt-20 pb-10 sm:pt-28">
        <Eyebrow>{t("legal.eyebrow")}</Eyebrow>
        <h1 className="mt-6 max-w-3xl text-4xl font-semibold tracking-[-0.035em] text-balance sm:text-6xl">
          {t("legal.title")}
        </h1>
      </Frame>

      <Frame className="pb-16">
        <Eyebrow className="border-t pt-6">{t("legal.company")}</Eyebrow>
        <dl className="mt-6 max-w-2xl">
          {rows.map((r) => (
            <div key={r.k} className="flex items-baseline justify-between gap-6 border-b py-4 text-sm">
              <dt className="text-muted-foreground">{r.k}</dt>
              <dd className="text-right font-medium">{r.v}</dd>
            </div>
          ))}
        </dl>
      </Frame>

      <Frame className="pb-24 sm:pb-32">
        <Eyebrow className="border-t pt-6">{t("legal.privacy")}</Eyebrow>
        <div className="text-muted-foreground mt-6 max-w-2xl space-y-4 text-sm text-pretty">
          <p>{t("legal.p1")}</p>
          <p>{t("legal.p2")}</p>
          <p>
            {t("legal.p3")}{" "}
            <a
              className="text-foreground underline-offset-4 hover:underline"
              href={`mailto:${company.email}`}
            >
              {company.email}
            </a>
            .
          </p>
          <p>
            {t("legal.p4a")}{" "}
            <Link className="text-foreground underline-offset-4 hover:underline" href="/terms/">
              {t("legal.p4.terms")}
            </Link>{" "}
            {t("legal.p4b")}
          </p>
        </div>
      </Frame>
    </>
  );
}
