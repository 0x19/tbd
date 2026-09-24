"use client";

import { LANGS, useLang } from "@/lib/i18n";
import { useT } from "@/lib/i18n";
import { cn } from "@/lib/utils";

/**
 * EN / HR, the one control that stores anything about the language: a choice
 * made here is kept in the visitor's own browser, and the country and browser
 * stop deciding for them.
 */
export function LangToggle({ className }: { className?: string }) {
  const { lang, setLang } = useLang();
  const t = useT();
  return (
    <div
      role="group"
      aria-label={t("common.language")}
      className={cn(
        "flex items-center font-mono text-[10px] tracking-[0.12em] uppercase sm:text-[11px]",
        className,
      )}
    >
      {LANGS.map((l, i) => (
        <button
          key={l.value}
          type="button"
          lang={l.value}
          onClick={() => setLang(l.value)}
          aria-pressed={lang === l.value}
          title={l.label}
          className={cn(
            "px-1.5 py-1 transition-colors",
            lang === l.value ? "text-foreground" : "text-muted-foreground hover:text-foreground",
            i > 0 && "border-l",
          )}
        >
          {l.value}
        </button>
      ))}
    </div>
  );
}
