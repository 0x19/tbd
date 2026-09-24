"use client";

import { useEffect, useState } from "react";

import { Frame } from "@/components/kit";
import { Button } from "@/components/ui/button";
import { company, notice } from "@/data/site";
import { useT } from "@/lib/i18n";

const KEY = "inorbit.notice";

/**
 * The bar above the header while the site is being rebuilt. Dismissing it
 * stores one key in the visitor's own browser (`notice.version`); that and a
 * language picked with the toggle are the only things the site ever keeps
 * there. The bar says so, and nothing else is needed, because the site sets no
 * cookies. `notice.version` and whether there is a notice at all stay in the
 * data file; the words are `common.notice.*`. Rendered only after mount so a
 * visitor who closed it does not see it flash.
 */
export function SiteNotice() {
  const t = useT();
  const [shown, setShown] = useState(false);

  useEffect(() => {
    if (!notice.text) return;
    try {
      setShown(window.localStorage.getItem(KEY) !== notice.version);
    } catch {
      setShown(true);
    }
  }, []);

  function dismiss() {
    setShown(false);
    try {
      window.localStorage.setItem(KEY, notice.version);
    } catch {
      // Storage refused (private mode, blocked site data): the bar returns next
      // visit, which is the honest outcome.
    }
  }

  if (!shown) return null;

  return (
    <div role="region" aria-label={t("common.notice.aria")} className="bg-muted/60 border-b">
      <Frame className="flex flex-col gap-3 py-3 text-sm sm:flex-row sm:items-center sm:gap-6">
        <div className="min-w-0 flex-1">
          <p className="text-pretty">
            {t("common.notice.text", { email: "\u0000" })
              .split("\u0000")
              .map((part, i, parts) => (
                <span key={i}>
                  {part}
                  {i < parts.length - 1 ? (
                    <a href={`mailto:${company.email}`} className="font-medium underline underline-offset-4">
                      {company.email}
                    </a>
                  ) : null}
                </span>
              ))}
          </p>
          <p className="text-muted-foreground mt-1 text-xs text-pretty">{t("common.notice.privacy")}</p>
        </div>
        <Button size="sm" variant="outline" onClick={dismiss} className="shrink-0 self-start sm:self-auto">
          {t("common.notice.ok")}
        </Button>
      </Frame>
    </div>
  );
}
