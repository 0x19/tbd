"use client";

import { useState } from "react";
import { toast } from "sonner";

import { useMe } from "@/app/providers";
import { Frame, SectionHead } from "@/components/kit";
import { StatusBadge } from "@/components/status-badge";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { api } from "@/lib/api/client";
import { describe, useFetch, when } from "@/lib/api/hooks";
import type { AccessRequest } from "@/lib/api/schema";
import { useT } from "@/lib/i18n";

const FILTERS = ["", "requested", "approved", "refused", "revoked"] as const;

/** The owner's list: everyone who asked, and the three buttons. */
export default function AdminPage() {
  const t = useT();
  const me = useMe();
  const [filter, setFilter] = useState<string>("");
  const owner = me.data?.role === "admin";
  const list = useFetch(() => (owner ? api.requests(filter) : Promise.resolve({ requests: [] })), 30_000, [
    owner,
    filter,
  ]);
  const [busy, setBusy] = useState<string | null>(null);

  async function decide(r: AccessRequest, decision: "approve" | "refuse" | "revoke") {
    setBusy(r.id);
    try {
      await api.decide(r.id, decision);
      list.reload();
    } catch (e) {
      toast.error(t("cv.error", { error: describe(e) }));
    } finally {
      setBusy(null);
    }
  }

  if (me.data && !owner)
    return (
      <Frame className="py-20">
        <p className="text-muted-foreground">{t("cv.admin.not_owner")}</p>
      </Frame>
    );

  return (
    <Frame className="grid gap-6 pt-16 pb-24 sm:pt-24 sm:pb-32">
      <SectionHead n="01" label={t("cv.admin.title")} lead={t("cv.admin.lead")} />
      <Tabs value={filter} onValueChange={setFilter}>
        <TabsList>
          {FILTERS.map((f) => (
            <TabsTrigger key={f} value={f} className="text-xs capitalize">
              {f ? t(`status.${f}`) : t("cv.admin.filter.all")}
            </TabsTrigger>
          ))}
        </TabsList>
      </Tabs>
      {list.loading && !list.data ? <Skeleton className="h-24 w-full" /> : null}
      {list.error ? <p className="text-destructive text-sm">{list.error}</p> : null}
      {list.data && list.data.requests.length === 0 ? (
        <p className="text-muted-foreground text-sm">{t("cv.admin.empty")}</p>
      ) : null}
      {list.data && list.data.requests.length > 0 ? (
        <div className="overflow-x-auto rounded-md border">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>{t("cv.admin.who")}</TableHead>
                <TableHead>{t("cv.admin.note")}</TableHead>
                <TableHead>{t("cv.admin.state")}</TableHead>
                <TableHead>{t("cv.admin.asked")}</TableHead>
                <TableHead>{t("cv.admin.downloads")}</TableHead>
                <TableHead />
              </TableRow>
            </TableHeader>
            <TableBody>
              {list.data.requests.map((r) => (
                <TableRow key={r.id}>
                  <TableCell>
                    <div className="font-medium">{r.name || r.email}</div>
                    <div className="text-muted-foreground font-mono text-xs">{r.email}</div>
                  </TableCell>
                  <TableCell className="max-w-xs text-pretty whitespace-normal">{r.note || "—"}</TableCell>
                  <TableCell>
                    <StatusBadge status={r.status} />
                    <div className="text-muted-foreground mt-1 text-xs">
                      {r.notified_at
                        ? t("cv.admin.mailed", { when: when(r.notified_at) })
                        : t("cv.admin.not_mailed")}
                      {r.decided_at
                        ? ` · ${t("cv.admin.decided")} ${when(r.decided_at)} (${r.decided_by})`
                        : ""}
                    </div>
                  </TableCell>
                  <TableCell className="whitespace-nowrap">{when(r.requested_at)}</TableCell>
                  <TableCell>
                    {r.downloads}
                    {r.last_download_at ? (
                      <div className="text-muted-foreground text-xs">
                        {t("cv.admin.last_download", { when: when(r.last_download_at) })}
                      </div>
                    ) : null}
                  </TableCell>
                  <TableCell className="whitespace-nowrap">
                    <div className="flex justify-end gap-1">
                      {r.status !== "approved" ? (
                        <Button size="sm" onClick={() => decide(r, "approve")} disabled={busy === r.id}>
                          {t("cv.admin.approve")}
                        </Button>
                      ) : null}
                      {r.status === "requested" ? (
                        <Button
                          size="sm"
                          variant="outline"
                          onClick={() => decide(r, "refuse")}
                          disabled={busy === r.id}
                        >
                          {t("cv.admin.refuse")}
                        </Button>
                      ) : null}
                      {r.status === "approved" ? (
                        <Button
                          size="sm"
                          variant="outline"
                          onClick={() => decide(r, "revoke")}
                          disabled={busy === r.id}
                        >
                          {t("cv.admin.revoke")}
                        </Button>
                      ) : null}
                    </div>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
      ) : null}
    </Frame>
  );
}
