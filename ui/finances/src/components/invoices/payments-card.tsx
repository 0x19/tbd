"use client";

// Money in against one invoice: what the bank showed under its number (the
// matcher's), what a person recorded, what is still owed. Recording offers
// the company's incoming transactions first, since that is almost always
// where the payment is; a payment the bank has not shown is an amount and a
// day. Undo puts the invoice back to open when nothing covers it any more.

import { Landmark, PenLine, Search, Undo2 } from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
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
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { api } from "@/lib/api/client";
import { describe, useFetch } from "@/lib/api/hooks";
import type { Invoice } from "@/lib/api/schema";
import { dateOnly, money } from "@/lib/format";
import { useT } from "@/lib/i18n";

export function PaymentsCard({
  invoice,
  clientName,
  onChanged,
}: {
  invoice: Invoice;
  clientName: string;
  onChanged: (inv: Invoice) => void;
}) {
  const t = useT();
  const [recording, setRecording] = useState(false);
  const [busy, setBusy] = useState(false);
  const open = invoice.status === "approved" || invoice.status === "sent" || invoice.status === "paid";
  const owed = BigInt(invoice.total_minor) - BigInt(invoice.paid_minor);
  const undo = async (id: string) => {
    setBusy(true);
    try {
      const r = await api.unlinkPayment(id);
      toast.success(t("invoices.payment.undone"));
      if (r.invoice) onChanged(r.invoice);
    } catch (e) {
      toast.error(describe(e));
    } finally {
      setBusy(false);
    }
  };
  if (!open) return null;
  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between">
        <div>
          <CardTitle>{t("invoices.payments")}</CardTitle>
          <CardDescription>
            {invoice.paid_minor !== "0"
              ? t("invoices.paid_of", {
                  paid: money(invoice.paid_minor, invoice.currency),
                  total: money(invoice.total_minor, invoice.currency),
                })
              : t("invoices.payments_hint")}
            {owed > 0n && invoice.paid_minor !== "0"
              ? ` · ${t("invoices.outstanding", { amount: money(owed.toString(), invoice.currency) })}`
              : ""}
          </CardDescription>
        </div>
        <Button variant="outline" size="sm" onClick={() => setRecording(true)} disabled={busy}>
          <PenLine /> {t("invoices.payment.record")}
        </Button>
      </CardHeader>
      <CardContent>
        {invoice.payments.length === 0 ? (
          <p className="text-muted-foreground text-sm">{t("invoices.payments_none")}</p>
        ) : (
          <ul className="divide-y text-sm">
            {invoice.payments.map((p) => (
              <li key={p.id} className="flex flex-wrap items-center justify-between gap-2 py-2">
                <span className="flex items-center gap-2">
                  {p.transaction_id ? <Landmark className="size-4" /> : <PenLine className="size-4" />}
                  <span className="tabular-nums">{dateOnly(p.paid_on)}</span>
                  <span className="text-muted-foreground">{p.counterparty || p.note || p.reason}</span>
                  <Badge variant="outline" className="text-[10px]">
                    {t(`invoices.payment.${p.source}`)}
                  </Badge>
                </span>
                <span className="flex items-center gap-2">
                  <span className="font-mono tabular-nums">{money(p.amount_minor, p.currency)}</span>
                  <Button
                    variant="ghost"
                    size="sm"
                    className="h-7 px-2 text-xs"
                    onClick={() => void undo(p.id)}
                    disabled={busy}
                  >
                    <Undo2 /> {t("invoices.payment.undo")}
                  </Button>
                </span>
              </li>
            ))}
          </ul>
        )}
      </CardContent>
      <RecordDialog
        open={recording}
        invoice={invoice}
        clientName={clientName}
        onClose={() => setRecording(false)}
        onChanged={(inv) => {
          setRecording(false);
          onChanged(inv);
        }}
      />
    </Card>
  );
}

function RecordDialog({
  open,
  invoice,
  clientName,
  onClose,
  onChanged,
}: {
  open: boolean;
  invoice: Invoice;
  clientName: string;
  onClose: () => void;
  onChanged: (inv: Invoice) => void;
}) {
  const t = useT();
  const [q, setQ] = useState(clientName);
  const [amount, setAmount] = useState("");
  const [date, setDate] = useState("");
  const [note, setNote] = useState("");
  const [busy, setBusy] = useState(false);
  const taken = new Set(invoice.payments.map((p) => p.transaction_id));
  // Money in on the company's accounts, newest first; the client's name is
  // the first search.
  const found = useFetch(
    () =>
      open
        ? api.transactions({ party_ids: [invoice.party_id], search: q, limit: 100 })
        : Promise.resolve(null),
    0,
    [open, q, invoice.party_id],
  );
  const credits = (found.data?.transactions ?? []).filter(
    (x) => BigInt(x.amount_minor) > 0n && !taken.has(x.id),
  );
  const record = async (p: { transaction_id?: string; amount_minor?: string; paid_on?: string }) => {
    setBusy(true);
    try {
      const r = await api.recordPayment(invoice.id, { ...p, note });
      toast.success(t("invoices.payment.recorded"));
      if (r.invoice) onChanged(r.invoice);
    } catch (e) {
      toast.error(describe(e));
    } finally {
      setBusy(false);
    }
  };
  const minor = (s: string) => {
    const m = /^(-?\d+)(?:[.,](\d{1,2}))?$/.exec(s.trim().replace(/\s/g, ""));
    if (!m) return null;
    return `${m[1]}${(m[2] ?? "").padEnd(2, "0")}`;
  };
  return (
    <Dialog open={open} onOpenChange={(o) => !o && onClose()}>
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>{t("invoices.payment.record")}</DialogTitle>
          <DialogDescription>{invoice.number}</DialogDescription>
        </DialogHeader>
        <Tabs defaultValue="bank">
          <TabsList>
            <TabsTrigger value="bank">{t("invoices.payment.from_bank")}</TabsTrigger>
            <TabsTrigger value="hand">{t("invoices.payment.by_hand")}</TabsTrigger>
          </TabsList>
          <TabsContent value="bank" className="space-y-2">
            <p className="text-muted-foreground text-xs">{t("invoices.payment.from_bank_hint")}</p>
            <div className="relative">
              <Search className="text-muted-foreground absolute top-1/2 left-2.5 size-4 -translate-y-1/2" />
              <Input
                value={q}
                onChange={(e) => setQ(e.target.value)}
                placeholder={t("invoices.payment.search")}
                className="pl-8"
              />
            </div>
            <div className="max-h-72 space-y-1 overflow-y-auto">
              {credits.length === 0 && !found.loading ? (
                <p className="text-muted-foreground py-4 text-center text-xs">
                  {t("invoices.payment.none_found")}
                </p>
              ) : null}
              {credits.map((x) => (
                <button
                  key={x.id}
                  type="button"
                  disabled={busy}
                  className="hover:bg-muted/50 flex w-full items-center justify-between gap-2 rounded-md border px-2 py-1.5 text-left text-xs"
                  onClick={() => void record({ transaction_id: x.id })}
                >
                  <span className="min-w-0 truncate">
                    <span className="tabular-nums">{dateOnly(x.booking_date)}</span>{" "}
                    <span className="font-medium">{x.counterparty_name}</span>
                    <span className="text-muted-foreground"> · {x.remittance}</span>
                  </span>
                  <span className="font-mono tabular-nums">{money(x.amount_minor, x.currency)}</span>
                </button>
              ))}
            </div>
          </TabsContent>
          <TabsContent value="hand" className="space-y-3">
            <p className="text-muted-foreground text-xs">{t("invoices.payment.by_hand_hint")}</p>
            <div className="grid grid-cols-2 gap-3">
              <div>
                <Label className="text-muted-foreground mb-1 block text-xs">
                  {t("invoices.payment.amount")} ({invoice.currency})
                </Label>
                <Input value={amount} onChange={(e) => setAmount(e.target.value)} placeholder="1450,82" />
              </div>
              <div>
                <Label className="text-muted-foreground mb-1 block text-xs">
                  {t("invoices.payment.date")}
                </Label>
                <Input type="date" value={date} onChange={(e) => setDate(e.target.value)} />
              </div>
            </div>
            <div>
              <Label className="text-muted-foreground mb-1 block text-xs">{t("invoices.payment.note")}</Label>
              <Input value={note} onChange={(e) => setNote(e.target.value)} />
            </div>
            <DialogFooter>
              <Button variant="outline" onClick={onClose}>
                {t("common.cancel")}
              </Button>
              <Button
                disabled={busy || minor(amount) === null}
                onClick={() => {
                  const m = minor(amount);
                  if (m) void record({ amount_minor: m, paid_on: date });
                }}
              >
                {t("invoices.payment.record")}
              </Button>
            </DialogFooter>
          </TabsContent>
        </Tabs>
      </DialogContent>
    </Dialog>
  );
}
