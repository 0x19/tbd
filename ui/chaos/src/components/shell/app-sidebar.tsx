"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { ExternalLink, Flame } from "lucide-react";
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuBadge,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarRail,
} from "@/components/ui/sidebar";
import { Badge } from "@/components/ui/badge";
import { NAV } from "./nav";
import { useChaos } from "./providers";

export function AppSidebar() {
  const pathname = usePathname();
  const { overview, connected } = useChaos();
  const active = overview?.active_run;
  const links = overview?.config.links;
  const external = [
    { title: "Grafana", href: links?.grafana },
    { title: "Logs", href: links?.victorialogs },
    { title: "Metrics", href: links?.metrics },
    { title: "Profiles", href: links?.pyroscope },
    { title: "Envoy admin", href: links?.envoy_admin },
  ].filter((l) => l.href);

  const isActive = (href: string) => {
    const clean = pathname.replace(/\/$/, "") || "/";
    const target = href.replace(/\/$/, "") || "/";
    return target === "/" ? clean === "/" : clean === target || clean.startsWith(`${target}/`);
  };

  return (
    <Sidebar collapsible="icon">
      <SidebarHeader>
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton size="lg" render={<Link href="/" />}>
              <div className="flex aspect-square size-8 items-center justify-center rounded-lg bg-sidebar-primary text-sidebar-primary-foreground">
                <Flame className="size-4" />
              </div>
              <div className="grid flex-1 text-left text-sm leading-tight">
                <span className="truncate font-semibold">chaos</span>
                <span className="truncate text-xs text-muted-foreground">
                  {overview ? `${overview.env} · v${overview.version}` : "connecting…"}
                </span>
              </div>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarHeader>
      <SidebarContent>
        {NAV.map((group) => (
          <SidebarGroup key={group.label}>
            <SidebarGroupLabel>{group.label}</SidebarGroupLabel>
            <SidebarGroupContent>
              <SidebarMenu>
                {group.items.map((item) => (
                  <SidebarMenuItem key={item.href}>
                    <SidebarMenuButton
                      isActive={isActive(item.href)}
                      tooltip={item.title}
                      render={<Link href={item.href} />}
                    >
                      <item.icon />
                      <span>{item.title}</span>
                    </SidebarMenuButton>
                    {item.href === "/runs/" && active ? (
                      <SidebarMenuBadge>
                        <span className="size-2 animate-pulse rounded-full bg-emerald-500" />
                      </SidebarMenuBadge>
                    ) : null}
                  </SidebarMenuItem>
                ))}
              </SidebarMenu>
            </SidebarGroupContent>
          </SidebarGroup>
        ))}
        {external.length ? (
          <SidebarGroup>
            <SidebarGroupLabel>Observability</SidebarGroupLabel>
            <SidebarGroupContent>
              <SidebarMenu>
                {external.map((l) => (
                  <SidebarMenuItem key={l.title}>
                    <SidebarMenuButton
                      tooltip={l.title}
                      render={<a href={l.href} target="_blank" rel="noreferrer" />}
                    >
                      <ExternalLink />
                      <span>{l.title}</span>
                    </SidebarMenuButton>
                  </SidebarMenuItem>
                ))}
              </SidebarMenu>
            </SidebarGroupContent>
          </SidebarGroup>
        ) : null}
      </SidebarContent>
      <SidebarFooter>
        <div className="flex items-center gap-2 px-2 text-xs text-muted-foreground group-data-[collapsible=icon]:hidden">
          <span className={`size-2 rounded-full ${connected ? "bg-emerald-500" : "bg-amber-500"}`} />
          {connected ? "live feed connected" : "live feed off"}
          {overview?.stack ? (
            <Badge variant="outline" className="ml-auto">
              {overview.stack.filter((i) => i.running).length}/{overview.stack.length} up
            </Badge>
          ) : null}
        </div>
      </SidebarFooter>
      <SidebarRail />
    </Sidebar>
  );
}
