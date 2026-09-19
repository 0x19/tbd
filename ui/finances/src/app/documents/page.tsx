"use client";

// Receipts: everything pulled from linked mailboxes, as a ledger. The
// server reads each PDF's text for vendor, date, amount and number and says
// how it found each one; a guess is shown as a guess, and a correction made
// here is declared and outlives any re-read. Search reaches the text.
import { Download, FileSearch, Pencil, RefreshCw, Search } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { toast } from "sonner";

import { useFinance } from "@/app/providers";
import { PageTitle } from "@/components/kit";
import { ScopeToggle } from "@/components/scope-toggle";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Sheet, SheetContent, SheetDescription, SheetHeader, SheetTitle } from "@/components/ui/sheet";
import { Skeleton } from "@/components/ui/skeleton";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { UploadReceipt } from "@/components/upload-receipt";
import { api, pdfUrl } from "@/lib/api/client";
import { describe, useFetch } from "@/lib/api/hooks";
import type { Document } from "@/lib/api/schema";
import { money, monthLabel, monthsBefore, thisMonth, when } from "@/lib/format";
import { type Lang, useLang, useT } from "@/lib/i18n";

const ALL = "__all";

/** The month's first and last day, for the server's inclusive bounds. */
function monthBounds(ym: string): { from: string; to: string } {
  const [y, m] = ym.split("-").map(Number);
  const last = new Date(Date.UTC(y, m, 0)).getUTCDate();
  return { from: `${ym}-01`, to: `${ym}-${String(last).padStart(2, "0")}` };
}

function minorToDecimal(minor: string): string {
  if (!minor) return "";
  const neg = minor.startsWith("-");
  const digits = (neg ? minor.slice(1) : minor).padStart(3, "0");
  return `${neg ? "-" : ""}${digits.slice(0, -2)}.${digits.slice(-2)}`;
}

function decimalToMinor(s: string): string {
  const t = s.trim().replace(/\s/g, "");
  if (!t) return "";
  // Either separator may be the decimal one; the last one wins, as in the reader.
  const lastDot = t.lastIndexOf(".");
  const lastComma = t.lastIndexOf(",");
  const at = Math.max(lastDot, lastComma);
  const whole = (at >= 0 ? t.slice(0, at) : t).replace(/[^\d-]/g, "");
  const frac = (at >= 0 ? t.slice(at + 1) : "").replace(/\D/g, "").padEnd(2, "0").slice(0, 2);
  if (!/^-?\d*$/.test(whole)) return "";
  return `${whole || "0"}${frac}`.replace(/^(-?)0+(?=\d)/, "$1");
}

/** How a field was found, as the key of a short word for the page. */
function foundKey(by: string | undefined): string {
  switch (by) {
    case "label":
    case "sender":
    case "first":
    case "received":
    case "payment":
    case "text":
    case "mailbox":
    case "declared":
      return `documents.found.${by}`;
    default:
      return "documents.found.none";
  }
}

/** The plural form of a count: Croatian has one, few and other. */
function plural(lang: Lang, n: number): "one" | "few" | "other" {
  if (lang === "hr") {
    const m10 = n % 10;
    const m100 = n % 100;
    if (m10 === 1 && m100 !== 11) return "one";
    if (m10 >= 2 && m10 <= 4 && (m100 < 12 || m100 > 14)) return "few";
    return "other";
  }
  return n === 1 ? "one" : "other";
}

function sureEnough(by: string | undefined): boolean {
  return by === "label" || by === "declared";
}

export default function DocumentsPage() {
  const t = useT();
  const { lang } = useLang();
  const { partyIds, partyName, multi } = useFinance();
  const key = partyIds.join(",");
  const [q, setQ] = useState("");
  const [term, setTerm] = useState("");
  useEffect(() => {
    const t = setTimeout(() => setTerm(q.trim()), 250);
    return () => clearTimeout(t);
  }, [q]);
  const [vendor, setVendor] = useState(ALL);
  const [month, setMonth] = useState(ALL);
  const months = useMemo(() => {
    const now = thisMonth();
    return Array.from({ length: 24 }, (_, i) => monthsBefore(now, i));
  }, []);
  const bounds = month === ALL ? {} : monthBounds(month);
  const docs = useFetch(
    () =>
      api.documents({
        party_ids: partyIds,
        q: term,
        vendor: vendor === ALL ? "" : vendor,
        ...bounds,
        limit: 200,
      }),
    0,
    [key, term, vendor, month],
  );
  const [openId, setOpenId] = useState<string | null>(null);
  const [uploadParty, setUploadParty] = useState("");
  const rows = useMemo(() => docs.data?.documents ?? [], [docs.data]);
  const sums = useMemo(() => {
    const by = new Map<string, bigint>();
    for (const d of rows) {
      if (!d.total_minor) continue;
      const c = d.currency || "EUR";
      by.set(c, (by.get(c) ?? 0n) + BigInt(d.total_minor));
    }
    return [...by.entries()];
  }, [rows]);
  const unpriced = rows.filter((d) => !d.total_minor).length;

  return (
    <>
      <PageTitle title={t("documents.title")} description={t("documents.description")}>
        <ScopeToggle className="md:hidden" />
        <div className="flex items-center gap-2">
          {partyIds.length > 1 ? (
            <Select value={uploadParty} onValueChange={setUploadParty}>
              <SelectTrigger className="w-44">
                <SelectValue placeholder={t("documents.upload_for")} />
              </SelectTrigger>
              <SelectContent>
                {partyIds.map((id) => (
                  <SelectItem key={id} value={id}>
                    {partyName(id)}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          ) : null}
          <UploadReceipt
            partyId={uploadParty || partyIds[0] || ""}
            onUploaded={(doc) => {
              docs.reload();
              setOpenId(doc.id);
            }}
          />
        </div>
      </PageTitle>

      <div className="flex flex-wrap items-center gap-2">
        <div className="relative min-w-64 flex-1">
          <Search className="text-muted-foreground pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2" />
          <Input
            value={q}
            onChange={(e) => setQ(e.target.value)}
            placeholder={t("documents.search_placeholder")}
            className="pl-8"
          />
        </div>
        <Select value={vendor} onValueChange={setVendor}>
          <SelectTrigger className="w-48">
            <SelectValue placeholder={t("common.vendor")} />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value={ALL}>{t("documents.all_vendors")}</SelectItem>
            {(docs.data?.vendors ?? []).map((v) => (
              <SelectItem key={v.vendor} value={v.vendor}>
                {v.vendor} · {v.count}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <Select value={month} onValueChange={setMonth}>
          <SelectTrigger className="w-40">
            <SelectValue placeholder={t("common.month")} />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value={ALL}>{t("common.any_month")}</SelectItem>
            {months.map((m) => (
              <SelectItem key={m} value={m}>
                {monthLabel(m)}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>

      {docs.data ? (
        <p className="text-muted-foreground text-xs">
          {t(`documents.count_${plural(lang, docs.data.total)}`, { n: docs.data.total })}
          {docs.data.total > rows.length ? ` (${t("common.showing", { n: rows.length })})` : ""}
          {sums.length ? " · " : ""}
          {sums.map(([c, m], i) => (
            <span key={c} className="tabular-nums">
              {i ? " + " : ""}
              {money(m.toString(), c)}
            </span>
          ))}
          {unpriced ? ` · ${t("documents.without_amount", { n: unpriced })}` : ""}
        </p>
      ) : null}

      <Card>
        <CardContent className="p-0">
          {docs.error ? (
            <p className="text-destructive p-4 text-sm">{docs.error}</p>
          ) : docs.loading && !docs.data ? (
            <Skeleton className="m-4 h-48" />
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="w-24">{t("common.date")}</TableHead>
                  <TableHead>{t("common.vendor")}</TableHead>
                  <TableHead className="hidden lg:table-cell">{t("documents.subject")}</TableHead>
                  <TableHead className="hidden md:table-cell">{t("common.number")}</TableHead>
                  <TableHead className="hidden xl:table-cell">{t("documents.mailbox")}</TableHead>
                  <TableHead className="text-right">{t("common.amount")}</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {rows.map((d) => {
                  const s = d.sources[0];
                  const guessedVendor = !sureEnough(d.found_by.vendor);
                  const guessedAmount = !sureEnough(d.found_by.amount);
                  return (
                    <TableRow key={d.id} className="cursor-pointer" onClick={() => setOpenId(d.id)}>
                      <TableCell className="text-xs whitespace-nowrap tabular-nums">
                        {d.doc_date || (
                          <span className="text-muted-foreground">{when(s?.received_at).slice(0, 11)}</span>
                        )}
                      </TableCell>
                      <TableCell className="max-w-48">
                        <div className="flex items-center gap-1.5">
                          <span
                            className={
                              "truncate " + (d.vendor ? "font-medium" : "text-muted-foreground italic")
                            }
                          >
                            {d.vendor || t("documents.unknown_vendor")}
                          </span>
                          {d.vendor && guessedVendor ? (
                            <Badge variant="outline" className="text-muted-foreground shrink-0 text-[10px]">
                              {t(foundKey(d.found_by.vendor))}
                            </Badge>
                          ) : null}
                          {multi ? (
                            <Badge variant="secondary" className="shrink-0 text-[10px]">
                              {partyName(d.party_id)}
                            </Badge>
                          ) : null}
                        </div>
                        <div className="text-muted-foreground max-w-48 truncate text-[11px] lg:hidden">
                          {s?.subject || d.filename}
                        </div>
                      </TableCell>
                      <TableCell className="text-muted-foreground hidden max-w-96 truncate text-xs lg:table-cell">
                        {s?.subject || d.filename || "—"}
                      </TableCell>
                      <TableCell className="text-muted-foreground hidden font-mono text-xs md:table-cell">
                        {d.invoice_no || "—"}
                      </TableCell>
                      <TableCell className="text-muted-foreground hidden max-w-56 truncate text-xs xl:table-cell">
                        {s?.sender || "—"}
                      </TableCell>
                      <TableCell className="text-right font-mono whitespace-nowrap tabular-nums">
                        {d.total_minor ? (
                          <span className={guessedAmount ? "text-muted-foreground" : undefined}>
                            {money(d.total_minor, d.currency || "EUR")}
                          </span>
                        ) : (
                          <span className="text-muted-foreground text-xs italic">
                            {d.extracted_at ? t("documents.found.none") : t("documents.reading")}
                          </span>
                        )}
                      </TableCell>
                    </TableRow>
                  );
                })}
                {docs.data && rows.length === 0 ? (
                  <TableRow>
                    <TableCell colSpan={6} className="text-muted-foreground py-10 text-center text-sm">
                      {term || vendor !== ALL || month !== ALL
                        ? t("common.nothing_matches")
                        : t("documents.nothing_pulled")}
                    </TableCell>
                  </TableRow>
                ) : null}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>

      <DocumentSheet
        id={openId}
        onClose={() => setOpenId(null)}
        onChanged={docs.reload}
        vendors={(docs.data?.vendors ?? []).map((v) => v.vendor)}
      />
    </>
  );
}

function DocumentSheet({
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
  const [doc, setDoc] = useState<Document | null>(null);
  const [url, setUrl] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState("");
  const [form, setForm] = useState({
    vendor: "",
    doc_date: "",
    amount: "",
    currency: "EUR",
    invoice_no: "",
    party_id: "",
  });
  const [editing, setEditing] = useState(false);
  const { parties, partyName } = useFinance();

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

  const s = doc?.sources[0];
  const hint = (key: string) => (
    <span className="text-muted-foreground ml-2 text-[10px]">{t(foundKey(doc?.found_by[key]))}</span>
  );

  return (
    <Sheet open={id !== null} onOpenChange={(o) => !o && onClose()}>
      <SheetContent className="flex w-full flex-col gap-4 overflow-y-auto sm:max-w-4xl">
        <SheetHeader className="pr-8">
          <SheetTitle className="flex flex-wrap items-center gap-2">
            {doc ? doc.vendor || t("documents.unknown_vendor_title") : t("documents.receipt")}
            {doc?.total_minor ? (
              <span className="font-mono text-base font-normal tabular-nums">
                {money(doc.total_minor, doc.currency || "EUR")}
              </span>
            ) : null}
            {doc?.declared ? (
              <Badge variant="outline" className="text-[10px]">
                {t("documents.corrected")}
              </Badge>
            ) : null}
          </SheetTitle>
          <SheetDescription>
            {doc ? (
              <>
                {doc.doc_date || t("documents.no_date")} · {doc.invoice_no || t("documents.no_number")} ·{" "}
                {doc.filename}
                {s ? ` · ${t("documents.from", { sender: s.sender })}` : ""}
              </>
            ) : (
              t("common.loading")
            )}
          </SheetDescription>
        </SheetHeader>

        {error ? <p className="text-destructive text-sm">{error}</p> : null}

        <div className="flex flex-wrap gap-1">
          <Button
            variant={editing ? "secondary" : "outline"}
            size="sm"
            onClick={() => setEditing((v) => !v)}
            disabled={!doc}
          >
            <Pencil /> {editing ? t("documents.close_editor") : t("documents.correct")}
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

        {editing && doc ? (
          <form
            className="grid gap-3 rounded-md border p-3 sm:grid-cols-2"
            onSubmit={(e) => {
              e.preventDefault();
              void act(
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
              );
            }}
          >
            <div className="sm:col-span-2">
              <Label className="text-xs">
                {t("documents.whose")} {hint("party")}
              </Label>
              <Select value={form.party_id} onValueChange={(v) => setForm({ ...form, party_id: v })}>
                <SelectTrigger className="mt-1">
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
            </div>
            <div className="sm:col-span-2">
              <Label className="text-xs">
                {t("common.vendor")} {hint("vendor")}
              </Label>
              <Input
                list="vendors"
                value={form.vendor}
                onChange={(e) => setForm({ ...form, vendor: e.target.value })}
                className="mt-1"
              />
              <datalist id="vendors">
                {vendors.map((v) => (
                  <option key={v} value={v} />
                ))}
              </datalist>
            </div>
            <div>
              <Label className="text-xs">
                {t("common.date")} {hint("date")}
              </Label>
              <Input
                type="date"
                value={form.doc_date}
                onChange={(e) => setForm({ ...form, doc_date: e.target.value })}
                className="mt-1"
              />
            </div>
            <div>
              <Label className="text-xs">
                {t("common.number")} {hint("invoice_no")}
              </Label>
              <Input
                value={form.invoice_no}
                onChange={(e) => setForm({ ...form, invoice_no: e.target.value })}
                className="mt-1 font-mono"
              />
            </div>
            <div>
              <Label className="text-xs">
                {t("common.amount")} {hint("amount")}
              </Label>
              <Input
                inputMode="decimal"
                value={form.amount}
                onChange={(e) => setForm({ ...form, amount: e.target.value })}
                placeholder="0.00"
                className="mt-1 font-mono"
              />
            </div>
            <div>
              <Label className="text-xs">{t("common.currency")}</Label>
              <Input
                value={form.currency}
                onChange={(e) => setForm({ ...form, currency: e.target.value.toUpperCase() })}
                maxLength={3}
                className="mt-1 font-mono uppercase"
              />
            </div>
            <div className="flex items-end gap-2 sm:col-span-2">
              <Button type="submit" size="sm" disabled={busy !== ""}>
                {busy === "save" ? t("common.saving") : t("common.save")}
              </Button>
              <span className="text-muted-foreground text-xs">{t("documents.clear_hint")}</span>
            </div>
          </form>
        ) : null}

        {doc && !editing ? (
          <dl className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1 text-xs">
            <dt className="text-muted-foreground">{t("documents.whose")}</dt>
            <dd>
              {partyName(doc.party_id)} {hint("party")}
            </dd>
            <dt className="text-muted-foreground">{t("common.vendor")}</dt>
            <dd>
              {doc.vendor || "—"} {hint("vendor")}
            </dd>
            <dt className="text-muted-foreground">{t("common.date")}</dt>
            <dd>
              {doc.doc_date || "—"} {hint("date")}
            </dd>
            <dt className="text-muted-foreground">{t("common.amount")}</dt>
            <dd className="font-mono tabular-nums">
              {doc.total_minor ? money(doc.total_minor, doc.currency || "EUR") : "—"} {hint("amount")}
            </dd>
            <dt className="text-muted-foreground">{t("common.number")}</dt>
            <dd className="font-mono">
              {doc.invoice_no || "—"} {hint("invoice_no")}
            </dd>
            {doc.sources.map((src) => (
              <div key={src.external_ref} className="contents">
                <dt className="text-muted-foreground">{t("documents.mail")}</dt>
                <dd className="truncate">
                  {src.sender} · {src.subject} · {when(src.received_at)}
                </dd>
              </div>
            ))}
            {doc.found_by.error ? (
              <>
                <dt className="text-muted-foreground">{t("documents.reader")}</dt>
                <dd className="text-amber-700 dark:text-amber-300">{doc.found_by.error}</dd>
              </>
            ) : null}
          </dl>
        ) : null}

        <div className="bg-muted/40 min-h-[60vh] flex-1 overflow-hidden rounded-md border">
          {url ? (
            <iframe title={t("documents.receipt")} src={url} className="h-[70vh] w-full" />
          ) : (
            <div className="text-muted-foreground flex h-[60vh] items-center justify-center gap-2 text-sm">
              <FileSearch className="size-4" />{" "}
              {error ? t("documents.file_error") : t("documents.file_loading")}
            </div>
          )}
        </div>
      </SheetContent>
    </Sheet>
  );
}
