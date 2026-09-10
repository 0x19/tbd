import {
  IconActivity,
  IconBook2,
  IconCalendarClock,
  IconChecklist,
  IconFlask,
  IconGauge,
  IconLayoutDashboard,
  IconServer,
} from "@tabler/icons-react";

import { type NavGroup } from "@/components/layout/types";

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
    title: "Operate",
    items: [{ title: "Runbook", url: "/runbook/", icon: IconBook2 }],
  },
];

const clean = (p: string) => (p.split("?")[0] ?? "").replace(/\/$/, "") || "/";

/** Breadcrumb trail for a pathname: [group, page]. */
export function crumbs(pathname: string): string[] {
  const c = clean(pathname);
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
