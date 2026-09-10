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

export type NavItem = {
  title: string;
  href: string;
  icon: LucideIcon;
  blurb: string;
};

export const NAV: { label: string; items: NavItem[] }[] = [
  {
    label: "Understand",
    items: [
      {
        title: "Overview",
        href: "/",
        icon: LayoutDashboard,
        blurb: "State of the stack and the last runs",
      },
      {
        title: "Stack",
        href: "/stack/",
        icon: Server,
        blurb: "Instances, faults, stop and start",
      },
    ],
  },
  {
    label: "Test",
    items: [
      {
        title: "Scenarios",
        href: "/scenarios/",
        icon: FlaskConical,
        blurb: "Load, faults, assertions",
      },
      {
        title: "Runs",
        href: "/runs/",
        icon: Activity,
        blurb: "Every run, live and past",
      },
      {
        title: "Load",
        href: "/load/",
        icon: Gauge,
        blurb: "Ad-hoc load with live latency",
      },
      {
        title: "Validate",
        href: "/validate/",
        icon: ListChecks,
        blurb: "Every surface, one check each",
      },
    ],
  },
  {
    label: "When it breaks",
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

export function titleFor(pathname: string): string {
  const clean = pathname.replace(/\/$/, "") || "/";
  for (const group of NAV) {
    for (const item of group.items) {
      const href = item.href.replace(/\/$/, "") || "/";
      if (clean === href) return item.title;
    }
  }
  if (clean.startsWith("/scenarios/")) return "Scenario";
  if (clean.startsWith("/runs/")) return "Run";
  return "chaos";
}
