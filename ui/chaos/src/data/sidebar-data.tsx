import {
  IconActivity,
  IconBook2,
  IconBug,
  IconCalendarClock,
  IconChecklist,
  IconFileText,
  IconFlask,
  IconGauge,
  IconLayoutDashboard,
  IconLifebuoy,
  IconServer,
  IconTestPipe,
} from "@tabler/icons-react";

import { type NavGroup, type NavItem } from "@/components/layout/types";
import type { KindDescriptor } from "@/lib/api/schema";

/** Navigation groups, the shape the kit's NavGroup and CommandMenu consume. */
export const navGroups: NavGroup[] = [
  {
    title: "Chaos",
    items: [
      { title: "Overview", url: "/", icon: IconLayoutDashboard },
      { title: "Stack", url: "/stack/", icon: IconServer },
    ],
  },
  {
    title: "Testing",
    items: [
      {
        title: "Scenarios",
        icon: IconFlask,
        items: [
          { title: "All scenarios", url: "/scenarios/" },
          { title: "New scenario", url: "/scenarios/view/?id=new" },
        ],
      },
      {
        title: "Runs",
        icon: IconActivity,
        items: [
          { title: "All runs", url: "/runs/" },
          { title: "Scenario runs", url: "/runs/?kind=scenario" },
          { title: "Load runs", url: "/runs/?kind=load" },
          { title: "Validate runs", url: "/runs/?kind=validate" },
        ],
      },
      { title: "Load", url: "/load/", icon: IconGauge },
      { title: "Validate", url: "/validate/", icon: IconChecklist },
      { title: "Schedules", url: "/schedules/", icon: IconCalendarClock },
    ],
  },
  {
    title: "Stress",
    items: [
      {
        title: "Campaigns",
        icon: IconTestPipe,
        items: [
          { title: "All campaigns", url: "/stress/" },
          { title: "New campaign", url: "/stress/view/?id=new" },
          { title: "Stress runs", url: "/runs/?kind=stress" },
        ],
      },
      { title: "Findings", url: "/findings/", icon: IconBug },
    ],
  },
  {
    title: "Knowledge base",
    items: [
      { title: "Browse", url: "/kb/", icon: IconBook2 },
      { title: "Runbook", url: "/kb/view/?doc=docs%2Fchaos%2Frunbook", icon: IconLifebuoy },
      {
        title: "References",
        icon: IconFileText,
        items: [
          { title: "Scenario files", url: "/kb/view/?doc=docs%2Fchaos%2Fscenarios" },
          { title: "Stress campaigns", url: "/kb/view/?doc=docs%2Fchaos%2Fstress" },
          { title: "Commands", url: "/kb/view/?doc=docs%2Fchaos%2Fcommands" },
          { title: "HTTP API", url: "/kb/view/?doc=docs%2Fchaos%2Fapi" },
          { title: "Configuration", url: "/kb/view/?doc=docs%2Fchaos%2Fconfig" },
        ],
      },
    ],
  },
];

/**
 * The groups with one "<Kind> runs" entry per registered service kind under
 * Runs (`/runs/?service=<kind>`), registry order. A kind scaffolded by
 * `tbd new service` shows up here without a UI change.
 */
export function navGroupsFor(kinds: KindDescriptor[]): NavGroup[] {
  if (!kinds.length) return navGroups;
  return navGroups.map((group) => ({
    ...group,
    items: group.items.map((item): NavItem => {
      if (item.title !== "Runs" || !item.items) return item;
      return {
        ...item,
        items: [
          ...item.items,
          ...kinds.map((k) => ({ title: `${k.label} runs`, url: `/runs/?service=${k.name}` })),
        ],
      };
    }),
  }));
}

const clean = (p: string) => (p.split("?")[0] ?? "").replace(/\/$/, "") || "/";

/** Breadcrumb trail for a pathname: [group, page]. */
export function crumbs(pathname: string): string[] {
  const c = clean(pathname);
  if (c === "/kb/view") return ["Knowledge base", "Article"];
  for (const group of navGroups) {
    for (const item of group.items) {
      const urls = item.items ? item.items.map((i) => i.url) : [item.url];
      for (const u of urls) {
        const t = clean(u);
        if (t === c) return [group.title, item.title];
      }
      const base = item.items ? clean(item.items[0]!.url) : clean(item.url);
      if (base !== "/" && c.startsWith(`${base}/`)) return [group.title, item.title, "Detail"];
    }
  }
  return ["Chaos"];
}
