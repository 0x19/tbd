"use client";

// One thread: the mail we sent and what came back, oldest first, each with
// its attachments; Reply hands the composer the mail to answer.
import { Archive, Mail as MailIcon, Paperclip, Reply } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Sheet, SheetContent, SheetDescription, SheetHeader, SheetTitle } from "@/components/ui/sheet";
import { Skeleton } from "@/components/ui/skeleton";
import { api } from "@/lib/api/client";
import { useFetch } from "@/lib/api/hooks";
import type { Mail } from "@/lib/api/schema";
import { when } from "@/lib/format";
import { useT } from "@/lib/i18n";
import { cn } from "@/lib/utils";

import { MailStatus } from "./sent-list";

export function ThreadSheet({
  id,
  mailbox,
  onClose,
  onReply,
}: {
  id: string | null;
  mailbox: (connectorId: string) => string;
  onClose: () => void;
  onReply: (m: Mail) => void;
}) {
  const t = useT();
  const thread = useFetch(() => (id ? api.mailThread(id) : Promise.resolve(null)), 0, [id]);
  const root = thread.data?.thread[0];
  return (
    <Sheet open={id !== null} onOpenChange={(o) => !o && onClose()}>
      <SheetContent className="flex w-full flex-col gap-4 overflow-y-auto sm:max-w-3xl">
        <SheetHeader>
          <SheetTitle className="flex items-center gap-2">
            <MailIcon className="size-4" /> {root?.subject ?? t("mail.thread")}
          </SheetTitle>
          <SheetDescription>
            {root ? `${mailbox(root.connector_id)} → ${root.to.join(", ")}` : ""}
          </SheetDescription>
        </SheetHeader>
        {thread.error ? <p className="text-destructive text-sm">{thread.error}</p> : null}
        {thread.loading && !thread.data ? <Skeleton className="h-40" /> : null}
        {(thread.data?.thread ?? []).map((m) => (
          <div key={m.id} className={cn("rounded-lg border p-4", m.direction === "in" && "bg-muted/40")}>
            <div className="mb-3 flex flex-wrap items-center justify-between gap-2 text-xs">
              <span>
                <span className="font-medium">{m.direction === "in" ? m.from : mailbox(m.connector_id)}</span>
                <span className="text-muted-foreground"> · {when(m.sent_at || m.received_at)}</span>
              </span>
              <span className="flex items-center gap-2">
                <MailStatus status={m.status} error={m.error} />
                <Button
                  size="sm"
                  variant="ghost"
                  className="h-6 px-1.5 text-[11px]"
                  onClick={() => onReply(m)}
                >
                  <Reply className="size-3" /> {t("mail.reply")}
                </Button>
              </span>
            </div>
            {m.error ? <p className="text-destructive mb-2 text-xs">{m.error}</p> : null}
            <pre className="max-w-prose font-sans text-sm leading-relaxed whitespace-pre-wrap">{m.body}</pre>
            {m.bundle ? (
              <p className="mt-3 text-xs">
                <Archive className="mr-1 inline size-3" />
                {m.bundle}
                <span className="text-muted-foreground">
                  {" "}
                  · {t("mail.bundle_contents", { n: m.documents.length })}
                </span>
              </p>
            ) : null}
            {m.documents.length ? (
              <div className="mt-3 flex flex-wrap gap-1">
                {m.documents.map((d) => (
                  <Badge key={d.document_id} variant="secondary" className="text-[11px] font-normal">
                    <Paperclip className="mr-1 size-3" /> {d.filename}
                  </Badge>
                ))}
              </div>
            ) : null}
          </div>
        ))}
      </SheetContent>
    </Sheet>
  );
}
