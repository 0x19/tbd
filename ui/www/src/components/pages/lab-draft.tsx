"use client";

import { useSearchParams } from "next/navigation";

import { Frame } from "@/components/kit";
import { LabDocContent } from "@/components/pages/lab-doc";
import { useT } from "@/lib/i18n";
import { useDrafts } from "@/lib/lab-drafts";

/**
 * One draft, `/lab/draft/?doc=rfc/<slug>`: the same page as a published
 * document, under a line that says it is not published and who can see it.
 * The prose arrives from the admin-only drafts file (`src/lib/lab-drafts.ts`);
 * the static page itself carries none of it.
 */
export function LabDraftContent() {
  const t = useT();
  const key = useSearchParams().get("doc") ?? "";
  const { docs, ready } = useDrafts();
  const doc = docs[key];
  if (!doc) {
    return (
      <Frame className="py-24">
        <p className="text-muted-foreground font-mono text-[11px] tracking-[0.18em] uppercase">
          {t("drafts.stamp")}
        </p>
        <p className="text-muted-foreground mt-4 max-w-xl text-pretty">{ready ? t("drafts.missing") : "…"}</p>
      </Frame>
    );
  }
  return (
    <>
      <div className="border-b border-dashed">
        <Frame className="flex flex-wrap items-baseline gap-x-4 gap-y-1 py-3">
          <span className="font-mono text-[11px] tracking-[0.18em] uppercase">{t("drafts.stamp")}</span>
          <span className="text-muted-foreground text-sm">{t("drafts.note")}</span>
        </Frame>
      </div>
      <LabDocContent doc={doc} />
    </>
  );
}
