"use client";

import { Plus } from "lucide-react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { useState } from "react";
import { toast } from "sonner";

import { useFinance } from "@/app/providers";
import { PageTitle } from "@/components/kit";
import { ScopeToggle } from "@/components/scope-toggle";
import { StatusBadge } from "@/components/status-badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Skeleton } from "@/components/ui/skeleton";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { api } from "@/lib/api/client";
import { describe, useFetch } from "@/lib/api/hooks";
import { day, money, when } from "@/lib/format";

export default function InvoicesPage() {
  const router = useRouter();
  const { partyIds, partyName, multi } = useFinance();
  const key = partyIds.join(",");
  const invoices = useFetch(() => api.invoices(partyIds), 30_000, [key]);
  const clients = useFetch(() => api.clients(partyIds), 0, [key]);
  const [client, setClient] = useState("");
  const [busy, setBusy] = useState(false);
  const list = clients.data?.clients.filter((c) => !c.archived) ?? [];
  const chosen = client || list[0]?.id || "";
  const clientName = (id: string) => clients.data?.clients.find((c) => c.id === id)?.name ?? "—";

  const create = async () => {
    setBusy(true);
    try {
      const r = await api.createInvoice(chosen);
      router.push(`/invoices/view/?id=${r.invoice!.id}`);
    } catch (e) {
      toast.error(describe(e));
      setBusy(false);
    }
  };

  return (
    <>
      <PageTitle title="Invoices" description="Every invoice ever issued, and the drafts that are not yet.">
        <ScopeToggle className="md:hidden" />
        <Select value={chosen} onValueChange={setClient}>
          <SelectTrigger className="w-44">
            <SelectValue placeholder="Client" />
          </SelectTrigger>
          <SelectContent>
            {list.map((c) => (
              <SelectItem key={c.id} value={c.id}>
                {c.name}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <Button size="sm" onClick={() => void create()} disabled={busy || !chosen}>
          <Plus /> New draft
        </Button>
      </PageTitle>

      {!clients.loading && list.length === 0 ? (
        <Card className="max-w-xl">
          <CardHeader>
            <CardTitle>No clients yet</CardTitle>
            <CardDescription>
              An invoice needs an issuer profile and a client. Set both under{" "}
              <Link href="/invoices/settings/" className="underline">
                Issuer &amp; clients
              </Link>
              .
            </CardDescription>
          </CardHeader>
        </Card>
      ) : null}

      <Card>
        <CardContent className="p-0">
          {invoices.error ? (
            <p className="text-destructive p-4 text-sm">{invoices.error}</p>
          ) : invoices.loading && !invoices.data ? (
            <Skeleton className="m-4 h-48" />
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Number</TableHead>
                  <TableHead>Client</TableHead>
                  {multi ? <TableHead className="hidden md:table-cell">Issuer</TableHead> : null}
                  <TableHead>Status</TableHead>
                  <TableHead>Issued</TableHead>
                  <TableHead>Due</TableHead>
                  <TableHead className="text-right">Total</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {(invoices.data?.invoices ?? []).map((i) => (
                  <TableRow
                    key={i.id}
                    className="cursor-pointer"
                    onClick={() => router.push(`/invoices/view/?id=${i.id}`)}
                  >
                    <TableCell className="font-mono font-medium">
                      {i.number || <span className="text-muted-foreground italic">draft</span>}
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
                    <TableCell className="text-muted-foreground text-xs">
                      {i.issued_at ? when(i.issued_at) : "—"}
                    </TableCell>
                    <TableCell className="text-muted-foreground text-xs">{day(i.due_date)}</TableCell>
                    <TableCell className="text-right font-mono tabular-nums">
                      {money(i.total_minor, i.currency)}
                    </TableCell>
                  </TableRow>
                ))}
                {invoices.data && invoices.data.invoices.length === 0 ? (
                  <TableRow>
                    <TableCell colSpan={7} className="text-muted-foreground py-10 text-center text-sm">
                      No invoices yet.
                    </TableCell>
                  </TableRow>
                ) : null}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>
    </>
  );
}
