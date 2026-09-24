"use client";

// The mail as the recipient will see it: a header block, the body in a
// readable measure, the attachments and what the bundle holds. Everything
// here is the rendered result, helpers filled; the composer edits the
// source. On a wide screen it stands beside the form at equal width; below
// that it is the second tab.
import { Archive, FileText, Paperclip } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/components/ui/collapsible";
import { useT } from "@/lib/i18n";
import type { BundlePlan } from "@/lib/mail-template";
import { cn } from "@/lib/utils";

import type { Attached } from "./draft";

export function MailPreview({
  from,
  to,
  cc,
  bcc,
  subject,
  body,
  attachments,
  bundle,
  className,
}: {
  from: string;
  to: string[];
  cc: string[];
  bcc: string[];
  subject: string;
  body: string;
  attachments: Attached[];
  bundle: BundlePlan | null;
  className?: string;
}) {
  const t = useT();
  const empty = !subject && !body.trim();
  const row = (label: string, value: string) =>
    value ? (
      <div className="contents">
        <dt className="text-muted-foreground text-xs">{label}</dt>
        <dd className="min-w-0 truncate text-xs" title={value}>
          {value}
        </dd>
      </div>
    ) : null;
  return (
    <div className={cn("bg-card flex min-h-[32rem] flex-col rounded-xl border", className)}>
      <div className="border-b px-5 py-4">
        <p className="text-muted-foreground mb-2 text-[10px] font-medium tracking-wide uppercase">
          {t("mail.preview")}
        </p>
        <h2
          className={cn("text-base leading-snug font-semibold", !subject && "text-muted-foreground italic")}
        >
          {subject || t("mail.preview.no_subject")}
        </h2>
        <dl className="mt-3 grid grid-cols-[4rem_1fr] gap-x-3 gap-y-1">
          {row(t("mail.from"), from)}
          {row(t("mail.to"), to.join(", "))}
          {row(t("mail.cc"), cc.join(", "))}
          {row(t("mail.bcc"), bcc.join(", "))}
        </dl>
      </div>
      <div className="flex-1 px-5 py-5">
        {empty ? (
          <p className="text-muted-foreground text-sm italic">{t("mail.preview.empty")}</p>
        ) : (
          <pre className="max-w-prose font-sans text-[15px] leading-relaxed whitespace-pre-wrap">{body}</pre>
        )}
      </div>
      {attachments.length || bundle ? (
        <div className="bg-muted/30 space-y-2 rounded-b-xl border-t px-5 py-3">
          <p className="text-muted-foreground text-[10px] font-medium tracking-wide uppercase">
            {t("mail.attachments")} · {attachments.length + (bundle ? 1 : 0)}
          </p>
          <ul className="space-y-1">
            {attachments.map((a) => (
              <li key={a.id} className="flex items-center gap-2 text-xs">
                <FileText className="text-muted-foreground size-3.5 shrink-0" />
                <span className="truncate">{a.filename || a.vendor}</span>
              </li>
            ))}
            {bundle ? (
              <li>
                <Collapsible>
                  <CollapsibleTrigger className="flex items-center gap-2 text-left text-xs hover:underline">
                    <Archive className="text-muted-foreground size-3.5 shrink-0" />
                    <span className="truncate">{bundle.filename}</span>
                    <Badge variant="secondary" className="text-[10px]">
                      {bundle.files.length + bundle.receipts.length}
                    </Badge>
                  </CollapsibleTrigger>
                  <CollapsibleContent>
                    <ul className="text-muted-foreground mt-1 ml-6 max-h-40 space-y-0.5 overflow-y-auto font-mono text-[11px]">
                      {bundle.files.map((f) => (
                        <li key={f.name}>{f.name}</li>
                      ))}
                      {bundle.receipts.map((r) => (
                        <li key={r.document_id} className="flex items-center gap-1">
                          <Paperclip className="size-3" /> {r.name}
                        </li>
                      ))}
                    </ul>
                  </CollapsibleContent>
                </Collapsible>
              </li>
            ) : null}
          </ul>
        </div>
      ) : null}
    </div>
  );
}
