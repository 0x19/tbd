"use client";

import { Plus } from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";

import { useFinance } from "@/app/providers";
import { PageTitle } from "@/components/kit";
import { StatusBadge } from "@/components/status-badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Skeleton } from "@/components/ui/skeleton";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { api } from "@/lib/api/client";
import { describe, useFetch } from "@/lib/api/hooks";
import { when } from "@/lib/format";

export default function ConnectionsPage() {
  const { parties, partyIds, partyName } = useFinance();
  const connections = useFetch(() => api.connections(partyIds), 30_000, [partyIds.join(",")]);
  const [party, setParty] = useState("");
  const [psu, setPsu] = useState<"business" | "personal">("business");
  const [busy, setBusy] = useState(false);
  const chosen = party || partyIds[0] || "";

  const start = async () => {
    setBusy(true);
    try {
      const s = await api.startConnection(chosen, psu);
      window.location.href = s.url;
    } catch (e) {
      toast.error(describe(e));
      setBusy(false);
    }
  };

  return (
    <>
      <PageTitle
        title="Connections"
        description="A consent at a bank lasts 180 days. Linking again finds the same accounts and keeps their history."
      >
        <Select value={chosen} onValueChange={setParty}>
          <SelectTrigger className="w-44">
            <SelectValue placeholder="Party" />
          </SelectTrigger>
          <SelectContent>
            {parties.map((p) => (
              <SelectItem key={p.id} value={p.id}>
                {p.display_name}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <Select value={psu} onValueChange={(v) => setPsu(v as "business" | "personal")}>
          <SelectTrigger className="w-36">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="business">Business login</SelectItem>
            <SelectItem value="personal">Personal login</SelectItem>
          </SelectContent>
        </Select>
        <Button size="sm" onClick={() => void start()} disabled={busy || !chosen}>
          <Plus /> {busy ? "Opening the bank…" : "Link Erste"}
        </Button>
      </PageTitle>
      <Card>
        <CardContent className="p-0">
          {connections.error ? (
            <p className="text-destructive p-4 text-sm">{connections.error}</p>
          ) : connections.loading && !connections.data ? (
            <Skeleton className="m-4 h-40" />
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Bank</TableHead>
                  <TableHead>Party</TableHead>
                  <TableHead>Login</TableHead>
                  <TableHead>Status</TableHead>
                  <TableHead>Authorized</TableHead>
                  <TableHead>Valid until</TableHead>
                  <TableHead className="text-right">Accounts</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {(connections.data?.connections ?? []).map((c) => (
                  <TableRow key={c.id}>
                    <TableCell className="font-medium">{c.aspsp_name}</TableCell>
                    <TableCell>{partyName(c.party_id)}</TableCell>
                    <TableCell className="capitalize">{c.psu_type}</TableCell>
                    <TableCell>
                      <StatusBadge status={c.status} />
                    </TableCell>
                    <TableCell className="text-muted-foreground text-xs">{when(c.authorized_at)}</TableCell>
                    <TableCell className="text-muted-foreground text-xs">{when(c.valid_until)}</TableCell>
                    <TableCell className="text-right tabular-nums">{c.accounts}</TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>
    </>
  );
}
