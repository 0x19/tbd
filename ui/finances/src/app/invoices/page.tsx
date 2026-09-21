"use client";

// The invoice ledger: every invoice ever issued and every draft that is not
// yet, for daily use. Tabs by state, a search that matches number, client and
// amount, filters in the URL so a view can be shared, and the numbers that
// matter at the top: what is outstanding, what is overdue, what this year
// has billed.
import {
  AlertTriangle,
  ArrowUpDown,
  Copy,
  Download,
  FileText,
  Plus,
  Search,
  Send,
  Trash2,
  Wallet,
  X,
} from "lucide-react";
import Link from "next/link";
import { useRouter, useSearchParams } from "next/navigation";
import { Suspense, useEffect, useMemo, useRef, useState } from "react";
import { toast } from "sonner";

import { useFinance } from "@/app/providers";
import { KpiStrip, PageTitle } from "@/components/kit";
import { ScopeToggle } from "@/components/scope-toggle";
import { StatusBadge } from "@/components/status-badge";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Skeleton } from "@/components/ui/skeleton";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { api, pdfUrl } from "@/lib/api/client";
import { describe, useFetch } from "@/lib/api/hooks";
import type { Invoice } from "@/lib/api/schema";
import { day, money, when } from "@/lib/format";
import { useT } from "@/lib/i18n";
import { stashPrefill } from "@/lib/mail-template";
import { cn } from "@/lib/utils";

type Tab = "all" | "draft" | "issued" | "paid" | "cancelled";
// `label` is a dictionary key, translated where it is rendered.
const TABS: { id: Tab; label: string; match: (i: Invoice) => boolean }[] = [
  { id: "all", label: "invoices.tab.all", match: () => true },
  { id: "draft", label: "invoices.tab.drafts", match: (i) => i.status === "draft" },
  {
    id: "issued",
    label: "invoices.tab.issued",
    match: (i) => i.status === "approved" || i.status === "sent",
  },
  { id: "paid", label: "invoices.tab.paid", match: (i) => i.status === "paid" },
  { id: "cancelled", label: "invoices.tab.cancelled", match: (i) => i.status === "cancelled" },
];
type SortKey = "issued" | "due" | "total" | "number";

export default function InvoicesPage() {
  return (
    <Suspense fallback={<Skeleton className="h-64 w-full" />}>
      <Invoices />
    </Suspense>
  );
}

function todayIso(): string {
  return new Date().toISOString().slice(0, 10);
}

function Invoices() {
  const t = useT();
  const router = useRouter();
  const params = useSearchParams();
  const { parties, partyIds, partyName, multi } = useFinance();
  // Invoices belong to companies. The personal scope has none, so the page
  // looks at the companies in the grant even when the toggle is on the
  // person; with several companies in scope it narrows to those.
  const issuerIds = useMemo(() => {
    const orgs = parties.filter((p) => p.kind === "org").map((p) => p.id);
    const inScope = orgs.filter((id) => partyIds.includes(id));
    return inScope.length ? inScope : orgs;
  }, [parties, partyIds]);
  const key = issuerIds.join(",");
  const invoices = useFetch(() => api.invoices(issuerIds), 30_000, [key]);
  const clients = useFetch(() => api.clients(issuerIds), 0, [key]);

  const tab = (params.get("tab") as Tab) || "all";
  const q = params.get("q") ?? "";
  const clientFilter = params.get("client") ?? "";
  const yearFilter = params.get("year") ?? "";
  const sort = (params.get("sort") as SortKey) || "issued";
  const dir = params.get("dir") === "asc" ? "asc" : "desc";
  const [search, setSearch] = useState(q);
  const searchRef = useRef<HTMLInputElement>(null);
  const [newClient, setNewClient] = useState("");
  const [busy, setBusy] = useState(false);

  const set = (patch: Record<string, string>) => {
    const next = new URLSearchParams(params.toString());
    for (const [k, v] of Object.entries(patch)) {
      if (v) next.set(k, v);
      else next.delete(k);
    }
    const s = next.toString();
    router.replace(s ? `/invoices/?${s}` : "/invoices/");
  };

  // `/` focuses search, `n` starts a draft, like the kit's lists.
  const createRef = useRef<() => void>(() => {});
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const el = e.target as HTMLElement | null;
      if (el && (el.tagName === "INPUT" || el.tagName === "TEXTAREA" || el.isContentEditable)) return;
      if (e.key === "/") {
        e.preventDefault();
        searchRef.current?.focus();
      } else if (e.key === "n" && !e.metaKey && !e.ctrlKey && !e.altKey) {
        e.preventDefault();
        createRef.current();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  const clientList = useMemo(() => clients.data?.clients.filter((c) => !c.archived) ?? [], [clients.data]);
  const clientName = (id: string) => clients.data?.clients.find((c) => c.id === id)?.name ?? "—";
  const all = useMemo(() => invoices.data?.invoices ?? [], [invoices.data]);
  const years = useMemo(() => [...new Set(all.map((i) => String(i.year)))].sort().reverse(), [all]);
  const today = todayIso();

  const filtered = useMemo(() => {
    const needle = q.trim().toLowerCase();
    const digits = needle.replace(/[^0-9]/g, "");
    const current = TABS.find((x) => x.id === tab) ?? TABS[0]!;
    const rows = all.filter((i) => {
      if (!current.match(i)) return false;
      if (clientFilter && i.client_id !== clientFilter) return false;
      if (yearFilter && String(i.year) !== yearFilter) return false;
      if (!needle) return true;
      const hay = `${i.number} ${clientName(i.client_id)} ${i.note}`.toLowerCase();
      if (hay.includes(needle)) return true;
      // "14500" or "14,500.82" finds the invoice with that total.
      return digits.length >= 3 && i.total_minor.replace("-", "").includes(digits);
    });
    const cmp = (a: Invoice, b: Invoice): number => {
      switch (sort) {
        case "due":
          return a.due_date.localeCompare(b.due_date);
        case "total":
          return (
            Number(BigInt(a.total_minor) - BigInt(b.total_minor) > 0n) -
            Number(BigInt(a.total_minor) - BigInt(b.total_minor) < 0n)
          );
        case "number":
          return a.year - b.year || Number(a.number.split("-")[0] || 0) - Number(b.number.split("-")[0] || 0);
        default:
          return (a.issued_at || a.created_at).localeCompare(b.issued_at || b.created_at);
      }
    };
    rows.sort((a, b) => (dir === "asc" ? cmp(a, b) : cmp(b, a)));
    return rows;
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [all, tab, q, clientFilter, yearFilter, sort, dir, clients.data]);

  const counts = useMemo(
    () => Object.fromEntries(TABS.map((x) => [x.id, all.filter(x.match).length])),
    [all],
  );
  const sums = useMemo(() => {
    // What is still owed, not what was billed: a part paid counts.
    const add = (rows: Invoice[]) =>
      rows.reduce((s, i) => s + BigInt(i.total_minor) - BigInt(i.paid_minor), 0n);
    const open = all.filter((i) => i.status === "approved" || i.status === "sent");
    const overdue = open.filter((i) => i.due_date < today);
    const thisYear = all.filter(
      (i) =>
        (i.status === "approved" || i.status === "sent" || i.status === "paid") &&
        i.year === new Date().getFullYear(),
    );
    const ccy = all[0]?.currency ?? "EUR";
    return {
      open: add(open),
      openN: open.length,
      overdue: add(overdue),
      overdueN: overdue.length,
      year: thisYear.reduce((s, i) => s + BigInt(i.total_minor), 0n),
      yearN: thisYear.length,
      ccy,
    };
  }, [all, today]);

  const create = async () => {
    const chosen = newClient || clientList[0]?.id;
    if (!chosen) return;
    setBusy(true);
    try {
      const r = await api.createInvoice(chosen);
      router.push(`/invoices/view/?id=${r.invoice!.id}`);
    } catch (e) {
      toast.error(describe(e));
      setBusy(false);
    }
  };
  useEffect(() => {
    createRef.current = () => void create();
  });

  const duplicate = async (i: Invoice) => {
    try {
      const r = await api.createInvoice("", i.id);
      toast.success(t("invoices.duplicated", { number: i.number || t("invoices.draft_title") }));
      router.push(`/invoices/view/?id=${r.invoice!.id}`);
    } catch (e) {
      toast.error(describe(e));
    }
  };

  // A draft is deleted, never cancelled; the dialog holds the one to delete.
  const [deleting, setDeleting] = useState<Invoice | null>(null);
  const remove = async () => {
    if (!deleting) return;
    try {
      await api.deleteInvoice(deleting.id);
      toast.success(t("invoices.deleted"));
      invoices.reload();
    } catch (e) {
      toast.error(describe(e));
    } finally {
      setDeleting(null);
    }
  };

  const download = async (i: Invoice) => {
    try {
      const d = await api.invoiceDocument(i.id);
      const a = document.createElement("a");
      a.href = pdfUrl(d.pdf);
      a.download = `inorbit-${i.number}.pdf`;
      a.click();
    } catch (e) {
      toast.error(describe(e));
    }
  };

  // An approved invoice goes to its client as a mail: the stored PDF as the
  // attachment, the client's addresses as recipients, the composer does the
  // rest (the same hand-off the reconciliation page makes).
  const sendByMail = (i: Invoice) => {
    const c = clients.data?.clients.find((x) => x.id === i.client_id);
    stashPrefill({
      kind: "invoice",
      party_id: i.party_id,
      company: partyName(i.party_id),
      invoice: {
        id: i.id,
        number: i.number,
        issued_at: i.issued_at,
        due_date: i.due_date,
        total: money(i.total_minor, i.currency),
        document_id: i.document_id,
        filename: `${i.number}.pdf`,
      },
      client: { id: i.client_id, name: c?.name ?? clientName(i.client_id), recipients: c?.recipients ?? [] },
    });
    router.push("/mail/");
  };

  const toggleSort = (k: SortKey) => set({ sort: k, dir: sort === k && dir === "desc" ? "asc" : "desc" });
  // A plain render helper, not a component: a component defined inside
  // render remounts on every pass.
  const sortHead = (k: SortKey, label: string, className?: string) => (
    <TableHead className={className}>
      <button
        type="button"
        className="hover:text-foreground inline-flex items-center gap-1"
        onClick={() => toggleSort(k)}
      >
        {label}
        <ArrowUpDown className={cn("size-3", sort === k ? "opacity-100" : "opacity-30")} />
      </button>
    </TableHead>
  );

  return (
    <>
      <PageTitle title={t("invoices.title")} description={t("invoices.description")}>
        <ScopeToggle className="md:hidden" />
        <Select value={newClient || clientList[0]?.id || ""} onValueChange={setNewClient}>
          <SelectTrigger className="w-44">
            <SelectValue placeholder={t("invoices.client")} />
          </SelectTrigger>
          <SelectContent>
            {clientList.map((c) => (
              <SelectItem key={c.id} value={c.id}>
                {c.name}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <Button size="sm" onClick={() => void create()} disabled={busy || clientList.length === 0}>
          <Plus /> {t("invoices.new_draft")}
        </Button>
      </PageTitle>

      {clients.data && clientList.length === 0 ? (
        <Alert>
          <AlertTriangle />
          <AlertTitle>{t("invoices.no_clients")}</AlertTitle>
          <AlertDescription>
            {t("invoices.needs.before")}{" "}
            <Link href="/issuer/" className="underline">
              {t("invoices.needs.issuer")}
            </Link>{" "}
            {t("invoices.needs.between")}{" "}
            <Link href="/clients/" className="underline">
              {t("invoices.needs.client")}
            </Link>
            {t("invoices.needs.after")}
          </AlertDescription>
        </Alert>
      ) : null}

      {invoices.loading && !invoices.data ? (
        <Skeleton className="h-24 w-full" />
      ) : (
        <KpiStrip
          items={[
            {
              icon: Wallet,
              label: t("invoices.kpi.outstanding"),
              value: money(sums.open.toString(), sums.ccy),
              hint: t("invoices.kpi.outstanding_hint", { n: sums.openN }),
            },
            {
              icon: AlertTriangle,
              label: t("invoices.kpi.overdue"),
              value: money(sums.overdue.toString(), sums.ccy),
              hint: sums.overdueN
                ? t("invoices.kpi.overdue_hint", { n: sums.overdueN })
                : t("invoices.kpi.overdue_none"),
            },
            {
              icon: FileText,
              label: t("invoices.kpi.billed", { year: new Date().getFullYear() }),
              value: money(sums.year.toString(), sums.ccy),
              hint: t("invoices.kpi.billed_hint", { n: sums.yearN }),
            },
            {
              icon: Plus,
              label: t("invoices.kpi.drafts"),
              value: String(counts.draft ?? 0),
              hint: counts.draft ? t("invoices.kpi.drafts_hint") : t("invoices.kpi.drafts_none"),
            },
          ]}
        />
      )}

      <Card>
        <CardHeader className="gap-3">
          <div className="flex flex-wrap items-center gap-2">
            <Tabs value={tab} onValueChange={(v) => set({ tab: v === "all" ? "" : v })}>
              <TabsList>
                {TABS.map((x) => (
                  <TabsTrigger key={x.id} value={x.id} className="gap-1.5">
                    {t(x.label)}
                    <span className="text-muted-foreground text-[11px] tabular-nums">
                      {counts[x.id] ?? 0}
                    </span>
                  </TabsTrigger>
                ))}
              </TabsList>
            </Tabs>
            <form
              className="relative ml-auto"
              onSubmit={(e) => {
                e.preventDefault();
                set({ q: search.trim() });
              }}
            >
              <Search className="text-muted-foreground absolute top-1/2 left-2.5 size-4 -translate-y-1/2" />
              <Input
                ref={searchRef}
                value={search}
                onChange={(e) => {
                  setSearch(e.target.value);
                  if (e.target.value === "") set({ q: "" });
                }}
                placeholder={t("invoices.search_placeholder")}
                className="w-64 pl-8"
              />
            </form>
            {clientList.length > 1 ? (
              <Select
                value={clientFilter || "any"}
                onValueChange={(v) => set({ client: v === "any" ? "" : v })}
              >
                <SelectTrigger className="w-40">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="any">{t("invoices.any_client")}</SelectItem>
                  {clientList.map((c) => (
                    <SelectItem key={c.id} value={c.id}>
                      {c.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            ) : null}
            {years.length > 1 ? (
              <Select value={yearFilter || "any"} onValueChange={(v) => set({ year: v === "any" ? "" : v })}>
                <SelectTrigger className="w-28">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="any">{t("invoices.any_year")}</SelectItem>
                  {years.map((y) => (
                    <SelectItem key={y} value={y}>
                      {y}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            ) : null}
            {q || clientFilter || yearFilter || tab !== "all" ? (
              <Button
                variant="ghost"
                size="sm"
                onClick={() => {
                  setSearch("");
                  router.replace("/invoices/");
                }}
              >
                <X /> {t("invoices.clear")}
              </Button>
            ) : null}
          </div>
        </CardHeader>
        <CardContent className="p-0">
          {invoices.error ? (
            <p className="text-destructive p-4 text-sm">{invoices.error}</p>
          ) : invoices.loading && !invoices.data ? (
            <Skeleton className="m-4 h-48" />
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  {sortHead("number", t("invoices.col.number"))}
                  <TableHead>{t("invoices.col.client")}</TableHead>
                  {multi ? (
                    <TableHead className="hidden md:table-cell">{t("invoices.col.issuer")}</TableHead>
                  ) : null}
                  <TableHead>{t("invoices.col.status")}</TableHead>
                  {sortHead("issued", t("invoices.col.issued"), "hidden sm:table-cell")}
                  {sortHead("due", t("invoices.col.due"))}
                  {sortHead("total", t("invoices.col.total"), "text-right")}
                  <TableHead className="w-12" />
                </TableRow>
              </TableHeader>
              <TableBody>
                {filtered.map((i) => {
                  const overdue = (i.status === "approved" || i.status === "sent") && i.due_date < today;
                  return (
                    <TableRow
                      key={i.id}
                      className="cursor-pointer"
                      onClick={() => router.push(`/invoices/view/?id=${i.id}`)}
                    >
                      <TableCell className="font-mono font-medium">
                        {i.number || (
                          <span className="text-muted-foreground italic">{t("status.draft")}</span>
                        )}
                      </TableCell>
                      <TableCell>{clientName(i.client_id)}</TableCell>
                      {multi ? (
                        <TableCell className="text-muted-foreground hidden text-xs md:table-cell">
                          {partyName(i.party_id)}
                        </TableCell>
                      ) : null}
                      <TableCell>
                        <StatusBadge status={i.status} />
                      </TableCell>
                      <TableCell className="text-muted-foreground hidden text-xs sm:table-cell">
                        {i.issued_at
                          ? when(i.issued_at)
                          : t("invoices.created", { when: when(i.created_at) })}
                      </TableCell>
                      <TableCell
                        className={cn(
                          "text-xs",
                          overdue ? "text-destructive font-medium" : "text-muted-foreground",
                        )}
                      >
                        {day(i.due_date)}
                        {overdue ? ` · ${t("invoices.overdue")}` : ""}
                      </TableCell>
                      <TableCell className="text-right font-mono tabular-nums">
                        {money(i.total_minor, i.currency)}
                      </TableCell>
                      <TableCell onClick={(e) => e.stopPropagation()}>
                        <span className="flex justify-end gap-0.5">
                          {i.document_id ? (
                            <>
                              <Button
                                variant="ghost"
                                size="icon"
                                className="size-8"
                                aria-label={t("invoices.download_pdf")}
                                title={t("invoices.download_pdf")}
                                onClick={() => void download(i)}
                              >
                                <Download />
                              </Button>
                              <Button
                                variant="ghost"
                                size="icon"
                                className="size-8"
                                aria-label={t("invoices.send_mail")}
                                title={t("invoices.send_mail")}
                                onClick={() => sendByMail(i)}
                              >
                                <Send />
                              </Button>
                            </>
                          ) : null}
                          <Button
                            variant="ghost"
                            size="icon"
                            className="size-8"
                            aria-label={t("invoices.duplicate")}
                            title={t("invoices.duplicate")}
                            onClick={() => void duplicate(i)}
                          >
                            <Copy />
                          </Button>
                          {i.status === "draft" ? (
                            <Button
                              variant="ghost"
                              size="icon"
                              className="size-8"
                              aria-label={t("invoices.delete_draft")}
                              title={t("invoices.delete_draft")}
                              onClick={() => setDeleting(i)}
                            >
                              <Trash2 />
                            </Button>
                          ) : null}
                        </span>
                      </TableCell>
                    </TableRow>
                  );
                })}
                {filtered.length === 0 ? (
                  <TableRow>
                    <TableCell colSpan={8} className="text-muted-foreground py-12 text-center text-sm">
                      {all.length === 0
                        ? t("invoices.empty")
                        : q
                          ? t("invoices.nothing_matches", { q })
                          : tab === "draft"
                            ? t("invoices.no_drafts")
                            : t("common.nothing_here")}
                    </TableCell>
                  </TableRow>
                ) : null}
              </TableBody>
            </Table>
          )}
        </CardContent>
        {filtered.length ? (
          <div className="text-muted-foreground flex items-center justify-between border-t px-4 py-2 text-xs">
            <span>{t("invoices.n_of_m", { n: filtered.length, m: all.length })}</span>
            <span className="font-mono tabular-nums">
              {t("invoices.in_view", {
                amount: money(filtered.reduce((s, i) => s + BigInt(i.total_minor), 0n).toString(), sums.ccy),
              })}
            </span>
          </div>
        ) : null}
      </Card>
      <AlertDialog open={deleting !== null} onOpenChange={(o) => !o && setDeleting(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t("invoices.delete_title")}</AlertDialogTitle>
            <AlertDialogDescription>{t("invoices.delete_confirm")}</AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t("common.cancel")}</AlertDialogCancel>
            <AlertDialogAction onClick={() => void remove()}>{t("invoices.delete_draft")}</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
}
