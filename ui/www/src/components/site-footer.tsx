"use client";

import Link from "next/link";

import { Eyebrow, Frame } from "@/components/kit";
import { Logo } from "@/components/logo";
import { company, nav, navVisible } from "@/data/site";
import { useT } from "@/lib/i18n";
import { useSite } from "@/lib/i18n/site";
import { useIsAdmin } from "@/lib/me";

export function SiteFooter() {
  const t = useT();
  const site = useSite();
  const admin = useIsAdmin();
  const columns = [
    {
      title: t("common.pages"),
      links: [
        ...nav
          .filter((i) => i.href !== "/" && navVisible(i, admin))
          .map((i) => ({ label: t(`common.${i.key}`), href: i.href, out: false })),
        { label: t("common.legal"), href: "/legal/", out: false },
        { label: t("common.terms"), href: "/terms/", out: false },
      ],
    },
    {
      title: t("common.elsewhere"),
      links: [
        { label: "GitHub", href: company.github, out: true },
        { label: "X", href: company.x, out: true },
        { label: "LinkedIn", href: company.linkedin, out: true },
      ],
    },
  ];

  return (
    <footer className="mt-auto border-t">
      <Frame className="py-14 sm:py-16">
        <div className="grid gap-10 sm:grid-cols-[minmax(0,1fr)_auto_auto] sm:gap-16">
          <div>
            <Logo size={26} />
            <p className="text-muted-foreground mt-4 max-w-xs text-sm text-pretty">{site.company.tagline}</p>
            <a
              href={`mailto:${company.email}`}
              className="mt-4 inline-block text-sm font-medium underline-offset-4 hover:underline"
            >
              {company.email}
            </a>
          </div>
          {columns.map((c) => (
            <div key={c.title}>
              <Eyebrow>{c.title}</Eyebrow>
              <ul className="mt-5 space-y-2.5">
                {c.links.map((l) => (
                  <li key={l.label}>
                    {l.out ? (
                      <a
                        href={l.href}
                        target="_blank"
                        rel="noreferrer"
                        className="text-muted-foreground hover:text-foreground text-sm transition-colors"
                      >
                        {l.label} ↗
                      </a>
                    ) : (
                      <Link
                        href={l.href}
                        className="text-muted-foreground hover:text-foreground text-sm transition-colors"
                      >
                        {l.label}
                      </Link>
                    )}
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>
        <div className="text-muted-foreground mt-14 flex flex-wrap items-center gap-x-6 gap-y-2 border-t pt-6 font-mono text-[11px] tracking-[0.14em] uppercase">
          <span>
            © {company.founded ? `${company.founded}–` : ""}
            {new Date().getFullYear()} {company.legalName}
          </span>
          <span>{company.city}</span>
        </div>
      </Frame>
    </footer>
  );
}
