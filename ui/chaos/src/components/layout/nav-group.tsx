"use client";

import { ChevronRight } from "lucide-react";
import Link from "next/link";
import { usePathname, useSearchParams } from "next/navigation";
import { ReactNode } from "react";

import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/components/ui/collapsible";
import {
  SidebarGroup,
  SidebarGroupLabel,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarMenuSub,
  SidebarMenuSubButton,
  SidebarMenuSubItem,
  useSidebar,
} from "@/components/ui/sidebar";

import { Badge } from "../ui/badge";
import { type NavGroup, NavItem } from "./types";

export function NavGroup({ title, items }: NavGroup) {
  const { setOpenMobile } = useSidebar();
  const pathname = usePathname();
  const search = useSearchParams();
  const current = search.size ? `${pathname}?${search.toString()}` : pathname;
  return (
    <SidebarGroup>
      <SidebarGroupLabel>{title}</SidebarGroupLabel>
      <SidebarMenu>
        {items.map((item) => {
          if (!item.items) {
            return (
              <SidebarMenuItem key={item.title}>
                <SidebarMenuButton asChild isActive={checkIsActive(current, item, true)} tooltip={item.title}>
                  <Link href={item.url} onClick={() => setOpenMobile(false)}>
                    {item.icon && <item.icon />}
                    <span>{item.title}</span>
                    {item.badge && <NavBadge>{item.badge}</NavBadge>}
                  </Link>
                </SidebarMenuButton>
              </SidebarMenuItem>
            );
          }
          return (
            <Collapsible
              key={item.title}
              asChild
              defaultOpen={checkIsActive(current, item, true)}
              className="group/collapsible"
            >
              <SidebarMenuItem>
                <CollapsibleTrigger asChild>
                  <SidebarMenuButton tooltip={item.title}>
                    {item.icon && <item.icon />}
                    <span>{item.title}</span>
                    {item.badge && <NavBadge>{item.badge}</NavBadge>}
                    <ChevronRight className="ml-auto transition-transform duration-200 group-data-[state=open]/collapsible:rotate-90" />
                  </SidebarMenuButton>
                </CollapsibleTrigger>
                <CollapsibleContent className="CollapsibleContent">
                  <SidebarMenuSub>
                    {item.items.map((subItem) => (
                      <SidebarMenuSubItem key={subItem.title}>
                        <SidebarMenuSubButton asChild isActive={checkIsActive(pathname, subItem)}>
                          <Link href={subItem.url} onClick={() => setOpenMobile(false)}>
                            {subItem.icon && <subItem.icon />}
                            <span>{subItem.title}</span>
                            {subItem.badge && <NavBadge>{subItem.badge}</NavBadge>}
                          </Link>
                        </SidebarMenuSubButton>
                      </SidebarMenuSubItem>
                    ))}
                  </SidebarMenuSub>
                </CollapsibleContent>
              </SidebarMenuItem>
            </Collapsible>
          );
        })}
      </SidebarMenu>
    </SidebarGroup>
  );
}

const NavBadge = ({ children }: { children: ReactNode }) => (
  <Badge className="rounded-full px-1 py-0 text-xs">{children}</Badge>
);

/**
 * Whether `item` is the current page. A sub-item with a query string
 * (`/runs/?kind=load`) is current only when the location carries those same
 * parameters; a plain one (`/runs/`) only when the location carries none, so
 * one Runs entry lights up at a time. A main item is also current for any
 * path below it.
 */
function checkIsActive(href: string, item: NavItem, mainNav = false): boolean {
  if (item.items) {
    return item.items.some((subItem) => checkIsActive(href, subItem));
  }
  const [currentPath, currentQuery = ""] = splitUrl(href);
  const [itemPath, itemQuery = ""] = splitUrl(item.url);
  if (currentPath === itemPath) {
    return sameQuery(currentQuery, itemQuery);
  }
  return mainNav && itemPath !== "/" && currentPath.startsWith(`${itemPath}/`);
}

function splitUrl(url: string): [string, string?] {
  const [path = "", query] = url.split("?");
  return [normalizePath(path), query];
}

function sameQuery(a: string, b: string): boolean {
  const pa = new URLSearchParams(a);
  const pb = new URLSearchParams(b);
  if ([...pa.keys()].length !== [...pb.keys()].length) return false;
  return [...pb.entries()].every(([k, v]) => pa.get(k) === v);
}

function normalizePath(url: string): string {
  return url.replace(/\/$/, "") || "/";
}
