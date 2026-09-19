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
import { useT } from "@/lib/i18n";

// The wire value of a login type, shown in the page's language.
const LOGIN_KEY: Record<string, string> = {
  business: "banking.login.business",
  personal: "banking.login.personal",
};

export default function ConnectionsPage() {
  const t = useT();
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
      <PageTitle title={t("banking.connections.title")} description={t("banking.connections.description")}>
        <Select value={chosen} onValueChange={setParty}>
          <SelectTrigger className="w-44">
            <SelectValue placeholder={t("common.party")} />
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
            <SelectItem value="business">{t("banking.business_login")}</SelectItem>
            <SelectItem value="personal">{t("banking.personal_login")}</SelectItem>
          </SelectContent>
        </Select>
        <Button size="sm" onClick={() => void start()} disabled={busy || !chosen}>
          <Plus /> {busy ? t("banking.opening_bank") : t("banking.link_erste")}
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
                  <TableHead>{t("banking.col.bank")}</TableHead>
                  <TableHead>{t("common.party")}</TableHead>
                  <TableHead>{t("banking.col.login")}</TableHead>
                  <TableHead>{t("banking.col.status")}</TableHead>
                  <TableHead>{t("banking.col.authorized")}</TableHead>
                  <TableHead>{t("banking.col.valid_until")}</TableHead>
                  <TableHead className="text-right">{t("banking.col.accounts")}</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {(connections.data?.connections ?? []).map((c) => (
                  <TableRow key={c.id}>
                    <TableCell className="font-medium">{c.aspsp_name}</TableCell>
                    <TableCell>{partyName(c.party_id)}</TableCell>
                    <TableCell className="capitalize">
                      {LOGIN_KEY[c.psu_type] ? t(LOGIN_KEY[c.psu_type]!) : c.psu_type}
                    </TableCell>
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
