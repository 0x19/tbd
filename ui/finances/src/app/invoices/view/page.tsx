"use client";

// One invoice: a draft is edited here, previewed as the PDF it would become,
// and approved with the hash of exactly that preview. An approved invoice is
// the immutable record: its PDF, its number, who approved it.
import {
  BellRing,
  Check,
  Copy,
  Download,
  ExternalLink,
  Eye,
  Maximize2,
  Plus,
  Send,
  Trash2,
  X,
} from "lucide-react";
import Link from "next/link";
import { useRouter, useSearchParams } from "next/navigation";
import { Suspense, useEffect, useMemo, useState } from "react";
import { toast } from "sonner";

import { useFinance } from "@/app/providers";
import { PaymentsCard } from "@/components/invoices/payments-card";
import { PageTitle } from "@/components/kit";
import { StatusBadge } from "@/components/status-badge";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Dialog, DialogContent, DialogTitle } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Skeleton } from "@/components/ui/skeleton";
import { Textarea } from "@/components/ui/textarea";
import { api, pdfUrl } from "@/lib/api/client";
import { describe, useFetch } from "@/lib/api/hooks";
import type { Invoice, InvoiceLine, LineTemplate } from "@/lib/api/schema";
import { money, when } from "@/lib/format";
import { useT } from "@/lib/i18n";
import { stashPrefill } from "@/lib/mail-template";

export default function InvoicePage() {
  return (
    <Suspense fallback={<Skeleton className="h-64 w-full" />}>
      <InvoiceView />
    </Suspense>
  );
}

/** Editable line, in the units a person types: "1.5" and "13750.00". */
type EditLine = { description: string; quantity: string; unit_price: string; template_id: string };

function toEdit(l: InvoiceLine): EditLine {
  return {
    description: l.description,
    quantity: (Number(l.quantity_milli) / 1000).toString(),
    unit_price: (Number(l.unit_price_minor) / 100).toFixed(2),
    template_id: l.template_id,
  };
}

function fromTemplate(t: LineTemplate): EditLine {
  return {
    description: t.description,
    quantity: (Number(t.quantity_milli) / 1000).toString(),
    unit_price: (Number(t.unit_price_minor) / 100).toFixed(2),
    template_id: t.id,
  };
}

/** "13750.00" or "13.750,00" → minor units as a decimal string, never a float. */
function toMinor(text: string, scale: number): string {
  const t = text.trim().replace(/\s/g, "");
  // Decide the decimal separator by whichever comes last.
  const lastComma = t.lastIndexOf(",");
  const lastDot = t.lastIndexOf(".");
  const sep = lastComma > lastDot ? "," : ".";
  const [wholeRaw, fracRaw = ""] = t.split(sep);
  const negative = wholeRaw!.startsWith("-");
  const whole = wholeRaw!.replace(/[^0-9]/g, "") || "0";
  const frac = (fracRaw.replace(/[^0-9]/g, "") + "0".repeat(scale)).slice(0, scale);
  const digits = `${whole}${frac}`.replace(/^0+(?=\d)/, "");
  return `${negative ? "-" : ""}${digits}`;
}

/** The VAT treatment and the template mode are wire values; these are their words. */
const VAT_LABEL: Record<string, string> = {
  standard_hr: "invoices.vat.standard_hr",
  reverse_charge_eu: "invoices.vat.reverse_charge_eu",
  outside_scope_non_eu: "invoices.vat.outside_scope_non_eu",
  exempt_issuer: "invoices.vat.exempt_issuer",
};
const TREATMENTS = ["standard_hr", "reverse_charge_eu", "outside_scope_non_eu", "exempt_issuer"];
const MODE_LABEL: Record<string, string> = {
  fixed: "invoices.mode.fixed",
  variable: "invoices.mode.variable",
};

function toApiLines(lines: EditLine[]): InvoiceLine[] {
  return lines.map((l, i) => ({
    position: i + 1,
    description: l.description,
    quantity_milli: toMinor(l.quantity, 3),
    unit_price_minor: toMinor(l.unit_price, 2),
    amount_minor: "0",
    template_id: l.template_id,
  }));
}

function InvoiceView() {
  const t = useT();
  const id = useSearchParams().get("id") ?? "";
  const loaded = useFetch(() => api.invoice(id), 0, [id]);
  const inv = loaded.data?.invoice ?? null;
  const router = useRouter();
  const { partyName } = useFinance();
  // The client's addresses, for sending the approved PDF by mail.
  const clients = useFetch(() => (inv?.party_id ? api.clients([inv.party_id]) : Promise.resolve(null)), 0, [
    inv?.party_id,
  ]);
  const sendByMail = (reminder = false) => {
    if (!inv) return;
    const c = clients.data?.clients.find((x) => x.id === inv.client_id);
    stashPrefill({
      kind: "invoice",
      party_id: inv.party_id,
      company: partyName(inv.party_id),
      invoice: {
        id: inv.id,
        number: inv.number,
        issued_at: inv.issued_at,
        due_date: inv.due_date,
        total: money(inv.total_minor, inv.currency),
        outstanding: money(inv.outstanding_minor, inv.currency),
        days_overdue: inv.days_overdue,
        document_id: inv.document_id,
        filename: `${inv.number}.pdf`,
      },
      client: { id: inv.client_id, name: c?.name ?? "", recipients: c?.recipients ?? [] },
      reminder,
    });
    router.push("/mail/");
  };
  const templates = useFetch(
    () =>
      inv?.client_id
        ? api.lineTemplates(inv.client_id)
        : Promise.resolve({ templates: [] as LineTemplate[] }),
    0,
    [inv?.client_id],
  );
  const templateById = useMemo(
    () => new Map((templates.data?.templates ?? []).map((tp) => [tp.id, tp])),
    [templates.data],
  );
  const [delivery, setDelivery] = useState("");
  const [due, setDue] = useState("");
  const [place, setPlace] = useState("");
  const [note, setNote] = useState("");
  // The header a draft may still change: client (same party), currency,
  // VAT treatment, series.
  const [clientId, setClientId] = useState("");
  const [currency, setCurrency] = useState("");
  const [treatment, setTreatment] = useState("");
  const [premises, setPremises] = useState("");
  const [device, setDevice] = useState("");
  const [deleting, setDeleting] = useState(false);
  const [lines, setLines] = useState<EditLine[]>([]);
  const [dirty, setDirty] = useState(false);
  const [busy, setBusy] = useState<"" | "save" | "preview" | "approve" | "cancel" | "duplicate" | "delete">(
    "",
  );
  const [preview, setPreview] = useState<{ url: string; hash: string; number: string; at: number } | null>(
    null,
  );
  const [docUrl, setDocUrl] = useState<string | null>(null);
  const [full, setFull] = useState(false);
  useEffect(() => {
    if (!full) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") setFull(false);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [full]);

  // Load the draft into the form once; later edits are the person's.
  useEffect(() => {
    if (!inv) return;
    setDelivery(inv.delivery_date);
    setDue(inv.due_date);
    setPlace(inv.place_of_issue);
    setNote(inv.note);
    setClientId(inv.client_id);
    setCurrency(inv.currency);
    setTreatment(inv.vat_treatment);
    setPremises(inv.premises);
    setDevice(inv.device);
    setLines(inv.lines.map(toEdit));
    setDirty(false);
    setPreview(null);
    if (inv.document_id) {
      api
        .invoiceDocument(inv.id)
        .then((d) => setDocUrl(pdfUrl(d.pdf)))
        .catch((e: unknown) => toast.error(describe(e)));
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [inv?.id, inv?.updated_at]);

  const editable = inv?.status === "draft";
  // Every edit is saved and re-rendered after a short pause, so the preview
  // is always the draft as it is now, and Approve is never one click away
  // from a stale one. `seq` drops responses that were overtaken by a newer
  // edit.
  const [seq, setSeq] = useState(0);
  const [autoState, setAutoState] = useState<"idle" | "pending" | "rendering">("idle");
  const edit =
    <T,>(setter: (v: T) => void) =>
    (v: T) => {
      setter(v);
      setDirty(true);
      setPreview(null);
    };
  const totals = useMemo(() => {
    let subtotal = 0n;
    for (const l of toApiLines(lines)) {
      const q = BigInt(l.quantity_milli);
      const p = BigInt(l.unit_price_minor);
      const n = q * p;
      const half = n < 0n ? -500n : 500n;
      subtotal += (n + half) / 1000n;
    }
    const rate = treatment === "standard_hr" ? 2500n : 0n;
    const vat = (subtotal * rate + 5000n) / 10000n;
    return { subtotal, vat, total: subtotal + vat };
  }, [lines, treatment]);

  const save = async (): Promise<Invoice | null> => {
    if (!inv) return null;
    setBusy("save");
    try {
      const r = await api.updateInvoice(inv.id, {
        delivery_date: delivery,
        due_date: due,
        place_of_issue: place,
        note,
        lines: toApiLines(lines),
        client_id: clientId,
        currency,
        vat_treatment: treatment,
        premises,
        device,
      });
      setDirty(false);
      loaded.setData(r);
      return r.invoice ?? null;
    } catch (e) {
      toast.error(describe(e));
      return null;
    } finally {
      setBusy("");
    }
  };

  const doPreview = async () => {
    if (!inv) return;
    if (dirty && !(await save())) return;
    setBusy("preview");
    try {
      const p = await api.previewInvoice(inv.id);
      setPreview({ url: pdfUrl(p.pdf), hash: p.content_hash, number: p.number, at: Date.now() });
    } catch (e) {
      toast.error(describe(e));
    } finally {
      setBusy("");
    }
  };

  // Auto-preview: 900 ms after the last edit, save then render; a draft
  // that loads with lines is rendered once on arrival.
  useEffect(() => {
    if (!inv || !editable) return;
    if (!dirty && preview) return;
    // The save inside a pending render flips `dirty` off; that is not a new
    // edit, so it must not start a second render.
    if (!dirty && autoState !== "idle") return;
    if (lines.length === 0) return;
    const mine = seq + 1;
    setSeq(mine);
    setAutoState("pending");
    const timer = setTimeout(
      async () => {
        let current: Invoice | null = inv;
        if (dirty) {
          current = await save();
          if (!current) {
            setAutoState("idle");
            return;
          }
        }
        setAutoState("rendering");
        try {
          const p = await api.previewInvoice(current.id);
          setSeq((latest) => {
            if (latest === mine) {
              setPreview({ url: pdfUrl(p.pdf), hash: p.content_hash, number: p.number, at: Date.now() });
            }
            return latest;
          });
        } catch (e) {
          toast.error(describe(e));
        } finally {
          setAutoState("idle");
        }
      },
      dirty ? 900 : 0,
    );
    return () => clearTimeout(timer);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [inv?.id, editable, dirty, delivery, due, place, note, lines]);

  const approve = async () => {
    if (!inv || !preview) return;
    setBusy("approve");
    try {
      const r = await api.approveInvoice(inv.id, preview.hash);
      toast.success(t("invoices.approved_toast", { number: r.invoice?.number ?? "" }));
      setPreview(null);
      loaded.reload();
    } catch (e) {
      toast.error(describe(e));
      setPreview(null);
    } finally {
      setBusy("");
    }
  };

  const cancel = async () => {
    if (!inv) return;
    const reason = window.prompt(t("invoices.cancel_confirm"), "");
    if (reason === null) return;
    setBusy("cancel");
    try {
      await api.cancelInvoice(inv.id, reason);
      loaded.reload();
    } catch (e) {
      toast.error(describe(e));
    } finally {
      setBusy("");
    }
  };

  // A draft is deleted, never cancelled: it took no number.
  const remove = async () => {
    if (!inv) return;
    setBusy("delete");
    try {
      await api.deleteInvoice(inv.id);
      toast.success(t("invoices.deleted"));
      router.push("/invoices/");
    } catch (e) {
      toast.error(describe(e));
      setBusy("");
    }
  };

  const duplicate = async () => {
    if (!inv) return;
    setBusy("duplicate");
    try {
      const r = await api.createInvoice("", inv.id);
      toast.success(t("invoices.duplicated", { number: inv.number || t("invoices.draft_title") }));
      router.push(`/invoices/view/?id=${r.invoice!.id}`);
    } catch (e) {
      toast.error(describe(e));
      setBusy("");
    }
  };

  if (loaded.error) return <p className="text-destructive text-sm">{loaded.error}</p>;
  if (!inv) return <Skeleton className="h-64 w-full" />;

  return (
    <>
      <PageTitle
        title={inv.number || t("invoices.draft_title")}
        description={
          inv.number
            ? t("invoices.issued_approved", { issued: when(inv.issued_at), approved: when(inv.approved_at) })
            : t("invoices.draft_hint")
        }
        back={
          <Button asChild variant="ghost" size="icon" className="mt-0.5">
            <Link href="/invoices/" aria-label={t("invoices.back")}>
              ←
            </Link>
          </Button>
        }
      >
        <StatusBadge status={inv.status} className="text-xs" />
        {editable ? (
          <>
            <Button variant="outline" size="sm" onClick={() => void save()} disabled={!dirty || busy !== ""}>
              {busy === "save" ? t("common.saving") : t("common.save")}
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={() => void doPreview()}
              disabled={busy !== "" || lines.length === 0}
            >
              <Eye /> {busy === "preview" ? t("invoices.rendering") : t("invoices.preview")}
            </Button>
            <Button size="sm" onClick={() => void approve()} disabled={!preview || dirty || busy !== ""}>
              <Check />{" "}
              {busy === "approve"
                ? t("invoices.approving")
                : preview
                  ? t("invoices.approve_as", { number: preview.number })
                  : t("invoices.approve")}
            </Button>
          </>
        ) : null}
        {docUrl ? (
          <>
            <Button asChild variant="outline" size="sm">
              <a href={docUrl} download={`inorbit-${inv.number}.pdf`}>
                <Download /> {t("invoices.pdf")}
              </a>
            </Button>
            <Button variant="outline" size="sm" onClick={() => sendByMail()} disabled={!inv.document_id}>
              <Send /> {inv.sent_at ? t("invoices.send_again") : t("invoices.send_mail")}
            </Button>
            {inv.days_overdue > 0 ? (
              <Button
                variant="outline"
                size="sm"
                className="text-destructive"
                onClick={() => sendByMail(true)}
                disabled={!inv.document_id}
                title={t("invoices.days_overdue", { n: inv.days_overdue })}
              >
                <BellRing /> {t("invoices.remind")}
              </Button>
            ) : null}
          </>
        ) : null}
        <Button variant="outline" size="sm" onClick={() => void duplicate()} disabled={busy !== ""}>
          <Copy /> {t("invoices.duplicate")}
        </Button>
        {inv.status === "approved" ? (
          <Button variant="ghost" size="sm" onClick={() => void cancel()} disabled={busy !== ""}>
            <X /> {t("invoices.cancel_invoice")}
          </Button>
        ) : null}
        {editable ? (
          <Button variant="ghost" size="sm" onClick={() => setDeleting(true)} disabled={busy !== ""}>
            <Trash2 /> {t("invoices.delete_draft")}
          </Button>
        ) : null}
      </PageTitle>
      <AlertDialog open={deleting} onOpenChange={setDeleting}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t("invoices.delete_title")}</AlertDialogTitle>
            <AlertDialogDescription>{t("invoices.delete_confirm")}</AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t("common.cancel")}</AlertDialogCancel>
            <AlertDialogAction onClick={() => void remove()}>{t("invoices.delete_draft")}</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      <div className="grid gap-6 xl:grid-cols-5">
        <div className="space-y-6 xl:col-span-3">
          <Card>
            <CardHeader>
              <CardTitle>{t("invoices.details")}</CardTitle>
              <CardDescription>
                {VAT_LABEL[inv.vat_treatment]
                  ? t(VAT_LABEL[inv.vat_treatment]!)
                  : inv.vat_treatment.replace(/_/g, " ")}{" "}
                · {inv.currency}
                {inv.prefilled_from ? ` · ${t("invoices.prefilled")}` : ""}
              </CardDescription>
            </CardHeader>
            <CardContent className="grid gap-4 sm:grid-cols-3">
              {editable ? (
                <>
                  <Field label={t("invoices.client")}>
                    <Select value={clientId} onValueChange={(v) => edit(setClientId)(v)}>
                      <SelectTrigger>
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        {(clients.data?.clients ?? [])
                          .filter((c) => !c.archived || c.id === clientId)
                          .map((c) => (
                            <SelectItem key={c.id} value={c.id}>
                              {c.name}
                            </SelectItem>
                          ))}
                      </SelectContent>
                    </Select>
                  </Field>
                  <Field label={t("invoices.vat_treatment")}>
                    <Select value={treatment} onValueChange={(v) => edit(setTreatment)(v)}>
                      <SelectTrigger>
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        {TREATMENTS.map((v) => (
                          <SelectItem key={v} value={v}>
                            {t(VAT_LABEL[v]!)}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </Field>
                  <div className="grid grid-cols-3 gap-2">
                    <Field label={t("invoices.currency")}>
                      <Input
                        value={currency}
                        maxLength={3}
                        onChange={(e) => edit(setCurrency)(e.target.value.toUpperCase())}
                      />
                    </Field>
                    <Field
                      label={t("invoices.series")}
                      className="col-span-2"
                      hint={t("invoices.series_hint")}
                    >
                      <div className="flex gap-1">
                        <Input value={premises} onChange={(e) => edit(setPremises)(e.target.value)} />
                        <Input value={device} onChange={(e) => edit(setDevice)(e.target.value)} />
                      </div>
                    </Field>
                  </div>
                </>
              ) : null}
              <Field label={t("invoices.delivery_date")}>
                <Input
                  type="date"
                  value={delivery}
                  disabled={!editable}
                  onChange={(e) => edit(setDelivery)(e.target.value)}
                />
              </Field>
              <Field label={t("invoices.due_date")}>
                <Input
                  type="date"
                  value={due}
                  disabled={!editable}
                  onChange={(e) => edit(setDue)(e.target.value)}
                />
              </Field>
              <Field label={t("invoices.place_of_issue")}>
                <Input value={place} disabled={!editable} onChange={(e) => edit(setPlace)(e.target.value)} />
              </Field>
              <Field label={t("invoices.note_label")} className="sm:col-span-3">
                <Textarea
                  value={note}
                  disabled={!editable}
                  rows={2}
                  onChange={(e) => edit(setNote)(e.target.value)}
                />
              </Field>
            </CardContent>
          </Card>

          {inv.deliveries.length > 0 ? (
            <Card>
              <CardHeader>
                <CardTitle>{t("invoices.sent_section")}</CardTitle>
                <CardDescription>{t("invoices.sent_hint")}</CardDescription>
              </CardHeader>
              <CardContent>
                <ul className="divide-y text-sm">
                  {inv.deliveries.map((d) => (
                    <li key={d.id} className="flex flex-wrap items-center justify-between gap-2 py-2">
                      <span className="flex flex-wrap items-center gap-2">
                        <span className="tabular-nums">{when(d.sent_at)}</span>
                        <span className="text-muted-foreground">{d.to.join(", ")}</span>
                        {d.kind === "reminder" ? (
                          <Badge variant="outline" className="text-[10px]">
                            {t("invoices.delivery.reminder")}
                          </Badge>
                        ) : null}
                      </span>
                      {d.mail_id ? (
                        <Button asChild variant="ghost" size="sm" className="h-7 text-xs">
                          <Link href={`/mail/?mail=${d.mail_id}`}>{t("invoices.open_thread")}</Link>
                        </Button>
                      ) : null}
                    </li>
                  ))}
                </ul>
              </CardContent>
            </Card>
          ) : null}

          <PaymentsCard
            invoice={inv}
            clientName={clients.data?.clients.find((x) => x.id === inv.client_id)?.name ?? ""}
            onChanged={(next) => loaded.setData({ invoice: next })}
          />

          <Card>
            <CardHeader className="flex flex-row items-center justify-between">
              <div>
                <CardTitle>{t("invoices.lines")}</CardTitle>
                <CardDescription>{t("invoices.lines_hint")}</CardDescription>
              </div>
              {editable ? (
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() =>
                    edit(setLines)([
                      ...lines,
                      { description: "", quantity: "1", unit_price: "0.00", template_id: "" },
                    ])
                  }
                >
                  <Plus /> {t("invoices.add_line")}
                </Button>
              ) : null}
            </CardHeader>
            <CardContent className="space-y-2">
              {editable &&
              (templates.data?.templates ?? []).some(
                (tp) => tp.enabled && !lines.some((l) => l.template_id === tp.id),
              ) ? (
                <div className="flex flex-wrap items-center gap-1.5 pb-1">
                  <span className="text-muted-foreground text-xs">{t("invoices.add_from_template")}</span>
                  {(templates.data?.templates ?? [])
                    .filter((tp) => tp.enabled && !lines.some((l) => l.template_id === tp.id))
                    .map((tp) => (
                      <Button
                        key={tp.id}
                        variant="outline"
                        size="sm"
                        className="h-7 text-xs"
                        onClick={() => edit(setLines)([...lines, fromTemplate(tp)])}
                      >
                        <Plus />{" "}
                        {tp.description.length > 40 ? `${tp.description.slice(0, 40)}…` : tp.description}
                        <span className="text-muted-foreground ml-1">
                          {MODE_LABEL[tp.mode] ? t(MODE_LABEL[tp.mode]!) : tp.mode}
                        </span>
                      </Button>
                    ))}
                </div>
              ) : null}
              <div className="text-muted-foreground grid grid-cols-[1fr_5rem_8rem_8rem_2rem] gap-2 px-1 text-xs">
                <span>{t("invoices.col.description")}</span>
                <span className="text-right">{t("invoices.col.qty")}</span>
                <span className="text-right">{t("invoices.col.price")}</span>
                <span className="text-right">{t("common.amount")}</span>
                <span />
              </div>
              {lines.map((l, i) => {
                const api = toApiLines([l])[0]!;
                const amount = (BigInt(api.quantity_milli) * BigInt(api.unit_price_minor) + 500n) / 1000n;
                return (
                  <div key={i} className="grid grid-cols-[1fr_5rem_8rem_8rem_2rem] items-start gap-2">
                    <Textarea
                      value={l.description}
                      disabled={!editable}
                      rows={2}
                      className="min-h-9 text-sm"
                      onChange={(e) =>
                        edit(setLines)(
                          lines.map((x, j) => (j === i ? { ...x, description: e.target.value } : x)),
                        )
                      }
                    />
                    <Input
                      value={l.quantity}
                      disabled={!editable}
                      inputMode="decimal"
                      className="text-right font-mono"
                      onChange={(e) =>
                        edit(setLines)(
                          lines.map((x, j) => (j === i ? { ...x, quantity: e.target.value } : x)),
                        )
                      }
                    />
                    <Input
                      value={l.unit_price}
                      disabled={!editable}
                      inputMode="decimal"
                      title={
                        templateById.get(l.template_id)?.mode === "variable"
                          ? t("invoices.variable_hint")
                          : undefined
                      }
                      className={
                        templateById.get(l.template_id)?.mode === "variable" && editable
                          ? "border-amber-400/70 text-right font-mono focus-visible:ring-amber-400/40"
                          : "text-right font-mono"
                      }
                      onChange={(e) =>
                        edit(setLines)(
                          lines.map((x, j) => (j === i ? { ...x, unit_price: e.target.value } : x)),
                        )
                      }
                    />
                    <div className="pt-2 text-right font-mono text-sm tabular-nums">
                      {money(amount.toString(), inv.currency)}
                    </div>
                    {editable ? (
                      <Button
                        variant="ghost"
                        size="icon"
                        className="size-8"
                        onClick={() => edit(setLines)(lines.filter((_, j) => j !== i))}
                        aria-label={t("invoices.remove_line")}
                      >
                        <Trash2 />
                      </Button>
                    ) : (
                      <span />
                    )}
                  </div>
                );
              })}
              <div className="flex justify-end pt-2">
                <dl className="grid grid-cols-[auto_auto] gap-x-6 gap-y-1 text-sm">
                  <dt className="text-muted-foreground text-right">{t("invoices.subtotal")}</dt>
                  <dd className="text-right font-mono tabular-nums">
                    {money(totals.subtotal.toString(), inv.currency)}
                  </dd>
                  <dt className="text-muted-foreground text-right">{t("invoices.vat")}</dt>
                  <dd className="text-right font-mono tabular-nums">
                    {money(totals.vat.toString(), inv.currency)}
                  </dd>
                  <dt className="text-right font-semibold">{t("invoices.total")}</dt>
                  <dd className="text-right font-mono font-semibold tabular-nums">
                    {money(totals.total.toString(), inv.currency)}
                  </dd>
                </dl>
              </div>
            </CardContent>
          </Card>
        </div>

        <Card className="xl:col-span-2">
          <CardHeader className="flex flex-row items-start justify-between gap-2">
            <div>
              <CardTitle>
                {inv.number
                  ? t("invoices.the_invoice")
                  : preview
                    ? t("invoices.preview_number", { number: preview.number })
                    : t("invoices.preview")}
              </CardTitle>
              <CardDescription>
                {inv.number
                  ? t("invoices.stored_pdf")
                  : preview
                    ? dirty
                      ? t("invoices.changed_since_preview")
                      : t("invoices.approve_exactly")
                    : t("invoices.renders_itself")}
              </CardDescription>
            </div>
            {docUrl || preview ? (
              <div className="flex shrink-0 gap-1">
                <Button
                  variant="outline"
                  size="icon"
                  className="size-8"
                  onClick={() => setFull(true)}
                  aria-label={t("invoices.full_size")}
                >
                  <Maximize2 />
                </Button>
                <Button asChild variant="outline" size="icon" className="size-8">
                  <a
                    href={docUrl ?? preview!.url}
                    target="_blank"
                    rel="noreferrer"
                    aria-label={t("invoices.open_new_tab")}
                  >
                    <ExternalLink />
                  </a>
                </Button>
              </div>
            ) : null}
          </CardHeader>
          <CardContent>
            {docUrl || preview ? (
              <div className="group relative aspect-[1/1.3] w-full">
                <iframe
                  title={t("invoices.frame_title")}
                  src={`${docUrl ?? preview!.url}#toolbar=0&view=Fit`}
                  className="bg-muted h-full w-full rounded-md border"
                />
                {/* An iframe keeps clicks to itself; this layer takes them and
                    opens the full-size view, with the cursor saying so. */}
                <button
                  type="button"
                  aria-label={t("invoices.open_full_size")}
                  onClick={() => setFull(true)}
                  className="hover:bg-foreground/[0.03] absolute inset-0 z-10 cursor-zoom-in rounded-md bg-transparent transition-colors"
                >
                  <span className="bg-background/90 text-muted-foreground pointer-events-none absolute right-2 bottom-2 rounded-md border px-2 py-1 text-xs opacity-0 transition-opacity group-hover:opacity-100">
                    {t("invoices.click_to_enlarge")}
                  </span>
                </button>
                {autoState !== "idle" ? (
                  <span className="bg-background/90 text-muted-foreground absolute top-2 left-2 z-20 rounded-md border px-2 py-1 text-xs">
                    {autoState === "pending" ? t("invoices.rerendering_soon") : t("invoices.rendering")}
                  </span>
                ) : null}
              </div>
            ) : (
              <div className="bg-muted/40 text-muted-foreground flex aspect-[1/1.3] w-full items-center justify-center rounded-md border border-dashed text-sm">
                {autoState !== "idle"
                  ? t("invoices.rendering")
                  : lines.length
                    ? t("invoices.nothing_rendered")
                    : t("invoices.add_line_hint")}
              </div>
            )}
          </CardContent>
        </Card>
      </div>

      <Dialog open={full} onOpenChange={setFull}>
        <DialogContent
          className="flex h-[94vh] w-[96vw] max-w-[96vw] flex-col gap-0 p-0 sm:max-w-[96vw]"
          showCloseButton={false}
          onEscapeKeyDown={() => setFull(false)}
        >
          <div className="flex items-center justify-between border-b px-4 py-2">
            <DialogTitle className="text-sm font-medium">
              {inv.number
                ? t("invoices.invoice_number", { number: inv.number })
                : preview
                  ? t("invoices.preview_number", { number: preview.number })
                  : t("invoices.preview")}
            </DialogTitle>
            {/* Focused on open so Escape is ours; once the PDF viewer has
                focus it keeps the keyboard, and this button is the way back. */}
            <Button variant="outline" size="sm" autoFocus onClick={() => setFull(false)}>
              <X /> {t("common.close")} <span className="text-muted-foreground ml-1 text-xs">Esc</span>
            </Button>
          </div>
          {docUrl || preview ? (
            <iframe
              title={t("invoices.frame_title_full")}
              src={`${docUrl ?? preview!.url}#view=FitH`}
              className="bg-muted min-h-0 w-full flex-1 rounded-b-lg"
            />
          ) : null}
        </DialogContent>
      </Dialog>
    </>
  );
}

function Field({
  label,
  children,
  className,
  hint,
}: {
  label: string;
  children: React.ReactNode;
  className?: string;
  hint?: string;
}) {
  return (
    <div className={className}>
      <Label className="text-muted-foreground mb-1.5 block text-xs" title={hint}>
        {label}
      </Label>
      {children}
    </div>
  );
}
