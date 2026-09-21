"use client";

// Mail from a linked mailbox: compose from a template with the month, the
// invoice or the accountant's bundle filled in, send as the mailbox, and
// keep the record. A reply in the thread arrives with the next pull and
// shows under its mail. The service refuses a send the environment's
// allowlist excludes; the page only asks for confirmation. The pieces live
// in src/components/mail/; this file is the orchestration: which tab, one
// draft, and the hand-offs from the reconciliation and invoices pages.
import { useEffect, useMemo, useState } from "react";
import { toast } from "sonner";

import { useFinance } from "@/app/providers";
import { PageTitle } from "@/components/kit";
import { Composer } from "@/components/mail/composer";
import { applyTemplate, type Draft, EMPTY, fromPrefill, replyDraft } from "@/components/mail/draft";
import { SentList } from "@/components/mail/sent-list";
import { Templates } from "@/components/mail/templates";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { api } from "@/lib/api/client";
import { useFetch } from "@/lib/api/hooks";
import { plan } from "@/lib/bundle";
import { monthLong } from "@/lib/format";
import { useT } from "@/lib/i18n";
import { type Prefill, takePrefill } from "@/lib/mail-template";

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

  // Another page's hand-off, applied once both lists are here: that party's
  // sender and first template, the source's recipients and attachments.
  const [prefill, setPrefill] = useState<Prefill | null>(null);
  useEffect(() => setPrefill(takePrefill()), []);
  useEffect(() => {
    if (!prefill || !connectors.data || !templates.data) return;
    const sender = senders.find((c) => c.party_id === prefill.party_id) ?? senders[0];
    const tpl = templates.data.templates.find((x) => x.party_id === prefill.party_id);
    setDraft(applyTemplate(fromPrefill(prefill, sender?.id ?? ""), tpl));
    setTab("compose");
    if (prefill.kind === "invoice") {
      toast.info(
        prefill.client.recipients.length
          ? t("mail.prefilled_invoice", { number: prefill.invoice.number, client: prefill.client.name })
          : t("mail.prefilled_invoice_no_address", {
              number: prefill.invoice.number,
              client: prefill.client.name,
            }),
      );
    } else {
      toast.info(
        t("mail.prefilled", { month: monthLong(prefill.month), n: plan(t, prefill).receipts.length }),
      );
    }
    setPrefill(null);
  }, [prefill, connectors.data, templates.data, senders, t]);

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
        <TabsContent value="compose" className="mt-4">
          <Composer
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
        <TabsContent value="sent" className="mt-4">
          <SentList
            version={listVersion}
            connectors={connectors.data?.connectors ?? []}
            onReply={(m) => {
              setDraft(replyDraft(m));
              setTab("compose");
            }}
          />
        </TabsContent>
        <TabsContent value="templates" className="mt-4">
          <Templates
            templates={templates.data?.templates ?? []}
            loading={templates.loading && !templates.data}
            onChanged={templates.reload}
            onUse={(tpl) => {
              setDraft((d) => applyTemplate({ ...d, in_reply_to: null }, tpl));
              setTab("compose");
            }}
          />
        </TabsContent>
      </Tabs>
    </>
  );
}
