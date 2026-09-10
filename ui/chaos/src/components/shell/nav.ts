import {
  Activity,
  BookOpenText,
  FlaskConical,
  Gauge,
  LayoutDashboard,
  ListChecks,
  Server,
  type LucideIcon,
} from "lucide-react";

export type NavLeaf = { title: string; href: string };
export type NavItem = { title: string; href: string; icon: LucideIcon; blurb: string; children?: NavLeaf[] };

/** Sidebar groups, the way the kit groups its sub-apps. */
export const NAV: { label: string; items: NavItem[] }[] = [
  {
    label: "Chaos",
    items: [
      { title: "Overview", href: "/", icon: LayoutDashboard, blurb: "State of the stack and the last runs" },
      {
        title: "Stack",
        href: "/stack/",
        icon: Server,
        blurb: "Instances, faults, stop and start",
      },
    ],
  },
  {
    label: "Testing",
    items: [
      {
        title: "Scenarios",
        href: "/scenarios/",
        icon: FlaskConical,
        blurb: "Load, faults, assertions",
        children: [
          { title: "All scenarios", href: "/scenarios/" },
          { title: "New scenario", href: "/scenarios/view/?id=new" },
        ],
      },
      {
        title: "Runs",
        href: "/runs/",
        icon: Activity,
        blurb: "Every run, live and past",
        children: [
          { title: "All runs", href: "/runs/" },
          { title: "Scenario runs", href: "/runs/?kind=scenario" },
          { title: "Load runs", href: "/runs/?kind=load" },
          { title: "Validate runs", href: "/runs/?kind=validate" },
        ],
      },
      { title: "Load", href: "/load/", icon: Gauge, blurb: "Ad-hoc load with live latency" },
      { title: "Validate", href: "/validate/", icon: ListChecks, blurb: "Every surface, one check each" },
    ],
  },
  {
    label: "Operate",
    items: [
      {
        title: "Runbook",
        href: "/runbook/",
        icon: BookOpenText,
        blurb: "What a failure means and where to look",
      },
    ],
  },
];

const clean = (p: string) => p.replace(/\/$/, "") || "/";

/** Breadcrumb trail for a pathname: [section, page]. */
export function crumbs(pathname: string): string[] {
  const c = clean(pathname);
  for (const group of NAV) {
    for (const item of group.items) {
      if (clean(item.href) === c) return [group.label, item.title];
      if (c.startsWith(`${clean(item.href)}/`))
        return [group.label, item.title, c.endsWith("/view") ? "Detail" : ""].filter(Boolean);
    }
  }
  return ["Chaos"];
}

export function titleFor(pathname: string): string {
  const parts = crumbs(pathname);
  return parts[parts.length - 1] ?? "Chaos";
}

export function isActive(pathname: string, href: string): boolean {
  const c = clean(pathname);
  const t = clean(href.split("?")[0]);
  return t === "/" ? c === "/" : c === t || c.startsWith(`${t}/`);
}
