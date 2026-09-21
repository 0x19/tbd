"use client";

// The record: every mail sent from a linked mailbox and every reply pulled
// into its thread. Search and direction as filters, a row opens the thread.
import { Archive, ArrowDownLeft, ArrowUpRight, Paperclip, Search } from "lucide-react";
import { useState } from "react";

import { useFinance } from "@/app/providers";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Skeleton } from "@/components/ui/skeleton";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { api } from "@/lib/api/client";
import { useFetch } from "@/lib/api/hooks";
import type { Connector, Mail } from "@/lib/api/schema";
import { when } from "@/lib/format";
import { useT } from "@/lib/i18n";
import { cn } from "@/lib/utils";

import { ThreadSheet } from "./thread-sheet";

export function MailStatus({
  status,
  error,
  className,
}: {
  status: string;
  error?: string;
  className?: string;
}) {
  const t = useT();
  return (
    <Badge
      variant="outline"
      className={cn(
        "border-transparent text-[10px]",
        status === "failed"
          ? "bg-destructive/12 text-destructive"
          : status === "received"
            ? "bg-sky-600/12 text-sky-700 dark:text-sky-300"
            : "bg-emerald-600/12 text-emerald-700 dark:text-emerald-300",
        className,
      )}
      title={error}
    >
      {t(`mail.status.${status}`)}
    </Badge>
  );
}

export function SentList({
  version,
  connectors,
  onReply,
  initialOpen = null,
}: {
  version: number;
  connectors: Connector[];
  onReply: (m: Mail) => void;
  /** A thread to open on arrival, from the URL. */
  initialOpen?: string | null;
}) {
  const t = useT();
  const { partyIds } = useFinance();
  const [q, setQ] = useState("");
  const [direction, setDirection] = useState("");
  const [openId, setOpenId] = useState<string | null>(initialOpen);
  const list = useFetch(() => api.mail({ party_ids: partyIds, q, direction, limit: 100 }), 0, [
    partyIds.join(","),
    q,
    direction,
    version,
  ]);
  const mailbox = (id: string) => connectors.find((c) => c.id === id)?.external_id ?? "";
  const mails = list.data?.mails ?? [];
  return (
    <div className="space-y-3">
      <div className="flex flex-wrap items-center gap-2">
        <div className="relative min-w-64 flex-1">
          <Search className="text-muted-foreground absolute top-1/2 left-2.5 size-4 -translate-y-1/2" />
          <Input
            value={q}
            onChange={(e) => setQ(e.target.value)}
            placeholder={t("mail.search_placeholder")}
            className="pl-8"
          />
        </div>
        {["", "out", "in"].map((d) => (
          <Button
            key={d}
            size="sm"
            variant={direction === d ? "default" : "outline"}
            onClick={() => setDirection(d)}
          >
            {t(`mail.direction.${d || "all"}`)}
          </Button>
        ))}
        {list.data ? (
          <span className="text-muted-foreground ml-auto text-xs">
            {t("mail.list.count", { n: list.data.total })}
          </span>
        ) : null}
      </div>
      <Card>
        <CardContent className="p-0">
          {list.error ? (
            <p className="text-destructive p-4 text-sm">{list.error}</p>
          ) : list.loading && !list.data ? (
            <Skeleton className="m-4 h-48" />
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="w-40">{t("mail.list.when")}</TableHead>
                  <TableHead className="w-64">{t("mail.list.who")}</TableHead>
                  <TableHead>{t("mail.list.subject")}</TableHead>
                  <TableHead className="w-24">{t("mail.list.status")}</TableHead>
                  <TableHead className="w-20 text-right">{t("mail.list.replies")}</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {mails.map((m) => (
                  <TableRow key={m.id} className="cursor-pointer" onClick={() => setOpenId(m.id)}>
                    <TableCell className="text-muted-foreground text-xs whitespace-nowrap">
                      {when(m.sent_at || m.received_at)}
                    </TableCell>
                    <TableCell className="max-w-64 text-xs">
                      <span className="flex items-center gap-1.5">
                        {m.direction === "in" ? (
                          <ArrowDownLeft className="size-3.5 shrink-0 text-sky-600" />
                        ) : (
                          <ArrowUpRight className="size-3.5 shrink-0 text-emerald-600" />
                        )}
                        <span className="truncate" title={m.direction === "in" ? m.from : m.to.join(", ")}>
                          {m.direction === "in" ? m.from : m.to.join(", ")}
                        </span>
                      </span>
                    </TableCell>
                    <TableCell className="max-w-96">
                      <span className="flex items-center gap-2">
                        <span className="truncate font-medium">{m.subject}</span>
                        {m.bundle ? (
                          <span
                            className="text-muted-foreground flex shrink-0 items-center gap-0.5 text-xs"
                            title={m.bundle}
                          >
                            <Archive className="size-3" /> {m.documents.length}
                          </span>
                        ) : m.documents.length ? (
                          <span className="text-muted-foreground flex shrink-0 items-center gap-0.5 text-xs">
                            <Paperclip className="size-3" /> {m.documents.length}
                          </span>
                        ) : null}
                      </span>
                      <span className="text-muted-foreground block truncate text-xs">
                        {m.body.split("\n")[0]}
                      </span>
                    </TableCell>
                    <TableCell>
                      <MailStatus status={m.status} error={m.error} />
                    </TableCell>
                    <TableCell className="text-right font-mono text-xs tabular-nums">
                      {m.direction === "out" && m.replies ? m.replies : ""}
                    </TableCell>
                  </TableRow>
                ))}
                {list.data && mails.length === 0 ? (
                  <TableRow>
                    <TableCell colSpan={5} className="text-muted-foreground py-12 text-center text-sm">
                      {q || direction ? t("mail.list.nothing_matches") : t("mail.list.empty")}
                    </TableCell>
                  </TableRow>
                ) : null}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>
      <ThreadSheet id={openId} mailbox={mailbox} onClose={() => setOpenId(null)} onReply={onReply} />
    </div>
  );
}
