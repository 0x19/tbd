"use client";

import Link from "next/link";

import { DOCK_OPEN } from "@/components/chat/dock";
import { WORKBENCH_HREF } from "@/components/chat/sessions";
import { latest, Timeline } from "@/components/pages/lab";
import { LabLiveStrip } from "@/components/workbench/lab-live";
import { rfcs, studies } from "@/generated/lab/index";
import { useAgents } from "@/lib/agents";
import { useT } from "@/lib/i18n";
import { messages } from "@/lib/i18n/messages";
import { useMe } from "@/lib/me";

/** Whether the home page shows the live lab: to an admin while the lab is private. */
export function useLabLive(): boolean {
  return useMe()?.role === "admin";
}

/**
 * The home page's live lab: the model tiers as they run now, the site guide
 * (which opens the chat at the bottom of the page), the workbench, and the
 * newest lines of the lab's status logs. Every piece links out to where it
 * lives rather than restating it.
 */
export function HomeLabLive() {
  const t = useT();
  const agents = useAgents(true);
  const guide = agents?.find((a) => a.id === "site" && a.available);
  const name = guide && messages.en[`dock.name.${guide.id}`] ? t(`dock.name.${guide.id}`) : guide?.name;
  return (
    <div className="mt-10 grid gap-10">
      <div className="border-y py-4">
        <p className="text-muted-foreground/70 mb-2 font-mono text-[11px] tracking-[0.18em] uppercase">
          {t("home.live.tiers")}
        </p>
        <LabLiveStrip />
      </div>
      <div className="bg-border/70 grid gap-px border sm:grid-cols-2">
        <div className="bg-background flex flex-col gap-3 p-6">
          <p className="font-mono text-[11px] tracking-[0.18em] uppercase">{name ?? t("home.live.guide")}</p>
          <p className="text-muted-foreground text-sm text-pretty">
            {guide?.persona ?? t("home.live.guide_off")}
          </p>
          {guide ? (
            <button
              type="button"
              className="mt-auto self-start text-sm font-medium underline-offset-4 hover:underline"
              onClick={() => window.dispatchEvent(new Event(DOCK_OPEN))}
            >
              {t("home.live.ask")} <kbd className="text-muted-foreground ml-1 font-mono text-[10px]">/</kbd>
            </button>
          ) : null}
        </div>
        <div className="bg-background flex flex-col gap-3 p-6">
          <p className="font-mono text-[11px] tracking-[0.18em] uppercase">{t("lab.demo.title")}</p>
          <p className="text-muted-foreground text-sm text-pretty">{t("home.live.workbench")}</p>
          <Link
            href={WORKBENCH_HREF}
            className="mt-auto self-start text-sm font-medium underline-offset-4 hover:underline"
          >
            {t("home.live.open_workbench")}
          </Link>
        </div>
      </div>
      <div>
        <p className="text-muted-foreground/70 mb-2 font-mono text-[11px] tracking-[0.18em] uppercase">
          {t("lab.latest.label")}
        </p>
        <Timeline lines={latest([...rfcs, ...studies], 5)} />
        <Link
          href="/lab/"
          className="text-muted-foreground hover:text-foreground mt-4 inline-block text-sm underline-offset-4 hover:underline"
        >
          {t("home.lab.cta")}
        </Link>
      </div>
    </div>
  );
}
