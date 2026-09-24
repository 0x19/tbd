"use client";

// English / Hrvatski. Everything on the page follows, including what a
// download carries; money is written the same way in both.
import { LANGS, useLang } from "@/lib/i18n";

import { Tabs, TabsList, TabsTrigger } from "./ui/tabs";

export function LangToggle({ className }: { className?: string }) {
  const { lang, setLang } = useLang();
  return (
    <Tabs value={lang} onValueChange={(v) => setLang(v as typeof lang)} className={className}>
      <TabsList>
        {LANGS.map((l) => (
          <TabsTrigger key={l.value} value={l.value} className="text-xs">
            {l.value.toUpperCase()}
          </TabsTrigger>
        ))}
      </TabsList>
    </Tabs>
  );
}
