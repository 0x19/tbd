"use client";

// One invoice: a draft is edited here, previewed as the PDF it would become,
// and approved with the hash of exactly that preview. An approved invoice is
// the immutable record: its PDF, its number, who approved it.
import { Check, Download, Eye, Plus, Trash2, X } from "lucide-react";
import Link from "next/link";
import { useSearchParams } from "next/navigation";
import { Suspense, useEffect, useMemo, useState } from "react";
import { toast } from "sonner";

import { PageTitle } from "@/components/kit";
import { StatusBadge } from "@/components/status-badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Skeleton } from "@/components/ui/skeleton";
import { Textarea } from "@/components/ui/textarea";
import { api, pdfUrl } from "@/lib/api/client";
import { describe, useFetch } from "@/lib/api/hooks";
import type { Invoice, InvoiceLine } from "@/lib/api/schema";
import { money, when } from "@/lib/format";

export default function InvoicePage() {
  return (
    <Suspense fallback={<Skeleton className="h-64 w-full" />}>
      <InvoiceView />
    </Suspense>
  );
}

/** Editable line, in the units a person types: "1.5" and "13750.00". */
type EditLine = { description: string; quantity: string; unit_price: string };

function toEdit(l: InvoiceLine): EditLine {
  return {
    description: l.description,
    quantity: (Number(l.quantity_milli) / 1000).toString(),
    unit_price: (Number(l.unit_price_minor) / 100).toFixed(2),
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
  }));
}

function InvoiceView() {
  const id = useSearchParams().get("id") ?? "";
  const loaded = useFetch(() => api.invoice(id), 0, [id]);
  const inv = loaded.data?.invoice ?? null;
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
                    edit(setLines)([...lines, { description: "", quantity: "1", unit_price: "0.00" }])
                  }
                >
                  <Plus /> Line
                </Button>
              ) : null}
            </CardHeader>
            <CardContent className="space-y-2">
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
                      className="text-right font-mono"
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
          <CardHeader>
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
                  : "Preview renders the draft with the number it would take."}
            </CardDescription>
          </CardHeader>
          <CardContent>
            {docUrl || preview ? (
              <iframe
                title="invoice"
                src={`${docUrl ?? preview!.url}#toolbar=0&view=FitH`}
                className="bg-muted h-[70vh] w-full rounded-md border"
              />
            ) : (
              <div className="bg-muted/40 text-muted-foreground flex h-[70vh] items-center justify-center rounded-md border border-dashed text-sm">
                Nothing rendered yet.
              </div>
            )}
          </CardContent>
        </Card>
      </div>
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
