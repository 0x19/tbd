"use client";

// The composer: the form and the preview side by side at equal width on a
// wide screen, as two tabs below that. The form edits the draft; the
// preview shows the rendered mail; the send bar says what is missing
// before the button is enabled, and the confirm dialog restates the
// recipients and attachments once more. The bundle and the summary follow
// the language: rebuilt from the rows whenever the translator changes.
import { Archive, Eye, FileText, Paperclip, PenLine, Send, Trash2, X } from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";
import { toast } from "sonner";

import { useFinance } from "@/app/providers";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Textarea } from "@/components/ui/textarea";
import { api, ApiError } from "@/lib/api/client";
import { describe } from "@/lib/api/hooks";
import type { Connector, MailTemplate } from "@/lib/api/schema";
import { plan, summaryText } from "@/lib/bundle";
import { monthLabel, monthsBefore, thisMonth } from "@/lib/format";
import { useLang, useT } from "@/lib/i18n";
import {
  base64Utf8,
  looksLikeAddress,
  render,
  splitAddresses,
  type TemplateContext,
} from "@/lib/mail-template";
import { cn } from "@/lib/utils";

import { AttachDialog } from "./attach-dialog";
import { applyTemplate, type Draft, EMPTY, insertAt } from "./draft";
import { HelpersMenu } from "./helpers-menu";
import { MailPreview } from "./preview";

export function Composer({
  senders,
  templates,
  draft,
  setDraft,
  onSent,
}: {
  senders: Connector[];
  templates: MailTemplate[];
  draft: Draft;
  setDraft: (d: Draft) => void;
  onSent: () => void;
}) {
  const t = useT();
  const { lang } = useLang();
  const { partyName } = useFinance();
  const [busy, setBusy] = useState(false);
  const [confirming, setConfirming] = useState(false);
  // The service refused because the invoice went out before: its words, and
  // the person's say-so to send once more.
  const [resend, setResend] = useState<string | null>(null);
  const [attaching, setAttaching] = useState(false);
  const [pane, setPane] = useState<"write" | "preview">("write");
  const [showCc, setShowCc] = useState(false);
  const [showBcc, setShowBcc] = useState(false);
  const bodyRef = useRef<HTMLTextAreaElement>(null);
  const subjectRef = useRef<HTMLInputElement>(null);
  const [lastFocus, setLastFocus] = useState<"subject" | "body">("body");

  const sender = senders.find((c) => c.id === draft.connector_id) ?? senders[0];
  useEffect(() => {
    if (!draft.connector_id && senders[0]) setDraft({ ...draft, connector_id: senders[0].id });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [senders.length]);
  const months = useMemo(() => Array.from({ length: 18 }, (_, i) => monthsBefore(thisMonth(), i)), []);
  const own = useMemo(
    () => templates.filter((x) => !sender || x.party_id === sender.party_id),
    [templates, sender],
  );

  const source = draft.source;
  const summaryNow = useMemo(() => (source?.kind === "month" ? summaryText(t, source) : ""), [t, source]);
  const bundleNow = useMemo(
    () => (source?.kind === "month" && draft.with_bundle ? plan(t, source) : null),
    [t, source, draft.with_bundle],
  );
  const ctx: TemplateContext = {
    month: draft.month,
    company: sender ? partyName(sender.party_id) : "",
    lang,
    summary: summaryNow,
    invoice:
      source?.kind === "invoice"
        ? {
            number: source.invoice.number,
            total: source.invoice.total,
            issued_at: source.invoice.issued_at,
            due_date: source.invoice.due_date,
            client: source.client.name,
          }
        : undefined,
  };
  const subjectRaw =
    draft.subject ||
    (source?.kind === "month"
      ? t("mail.default_subject")
      : source?.kind === "invoice"
        ? t("mail.default_subject_invoice")
        : "");
  const subject = render(subjectRaw, ctx);
  const body = render(draft.body, ctx);
  const to = splitAddresses(draft.to);
  const cc = splitAddresses(draft.cc);
  const bcc = splitAddresses(draft.bcc);
  const bad = [...to, ...cc, ...bcc].filter((a) => !looksLikeAddress(a));
  const missing: string[] = [];
  if (!sender) missing.push(t("mail.needs.sender"));
  if (to.length === 0) missing.push(t("mail.needs.to"));
  if (bad.length) missing.push(t("mail.needs.address", { address: bad[0] ?? "" }));
  if (!subject.trim()) missing.push(t("mail.needs.subject"));
  if (!body.trim()) missing.push(t("mail.needs.body"));
  const ready = missing.length === 0;
  const nAttachments = draft.attachments.length + (bundleNow ? 1 : 0);

  const insert = (token: string) => {
    if (lastFocus === "subject") {
      const el = subjectRef.current;
      const r = insertAt(subjectRaw, el?.selectionStart ?? null, token);
      setDraft({ ...draft, subject: r.text });
      requestAnimationFrame(() => el?.setSelectionRange(r.caret, r.caret));
    } else {
      const el = bodyRef.current;
      const r = insertAt(draft.body, el?.selectionStart ?? null, token);
      setDraft({ ...draft, body: r.text });
      requestAnimationFrame(() => {
        el?.focus();
        el?.setSelectionRange(r.caret, r.caret);
      });
    }
  };

  const send = async (force = false) => {
    if (!sender) return;
    setBusy(true);
    try {
      const r = await api.sendMail({
        invoice_id: draft.source?.kind === "invoice" ? draft.source.invoice.id : "",
        force,
        connector_id: sender.id,
        template_id: draft.template_id,
        to,
        cc,
        bcc,
        subject,
        body,
        html: "",
        attachment_document_ids: draft.attachments.map((d) => d.id),
        in_reply_to_mail_id: draft.in_reply_to?.id ?? "",
        bundle: bundleNow
          ? {
              filename: bundleNow.filename,
              files: bundleNow.files.map((f) => ({ name: f.name, bytes: base64Utf8(f.text) })),
              receipts: bundleNow.receipts,
            }
          : undefined,
      });
      if (r.mail?.status === "sent") {
        toast.success(t("mail.sent"));
        onSent();
      } else {
        toast.error(t("mail.failed", { error: r.mail?.error ?? "" }));
      }
    } catch (e) {
      if (e instanceof ApiError && e.code === "failed_precondition" && e.message.includes("already sent")) {
        setResend(e.message);
      } else {
        toast.error(describe(e));
      }
    } finally {
      setBusy(false);
      setConfirming(false);
    }
  };

  const label = (text: string, extra?: React.ReactNode) => (
    <div className="mb-1 flex items-center justify-between">
      <Label className="text-muted-foreground text-xs">{text}</Label>
      {extra}
    </div>
  );

  const preview = (
    <MailPreview
      from={sender ? `${sender.external_id || sender.label} · ${partyName(sender.party_id)}` : ""}
      to={to}
      cc={cc}
      bcc={bcc}
      subject={subject}
      body={body}
      attachments={draft.attachments}
      bundle={bundleNow}
      className="xl:sticky xl:top-20"
    />
  );

  return (
    <div className="space-y-4">
      {senders.length === 0 ? (
        <div className="rounded-lg border border-amber-500/40 bg-amber-500/10 px-4 py-3 text-sm text-amber-800 dark:text-amber-200">
          {t("mail.no_sender")}
        </div>
      ) : null}
      {source ? (
        <div className="bg-muted/50 flex flex-wrap items-center justify-between gap-2 rounded-lg px-4 py-2.5 text-sm">
          <span className="flex items-center gap-2">
            {source.kind === "invoice" ? <FileText className="size-4" /> : <Archive className="size-4" />}
            {source.kind === "invoice"
              ? t("mail.source.invoice", { number: source.invoice.number, client: source.client.name })
              : t("mail.source.month", { month: monthLabel(source.month), company: source.company })}
          </span>
          <Button
            size="sm"
            variant="ghost"
            onClick={() => setDraft({ ...EMPTY, connector_id: draft.connector_id })}
          >
            <X /> {t("mail.discard")}
          </Button>
        </div>
      ) : null}
      {draft.in_reply_to ? (
        <div className="bg-muted/50 flex items-center justify-between rounded-lg px-4 py-2.5 text-sm">
          <span>{t("mail.replying_to", { subject: draft.in_reply_to.subject })}</span>
          <Button size="sm" variant="ghost" onClick={() => setDraft({ ...draft, in_reply_to: null })}>
            <X /> {t("mail.stop_replying")}
          </Button>
        </div>
      ) : null}

      <Tabs value={pane} onValueChange={(v) => setPane(v as "write" | "preview")} className="xl:hidden">
        <TabsList className="grid w-full grid-cols-2">
          <TabsTrigger value="write">
            <PenLine className="mr-1.5 size-3.5" /> {t("mail.write")}
          </TabsTrigger>
          <TabsTrigger value="preview">
            <Eye className="mr-1.5 size-3.5" /> {t("mail.preview")}
          </TabsTrigger>
        </TabsList>
      </Tabs>

      <div className="grid items-start gap-6 xl:grid-cols-2">
        <div
          className={cn("bg-card space-y-5 rounded-xl border p-5", pane === "preview" && "hidden xl:block")}
        >
          <div className="grid gap-4 sm:grid-cols-3">
            <div>
              {label(t("mail.from"))}
              <Select value={sender?.id ?? ""} onValueChange={(v) => setDraft({ ...draft, connector_id: v })}>
                <SelectTrigger>
                  <SelectValue placeholder={t("mail.from_placeholder")} />
                </SelectTrigger>
                <SelectContent>
                  {senders.map((c) => (
                    <SelectItem key={c.id} value={c.id}>
                      {c.external_id || c.label} · {partyName(c.party_id)}
                      {c.can_read ? "" : ` · ${t("mail.send_only")}`}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div>
              {label(t("mail.template"))}
              <Select
                value={draft.template_id || "none"}
                onValueChange={(v) => {
                  const tpl = own.find((x) => x.id === v);
                  setDraft(tpl ? applyTemplate(draft, tpl) : { ...draft, template_id: "" });
                }}
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="none">{t("mail.template_none")}</SelectItem>
                  {own.map((tpl) => (
                    <SelectItem key={tpl.id} value={tpl.id}>
                      {tpl.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div>
              {label(t("mail.month"))}
              <Select value={draft.month} onValueChange={(v) => setDraft({ ...draft, month: v })}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {months.map((m) => (
                    <SelectItem key={m} value={m}>
                      {monthLabel(m)}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </div>

          <div className="space-y-3">
            <div>
              {label(
                t("mail.to"),
                <span className="flex gap-1">
                  {!showCc && !cc.length ? (
                    <Button
                      size="sm"
                      variant="ghost"
                      className="h-6 px-1.5 text-xs"
                      onClick={() => setShowCc(true)}
                    >
                      {t("mail.cc")}
                    </Button>
                  ) : null}
                  {!showBcc && !bcc.length ? (
                    <Button
                      size="sm"
                      variant="ghost"
                      className="h-6 px-1.5 text-xs"
                      onClick={() => setShowBcc(true)}
                    >
                      {t("mail.bcc")}
                    </Button>
                  ) : null}
                </span>,
              )}
              <Input
                value={draft.to}
                onChange={(e) => setDraft({ ...draft, to: e.target.value })}
                placeholder={t("mail.to_placeholder")}
                aria-invalid={to.some((a) => !looksLikeAddress(a)) || undefined}
              />
              <AddressChips addresses={to} />
            </div>
            {showCc || cc.length ? (
              <div>
                {label(t("mail.cc"))}
                <Input value={draft.cc} onChange={(e) => setDraft({ ...draft, cc: e.target.value })} />
                <AddressChips addresses={cc} />
              </div>
            ) : null}
            {showBcc || bcc.length ? (
              <div>
                {label(t("mail.bcc"))}
                <Input value={draft.bcc} onChange={(e) => setDraft({ ...draft, bcc: e.target.value })} />
                <AddressChips addresses={bcc} />
              </div>
            ) : null}
          </div>

          <div>
            {label(t("mail.subject"))}
            <Input
              ref={subjectRef}
              value={subjectRaw}
              onFocus={() => setLastFocus("subject")}
              onChange={(e) => setDraft({ ...draft, subject: e.target.value })}
              placeholder={t("mail.subject_placeholder")}
            />
            {subject !== subjectRaw ? (
              <p className="text-muted-foreground mt-1 truncate text-xs">
                {t("mail.subject_becomes")} <span className="text-foreground">{subject}</span>
              </p>
            ) : null}
          </div>

          <div>
            {label(t("mail.body"), <HelpersMenu ctx={ctx} onInsert={insert} />)}
            <Textarea
              ref={bodyRef}
              value={draft.body}
              onFocus={() => setLastFocus("body")}
              onChange={(e) => setDraft({ ...draft, body: e.target.value })}
              rows={14}
              className="min-h-[18rem] font-mono text-sm leading-relaxed"
              placeholder={t("mail.body_placeholder")}
            />
          </div>

          <div>
            {label(
              t("mail.attachments"),
              <Button
                size="sm"
                variant="ghost"
                className="h-7 gap-1 px-2 text-xs"
                onClick={() => setAttaching(true)}
                disabled={!sender}
              >
                <Paperclip className="size-3.5" /> {t("mail.attach")}
              </Button>,
            )}
            <div className="flex flex-wrap items-center gap-2">
              {nAttachments === 0 ? (
                <span className="text-muted-foreground text-xs">{t("mail.attached_none")}</span>
              ) : null}
              {bundleNow ? (
                <Chip
                  icon={<Archive className="size-3.5" />}
                  text={bundleNow.filename}
                  hint={t("mail.bundle_contents", { n: bundleNow.receipts.length })}
                  onRemove={() => setDraft({ ...draft, with_bundle: false })}
                />
              ) : null}
              {draft.attachments.map((d) => (
                <Chip
                  key={d.id}
                  icon={<FileText className="size-3.5" />}
                  text={d.filename || d.vendor}
                  onRemove={() =>
                    setDraft({ ...draft, attachments: draft.attachments.filter((x) => x.id !== d.id) })
                  }
                />
              ))}
            </div>
          </div>

          <div className="flex flex-wrap items-center justify-between gap-3 border-t pt-4">
            <p
              className={cn(
                "text-xs",
                ready ? "text-muted-foreground" : "text-amber-700 dark:text-amber-300",
              )}
            >
              {ready
                ? t("mail.ready", { n: to.length + cc.length + bcc.length, a: nAttachments })
                : missing[0]}
            </p>
            <div className="flex items-center gap-2">
              <Button
                variant="ghost"
                size="sm"
                onClick={() => setDraft({ ...EMPTY, connector_id: draft.connector_id })}
                disabled={busy}
              >
                <Trash2 /> {t("mail.discard")}
              </Button>
              <Button onClick={() => setConfirming(true)} disabled={!ready || busy}>
                <Send /> {busy ? t("mail.sending") : t("mail.send")}
              </Button>
            </div>
          </div>
        </div>

        <div className={cn(pane === "write" && "hidden xl:block")}>{preview}</div>
      </div>

      <AttachDialog
        open={attaching}
        partyId={sender?.party_id ?? ""}
        chosen={draft.attachments}
        onClose={() => setAttaching(false)}
        onPick={(d) =>
          setDraft({
            ...draft,
            attachments: draft.attachments.some((x) => x.id === d.id)
              ? draft.attachments.filter((x) => x.id !== d.id)
              : [...draft.attachments, d],
          })
        }
      />
      <Dialog open={resend !== null} onOpenChange={(o) => !o && setResend(null)}>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle>{t("mail.resend.title")}</DialogTitle>
            <DialogDescription>{resend}</DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setResend(null)}>
              {t("common.cancel")}
            </Button>
            <Button
              onClick={() => {
                setResend(null);
                void send(true);
              }}
              disabled={busy}
            >
              <Send /> {t("mail.resend.confirm")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
      <Dialog open={confirming} onOpenChange={(o) => !o && setConfirming(false)}>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle>{t("mail.confirm.title")}</DialogTitle>
            <DialogDescription>
              {t("mail.confirm.description", {
                from: sender?.external_id ?? "",
                n: to.length + cc.length + bcc.length,
              })}
            </DialogDescription>
          </DialogHeader>
          <dl className="grid grid-cols-[5rem_1fr] gap-x-3 gap-y-1.5 text-sm">
            <dt className="text-muted-foreground">{t("mail.to")}</dt>
            <dd className="break-words">{to.join(", ")}</dd>
            {cc.length ? (
              <>
                <dt className="text-muted-foreground">{t("mail.cc")}</dt>
                <dd className="break-words">{cc.join(", ")}</dd>
              </>
            ) : null}
            {bcc.length ? (
              <>
                <dt className="text-muted-foreground">{t("mail.bcc")}</dt>
                <dd className="break-words">{bcc.join(", ")}</dd>
              </>
            ) : null}
            <dt className="text-muted-foreground">{t("mail.subject")}</dt>
            <dd className="font-medium">{subject}</dd>
            <dt className="text-muted-foreground">{t("mail.attachments")}</dt>
            <dd>
              {nAttachments === 0
                ? t("mail.attached_none")
                : [bundleNow?.filename, ...draft.attachments.map((a) => a.filename || a.vendor)]
                    .filter(Boolean)
                    .join(", ")}
            </dd>
          </dl>
          <DialogFooter>
            <Button variant="outline" onClick={() => setConfirming(false)}>
              {t("common.cancel")}
            </Button>
            <Button onClick={() => void send()} disabled={busy}>
              <Send /> {busy ? t("mail.sending") : t("mail.confirm.send")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}

/** The addresses as parsed, a bad one marked, so a typo shows before the
 *  service refuses it. */
function AddressChips({ addresses }: { addresses: string[] }) {
  if (addresses.length < 2 && addresses.every(looksLikeAddress)) return null;
  return (
    <div className="mt-1.5 flex flex-wrap gap-1">
      {addresses.map((a, i) => (
        <Badge
          key={`${a}-${i}`}
          variant="outline"
          className={cn(
            "text-[11px] font-normal",
            !looksLikeAddress(a) && "border-destructive text-destructive",
          )}
        >
          {a}
        </Badge>
      ))}
    </div>
  );
}

function Chip({
  icon,
  text,
  hint,
  onRemove,
}: {
  icon: React.ReactNode;
  text: string;
  hint?: string;
  onRemove: () => void;
}) {
  return (
    <Badge variant="secondary" className="h-7 max-w-full gap-1.5 pr-1 pl-2 text-xs font-normal" title={hint}>
      {icon}
      <span className="truncate">{text}</span>
      {hint ? <span className="text-muted-foreground hidden truncate sm:inline">· {hint}</span> : null}
      <button
        type="button"
        aria-label="remove"
        className="hover:bg-muted-foreground/20 ml-0.5 rounded-sm p-0.5"
        onClick={onRemove}
      >
        <X className="size-3" />
      </button>
    </Badge>
  );
}
