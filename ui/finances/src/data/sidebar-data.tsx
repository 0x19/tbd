import {
  IconBook2,
  IconBuildingBank,
  IconCategory,
  IconChecklist,
  IconFileCertificate,
  IconFileInvoice,
  IconLayoutDashboard,
  IconListDetails,
  IconMail,
  IconPlugConnected,
  IconReceipt2,
  IconSettings,
  IconUsers,
} from "@tabler/icons-react";

import { type NavGroup } from "@/components/layout/types";

/** Navigation groups, the shape the kit's NavGroup and CommandMenu consume. Titles are
 * translation keys; whoever renders them calls `t()`. */
export const navGroups: NavGroup[] = [
  {
    title: "nav.group.money",
    items: [
      { title: "nav.overview", url: "/", icon: IconLayoutDashboard },
      { title: "nav.transactions", url: "/transactions/", icon: IconListDetails },
      { title: "nav.categories", url: "/categories/", icon: IconCategory },
    ],
  },
  {
    title: "nav.group.banking",
    items: [
      { title: "nav.accounts", url: "/accounts/", icon: IconBuildingBank },
      { title: "nav.connections", url: "/connections/", icon: IconPlugConnected },
    ],
  },
  {
    title: "nav.group.documents",
    items: [
      { title: "nav.receipts", url: "/documents/", icon: IconFileInvoice },
      { title: "nav.connectors", url: "/connectors/", icon: IconPlugConnected },
    ],
  },
  {
    title: "nav.group.accountant",
    items: [
      { title: "nav.reconciliation", url: "/reconciliation/", icon: IconChecklist },
      { title: "nav.filings", url: "/filings/", icon: IconFileCertificate },
      { title: "nav.books", url: "/books/", icon: IconBook2 },
    ],
  },
  {
    title: "nav.group.communication",
    items: [{ title: "nav.mail", url: "/mail/", icon: IconMail }],
  },
  {
    title: "nav.group.invoicing",
    items: [
      { title: "nav.invoices", url: "/invoices/", icon: IconReceipt2 },
      { title: "nav.clients", url: "/clients/", icon: IconUsers },
      { title: "nav.issuer", url: "/issuer/", icon: IconSettings },
    ],
  },
];

const clean = (p: string) => (p.split("?")[0] ?? "").replace(/\/$/, "") || "/";

/** Breadcrumb trail for a pathname, as translation keys: [group, page]. */
export function crumbs(pathname: string): string[] {
  const c = clean(pathname);
  if (c === "/connect/callback") return ["nav.group.banking", "nav.connections", "nav.bank_authorization"];
  if (c === "/invoices/view") return ["nav.group.invoicing", "nav.invoices", "nav.invoice"];
  if (c === "/connectors/callback") return ["nav.group.documents", "nav.connectors", "nav.authorization"];
  for (const group of navGroups) {
    for (const item of group.items) {
      const urls = item.items ? item.items.map((i) => i.url) : [item.url];
      for (const u of urls) {
        if (clean(u) === c) return [group.title, item.title];
      }
      const base = item.items ? clean(item.items[0]!.url) : clean(item.url);
      if (base !== "/" && c.startsWith(`${base}/`)) return [group.title, item.title, "nav.detail"];
    }
  }
  return ["nav.group.money"];
}
