"use client";

import { ChevronsUpDown, ExternalLink, Globe } from "lucide-react";
import { Suspense } from "react";
import * as React from "react";

import { NavGroup } from "@/components/layout/nav-group";
import { TeamSwitcher } from "@/components/layout/team-switcher";
import { Logo } from "@/components/logo";
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
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarRail,
  useSidebar,
} from "@/components/ui/sidebar";
import { navGroupsFor } from "@/data/sidebar-data";
import { site } from "@/data/site";
import { cn } from "@/lib/utils";

import { useChaos } from "../../app/providers";

/**
 * The kit's AppSidebar with the chaos navigation: team switcher on top, the
 * observability links from the environment's config, and an environment
 * block in place of the user block.
 */
export function AppSidebar({ ...props }: React.ComponentProps<typeof Sidebar>) {
  const [isMounted, setIsMounted] = React.useState(false);
  const { state } = useSidebar();
  const { overview, kinds } = useChaos();

  React.useEffect(() => {
    setIsMounted(true);
  }, []);

  if (!isMounted) {
    return (
      <div
        aria-hidden="true"
        className={cn(
          "hidden h-svh shrink-0 bg-transparent md:block",
          state === "collapsed" ? "w-(--sidebar-width-icon)" : "w-(--sidebar-width)",
        )}
      />
    );
  }

  const links = overview?.config.links;
  const external = [
    { title: "Grafana", url: links?.grafana },
    { title: "Logs", url: links?.victorialogs },
    { title: "Metrics", url: links?.metrics },
    { title: "Profiles", url: links?.pyroscope },
    { title: "Envoy admin", url: links?.envoy_admin },
  ].filter((l): l is { title: string; url: string } => Boolean(l.url));

  return (
    <div className="relative">
      <Sidebar collapsible="icon" {...props}>
        <SidebarHeader>
          <TeamSwitcher
            teams={[
              {
                name: site.title,
                logo: ({ className }: { className: string }) => <Logo className={className} />,
                plan: site.plan,
              },
            ]}
          />
        </SidebarHeader>
        <SidebarContent>
          <Suspense fallback={null}>
            {navGroupsFor(kinds).map((group) => (
              <NavGroup key={group.title} {...group} />
            ))}
          </Suspense>
          {external.length ? (
            <NavGroup
              title="Observability"
              items={external.map((l) => ({
                title: l.title,
                url: l.url,
                icon: ExternalLink,
              }))}
            />
          ) : null}
        </SidebarContent>
        <SidebarFooter>
          <NavEnvironment />
        </SidebarFooter>
        <SidebarRail />
      </Sidebar>
    </div>
  );
}

/** The kit's NavUser block, holding the environment instead of a person. */
function NavEnvironment() {
  const { isMobile } = useSidebar();
  const { overview, connected } = useChaos();
  const env = overview?.env ?? "…";
  const domain = overview?.config.links.domain || "local machine";

  return (
    <SidebarMenu>
      <SidebarMenuItem>
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <SidebarMenuButton
              size="lg"
              className="data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground"
            >
              <div className="bg-muted flex size-8 items-center justify-center rounded-lg">
                <Globe className="size-4" />
              </div>
              <div className="grid flex-1 text-left text-sm leading-tight">
                <span className="truncate font-semibold">{env}</span>
                <span className="truncate text-xs">{domain}</span>
              </div>
              <span
                className={cn("size-2 rounded-full", connected ? "bg-success" : "bg-warning")}
                title={connected ? "live feed connected" : "live feed off"}
              />
              <ChevronsUpDown className="ml-1 size-4" />
            </SidebarMenuButton>
          </DropdownMenuTrigger>
          <DropdownMenuContent
            className="w-(--radix-dropdown-menu-trigger-width) min-w-64 rounded-lg"
            side={isMobile ? "bottom" : "right"}
            align="end"
            sideOffset={4}
          >
            <DropdownMenuLabel className="text-muted-foreground text-xs">Environment</DropdownMenuLabel>
            <DropdownMenuItem className="text-xs">
              <span className="text-muted-foreground">config</span>
              <span className="ml-auto truncate font-mono">
                {overview?.config_files.map((f) => f.split("/").pop()).join(" + ") ?? "…"}
              </span>
            </DropdownMenuItem>
            <DropdownMenuItem className="text-xs">
              <span className="text-muted-foreground">version</span>
              <span className="ml-auto font-mono">{overview?.version ?? "…"}</span>
            </DropdownMenuItem>
            <DropdownMenuItem className="text-xs">
              <span className="text-muted-foreground">live feed</span>
              <span className="ml-auto">{connected ? "connected" : "off"}</span>
            </DropdownMenuItem>
            <DropdownMenuSeparator />
            <DropdownMenuItem className="text-xs">
              <span className="text-muted-foreground">API</span>
              <span className="ml-auto font-mono">{overview?.config.serve.base_path ?? "/api/chaos"}</span>
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </SidebarMenuItem>
    </SidebarMenu>
  );
}
