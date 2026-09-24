"use client";

import { ChevronsUpDown, Wallet } from "lucide-react";
import { Suspense } from "react";
import * as React from "react";

import { useFinance } from "@/app/providers";
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
import { navGroups } from "@/data/sidebar-data";
import { site } from "@/data/site";
import { useT } from "@/lib/i18n";
import { cn } from "@/lib/utils";

/** The kit's AppSidebar with the finance navigation and the scope block in the footer. */
export function AppSidebar({ ...props }: React.ComponentProps<typeof Sidebar>) {
  const [isMounted, setIsMounted] = React.useState(false);
  const { state } = useSidebar();

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
            {navGroups.map((group) => (
              <NavGroup key={group.title} {...group} />
            ))}
          </Suspense>
        </SidebarContent>
        <SidebarFooter>
          <NavScope />
        </SidebarFooter>
        <SidebarRail />
      </Sidebar>
    </div>
  );
}

/** The kit's NavUser block, holding the scope: whose money the pages show. */
function NavScope() {
  const { isMobile } = useSidebar();
  const { parties, scope, setScope, loading, error } = useFinance();
  const t = useT();
  const current = scope === "all" ? "Combined" : (parties.find((p) => p.id === scope)?.display_name ?? "…");
  const detail = error
    ? "not signed in"
    : loading
      ? "…"
      : `${parties.length} ${parties.length === 1 ? "party" : "parties"}`;

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
                <Wallet className="size-4" />
              </div>
              <div className="grid flex-1 text-left text-sm leading-tight">
                <span className="truncate font-semibold">{current}</span>
                <span className="truncate text-xs">{detail}</span>
              </div>
              <ChevronsUpDown className="ml-1 size-4" />
            </SidebarMenuButton>
          </DropdownMenuTrigger>
          <DropdownMenuContent
            className="w-(--radix-dropdown-menu-trigger-width) min-w-64 rounded-lg"
            side={isMobile ? "bottom" : "right"}
            align="end"
            sideOffset={4}
          >
            <DropdownMenuLabel className="text-muted-foreground text-xs">{t("nav.show")}</DropdownMenuLabel>
            <DropdownMenuItem onSelect={() => setScope("all")} className="text-xs">
              {t("common.combined")}
              {scope === "all" ? <span className="ml-auto">✓</span> : null}
            </DropdownMenuItem>
            <DropdownMenuSeparator />
            {parties.map((p) => (
              <DropdownMenuItem key={p.id} onSelect={() => setScope(p.id)} className="text-xs">
                <span className="truncate">{p.display_name}</span>
                <span className="text-muted-foreground ml-auto pl-2">{p.capability}</span>
                {scope === p.id ? <span className="pl-2">✓</span> : null}
              </DropdownMenuItem>
            ))}
          </DropdownMenuContent>
        </DropdownMenu>
      </SidebarMenuItem>
    </SidebarMenu>
  );
}
