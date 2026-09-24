// The composer's state and the pure moves on it. A draft is what the page
// holds between keystrokes; what is *sent* is rendered from it at the moment
// of sending (`render` in mail-template.ts), so helpers and the bundle
// follow the language the page is in then.
import type { Mail, MailTemplate } from "@/lib/api/schema";
import { monthsBefore, thisMonth } from "@/lib/format";
import type { Prefill } from "@/lib/mail-template";

export type Attached = { id: string; filename: string; vendor: string };

export type Draft = {
  connector_id: string;
  template_id: string;
  month: string;
  to: string;
  cc: string;
  bcc: string;
  subject: string;
  body: string;
  attachments: Attached[];
  /** The page that handed the composer its content: the reconciliation
   *  month behind `{{Summary}}` and the bundle, or the invoice behind
   *  `{{InvoiceNumber}}` and the attached PDF. */
  source: Prefill | null;
  /** Whether the bundle goes with the mail (the chip's X drops it). */
  with_bundle: boolean;
  in_reply_to: Mail | null;
};

export const EMPTY: Draft = {
  connector_id: "",
  template_id: "",
  month: monthsBefore(thisMonth(), 1),
  to: "",
  cc: "",
  bcc: "",
  subject: "",
  body: "",
  attachments: [],
  source: null,
  with_bundle: false,
  in_reply_to: null,
};

/** A template's recipients, subject and body over a draft. The summary keeps
 *  its place if the template names it, else follows the body. An invoice
 *  mail keeps the client's addresses when the template has none of its own.
 *  Without a template the subject stays empty, and the composer shows the
 *  source's default until something is typed. */
export function applyTemplate(d: Draft, tpl: MailTemplate | undefined): Draft {
  const month = d.source?.kind === "month";
  const summaryTail = month ? "{{Summary}}" : "";
  if (!tpl) {
    return { ...d, template_id: "", body: d.body || summaryTail };
  }
  const body =
    tpl.body.includes("{{Summary}}") || !summaryTail ? tpl.body : `${tpl.body.trimEnd()}\n\n${summaryTail}`;
  const to = tpl.to.length ? tpl.to.join(", ") : d.source?.kind === "invoice" ? d.to : "";
  return {
    ...d,
    template_id: tpl.id,
    to,
    cc: tpl.cc.join(", "),
    bcc: tpl.bcc.join(", "),
    subject: tpl.subject,
    body,
  };
}

/** The draft a hand-off from another page starts from, before a template. */
export function fromPrefill(p: Prefill, connectorId: string): Draft {
  if (p.kind === "invoice") {
    return {
      ...EMPTY,
      connector_id: connectorId,
      month: p.invoice.issued_at.slice(0, 7) || EMPTY.month,
      to: p.client.recipients.join(", "),
      attachments: [{ id: p.invoice.document_id, filename: p.invoice.filename, vendor: p.client.name }],
      source: p,
    };
  }
  return { ...EMPTY, connector_id: connectorId, month: p.month, source: p, with_bundle: true };
}

/** A reply: same mailbox, the other side as recipient, the subject prefixed. */
export function replyDraft(m: Mail): Draft {
  return {
    ...EMPTY,
    connector_id: m.connector_id,
    to: m.direction === "in" ? m.from : m.to.join(", "),
    cc: m.cc.join(", "),
    subject: m.subject.startsWith("Re:") ? m.subject : `Re: ${m.subject}`,
    in_reply_to: m,
  };
}

/** `{{Name}}` put where the caret is, or at the end. */
export function insertAt(text: string, caret: number | null, token: string): { text: string; caret: number } {
  const at = caret ?? text.length;
  const before = text.slice(0, at);
  const after = text.slice(at);
  const pad = before.length && !/\s$/.test(before) ? " " : "";
  const next = `${before}${pad}${token}${after}`;
  return { text: next, caret: before.length + pad.length + token.length };
}
