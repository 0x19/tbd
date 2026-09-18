import {
  IconBuildingBank,
  IconCategory,
  IconFileInvoice,
  IconLayoutDashboard,
  IconListDetails,
  IconPlugConnected,
  IconReceipt2,
  IconSettings,
} from "@tabler/icons-react";

import { type NavGroup } from "@/components/layout/types";

/** Navigation groups, the shape the kit's NavGroup and CommandMenu consume. */
export const navGroups: NavGroup[] = [
  {
    title: "Money",
    items: [
      { title: "Overview", url: "/", icon: IconLayoutDashboard },
      { title: "Transactions", url: "/transactions/", icon: IconListDetails },
      { title: "Categories & rules", url: "/categories/", icon: IconCategory },
    ],
  },
  {
    title: "Banking",
    items: [
      { title: "Accounts", url: "/accounts/", icon: IconBuildingBank },
      { title: "Connections", url: "/connections/", icon: IconPlugConnected },
    ],
  },
  {
    title: "Documents",
    items: [
      { title: "Receipts", url: "/documents/", icon: IconFileInvoice },
      { title: "Connectors", url: "/connectors/", icon: IconPlugConnected },
    ],
  },
  {
    title: "Invoicing",
    items: [
      { title: "Invoices", url: "/invoices/", icon: IconReceipt2 },
      { title: "Issuer & clients", url: "/invoices/settings/", icon: IconSettings },
    ],
  },
];

const clean = (p: string) => (p.split("?")[0] ?? "").replace(/\/$/, "") || "/";

/** Breadcrumb trail for a pathname: [group, page]. */
export function crumbs(pathname: string): string[] {
  const c = clean(pathname);
  if (c === "/connect/callback") return ["Banking", "Connections", "Bank authorization"];
  if (c === "/invoices/view") return ["Invoicing", "Invoices", "Invoice"];
  if (c === "/connectors/callback") return ["Documents", "Connectors", "Authorization"];
  for (const group of navGroups) {
    for (const item of group.items) {
      const urls = item.items ? item.items.map((i) => i.url) : [item.url];
      for (const u of urls) {
        if (clean(u) === c) return [group.title, item.title];
      }
      const base = item.items ? clean(item.items[0]!.url) : clean(item.url);
      if (base !== "/" && c.startsWith(`${base}/`)) return [group.title, item.title, "Detail"];
    }
  }
  return ["Money"];
}
