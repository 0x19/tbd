"use client";

// Mail from a linked mailbox: compose from a template with the month filled
// in, attach receipts, send as the mailbox, and keep the record. A reply in
// the thread arrives with the next pull and shows under its mail. The
// service refuses a send the environment's allowlist excludes; the page
// only asks for confirmation.

import {
  Archive,
  Mail as MailIcon,
  Paperclip,
  Pencil,
  Plus,
  Reply,
  Search,
  Send,
  Trash2,
  X,
} from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { toast } from "sonner";

import { useFinance } from "@/app/providers";
import { PageTitle } from "@/components/kit";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
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
import { Sheet, SheetContent, SheetDescription, SheetHeader, SheetTitle } from "@/components/ui/sheet";
import { Skeleton } from "@/components/ui/skeleton";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Textarea } from "@/components/ui/textarea";
import { api } from "@/lib/api/client";
import { describe, useFetch } from "@/lib/api/hooks";
import type { Connector, Mail, MailTemplate } from "@/lib/api/schema";
import { monthLabel, monthLong, monthsBefore, thisMonth, when } from "@/lib/format";
import { useLang, useT } from "@/lib/i18n";
import {
  base64Utf8,
  type BundlePlan,
  helpers,
  type Prefill,
  render,
  splitAddresses,
  takePrefill,
} from "@/lib/mail-template";

type Attached = { id: string; filename: string; vendor: string };

type Draft = {
  connector_id: string;
  template_id: string;
  month: string;
  to: string;
  cc: string;
  bcc: string;
  subject: string;
  body: string;
  /** The reconciliation summary, for `{{Summary}}`. */
  summary: string;
  attachments: Attached[];
  /** The accountant's bundle, zipped by the service on send. */
  bundle: BundlePlan | null;
  in_reply_to: Mail | null;
};

const EMPTY: Draft = {
  connector_id: "",
  template_id: "",
  month: monthsBefore(thisMonth(), 1),
  to: "",
  cc: "",
  bcc: "",
  subject: "",
  body: "",
  summary: "",
  attachments: [],
  bundle: null,
  in_reply_to: null,
};

/** A template's recipients, subject and body over a draft; the summary keeps
 *  its place if the template names it, else follows the body. */
function applyTemplate(d: Draft, tpl: MailTemplate | undefined, fallbackSubject: string): Draft {
  const summaryTail = d.summary ? "{{Summary}}" : "";
  if (!tpl) {
    return { ...d, template_id: "", subject: d.subject || fallbackSubject, body: d.body || summaryTail };
  }
  const body =
    tpl.body.includes("{{Summary}}") || !summaryTail ? tpl.body : `${tpl.body.trimEnd()}\n\n${summaryTail}`;
  return {
    ...d,
    template_id: tpl.id,
    to: tpl.to.join(", "),
    cc: tpl.cc.join(", "),
    bcc: tpl.bcc.join(", "),
    subject: tpl.subject,
    body,
  };
}

export default function MailPage() {
  const t = useT();
  const { partyIds } = useFinance();
  const key = partyIds.join(",");
  const connectors = useFetch(() => api.connectors(partyIds), 0, [key]);
  const templates = useFetch(() => api.mailTemplates(partyIds), 0, [key]);
  const senders = useMemo(
    () => (connectors.data?.connectors ?? []).filter((c) => c.status === "linked" && c.can_send),
    [connectors.data],
  );
  const [tab, setTab] = useState("compose");
  const [draft, setDraft] = useState<Draft>(EMPTY);
  const [listVersion, setListVersion] = useState(0);

  // The reconciliation page's handoff, applied once both lists are here.
  const [prefill, setPrefill] = useState<Prefill | null>(null);
  useEffect(() => setPrefill(takePrefill()), []);
  useEffect(() => {
    if (!prefill || !connectors.data || !templates.data) return;
    const sender = senders.find((c) => c.party_id === prefill.party_id) ?? senders[0];
    const tpl = templates.data.templates.find((x) => x.party_id === prefill.party_id);
    setDraft(
      applyTemplate(
        {
          ...EMPTY,
          connector_id: sender?.id ?? "",
          month: prefill.month,
          summary: prefill.summary,
          bundle: prefill.bundle,
        },
        tpl,
        t("mail.default_subject"),
      ),
    );
    setTab("compose");
    toast.info(t("mail.prefilled", { month: monthLong(prefill.month), n: prefill.bundle.receipts.length }));
    setPrefill(null);
  }, [prefill, connectors.data, templates.data, senders, t]);

  const replyTo = (m: Mail) => {
    setDraft({
      ...EMPTY,
      connector_id: m.connector_id,
      to: m.direction === "in" ? m.from : m.to.join(", "),
      cc: m.cc.join(", "),
      subject: m.subject.startsWith("Re:") ? m.subject : `Re: ${m.subject}`,
      in_reply_to: m,
    });
    setTab("compose");
  };

  return (
    <>
      <PageTitle title={t("mail.title")} description={t("mail.description")} />
      <Tabs value={tab} onValueChange={setTab}>
        <TabsList>
          <TabsTrigger value="compose">{t("mail.tab.compose")}</TabsTrigger>
          <TabsTrigger value="sent">{t("mail.tab.sent")}</TabsTrigger>
          <TabsTrigger value="templates">
            {t("mail.tab.templates")} ({templates.data?.templates.length ?? 0})
          </TabsTrigger>
        </TabsList>
        <TabsContent value="compose">
          <Compose
            senders={senders}
            templates={templates.data?.templates ?? []}
            draft={draft}
            setDraft={setDraft}
            onSent={() => {
              setDraft({ ...EMPTY, connector_id: draft.connector_id });
              setListVersion((v) => v + 1);
              setTab("sent");
            }}
          />
        </TabsContent>
        <TabsContent value="sent">
          <SentList version={listVersion} connectors={connectors.data?.connectors ?? []} onReply={replyTo} />
        </TabsContent>
        <TabsContent value="templates">
          <Templates
            templates={templates.data?.templates ?? []}
            loading={templates.loading && !templates.data}
            onChanged={templates.reload}
            onUse={(tpl) => {
              setDraft((d) => applyTemplate({ ...d, in_reply_to: null }, tpl, t("mail.default_subject")));
              setTab("compose");
            }}
          />
        </TabsContent>
      </Tabs>
    </>
  );
}

function Compose({
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
  const [attaching, setAttaching] = useState(false);
  const sender = senders.find((c) => c.id === draft.connector_id) ?? senders[0];
  useEffect(() => {
    if (!draft.connector_id && senders[0]) setDraft({ ...draft, connector_id: senders[0].id });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [senders.length]);
  const months = useMemo(() => Array.from({ length: 18 }, (_, i) => monthsBefore(thisMonth(), i)), []);
  const ctx = {
    month: draft.month,
    company: sender ? partyName(sender.party_id) : "",
    lang,
    summary: draft.summary,
  };
  const subject = render(draft.subject, ctx);
  const body = render(draft.body, ctx);
  const to = splitAddresses(draft.to);
  const cc = splitAddresses(draft.cc);
  const bcc = splitAddresses(draft.bcc);
  const ready = Boolean(sender) && to.length > 0 && subject.trim().length > 0;

  const send = async () => {
    if (!sender) return;
    setBusy(true);
    try {
      const r = await api.sendMail({
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
        bundle: draft.bundle
          ? {
              filename: draft.bundle.filename,
              files: draft.bundle.files.map((f) => ({ name: f.name, bytes: base64Utf8(f.text) })),
              receipts: draft.bundle.receipts,
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
      toast.error(describe(e));
    } finally {
      setBusy(false);
      setConfirming(false);
    }
  };

  const field = (label: string, node: React.ReactNode, hint?: string) => (
    <div>
      <Label className="text-muted-foreground mb-1 block text-xs">
        {label}
        {hint ? <span className="ml-2 opacity-70">{hint}</span> : null}
      </Label>
      {node}
    </div>
  );

  return (
    <div className="grid gap-4 xl:grid-cols-[1fr_22rem]">
      <Card>
        <CardContent className="space-y-4 pt-4">
          {senders.length === 0 ? (
            <p className="text-sm text-amber-700 dark:text-amber-300">{t("mail.no_sender")}</p>
          ) : null}
          {draft.in_reply_to ? (
            <div className="bg-muted/50 flex items-center justify-between rounded-md px-3 py-2 text-xs">
              <span>{t("mail.replying_to", { subject: draft.in_reply_to.subject })}</span>
              <Button size="sm" variant="ghost" onClick={() => setDraft({ ...draft, in_reply_to: null })}>
                <X /> {t("mail.stop_replying")}
              </Button>
            </div>
          ) : null}
          <div className="grid gap-3 sm:grid-cols-3">
            {field(
              t("mail.from"),
              <Select value={sender?.id ?? ""} onValueChange={(v) => setDraft({ ...draft, connector_id: v })}>
                <SelectTrigger>
                  <SelectValue placeholder={t("mail.from_placeholder")} />
                </SelectTrigger>
                <SelectContent>
                  {senders.map((c) => (
                    <SelectItem key={c.id} value={c.id}>
                      {c.external_id || c.label} · {partyName(c.party_id)}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>,
            )}
            {field(
              t("mail.template"),
              <Select
                value={draft.template_id || "none"}
                onValueChange={(v) => {
                  const tpl = templates.find((x) => x.id === v);
                  setDraft(
                    tpl
                      ? applyTemplate(draft, tpl, t("mail.default_subject"))
                      : { ...draft, template_id: "" },
                  );
                }}
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="none">{t("mail.template_none")}</SelectItem>
                  {templates.map((tpl) => (
                    <SelectItem key={tpl.id} value={tpl.id}>
                      {tpl.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>,
            )}
            {field(
              t("mail.month"),
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
              </Select>,
            )}
          </div>
          {field(
            t("mail.to"),
            <Input
              value={draft.to}
              onChange={(e) => setDraft({ ...draft, to: e.target.value })}
              placeholder="books@accountant.hr"
            />,
            t("mail.addresses_hint"),
          )}
          <div className="grid gap-3 sm:grid-cols-2">
            {field(
              t("mail.cc"),
              <Input value={draft.cc} onChange={(e) => setDraft({ ...draft, cc: e.target.value })} />,
            )}
            {field(
              t("mail.bcc"),
              <Input value={draft.bcc} onChange={(e) => setDraft({ ...draft, bcc: e.target.value })} />,
            )}
          </div>
          {field(
            t("mail.subject"),
            <Input
              value={draft.subject}
              onChange={(e) => setDraft({ ...draft, subject: e.target.value })}
              placeholder="Računi {{MonthName}} {{Year}}"
            />,
          )}
          {field(
            t("mail.body"),
            <Textarea
              value={draft.body}
              onChange={(e) => setDraft({ ...draft, body: e.target.value })}
              rows={10}
              className="font-mono text-sm"
            />,
          )}
          <div>
            <Label className="text-muted-foreground mb-1 block text-xs">{t("mail.attachments")}</Label>
            <div className="flex flex-wrap items-center gap-2">
              {draft.attachments.length === 0 && !draft.bundle ? (
                <span className="text-muted-foreground text-xs">{t("mail.attached_none")}</span>
              ) : null}
              {draft.bundle ? (
                <Badge
                  variant="secondary"
                  className="gap-1 text-[11px]"
                  title={draft.bundle.receipts.map((r) => r.name).join("\n")}
                >
                  <Archive className="size-3" /> {draft.bundle.filename}
                  <span className="text-muted-foreground">
                    · {t("mail.bundle_contents", { n: draft.bundle.receipts.length })}
                  </span>
                  <button
                    type="button"
                    aria-label="remove"
                    onClick={() => setDraft({ ...draft, bundle: null })}
                  >
                    <X className="size-3" />
                  </button>
                </Badge>
              ) : null}
              {draft.attachments.map((d) => (
                <Badge key={d.id} variant="secondary" className="gap-1 text-[11px]">
                  <Paperclip className="size-3" /> {d.filename || d.vendor}
                  <button
                    type="button"
                    aria-label="remove"
                    onClick={() =>
                      setDraft({ ...draft, attachments: draft.attachments.filter((x) => x.id !== d.id) })
                    }
                  >
                    <X className="size-3" />
                  </button>
                </Badge>
              ))}
              <Button size="sm" variant="outline" onClick={() => setAttaching(true)} disabled={!sender}>
                <Paperclip /> {t("mail.attach")}
              </Button>
            </div>
          </div>
          <div className="flex items-center justify-end gap-2">
            <Button onClick={() => setConfirming(true)} disabled={!ready || busy}>
              <Send /> {busy ? t("mail.sending") : t("mail.send")}
            </Button>
          </div>
        </CardContent>
      </Card>

      <div className="space-y-4">
        <Card>
          <CardContent className="space-y-2 pt-4 text-xs">
            <p className="font-medium">{t("mail.helpers")}</p>
            <p className="text-muted-foreground">{t("mail.helpers_hint")}</p>
            <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1">
              {Object.entries(helpers(ctx)).map(([k, v]) => (
                <div key={k} className="contents">
                  <dt className="font-mono">{`{{${k}}}`}</dt>
                  <dd className="text-muted-foreground truncate">{k === "Summary" ? v.split("\n")[0] : v}</dd>
                </div>
              ))}
            </dl>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="space-y-2 pt-4 text-sm">
            <p className="text-muted-foreground text-xs font-medium">{t("mail.preview")}</p>
            <p className="font-medium">{subject || "—"}</p>
            <pre className="font-sans text-sm whitespace-pre-wrap">{body}</pre>
          </CardContent>
        </Card>
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
          <div className="space-y-1 text-sm">
            <p>
              <span className="text-muted-foreground">{t("mail.to")}:</span> {to.join(", ")}
            </p>
            {cc.length ? (
              <p>
                <span className="text-muted-foreground">{t("mail.cc")}:</span> {cc.join(", ")}
              </p>
            ) : null}
            {bcc.length ? (
              <p>
                <span className="text-muted-foreground">{t("mail.bcc")}:</span> {bcc.join(", ")}
              </p>
            ) : null}
            <p className="font-medium">{subject}</p>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setConfirming(false)}>
              {t("common.cancel")}
            </Button>
            <Button onClick={() => void send()} disabled={busy}>
              <Send /> {t("mail.confirm.send")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}

function AttachDialog({
  open,
  partyId,
  chosen,
  onClose,
  onPick,
}: {
  open: boolean;
  partyId: string;
  chosen: Attached[];
  onClose: () => void;
  onPick: (d: Attached) => void;
}) {
  const t = useT();
  const [q, setQ] = useState("");
  const found = useFetch(
    () => (open && partyId ? api.documents({ party_ids: [partyId], q, limit: 30 }) : Promise.resolve(null)),
    0,
    [open, q, partyId],
  );
  return (
    <Dialog open={open} onOpenChange={(o) => !o && onClose()}>
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>{t("mail.attach")}</DialogTitle>
          <DialogDescription>{t("mail.attach_search")}</DialogDescription>
        </DialogHeader>
        <div className="relative">
          <Search className="text-muted-foreground absolute top-1/2 left-2.5 size-4 -translate-y-1/2" />
          <Input value={q} onChange={(e) => setQ(e.target.value)} className="pl-8" autoFocus />
        </div>
        <div className="max-h-80 space-y-1 overflow-y-auto">
          {(found.data?.documents ?? []).map((d) => {
            const on = chosen.some((x) => x.id === d.id);
            return (
              <button
                key={d.id}
                type="button"
                className={
                  "hover:bg-muted/50 flex w-full items-center justify-between gap-2 rounded-md border px-2 py-1.5 text-left text-xs " +
                  (on ? "border-primary" : "")
                }
                onClick={() => onPick({ id: d.id, filename: d.filename, vendor: d.vendor })}
              >
                <span className="truncate">
                  <span className="font-medium">{d.vendor || d.filename}</span>
                  <span className="text-muted-foreground">
                    {" "}
                    · {d.doc_date || "—"} · {d.filename}
                  </span>
                </span>
                {on ? <Badge variant="secondary">✓</Badge> : null}
              </button>
            );
          })}
        </div>
      </DialogContent>
    </Dialog>
  );
}

function SentList({
  version,
  connectors,
  onReply,
}: {
  version: number;
  connectors: Connector[];
  onReply: (m: Mail) => void;
}) {
  const t = useT();
  const { partyIds } = useFinance();
  const [q, setQ] = useState("");
  const [direction, setDirection] = useState("");
  const [openId, setOpenId] = useState<string | null>(null);
  const list = useFetch(() => api.mail({ party_ids: partyIds, q, direction, limit: 100 }), 0, [
    partyIds.join(","),
    q,
    direction,
    version,
  ]);
  const mailbox = (id: string) => connectors.find((c) => c.id === id)?.external_id ?? "";
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
                  <TableHead className="w-36">{t("mail.list.when")}</TableHead>
                  <TableHead>{t("mail.list.who")}</TableHead>
                  <TableHead>{t("mail.list.subject")}</TableHead>
                  <TableHead>{t("mail.list.status")}</TableHead>
                  <TableHead className="text-right">{t("mail.list.replies")}</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {(list.data?.mails ?? []).map((m) => (
                  <TableRow key={m.id} className="cursor-pointer" onClick={() => setOpenId(m.id)}>
                    <TableCell className="text-xs whitespace-nowrap">
                      {when(m.sent_at || m.received_at)}
                    </TableCell>
                    <TableCell className="max-w-64 truncate text-xs">
                      {m.direction === "in" ? `← ${m.from}` : `→ ${m.to.join(", ")}`}
                    </TableCell>
                    <TableCell className="max-w-80 truncate">
                      {m.subject}
                      {m.bundle ? (
                        <span className="text-muted-foreground ml-1 text-xs" title={m.bundle}>
                          <Archive className="inline size-3" /> {m.documents.length}
                        </span>
                      ) : m.documents.length ? (
                        <span className="text-muted-foreground ml-1 text-xs">
                          <Paperclip className="inline size-3" /> {m.documents.length}
                        </span>
                      ) : null}
                    </TableCell>
                    <TableCell>
                      <Badge
                        variant="outline"
                        className={
                          m.status === "failed"
                            ? "bg-destructive/12 text-destructive border-transparent text-[10px]"
                            : "border-transparent bg-emerald-600/12 text-[10px] text-emerald-700 dark:text-emerald-300"
                        }
                        title={m.error}
                      >
                        {t(`mail.status.${m.status}`)}
                      </Badge>
                    </TableCell>
                    <TableCell className="text-right font-mono text-xs">
                      {m.direction === "out" ? m.replies : ""}
                    </TableCell>
                  </TableRow>
                ))}
                {list.data && list.data.mails.length === 0 ? (
                  <TableRow>
                    <TableCell colSpan={5} className="text-muted-foreground py-10 text-center text-sm">
                      {t("mail.list.empty")}
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

function ThreadSheet({
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
      <SheetContent className="flex w-full flex-col gap-4 overflow-y-auto sm:max-w-2xl">
        <SheetHeader>
          <SheetTitle className="flex items-center gap-2">
            <MailIcon className="size-4" /> {root?.subject ?? t("mail.thread")}
          </SheetTitle>
          <SheetDescription>
            {root ? `${mailbox(root.connector_id)} → ${root.to.join(", ")}` : ""}
          </SheetDescription>
        </SheetHeader>
        {thread.error ? <p className="text-destructive text-sm">{thread.error}</p> : null}
        {(thread.data?.thread ?? []).map((m) => (
          <div key={m.id} className={"rounded-md border p-3 " + (m.direction === "in" ? "bg-muted/40" : "")}>
            <div className="mb-2 flex flex-wrap items-center justify-between gap-2 text-xs">
              <span>
                <span className="font-medium">{m.direction === "in" ? m.from : mailbox(m.connector_id)}</span>
                <span className="text-muted-foreground"> · {when(m.sent_at || m.received_at)}</span>
              </span>
              <span className="flex items-center gap-2">
                <Badge variant="outline" className="text-[10px]">
                  {t(`mail.status.${m.status}`)}
                </Badge>
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
            <pre className="font-sans text-sm whitespace-pre-wrap">{m.body}</pre>
            {m.bundle ? (
              <p className="mt-2 text-xs">
                <Archive className="mr-1 inline size-3" />
                {m.bundle}
                <span className="text-muted-foreground">
                  {" "}
                  · {t("mail.bundle_contents", { n: m.documents.length })}
                </span>
              </p>
            ) : null}
            {m.documents.length ? (
              <div className="mt-2 flex flex-wrap gap-1">
                {m.documents.map((d) => (
                  <Badge key={d.document_id} variant="secondary" className="text-[11px]">
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

function Templates({
  templates,
  loading,
  onChanged,
  onUse,
}: {
  templates: MailTemplate[];
  loading: boolean;
  onChanged: () => void;
  onUse: (t: MailTemplate) => void;
}) {
  const t = useT();
  const { parties, partyIds } = useFinance();
  const [editing, setEditing] = useState<MailTemplate | "new" | null>(null);
  const [form, setForm] = useState({
    party_id: "",
    name: "",
    subject: "",
    body: "",
    to: "",
    cc: "",
    bcc: "",
  });
  const [busy, setBusy] = useState(false);
  const open = (tpl: MailTemplate | "new") => {
    setEditing(tpl);
    setForm(
      tpl === "new"
        ? {
            party_id: partyIds[0] ?? "",
            name: "",
            subject: "Računi {{MonthName}} {{Year}}",
            body: "",
            to: "",
            cc: "",
            bcc: "",
          }
        : {
            party_id: tpl.party_id,
            name: tpl.name,
            subject: tpl.subject,
            body: tpl.body,
            to: tpl.to.join(", "),
            cc: tpl.cc.join(", "),
            bcc: tpl.bcc.join(", "),
          },
    );
  };
  const save = async () => {
    setBusy(true);
    try {
      await api.upsertMailTemplate({
        id: editing && editing !== "new" ? editing.id : undefined,
        party_id: form.party_id,
        name: form.name,
        subject: form.subject,
        body: form.body,
        to: splitAddresses(form.to),
        cc: splitAddresses(form.cc),
        bcc: splitAddresses(form.bcc),
      });
      toast.success(t("mail.templates.saved"));
      setEditing(null);
      onChanged();
    } catch (e) {
      toast.error(describe(e));
    } finally {
      setBusy(false);
    }
  };
  const remove = async (tpl: MailTemplate) => {
    try {
      await api.deleteMailTemplate(tpl.id);
      toast.success(t("mail.templates.deleted"));
      onChanged();
    } catch (e) {
      toast.error(describe(e));
    }
  };
  return (
    <div className="space-y-3">
      <div className="flex justify-end">
        <Button size="sm" onClick={() => open("new")}>
          <Plus /> {t("mail.templates.new")}
        </Button>
      </div>
      {loading ? <Skeleton className="h-32 w-full" /> : null}
      {!loading && templates.length === 0 ? (
        <p className="text-muted-foreground py-6 text-center text-sm">{t("mail.templates.empty")}</p>
      ) : null}
      <div className="grid gap-3 md:grid-cols-2">
        {templates.map((tpl) => (
          <Card key={tpl.id}>
            <CardContent className="space-y-2 pt-4">
              <div className="flex items-start justify-between gap-2">
                <div className="min-w-0">
                  <p className="truncate font-medium">{tpl.name}</p>
                  <p className="text-muted-foreground truncate text-xs">{tpl.subject}</p>
                </div>
                <div className="flex gap-1">
                  <Button
                    size="icon-sm"
                    variant="ghost"
                    onClick={() => open(tpl)}
                    aria-label={t("mail.templates.edit")}
                  >
                    <Pencil />
                  </Button>
                  <Button
                    size="icon-sm"
                    variant="ghost"
                    onClick={() => void remove(tpl)}
                    aria-label={t("mail.templates.delete")}
                  >
                    <Trash2 />
                  </Button>
                </div>
              </div>
              <pre className="text-muted-foreground max-h-24 overflow-hidden font-sans text-xs whitespace-pre-wrap">
                {tpl.body}
              </pre>
              <div className="flex items-center justify-between text-xs">
                <span className="text-muted-foreground truncate">
                  {t("mail.templates.default_to")}: {tpl.to.join(", ") || "—"}
                </span>
                <Button size="sm" variant="outline" onClick={() => onUse(tpl)}>
                  {t("mail.templates.use")}
                </Button>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>
      <Dialog open={editing !== null} onOpenChange={(o) => !o && setEditing(null)}>
        <DialogContent className="sm:max-w-lg">
          <DialogHeader>
            <DialogTitle>
              {editing === "new" ? t("mail.templates.new") : t("mail.templates.edit")}
            </DialogTitle>
            <DialogDescription>{t("mail.helpers_hint")}</DialogDescription>
          </DialogHeader>
          <div className="grid gap-3">
            {editing === "new" && parties.length > 1 ? (
              <Select value={form.party_id} onValueChange={(v) => setForm({ ...form, party_id: v })}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {parties.map((p) => (
                    <SelectItem key={p.id} value={p.id}>
                      {p.display_name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            ) : null}
            <Input
              value={form.name}
              onChange={(e) => setForm({ ...form, name: e.target.value })}
              placeholder={t("mail.templates.name")}
            />
            <Input
              value={form.subject}
              onChange={(e) => setForm({ ...form, subject: e.target.value })}
              placeholder={t("mail.subject")}
            />
            <Textarea
              value={form.body}
              onChange={(e) => setForm({ ...form, body: e.target.value })}
              rows={8}
              placeholder={t("mail.body")}
              className="font-mono text-sm"
            />
            <Input
              value={form.to}
              onChange={(e) => setForm({ ...form, to: e.target.value })}
              placeholder={`${t("mail.to")} · ${t("mail.addresses_hint")}`}
            />
            <div className="grid grid-cols-2 gap-3">
              <Input
                value={form.cc}
                onChange={(e) => setForm({ ...form, cc: e.target.value })}
                placeholder={t("mail.cc")}
              />
              <Input
                value={form.bcc}
                onChange={(e) => setForm({ ...form, bcc: e.target.value })}
                placeholder={t("mail.bcc")}
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setEditing(null)}>
              {t("common.cancel")}
            </Button>
            <Button onClick={() => void save()} disabled={busy || !form.name.trim() || !form.party_id}>
              {busy ? t("common.saving") : t("common.save")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
