"use client";

import { IconBrandGithub, IconBrandLinkedin, IconBrandX } from "@tabler/icons-react";
import { MapPin } from "lucide-react";
import Link from "next/link";

import { Eyebrow, Frame, Reveal } from "@/components/kit";
import { company } from "@/data/site";
import { useT } from "@/lib/i18n";

/** The contact page in the reader's language; `app/contact/page.tsx` carries the metadata. */
export function ContactContent() {
  const t = useT();
  const elsewhere = [
    { icon: IconBrandGithub, label: "GitHub", value: "github.com/0x19", href: company.github },
    { icon: IconBrandX, label: "X", value: "@vesicnevio", href: company.x },
    { icon: IconBrandLinkedin, label: "LinkedIn", value: "in/neviovesic", href: company.linkedin },
  ];

  return (
    <>
      <Frame className="pt-20 pb-12 sm:pt-28">
        <Eyebrow>{t("contact.eyebrow")}</Eyebrow>
        <h1 className="mt-6 max-w-3xl text-4xl font-semibold tracking-[-0.035em] text-balance sm:text-6xl">
          {t("contact.title")}
        </h1>
        <p className="text-muted-foreground mt-6 max-w-xl text-lg text-pretty">{t("contact.lead")}</p>
      </Frame>

      <Frame className="pb-16">
        <Reveal>
          <div className="border-t pt-10">
            <Eyebrow>{t("contact.email")}</Eyebrow>
            <a
              href={`mailto:${company.email}`}
              className="mt-5 block text-3xl font-semibold tracking-[-0.03em] break-all transition-opacity hover:opacity-60 sm:text-6xl"
            >
              {company.email}
            </a>
          </div>
        </Reveal>
      </Frame>

      <Frame className="pb-24 sm:pb-32">
        <div className="grid gap-10 border-t pt-10 sm:grid-cols-2">
          <div>
            <Eyebrow>{t("contact.elsewhere")}</Eyebrow>
            <ul className="mt-5 space-y-1">
              {elsewhere.map((e) => (
                <li key={e.label}>
                  <a
                    href={e.href}
                    target="_blank"
                    rel="noreferrer"
                    className="group flex items-center gap-3 py-2"
                  >
                    <e.icon className="text-muted-foreground size-4" />
                    <span className="font-medium">{e.label}</span>
                    <span className="text-muted-foreground font-mono text-xs">{e.value}</span>
                    <span className="text-muted-foreground ml-auto opacity-0 transition-opacity group-hover:opacity-100">
                      ↗
                    </span>
                  </a>
                </li>
              ))}
            </ul>
          </div>
          <div>
            <Eyebrow>{t("contact.where")}</Eyebrow>
            <p className="mt-5 flex items-center gap-2 font-medium">
              <MapPin className="text-muted-foreground size-4" />
              {company.city}
            </p>
            <p className="text-muted-foreground mt-2 text-sm text-pretty">
              {t("contact.hours", { company: company.legalName })}{" "}
              <Link className="text-foreground underline-offset-4 hover:underline" href="/legal/">
                {t("contact.legal_page")}
              </Link>
              .
            </p>
          </div>
        </div>
      </Frame>
    </>
  );
}
