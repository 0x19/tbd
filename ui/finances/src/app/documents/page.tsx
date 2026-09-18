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
import { api, pdfUrl } from "@/lib/api/client";
import { describe, useFetch } from "@/lib/api/hooks";
import type { Document } from "@/lib/api/schema";
import { money, monthLabel, monthsBefore, thisMonth, when } from "@/lib/format";

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

/** How a field was found, as a short word for the page. */
function foundLabel(by: string | undefined): string {
  switch (by) {
    case "label":
      return "read";
    case "sender":
      return "from sender";
    case "first":
      return "guessed";
    case "received":
      return "mail date";
    case "payment":
      return "from the account that paid";
    case "text":
      return "named in the document";
    case "mailbox":
      return "the mailbox's";
    case "declared":
      return "set by you";
    default:
      return "not found";
  }
}

function sureEnough(by: string | undefined): boolean {
  return by === "label" || by === "declared";
}

export default function DocumentsPage() {
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
      <PageTitle
        title="Receipts"
        description="Every supplier document pulled from your mailboxes, read for vendor, date and amount. Click a row to see it and correct what was read."
      >
        <ScopeToggle className="md:hidden" />
      </PageTitle>

      <div className="flex flex-wrap items-center gap-2">
        <div className="relative min-w-64 flex-1">
          <Search className="text-muted-foreground pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2" />
          <Input
            value={q}
            onChange={(e) => setQ(e.target.value)}
            placeholder="Search vendor, number, subject, or anything in the document"
            className="pl-8"
          />
        </div>
        <Select value={vendor} onValueChange={setVendor}>
          <SelectTrigger className="w-48">
            <SelectValue placeholder="Vendor" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value={ALL}>All vendors</SelectItem>
            {(docs.data?.vendors ?? []).map((v) => (
              <SelectItem key={v.vendor} value={v.vendor}>
                {v.vendor} · {v.count}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <Select value={month} onValueChange={setMonth}>
          <SelectTrigger className="w-40">
            <SelectValue placeholder="Month" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value={ALL}>Any month</SelectItem>
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
          {docs.data.total} {docs.data.total === 1 ? "receipt" : "receipts"}
          {docs.data.total > rows.length ? ` (showing ${rows.length})` : ""}
          {sums.length ? " · " : ""}
          {sums.map(([c, m], i) => (
            <span key={c} className="tabular-nums">
              {i ? " + " : ""}
              {money(m.toString(), c)}
            </span>
          ))}
          {unpriced ? ` · ${unpriced} without an amount yet` : ""}
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
                  <TableHead className="w-24">Date</TableHead>
                  <TableHead>Vendor</TableHead>
                  <TableHead className="hidden lg:table-cell">Subject</TableHead>
                  <TableHead className="hidden md:table-cell">Number</TableHead>
                  <TableHead className="hidden xl:table-cell">Mailbox</TableHead>
                  <TableHead className="text-right">Amount</TableHead>
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
                            {d.vendor || "unknown vendor"}
                          </span>
                          {d.vendor && guessedVendor ? (
                            <Badge variant="outline" className="text-muted-foreground shrink-0 text-[10px]">
                              {foundLabel(d.found_by.vendor)}
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
                            {d.extracted_at ? "not found" : "reading…"}
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
                        ? "Nothing matches."
                        : "Nothing pulled yet. Link a mailbox under Connectors and pull."}
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
    <span className="text-muted-foreground ml-2 text-[10px]">{foundLabel(doc?.found_by[key])}</span>
  );

  return (
    <Sheet open={id !== null} onOpenChange={(o) => !o && onClose()}>
      <SheetContent className="flex w-full flex-col gap-4 overflow-y-auto sm:max-w-4xl">
        <SheetHeader className="pr-8">
          <SheetTitle className="flex flex-wrap items-center gap-2">
            {doc ? doc.vendor || "Unknown vendor" : "Receipt"}
            {doc?.total_minor ? (
              <span className="font-mono text-base font-normal tabular-nums">
                {money(doc.total_minor, doc.currency || "EUR")}
              </span>
            ) : null}
            {doc?.declared ? (
              <Badge variant="outline" className="text-[10px]">
                corrected
              </Badge>
            ) : null}
          </SheetTitle>
          <SheetDescription>
            {doc ? (
              <>
                {doc.doc_date || "no date"} · {doc.invoice_no || "no number"} · {doc.filename}
                {s ? ` · from ${s.sender}` : ""}
              </>
            ) : (
              "Loading…"
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
            <Pencil /> {editing ? "Close editor" : "Correct"}
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
                doc.declared ? "Read again; your corrections kept." : "Read again.",
              )
            }
          >
            <RefreshCw className={busy === "extract" ? "animate-spin" : undefined} /> Read again
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
            <Download /> Download
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
                "Saved. These fields are yours now; a re-read will not change them.",
              );
            }}
          >
            <div className="sm:col-span-2">
              <Label className="text-xs">Whose {hint("party")}</Label>
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
              <Label className="text-xs">Vendor {hint("vendor")}</Label>
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
              <Label className="text-xs">Date {hint("date")}</Label>
              <Input
                type="date"
                value={form.doc_date}
                onChange={(e) => setForm({ ...form, doc_date: e.target.value })}
                className="mt-1"
              />
            </div>
            <div>
              <Label className="text-xs">Number {hint("invoice_no")}</Label>
              <Input
                value={form.invoice_no}
                onChange={(e) => setForm({ ...form, invoice_no: e.target.value })}
                className="mt-1 font-mono"
              />
            </div>
            <div>
              <Label className="text-xs">Amount {hint("amount")}</Label>
              <Input
                inputMode="decimal"
                value={form.amount}
                onChange={(e) => setForm({ ...form, amount: e.target.value })}
                placeholder="0.00"
                className="mt-1 font-mono"
              />
            </div>
            <div>
              <Label className="text-xs">Currency</Label>
              <Input
                value={form.currency}
                onChange={(e) => setForm({ ...form, currency: e.target.value.toUpperCase() })}
                maxLength={3}
                className="mt-1 font-mono uppercase"
              />
            </div>
            <div className="flex items-end gap-2 sm:col-span-2">
              <Button type="submit" size="sm" disabled={busy !== ""}>
                {busy === "save" ? "Saving…" : "Save"}
              </Button>
              <span className="text-muted-foreground text-xs">
                Leave a field empty to clear it. Saving marks all of them as yours.
              </span>
            </div>
          </form>
        ) : null}

        {doc && !editing ? (
          <dl className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1 text-xs">
            <dt className="text-muted-foreground">Whose</dt>
            <dd>
              {partyName(doc.party_id)} {hint("party")}
            </dd>
            <dt className="text-muted-foreground">Vendor</dt>
            <dd>
              {doc.vendor || "—"} {hint("vendor")}
            </dd>
            <dt className="text-muted-foreground">Date</dt>
            <dd>
              {doc.doc_date || "—"} {hint("date")}
            </dd>
            <dt className="text-muted-foreground">Amount</dt>
            <dd className="font-mono tabular-nums">
              {doc.total_minor ? money(doc.total_minor, doc.currency || "EUR") : "—"} {hint("amount")}
            </dd>
            <dt className="text-muted-foreground">Number</dt>
            <dd className="font-mono">
              {doc.invoice_no || "—"} {hint("invoice_no")}
            </dd>
            {doc.sources.map((src) => (
              <div key={src.external_ref} className="contents">
                <dt className="text-muted-foreground">Mail</dt>
                <dd className="truncate">
                  {src.sender} · {src.subject} · {when(src.received_at)}
                </dd>
              </div>
            ))}
            {doc.found_by.error ? (
              <>
                <dt className="text-muted-foreground">Reader</dt>
                <dd className="text-amber-700 dark:text-amber-300">{doc.found_by.error}</dd>
              </>
            ) : null}
          </dl>
        ) : null}

        <div className="bg-muted/40 min-h-[60vh] flex-1 overflow-hidden rounded-md border">
          {url ? (
            <iframe title="Receipt" src={url} className="h-[70vh] w-full" />
          ) : (
            <div className="text-muted-foreground flex h-[60vh] items-center justify-center gap-2 text-sm">
              <FileSearch className="size-4" /> {error ? "Could not load the file." : "Loading the file…"}
            </div>
          )}
        </div>
      </SheetContent>
    </Sheet>
  );
}
