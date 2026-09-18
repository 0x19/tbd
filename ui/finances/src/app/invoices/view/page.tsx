"use client";

// One invoice: a draft is edited here, previewed as the PDF it would become,
// and approved with the hash of exactly that preview. An approved invoice is
// the immutable record: its PDF, its number, who approved it.
import { Check, Download, ExternalLink, Eye, Maximize2, Plus, Trash2, X } from "lucide-react";
import Link from "next/link";
import { useSearchParams } from "next/navigation";
import { Suspense, useEffect, useMemo, useState } from "react";
import { toast } from "sonner";

import { PageTitle } from "@/components/kit";
import { StatusBadge } from "@/components/status-badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Dialog, DialogContent, DialogTitle } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Skeleton } from "@/components/ui/skeleton";
import { Textarea } from "@/components/ui/textarea";
import { api, pdfUrl } from "@/lib/api/client";
import { describe, useFetch } from "@/lib/api/hooks";
import type { Invoice, InvoiceLine, LineTemplate } from "@/lib/api/schema";
import { money, when } from "@/lib/format";

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
  const id = useSearchParams().get("id") ?? "";
  const loaded = useFetch(() => api.invoice(id), 0, [id]);
  const inv = loaded.data?.invoice ?? null;
  const templates = useFetch(
    () =>
      inv?.client_id
        ? api.lineTemplates(inv.client_id)
        : Promise.resolve({ templates: [] as LineTemplate[] }),
    0,
    [inv?.client_id],
  );
  const templateById = useMemo(
    () => new Map((templates.data?.templates ?? []).map((t) => [t.id, t])),
    [templates.data],
  );
  const [delivery, setDelivery] = useState("");
  const [due, setDue] = useState("");
  const [place, setPlace] = useState("");
  const [note, setNote] = useState("");
  const [lines, setLines] = useState<EditLine[]>([]);
  const [dirty, setDirty] = useState(false);
  const [busy, setBusy] = useState<"" | "save" | "preview" | "approve" | "cancel">("");
  const [preview, setPreview] = useState<{ url: string; hash: string; number: string; at: number } | null>(
    null,
  );
  const [docUrl, setDocUrl] = useState<string | null>(null);
  const [full, setFull] = useState(false);

  // Load the draft into the form once; later edits are the person's.
  useEffect(() => {
    if (!inv) return;
    setDelivery(inv.delivery_date);
    setDue(inv.due_date);
    setPlace(inv.place_of_issue);
    setNote(inv.note);
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
    const rate = inv?.vat_treatment === "standard_hr" ? 2500n : 0n;
    const vat = (subtotal * rate + 5000n) / 10000n;
    return { subtotal, vat, total: subtotal + vat };
  }, [lines, inv?.vat_treatment]);

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
      toast.success(`Approved as ${r.invoice?.number}. The number is taken; the PDF is stored.`);
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
    const reason = window.prompt(
      inv.status === "draft" ? "Discard this draft?" : "Cancel this invoice? It keeps its number.",
      "",
    );
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

  if (loaded.error) return <p className="text-destructive text-sm">{loaded.error}</p>;
  if (!inv) return <Skeleton className="h-64 w-full" />;

  return (
    <>
      <PageTitle
        title={inv.number || "Draft invoice"}
        description={
          inv.number
            ? `Issued ${when(inv.issued_at)} · approved ${when(inv.approved_at)}`
            : "Edit, preview, then approve. Nothing is numbered until you approve exactly what you previewed."
        }
        back={
          <Button asChild variant="ghost" size="icon" className="mt-0.5">
            <Link href="/invoices/" aria-label="Back to invoices">
              ←
            </Link>
          </Button>
        }
      >
        <StatusBadge status={inv.status} className="text-xs" />
        {editable ? (
          <>
            <Button variant="outline" size="sm" onClick={() => void save()} disabled={!dirty || busy !== ""}>
              {busy === "save" ? "Saving…" : "Save"}
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={() => void doPreview()}
              disabled={busy !== "" || lines.length === 0}
            >
              <Eye /> {busy === "preview" ? "Rendering…" : "Preview"}
            </Button>
            <Button size="sm" onClick={() => void approve()} disabled={!preview || dirty || busy !== ""}>
              <Check />{" "}
              {busy === "approve" ? "Approving…" : preview ? `Approve as ${preview.number}` : "Approve"}
            </Button>
          </>
        ) : null}
        {docUrl ? (
          <Button asChild variant="outline" size="sm">
            <a href={docUrl} download={`inorbit-${inv.number}.pdf`}>
              <Download /> PDF
            </a>
          </Button>
        ) : null}
        {inv.status === "draft" || inv.status === "approved" ? (
          <Button variant="ghost" size="sm" onClick={() => void cancel()} disabled={busy !== ""}>
            <X /> {inv.status === "draft" ? "Discard" : "Cancel"}
          </Button>
        ) : null}
      </PageTitle>

      <div className="grid gap-6 xl:grid-cols-5">
        <div className="space-y-6 xl:col-span-3">
          <Card>
            <CardHeader>
              <CardTitle>Details</CardTitle>
              <CardDescription>
                {inv.vat_treatment.replace(/_/g, " ")} · {inv.currency}
                {inv.prefilled_from ? " · pre-filled from the previous invoice" : ""}
              </CardDescription>
            </CardHeader>
            <CardContent className="grid gap-4 sm:grid-cols-3">
              <Field label="Delivery date">
                <Input
                  type="date"
                  value={delivery}
                  disabled={!editable}
                  onChange={(e) => edit(setDelivery)(e.target.value)}
                />
              </Field>
              <Field label="Due date">
                <Input
                  type="date"
                  value={due}
                  disabled={!editable}
                  onChange={(e) => edit(setDue)(e.target.value)}
                />
              </Field>
              <Field label="Place of issue">
                <Input value={place} disabled={!editable} onChange={(e) => edit(setPlace)(e.target.value)} />
              </Field>
              <Field label="Note (printed under the totals)" className="sm:col-span-3">
                <Textarea
                  value={note}
                  disabled={!editable}
                  rows={2}
                  onChange={(e) => edit(setNote)(e.target.value)}
                />
              </Field>
            </CardContent>
          </Card>

          <Card>
            <CardHeader className="flex flex-row items-center justify-between">
              <div>
                <CardTitle>Lines</CardTitle>
                <CardDescription>
                  Quantity in units, price per unit. Amounts are computed on the server.
                </CardDescription>
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
                  <Plus /> Line
                </Button>
              ) : null}
            </CardHeader>
            <CardContent className="space-y-2">
              {editable &&
              (templates.data?.templates ?? []).some(
                (t) => t.enabled && !lines.some((l) => l.template_id === t.id),
              ) ? (
                <div className="flex flex-wrap items-center gap-1.5 pb-1">
                  <span className="text-muted-foreground text-xs">Add from template:</span>
                  {(templates.data?.templates ?? [])
                    .filter((t) => t.enabled && !lines.some((l) => l.template_id === t.id))
                    .map((t) => (
                      <Button
                        key={t.id}
                        variant="outline"
                        size="sm"
                        className="h-7 text-xs"
                        onClick={() => edit(setLines)([...lines, fromTemplate(t)])}
                      >
                        <Plus />{" "}
                        {t.description.length > 40 ? `${t.description.slice(0, 40)}…` : t.description}
                        <span className="text-muted-foreground ml-1">{t.mode}</span>
                      </Button>
                    ))}
                </div>
              ) : null}
              <div className="text-muted-foreground grid grid-cols-[1fr_5rem_8rem_8rem_2rem] gap-2 px-1 text-xs">
                <span>Description</span>
                <span className="text-right">Qty</span>
                <span className="text-right">Price</span>
                <span className="text-right">Amount</span>
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
                          ? "Variable row: pre-filled from last month; set this month's price"
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
                        aria-label="Remove line"
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
                  <dt className="text-muted-foreground text-right">Subtotal</dt>
                  <dd className="text-right font-mono tabular-nums">
                    {money(totals.subtotal.toString(), inv.currency)}
                  </dd>
                  <dt className="text-muted-foreground text-right">VAT</dt>
                  <dd className="text-right font-mono tabular-nums">
                    {money(totals.vat.toString(), inv.currency)}
                  </dd>
                  <dt className="text-right font-semibold">Total</dt>
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
                {inv.number ? "The invoice" : preview ? `Preview · ${preview.number}` : "Preview"}
              </CardTitle>
              <CardDescription>
                {inv.number
                  ? "The stored PDF, byte for byte what was approved."
                  : preview
                    ? dirty
                      ? "The draft changed since this preview; preview again before approving."
                      : "Approve exactly this. The hash of what you see is what gets approved."
                    : "The draft renders itself after every change, with the number it would take."}
              </CardDescription>
            </div>
            {docUrl || preview ? (
              <div className="flex shrink-0 gap-1">
                <Button
                  variant="outline"
                  size="icon"
                  className="size-8"
                  onClick={() => setFull(true)}
                  aria-label="Full size"
                >
                  <Maximize2 />
                </Button>
                <Button asChild variant="outline" size="icon" className="size-8">
                  <a
                    href={docUrl ?? preview!.url}
                    target="_blank"
                    rel="noreferrer"
                    aria-label="Open in a new tab"
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
                  title="invoice"
                  src={`${docUrl ?? preview!.url}#toolbar=0&view=Fit`}
                  className="bg-muted h-full w-full rounded-md border"
                />
                {/* An iframe keeps clicks to itself; this layer takes them and
                    opens the full-size view, with the cursor saying so. */}
                <button
                  type="button"
                  aria-label="Open full size"
                  onClick={() => setFull(true)}
                  className="hover:bg-foreground/[0.03] absolute inset-0 z-10 cursor-zoom-in rounded-md bg-transparent transition-colors"
                >
                  <span className="bg-background/90 text-muted-foreground pointer-events-none absolute right-2 bottom-2 rounded-md border px-2 py-1 text-xs opacity-0 transition-opacity group-hover:opacity-100">
                    Click to enlarge
                  </span>
                </button>
                {autoState !== "idle" ? (
                  <span className="bg-background/90 text-muted-foreground absolute top-2 left-2 z-20 rounded-md border px-2 py-1 text-xs">
                    {autoState === "pending" ? "Re-rendering soon…" : "Rendering…"}
                  </span>
                ) : null}
              </div>
            ) : (
              <div className="bg-muted/40 text-muted-foreground flex aspect-[1/1.3] w-full items-center justify-center rounded-md border border-dashed text-sm">
                {autoState !== "idle"
                  ? "Rendering…"
                  : lines.length
                    ? "Nothing rendered yet."
                    : "Add a line and the preview renders itself."}
              </div>
            )}
          </CardContent>
        </Card>
      </div>

      <Dialog open={full} onOpenChange={setFull}>
        <DialogContent className="h-[94vh] w-[96vw] max-w-[96vw] gap-0 p-0 sm:max-w-[96vw]">
          <DialogTitle className="sr-only">Invoice preview</DialogTitle>
          {docUrl || preview ? (
            <iframe
              title="invoice, full size"
              src={`${docUrl ?? preview!.url}#view=FitH`}
              className="bg-muted h-full w-full rounded-lg"
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
}: {
  label: string;
  children: React.ReactNode;
  className?: string;
}) {
  return (
    <div className={className}>
      <Label className="text-muted-foreground mb-1.5 block text-xs">{label}</Label>
      {children}
    </div>
  );
}
