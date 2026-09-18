"use client";

import { Download } from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";

import { useFinance } from "@/app/providers";
import { PageTitle } from "@/components/kit";
import { ScopeToggle } from "@/components/scope-toggle";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { api, pdfUrl } from "@/lib/api/client";
import { describe, useFetch } from "@/lib/api/hooks";
import type { Document } from "@/lib/api/schema";
import { money, when } from "@/lib/format";

export default function DocumentsPage() {
  const { partyIds } = useFinance();
  const docs = useFetch(() => api.documents(partyIds, "receipt", 200), 30_000, [partyIds.join(",")]);
  const [busy, setBusy] = useState("");
  const open = async (d: Document) => {
    setBusy(d.id);
    try {
      const r = await api.document(d.id);
      const url = pdfUrl(r.bytes);
      const a = window.document.createElement("a");
      a.href = url;
      a.download = d.filename || `${d.id}.pdf`;
      a.click();
    } catch (e) {
      toast.error(describe(e));
    } finally {
      setBusy("");
    }
  };
  return (
    <>
      <PageTitle
        title="Receipts"
        description="Everything pulled from linked mailboxes and portals, newest first. Vendor, date and amount are filled in as they are recognised."
      >
        <ScopeToggle className="md:hidden" />
      </PageTitle>
      <Card>
        <CardContent className="p-0">
          {docs.error ? (
            <p className="text-destructive p-4 text-sm">{docs.error}</p>
          ) : docs.loading && !docs.data ? (
            <Skeleton className="m-4 h-48" />
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>File</TableHead>
                  <TableHead>From</TableHead>
                  <TableHead className="hidden lg:table-cell">Subject</TableHead>
                  <TableHead>Received</TableHead>
                  <TableHead>Vendor</TableHead>
                  <TableHead className="text-right">Amount</TableHead>
                  <TableHead className="w-10" />
                </TableRow>
              </TableHeader>
              <TableBody>
                {(docs.data?.documents ?? []).map((d) => {
                  const s = d.sources[0];
                  return (
                    <TableRow key={d.id}>
                      <TableCell className="max-w-64 truncate font-medium">
                        {d.filename || d.id.slice(0, 8)}
                      </TableCell>
                      <TableCell className="text-muted-foreground max-w-56 truncate text-xs">
                        {s?.sender ?? "—"}
                      </TableCell>
                      <TableCell className="text-muted-foreground hidden max-w-80 truncate text-xs lg:table-cell">
                        {s?.subject ?? ""}
                      </TableCell>
                      <TableCell className="text-muted-foreground text-xs whitespace-nowrap">
                        {when(s?.received_at || d.created_at)}
                      </TableCell>
                      <TableCell>
                        {d.vendor ? (
                          <Badge variant="outline">{d.vendor}</Badge>
                        ) : (
                          <span className="text-muted-foreground text-xs italic">not yet</span>
                        )}
                      </TableCell>
                      <TableCell className="text-right font-mono tabular-nums">
                        {d.total_minor ? money(d.total_minor, d.currency || "EUR") : "—"}
                      </TableCell>
                      <TableCell>
                        <Button
                          variant="ghost"
                          size="icon"
                          className="size-8"
                          disabled={busy === d.id}
                          onClick={() => void open(d)}
                          aria-label="Download"
                        >
                          <Download />
                        </Button>
                      </TableCell>
                    </TableRow>
                  );
                })}
                {docs.data && docs.data.documents.length === 0 ? (
                  <TableRow>
                    <TableCell colSpan={7} className="text-muted-foreground py-10 text-center text-sm">
                      Nothing pulled yet. Link a mailbox under Connectors and pull.
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
