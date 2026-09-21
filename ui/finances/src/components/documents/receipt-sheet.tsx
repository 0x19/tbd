"use client";

// One receipt: the file large on the left, what was read of it on the
// right, each field with how it was found, and the editor in the same
// column so a correction is made next to the page it corrects. A
// correction is declared and outlives any re-read.
import { Download, FileSearch, Pencil, RefreshCw, Upload, X } from "lucide-react";
import { useEffect, useState } from "react";
import { toast } from "sonner";

import { useFinance } from "@/app/providers";
import { Provenance, ReceiptStatusChip } from "@/components/documents/status";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Sheet, SheetContent, SheetDescription, SheetHeader, SheetTitle } from "@/components/ui/sheet";
import { Skeleton } from "@/components/ui/skeleton";
import { api, pdfUrl } from "@/lib/api/client";
import { describe } from "@/lib/api/hooks";
import type { Document } from "@/lib/api/schema";
import { day, money, when } from "@/lib/format";
import { useT } from "@/lib/i18n";
import { decimalToMinor, fileSize, minorToDecimal, statusOf } from "@/lib/receipts";

export function ReceiptSheet({
  id,
  onClose,
  onChanged,
  vendors,
}: {
  id: string | null;
  onClose: () => void;
  onChanged: () => void;
  vendors: string[];
}) {
  const t = useT();
  const { parties, partyName } = useFinance();
  const [doc, setDoc] = useState<Document | null>(null);
  const [url, setUrl] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState("");
  const [editing, setEditing] = useState(false);
  const [form, setForm] = useState({
    vendor: "",
    doc_date: "",
    amount: "",
    currency: "EUR",
    invoice_no: "",
    party_id: "",
  });

  const fill = (d: Document) => {
    setDoc(d);
    setForm({
      vendor: d.vendor,
      doc_date: d.doc_date,
      amount: minorToDecimal(d.total_minor),
      currency: d.currency || "EUR",
      invoice_no: d.invoice_no,
      party_id: d.party_id,
    });
  };

  useEffect(() => {
    if (!id) {
      setDoc(null);
      setError(null);
      setEditing(false);
      return;
    }
    let cancelled = false;
    let objectUrl: string | null = null;
    setDoc(null);
    setUrl(null);
    setError(null);
    api
      .document(id)
      .then((r) => {
        if (cancelled || !r.document) return;
        fill(r.document);
        objectUrl = pdfUrl(r.bytes);
        setUrl(objectUrl);
        setEditing(false);
      })
      .catch((e: unknown) => {
        if (!cancelled) setError(describe(e));
      });
    return () => {
      cancelled = true;
      if (objectUrl) URL.revokeObjectURL(objectUrl);
    };
  }, [id]);

  const act = async (what: string, f: () => Promise<Document | null | undefined>, done: string) => {
    setBusy(what);
    try {
      const d = await f();
      if (d) fill(d);
      toast.success(done);
      onChanged();
    } catch (e) {
      toast.error(describe(e));
    } finally {
      setBusy("");
    }
  };

  const save = () =>
    doc &&
    act(
      "save",
      async () =>
        (
          await api.updateDocument(doc.id, {
            vendor: form.vendor,
            doc_date: form.doc_date,
            total_minor: decimalToMinor(form.amount),
            currency: form.currency,
            invoice_no: form.invoice_no,
            party_id: form.party_id,
          })
        ).document,
      t("documents.saved_yours"),
    ).then(() => setEditing(false));

  const status = doc ? statusOf(doc) : null;
  const uploaded = doc?.sources.length === 0;
  const fact = (label: string, value: React.ReactNode, by?: string, mono?: boolean) => (
    <div className="flex items-baseline justify-between gap-3 py-2">
      <dt className="text-muted-foreground shrink-0 text-xs">{label}</dt>
      <dd className={"min-w-0 truncate text-right text-sm " + (mono ? "font-mono tabular-nums" : "")}>
        {value}
        <Provenance by={by} />
      </dd>
    </div>
  );
  const field = (label: string, by: string | undefined, node: React.ReactNode) => (
    <div>
      <Label className="text-muted-foreground mb-1 flex items-center text-xs">
        {label}
        <Provenance by={by} always />
      </Label>
      {node}
    </div>
  );

  return (
    <Sheet open={id !== null} onOpenChange={(o) => !o && onClose()}>
      <SheetContent className="flex w-full flex-col gap-4 overflow-y-auto sm:max-w-6xl">
        <SheetHeader className="pr-8">
          <SheetTitle className="flex flex-wrap items-center gap-2">
            {doc ? doc.vendor || t("documents.unknown_vendor_title") : t("documents.receipt")}
            {doc?.total_minor ? (
              <span className="font-mono text-base font-normal tabular-nums">
                {money(doc.total_minor, doc.currency || "EUR")}
              </span>
            ) : null}
            {status ? <ReceiptStatusChip status={status} /> : null}
          </SheetTitle>
          <SheetDescription>
            {doc
              ? [
                  doc.doc_date ? day(doc.doc_date) : t("documents.no_date"),
                  doc.invoice_no || t("documents.no_number"),
                  doc.filename,
                ]
                  .filter(Boolean)
                  .join(" · ")
              : t("common.loading")}
          </SheetDescription>
        </SheetHeader>

        {error ? <p className="text-destructive text-sm">{error}</p> : null}

        <div className="grid flex-1 gap-5 lg:grid-cols-[1fr_22rem]">
          <div className="bg-muted/40 min-h-[60vh] overflow-hidden rounded-lg border">
            {url ? (
              <iframe title={t("documents.receipt")} src={url} className="h-[78vh] w-full" />
            ) : (
              <div className="text-muted-foreground flex h-[60vh] items-center justify-center gap-2 text-sm">
                <FileSearch className="size-4" />{" "}
                {error ? t("documents.file_error") : t("documents.file_loading")}
              </div>
            )}
          </div>

          <div className="space-y-4 lg:sticky lg:top-0 lg:self-start">
            <div className="flex flex-wrap gap-1.5">
              <Button
                variant={editing ? "secondary" : "default"}
                size="sm"
                onClick={() => setEditing((v) => !v)}
                disabled={!doc}
              >
                {editing ? <X /> : <Pencil />} {editing ? t("common.cancel") : t("documents.correct")}
              </Button>
              <Button
                variant="outline"
                size="sm"
                disabled={!doc || busy !== ""}
                onClick={() =>
                  doc &&
                  void act(
                    "extract",
                    async () => (await api.extractDocument(doc.id)).document,
                    doc.declared ? t("documents.read_again_kept") : t("documents.read_again_done"),
                  )
                }
              >
                <RefreshCw className={busy === "extract" ? "animate-spin" : undefined} />{" "}
                {t("documents.read_again")}
              </Button>
              <Button
                variant="outline"
                size="sm"
                disabled={!url}
                onClick={() => {
                  if (!url || !doc) return;
                  const a = window.document.createElement("a");
                  a.href = url;
                  a.download = doc.filename || `${doc.id}.pdf`;
                  a.click();
                }}
              >
                <Download /> {t("common.download")}
              </Button>
            </div>

            {!doc ? <Skeleton className="h-48" /> : null}

            {doc && status && status !== "read" && status !== "corrected" && !editing ? (
              <p className="rounded-md border border-amber-500/40 bg-amber-500/10 px-3 py-2 text-xs text-amber-800 dark:text-amber-200">
                {t(`documents.why.${status}`)}
              </p>
            ) : null}

            {doc && editing ? (
              <form
                className="bg-card space-y-3 rounded-lg border p-4"
                onSubmit={(e) => {
                  e.preventDefault();
                  void save();
                }}
              >
                {parties.length > 1
                  ? field(
                      t("documents.whose"),
                      doc.found_by.party,
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
                      </Select>,
                    )
                  : null}
                {field(
                  t("common.vendor"),
                  doc.found_by.vendor,
                  <>
                    <Input
                      list="vendors"
                      value={form.vendor}
                      onChange={(e) => setForm({ ...form, vendor: e.target.value })}
                      autoFocus
                    />
                    <datalist id="vendors">
                      {vendors.map((v) => (
                        <option key={v} value={v} />
                      ))}
                    </datalist>
                  </>,
                )}
                {field(
                  t("common.date"),
                  doc.found_by.date,
                  <Input
                    type="date"
                    value={form.doc_date}
                    onChange={(e) => setForm({ ...form, doc_date: e.target.value })}
                  />,
                )}
                <div className="grid grid-cols-[1fr_5rem] gap-2">
                  {field(
                    t("common.amount"),
                    doc.found_by.amount,
                    <Input
                      inputMode="decimal"
                      value={form.amount}
                      onChange={(e) => setForm({ ...form, amount: e.target.value })}
                      placeholder="0.00"
                      className="font-mono"
                    />,
                  )}
                  {field(
                    t("common.currency"),
                    undefined,
                    <Input
                      value={form.currency}
                      onChange={(e) => setForm({ ...form, currency: e.target.value.toUpperCase() })}
                      maxLength={3}
                      className="font-mono uppercase"
                    />,
                  )}
                </div>
                {field(
                  t("common.number"),
                  doc.found_by.invoice_no,
                  <Input
                    value={form.invoice_no}
                    onChange={(e) => setForm({ ...form, invoice_no: e.target.value })}
                    className="font-mono"
                  />,
                )}
                <div className="flex items-center gap-2 pt-1">
                  <Button type="submit" size="sm" disabled={busy !== ""}>
                    {busy === "save" ? t("common.saving") : t("common.save")}
                  </Button>
                  <span className="text-muted-foreground text-[11px]">{t("documents.clear_hint")}</span>
                </div>
              </form>
            ) : null}

            {doc && !editing ? (
              <dl className="bg-card divide-y rounded-lg border px-4">
                {fact(t("documents.whose"), partyName(doc.party_id), doc.found_by.party)}
                {fact(t("common.vendor"), doc.vendor || "—", doc.found_by.vendor)}
                {fact(t("common.date"), doc.doc_date ? day(doc.doc_date) : "—", doc.found_by.date, true)}
                {fact(
                  t("common.amount"),
                  doc.total_minor ? money(doc.total_minor, doc.currency || "EUR") : "—",
                  doc.found_by.amount,
                  true,
                )}
                {fact(t("common.number"), doc.invoice_no || "—", doc.found_by.invoice_no, true)}
              </dl>
            ) : null}

            {doc ? (
              <div className="text-muted-foreground space-y-1.5 text-xs">
                <p className="text-foreground font-medium">{t("documents.origin")}</p>
                {uploaded ? (
                  <p className="flex items-center gap-1.5">
                    <Upload className="size-3.5" />{" "}
                    {t("documents.origin_upload", { when: when(doc.created_at) })}
                  </p>
                ) : (
                  doc.sources.map((src) => (
                    <div key={src.external_ref} className="space-y-0.5">
                      <p className="text-foreground truncate" title={src.subject}>
                        {src.subject || "—"}
                      </p>
                      <p className="truncate">
                        {src.sender} · {when(src.received_at)}
                      </p>
                    </div>
                  ))
                )}
                <p className="truncate font-mono text-[11px]" title={doc.filename}>
                  {doc.filename}
                  {doc.size_bytes && fileSize(doc.size_bytes) ? ` · ${fileSize(doc.size_bytes)}` : ""}
                </p>
                {doc.found_by.error ? (
                  <p className="text-amber-700 dark:text-amber-300">
                    {t("documents.reader")}: {doc.found_by.error}
                  </p>
                ) : null}
              </div>
            ) : null}
          </div>
        </div>
      </SheetContent>
    </Sheet>
  );
}
