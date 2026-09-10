"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { ChevronRight, ChevronsUpDown, ExternalLink, Flame, Globe } from "lucide-react";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/components/ui/collapsible";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarMenuSub,
  SidebarMenuSubButton,
  SidebarMenuSubItem,
  SidebarRail,
} from "@/components/ui/sidebar";
import { NAV, isActive } from "./nav";
import { useChaos } from "./providers";

/**
 * The kit's sidebar: a workspace block on top, labelled groups with
 * collapsible items, external tools, and an identity block at the bottom.
 */
export function AppSidebar() {
  const pathname = usePathname();
  const { overview, connected } = useChaos();
  const links = overview?.config.links;
  const external = [
    { title: "Grafana", href: links?.grafana },
    { title: "Logs", href: links?.victorialogs },
    { title: "Metrics", href: links?.metrics },
    { title: "Profiles", href: links?.pyroscope },
    { title: "Envoy admin", href: links?.envoy_admin },
  ].filter((l): l is { title: string; href: string } => Boolean(l.href));
  const env = overview?.env ?? "…";
  const domain = overview?.config.links.domain || "local machine";

  return (
    <Sidebar collapsible="icon">
      <SidebarHeader>
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton size="lg" render={<Link href="/" />}>
              <div className="flex aspect-square size-8 items-center justify-center rounded-lg bg-primary text-primary-foreground">
                <Flame className="size-4" />
              </div>
              <div className="grid flex-1 text-left text-sm leading-tight">
                <span className="truncate font-semibold">Chaos Admin</span>
                <span className="truncate text-xs text-muted-foreground">tbd · chaos serve</span>
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
                {group.items.map((item) =>
                  item.children ? (
                    <Collapsible
                      key={item.href}
                      defaultOpen={isActive(pathname, item.href)}
                      className="group/collapsible"
                    >
                      <SidebarMenuItem>
                        <CollapsibleTrigger
                          render={
                            <SidebarMenuButton
                              tooltip={item.title}
                              isActive={isActive(pathname, item.href)}
                            />
                          }
                        >
                          <item.icon />
                          <span>{item.title}</span>
                          <ChevronRight className="ml-auto transition-transform duration-200 group-data-[panel-open]/collapsible:rotate-90" />
                        </CollapsibleTrigger>
                        <CollapsibleContent>
                          <SidebarMenuSub>
                            {item.children.map((leaf) => (
                              <SidebarMenuSubItem key={leaf.href}>
                                <SidebarMenuSubButton
                                  isActive={
                                    leaf.href.includes("?")
                                      ? false
                                      : isActive(pathname, leaf.href) &&
                                        pathname.replace(/\/$/, "") === leaf.href.replace(/\/$/, "")
                                  }
                                  render={<Link href={leaf.href} />}
                                >
                                  <span>{leaf.title}</span>
                                </SidebarMenuSubButton>
                              </SidebarMenuSubItem>
                            ))}
                          </SidebarMenuSub>
                        </CollapsibleContent>
                      </SidebarMenuItem>
                    </Collapsible>
                  ) : (
                    <SidebarMenuItem key={item.href}>
                      <SidebarMenuButton
                        isActive={isActive(pathname, item.href)}
                        tooltip={item.title}
                        render={<Link href={item.href} />}
                      >
                        <item.icon />
                        <span>{item.title}</span>
                      </SidebarMenuButton>
                    </SidebarMenuItem>
                  ),
                )}
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
        <SidebarMenu>
          <SidebarMenuItem>
            <DropdownMenu>
              <DropdownMenuTrigger render={<SidebarMenuButton size="lg" />}>
                <div className="flex aspect-square size-8 items-center justify-center rounded-lg bg-muted">
                  <Globe className="size-4" />
                </div>
                <div className="grid flex-1 text-left text-sm leading-tight">
                  <span className="truncate font-semibold">{env}</span>
                  <span className="truncate text-xs text-muted-foreground">{domain}</span>
                </div>
                <span className={`size-2 rounded-full ${connected ? "bg-emerald-500" : "bg-amber-500"}`} />
                <ChevronsUpDown className="ml-1 size-4" />
              </DropdownMenuTrigger>
              <DropdownMenuContent side="top" align="start" className="w-64">
                <DropdownMenuLabel>Environment</DropdownMenuLabel>
                <DropdownMenuItem className="text-xs">
                  <span className="text-muted-foreground">config</span>
                  <span className="ml-auto truncate font-mono">
                    {overview?.config_files.map((f) => f.split("/").pop()).join(" + ")}
                  </span>
                </DropdownMenuItem>
                <DropdownMenuItem className="text-xs">
                  <span className="text-muted-foreground">version</span>
                  <span className="ml-auto font-mono">{overview?.version}</span>
                </DropdownMenuItem>
                <DropdownMenuItem className="text-xs">
                  <span className="text-muted-foreground">live feed</span>
                  <span className="ml-auto">{connected ? "connected" : "off"}</span>
                </DropdownMenuItem>
                <DropdownMenuSeparator />
                <DropdownMenuItem className="text-xs">
                  <span className="text-muted-foreground">API</span>
                  <span className="ml-auto font-mono">{overview?.config.serve.base_path}</span>
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarFooter>
      <SidebarRail />
    </Sidebar>
  );
}
