"use client";

import { Eyebrow, Frame } from "@/components/kit";
import { Logo } from "@/components/logo";
import { company, publicNav, publicUrl } from "@/data/site";
import { useT } from "@/lib/i18n";

/** The public site's footer, its links absolute; the same shape as `ui/www`'s. */
export function SiteFooter() {
  const t = useT();
  const columns = [
    {
      title: t("nav.pages"),
      links: [
        ...publicNav.map((p) => ({ label: t(`nav.${p.key}`), href: p.href })),
        { label: t("nav.legal"), href: `${publicUrl}/legal/` },
        { label: t("nav.terms"), href: `${publicUrl}/terms/` },
      ],
    },
    {
      title: t("nav.elsewhere"),
      links: [
        { label: "GitHub", href: company.github },
        { label: "X", href: company.x },
        { label: "LinkedIn", href: company.linkedin },
      ],
    },
  ];

  return (
    <footer className="mt-auto border-t">
      <Frame className="py-14 sm:py-16">
        <div className="grid gap-10 sm:grid-cols-[minmax(0,1fr)_auto_auto] sm:gap-16">
          <div>
            <Logo size={26} />
            <p className="text-muted-foreground mt-4 max-w-xs text-sm text-pretty">{t("nav.tagline")}</p>
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
                    <a
                      href={l.href}
                      className="text-muted-foreground hover:text-foreground text-sm transition-colors"
                    >
                      {l.label}
                    </a>
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>
        <div className="text-muted-foreground mt-14 flex flex-wrap items-center gap-x-6 gap-y-2 border-t pt-6 font-mono text-[11px] tracking-[0.14em] uppercase">
          <span>
            © {company.founded}–{new Date().getFullYear()} {company.legalName}
          </span>
          <span>{company.city}</span>
        </div>
      </Frame>
    </footer>
  );
}
