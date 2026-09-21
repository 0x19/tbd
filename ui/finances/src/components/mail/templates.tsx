"use client";

// Templates: a card each, and an editor with the same live preview as the
// composer, filled with sample values, so a helper typo shows while typing.
import { Pencil, Plus, Trash2 } from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";

import { useFinance } from "@/app/providers";
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
import { Skeleton } from "@/components/ui/skeleton";
import { Textarea } from "@/components/ui/textarea";
import { api } from "@/lib/api/client";
import { describe } from "@/lib/api/hooks";
import type { MailTemplate } from "@/lib/api/schema";
import { monthsBefore, thisMonth } from "@/lib/format";
import { useLang, useT } from "@/lib/i18n";
import { render, splitAddresses, type TemplateContext } from "@/lib/mail-template";

import { insertAt } from "./draft";
import { HelpersMenu } from "./helpers-menu";
import { MailPreview } from "./preview";

type Form = {
  party_id: string;
  name: string;
  subject: string;
  body: string;
  to: string;
  cc: string;
  bcc: string;
};

export function Templates({
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
  const { lang } = useLang();
  const { parties, partyIds, partyName } = useFinance();
  const [editing, setEditing] = useState<MailTemplate | "new" | null>(null);
  const [form, setForm] = useState<Form>({
    party_id: "",
    name: "",
    subject: "",
    body: "",
    to: "",
    cc: "",
    bcc: "",
  });
  const [busy, setBusy] = useState(false);
  const [removing, setRemoving] = useState<MailTemplate | null>(null);

  const open = (tpl: MailTemplate | "new") => {
    setEditing(tpl);
    setForm(
      tpl === "new"
        ? {
            party_id: partyIds[0] ?? "",
            name: "",
            subject: t("mail.default_subject"),
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
    } finally {
      setRemoving(null);
    }
  };

  // Sample values: last month, this party, and an invoice so the invoice
  // helpers show what they would become.
  const sample: TemplateContext = {
    month: monthsBefore(thisMonth(), 1),
    company: partyName(form.party_id),
    lang,
    summary: t("mail.templates.sample_summary"),
    invoice: {
      number: "2026-000042",
      total: "1.250,00 €",
      issued_at: `${thisMonth()}-01`,
      due_date: `${thisMonth()}-15`,
      client: t("mail.templates.sample_client"),
      outstanding: "1.250,00 €",
      days_overdue: 12,
    },
  };
  const [lastFocus, setLastFocus] = useState<"subject" | "body">("body");
  const insert = (token: string) => {
    if (lastFocus === "subject") setForm({ ...form, subject: insertAt(form.subject, null, token).text });
    else setForm({ ...form, body: insertAt(form.body, null, token).text });
  };
  const field = (label: string, node: React.ReactNode, extra?: React.ReactNode) => (
    <div>
      <div className="mb-1 flex items-center justify-between">
        <Label className="text-muted-foreground text-xs">{label}</Label>
        {extra}
      </div>
      {node}
    </div>
  );

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between">
        <p className="text-muted-foreground text-sm">{t("mail.templates.intro")}</p>
        <Button size="sm" onClick={() => open("new")}>
          <Plus /> {t("mail.templates.new")}
        </Button>
      </div>
      {loading ? <Skeleton className="h-32 w-full" /> : null}
      {!loading && templates.length === 0 ? (
        <p className="text-muted-foreground py-10 text-center text-sm">{t("mail.templates.empty")}</p>
      ) : null}
      <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-3">
        {templates.map((tpl) => (
          <Card key={tpl.id} className="flex flex-col">
            <CardContent className="flex flex-1 flex-col gap-3 pt-4">
              <div className="flex items-start justify-between gap-2">
                <div className="min-w-0">
                  <p className="truncate font-medium">{tpl.name}</p>
                  <p className="text-muted-foreground truncate text-xs">{tpl.subject}</p>
                  {parties.length > 1 ? (
                    <p className="text-muted-foreground truncate text-[11px]">{partyName(tpl.party_id)}</p>
                  ) : null}
                </div>
                <div className="flex shrink-0 gap-1">
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
                    onClick={() => setRemoving(tpl)}
                    aria-label={t("mail.templates.delete")}
                  >
                    <Trash2 />
                  </Button>
                </div>
              </div>
              <pre className="text-muted-foreground max-h-28 flex-1 overflow-hidden font-sans text-xs whitespace-pre-wrap">
                {tpl.body || t("mail.templates.no_body")}
              </pre>
              <div className="flex items-center justify-between gap-2 border-t pt-3 text-xs">
                <span className="text-muted-foreground truncate" title={tpl.to.join(", ")}>
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
        <DialogContent className="max-h-[92vh] overflow-y-auto sm:max-w-5xl">
          <DialogHeader>
            <DialogTitle>
              {editing === "new" ? t("mail.templates.new") : t("mail.templates.edit")}
            </DialogTitle>
            <DialogDescription>{t("mail.templates.editor_hint")}</DialogDescription>
          </DialogHeader>
          <div className="grid gap-6 lg:grid-cols-2">
            <div className="space-y-4">
              <div className="grid gap-4 sm:grid-cols-2">
                {field(
                  t("mail.templates.name"),
                  <Input
                    value={form.name}
                    onChange={(e) => setForm({ ...form, name: e.target.value })}
                    autoFocus
                  />,
                )}
                {parties.length > 1
                  ? field(
                      t("mail.templates.party"),
                      <Select
                        value={form.party_id}
                        onValueChange={(v) => setForm({ ...form, party_id: v })}
                        disabled={editing !== "new"}
                      >
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
                      </Select>,
                    )
                  : null}
              </div>
              {field(
                t("mail.subject"),
                <Input
                  value={form.subject}
                  onFocus={() => setLastFocus("subject")}
                  onChange={(e) => setForm({ ...form, subject: e.target.value })}
                />,
              )}
              {field(
                t("mail.body"),
                <Textarea
                  value={form.body}
                  onFocus={() => setLastFocus("body")}
                  onChange={(e) => setForm({ ...form, body: e.target.value })}
                  rows={12}
                  className="min-h-[14rem] font-mono text-sm leading-relaxed"
                />,
                <HelpersMenu ctx={sample} onInsert={insert} />,
              )}
              {field(
                t("mail.templates.default_to"),
                <Input
                  value={form.to}
                  onChange={(e) => setForm({ ...form, to: e.target.value })}
                  placeholder={t("mail.addresses_hint")}
                />,
              )}
              <div className="grid gap-4 sm:grid-cols-2">
                {field(
                  t("mail.cc"),
                  <Input value={form.cc} onChange={(e) => setForm({ ...form, cc: e.target.value })} />,
                )}
                {field(
                  t("mail.bcc"),
                  <Input value={form.bcc} onChange={(e) => setForm({ ...form, bcc: e.target.value })} />,
                )}
              </div>
            </div>
            <MailPreview
              from={partyName(form.party_id)}
              to={splitAddresses(form.to)}
              cc={splitAddresses(form.cc)}
              bcc={splitAddresses(form.bcc)}
              subject={render(form.subject, sample)}
              body={render(form.body, sample)}
              attachments={[]}
              bundle={null}
              className="min-h-[24rem]"
            />
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

      <Dialog open={removing !== null} onOpenChange={(o) => !o && setRemoving(null)}>
        <DialogContent className="sm:max-w-sm">
          <DialogHeader>
            <DialogTitle>{t("mail.templates.delete_title", { name: removing?.name ?? "" })}</DialogTitle>
            <DialogDescription>{t("mail.templates.delete_hint")}</DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setRemoving(null)}>
              {t("common.cancel")}
            </Button>
            <Button variant="destructive" onClick={() => removing && void remove(removing)}>
              {t("mail.templates.delete")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
