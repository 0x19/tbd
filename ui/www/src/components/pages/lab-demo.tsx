"use client";

import Link from "next/link";

import { Frame, SectionHead } from "@/components/kit";
import { useT } from "@/lib/i18n";

/**
 * The demo's page before there is a demo: what it will be, why it sits behind
 * a sign-in, and the "what this page sends" paragraph every page that could
 * process a visitor's words owes the reader (`/legal/`). Today it sends nothing.
 * `app/lab/demo/page.tsx` carries the metadata.
 */
export function LabDemoContent() {
  const t = useT();
  return (
    <>
      <Frame className="pt-20 pb-10 sm:pt-28">
        <p className="text-muted-foreground font-mono text-[11px] tracking-[0.18em] uppercase">
          {t("lab.demo.eyebrow")}
        </p>
        <h1 className="mt-4 max-w-3xl text-4xl font-semibold tracking-[-0.035em] text-balance sm:text-6xl">
          {t("lab.demo.title")}
        </h1>
        <p className="text-muted-foreground mt-6 max-w-xl text-lg text-pretty">{t("lab.demo.lead")}</p>
      </Frame>
      <Frame className="pb-16">
        <SectionHead n="01" label={t("lab.demo.gated.label")} />
        <p className="text-muted-foreground mt-6 max-w-xl text-pretty">{t("lab.demo.gated")}</p>
        <p className="mt-4 max-w-xl text-sm">
          <a href="/account/" className="text-foreground underline underline-offset-4">
            {t("lab.demo.signin")}
          </a>
        </p>
      </Frame>
      <Frame className="pb-24 sm:pb-32">
        <SectionHead n="02" label={t("lab.demo.sends.label")} />
        <p className="text-muted-foreground mt-6 max-w-xl text-pretty">{t("lab.demo.sends.text")}</p>
        <p className="mt-10">
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
