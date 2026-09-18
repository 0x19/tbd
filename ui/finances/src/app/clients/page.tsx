"use client";

// Clients: who is billed. A table of them, and a panel per client with its
// details, the VAT treatment that decides the invoice's note, and the line
// templates a new draft for it starts from.
import { Plus } from "lucide-react";
import { useEffect, useState } from "react";
import { toast } from "sonner";

import { useFinance } from "@/app/providers";
import { PageTitle } from "@/components/kit";
import { TextField } from "@/components/text-field";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Sheet, SheetContent, SheetDescription, SheetHeader, SheetTitle } from "@/components/ui/sheet";
import { Skeleton } from "@/components/ui/skeleton";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Textarea } from "@/components/ui/textarea";
import { api } from "@/lib/api/client";
import { describe, useFetch } from "@/lib/api/hooks";
import type { ClientProfile, LineTemplate } from "@/lib/api/schema";

const EMPTY_CLIENT = (party_id: string): ClientProfile => ({
  id: "",
  party_id,
  name: "",
  address_lines: [],
  country_code: "",
  tax_id: "",
  vat_treatment: "outside_scope_non_eu",
  recipients: [],
  currency: "EUR",
  archived: false,
});

const TREATMENTS: Record<string, string> = {
  outside_scope_non_eu: "Outside EU · reverse charge, no VAT",
  reverse_charge_eu: "EU business · reverse charge",
  standard_hr: "Croatia · 25% VAT",
  exempt_issuer: "Issuer not in VAT system",
};

export default function ClientsPage() {
  const { parties, partyIds, partyName, multi } = useFinance();
  const orgs = parties.filter((p) => p.kind === "org");
  const [party, setParty] = useState("");
  const chosen = party || orgs[0]?.id || partyIds[0] || "";
  const loaded = useFetch(() => api.clients(chosen ? [chosen] : []), 0, [chosen]);
  const [editing, setEditing] = useState<ClientProfile | null>(null);
  const list = loaded.data?.clients ?? [];
  return (
    <>
      <PageTitle
        title="Clients"
        description="Who is billed. The VAT treatment decides the note each invoice carries."
      >
        <div className="flex items-center gap-2">
          {parties.length > 1 ? (
            <Select value={chosen} onValueChange={setParty}>
              <SelectTrigger className="w-44">
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
          ) : null}
          <Button size="sm" onClick={() => setEditing(EMPTY_CLIENT(chosen))} disabled={!chosen}>
            <Plus /> New client
          </Button>
        </div>
      </PageTitle>

      <Card>
        <CardContent className="p-0">
          {loaded.error ? (
            <p className="text-destructive p-4 text-sm">{loaded.error}</p>
          ) : loaded.loading && !loaded.data ? (
            <Skeleton className="m-4 h-40" />
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Client</TableHead>
                  <TableHead className="hidden md:table-cell">Address</TableHead>
                  <TableHead>Country</TableHead>
                  <TableHead className="hidden lg:table-cell">Tax ID</TableHead>
                  <TableHead>VAT</TableHead>
                  <TableHead>Currency</TableHead>
                  {multi ? <TableHead className="hidden xl:table-cell">Billed by</TableHead> : null}
                </TableRow>
              </TableHeader>
              <TableBody>
                {list.map((c) => (
                  <TableRow key={c.id} className="cursor-pointer" onClick={() => setEditing(c)}>
                    <TableCell className="font-medium">{c.name}</TableCell>
                    <TableCell className="text-muted-foreground hidden max-w-80 truncate text-xs md:table-cell">
                      {c.address_lines.join(", ")}
                    </TableCell>
                    <TableCell className="font-mono text-xs">{c.country_code}</TableCell>
                    <TableCell className="text-muted-foreground hidden font-mono text-xs lg:table-cell">
                      {c.tax_id || "—"}
                    </TableCell>
                    <TableCell>
                      <Badge variant="outline" className="text-[10px]">
                        {c.vat_treatment.replace(/_/g, " ")}
                      </Badge>
                    </TableCell>
                    <TableCell className="font-mono text-xs">{c.currency}</TableCell>
                    {multi ? (
                      <TableCell className="text-muted-foreground hidden text-xs xl:table-cell">
                        {partyName(c.party_id)}
                      </TableCell>
                    ) : null}
                  </TableRow>
                ))}
                {loaded.data && list.length === 0 ? (
                  <TableRow>
                    <TableCell colSpan={7} className="text-muted-foreground py-10 text-center text-sm">
                      No clients yet. Add one to start invoicing.
                    </TableCell>
                  </TableRow>
                ) : null}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>

      <ClientSheet client={editing} onClose={() => setEditing(null)} onSaved={loaded.reload} />
    </>
  );
}

function ClientSheet({
  client,
  onClose,
  onSaved,
}: {
  client: ClientProfile | null;
  onClose: () => void;
  onSaved: () => void;
}) {
  const [form, setForm] = useState<ClientProfile | null>(client);
  const [busy, setBusy] = useState(false);
  useEffect(() => setForm(client), [client]);
  const set = (k: keyof ClientProfile) => (v: string) => form && setForm({ ...form, [k]: v });
  const save = async () => {
    if (!form) return;
    setBusy(true);
    try {
      const { archived: _archived, ...input } = form;
      const r = await api.upsertClient(input);
      toast.success("Client saved.");
      if (r.client) setForm(r.client);
      onSaved();
    } catch (e) {
      toast.error(describe(e));
    } finally {
      setBusy(false);
    }
  };
  return (
    <Sheet open={client !== null} onOpenChange={(o) => !o && onClose()}>
      <SheetContent className="flex w-full flex-col gap-4 overflow-y-auto sm:max-w-2xl">
        <SheetHeader className="pr-8">
          <SheetTitle>{form?.id ? form.name || "Client" : "New client"}</SheetTitle>
          <SheetDescription>
            {form?.id
              ? `${TREATMENTS[form.vat_treatment] ?? form.vat_treatment} · billed in ${form.currency}`
              : "Name, address and country print on the invoice; the VAT treatment decides its note."}
          </SheetDescription>
        </SheetHeader>
        {form ? (
          <>
            <div className="grid gap-3 sm:grid-cols-2">
              <TextField label="Name" value={form.name} onChange={set("name")} className="sm:col-span-2" />
              <div className="sm:col-span-2">
                <Label className="text-muted-foreground mb-1.5 block text-xs">
                  Address, one line per row
                </Label>
                <Textarea
                  rows={3}
                  value={form.address_lines.join("\n")}
                  onChange={(e) => setForm({ ...form, address_lines: e.target.value.split("\n") })}
                />
              </div>
              <TextField
                label="Country (ISO code)"
                value={form.country_code}
                onChange={(v) => set("country_code")(v.toUpperCase())}
                mono
              />
              <TextField label="Tax ID (their number)" value={form.tax_id} onChange={set("tax_id")} mono />
              <div>
                <Label className="text-muted-foreground mb-1.5 block text-xs">VAT treatment</Label>
                <Select value={form.vat_treatment} onValueChange={set("vat_treatment")}>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {Object.entries(TREATMENTS).map(([v, label]) => (
                      <SelectItem key={v} value={v}>
                        {label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <TextField
                label="Currency"
                value={form.currency}
                onChange={(v) => set("currency")(v.toUpperCase())}
                mono
              />
              <div className="sm:col-span-2">
                <Label className="text-muted-foreground mb-1.5 block text-xs">
                  Recipients (emails, one per row) — for sending, later
                </Label>
                <Textarea
                  rows={2}
                  value={form.recipients.join("\n")}
                  onChange={(e) =>
                    setForm({ ...form, recipients: e.target.value.split("\n").filter(Boolean) })
                  }
                />
              </div>
            </div>
            <div className="flex gap-2">
              <Button onClick={() => void save()} disabled={busy || !form.name || !form.country_code}>
                {busy ? "Saving…" : form.id ? "Save client" : "Create client"}
              </Button>
              <Button variant="ghost" onClick={onClose}>
                Close
              </Button>
            </div>
            {form.id ? (
              <TemplatesEditor client={form.id} />
            ) : (
              <p className="text-muted-foreground text-xs">
                Line templates can be added once the client is created.
              </p>
            )}
          </>
        ) : null}
      </SheetContent>
    </Sheet>
  );
}

const MODES = [
  ["fixed", "Fixed — on every draft with this price"],
  ["variable", "Variable — on every draft, price asked each month"],
  ["optional", "Optional — offered, off until chosen"],
] as const;

/** The rows a draft for this client starts from. */
function TemplatesEditor({ client }: { client: string }) {
  const loaded = useFetch(() => api.lineTemplates(client), 0, [client]);
  const [rows, setRows] = useState<LineTemplate[]>([]);
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    setRows(loaded.data?.templates ?? []);
  }, [loaded.data]);
  const blank = (): LineTemplate => ({
    id: "",
    client_id: client,
    position: rows.length + 1,
    description: "",
    mode: "fixed",
    quantity_milli: "1000",
    unit_price_minor: "0",
    enabled: true,
  });
  const set = (i: number, patch: Partial<LineTemplate>) =>
    setRows(rows.map((r, j) => (j === i ? { ...r, ...patch } : r)));
  const save = async (t: LineTemplate) => {
    setBusy(true);
    try {
      await api.upsertLineTemplate(client, t);
      toast.success("Template saved.");
      loaded.reload();
    } catch (e) {
      toast.error(describe(e));
    } finally {
      setBusy(false);
    }
  };
  const remove = async (t: LineTemplate, i: number) => {
    if (!t.id) return setRows(rows.filter((_, j) => j !== i));
    setBusy(true);
    try {
      await api.deleteLineTemplate(client, t.id);
      loaded.reload();
    } catch (e) {
      toast.error(describe(e));
    } finally {
      setBusy(false);
    }
  };
  return (
    <div className="border-t pt-4">
      <div className="mb-2 flex items-center justify-between">
        <div>
          <Label className="text-xs font-medium">Line templates</Label>
          <p className="text-muted-foreground text-xs">
            What a new draft for this client starts with, in order.
          </p>
        </div>
        <Button variant="outline" size="sm" onClick={() => setRows([...rows, blank()])}>
          <Plus /> Row
        </Button>
      </div>
      <div className="space-y-2">
        {rows.map((t, i) => (
          <div key={t.id || `new-${i}`} className="space-y-2 rounded-md border p-2">
            <div className="grid gap-2 sm:grid-cols-[3rem_1fr]">
              <Input
                value={String(t.position)}
                inputMode="numeric"
                className="font-mono"
                title="Position"
                onChange={(e) => set(i, { position: Number(e.target.value) || 0 })}
              />
              <Textarea
                rows={2}
                value={t.description}
                className="min-h-9 text-sm"
                placeholder="Description as printed"
                onChange={(e) => set(i, { description: e.target.value })}
              />
            </div>
            <div className="grid gap-2 sm:grid-cols-[1fr_6rem_8rem_auto]">
              <Select value={t.mode} onValueChange={(v) => set(i, { mode: v })}>
                <SelectTrigger className="text-xs">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {MODES.map(([v, label]) => (
                    <SelectItem key={v} value={v}>
                      {label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              <Input
                value={(Number(t.quantity_milli) / 1000).toString()}
                inputMode="decimal"
                className="text-right font-mono"
                title="Quantity"
                onChange={(e) =>
                  set(i, {
                    quantity_milli: String(
                      Math.round(Number(e.target.value.replace(",", ".")) * 1000) || 1000,
                    ),
                  })
                }
              />
              <Input
                value={(Number(t.unit_price_minor) / 100).toFixed(2)}
                inputMode="decimal"
                className="text-right font-mono"
                title={t.mode === "variable" ? "Only used when there is no previous invoice" : "Unit price"}
                onChange={(e) =>
                  set(i, {
                    unit_price_minor: String(Math.round(Number(e.target.value.replace(",", ".")) * 100) || 0),
                  })
                }
              />
              <div className="flex gap-1">
                <Button
                  size="sm"
                  variant="outline"
                  disabled={busy || !t.description.trim()}
                  onClick={() => void save(t)}
                >
                  Save
                </Button>
                <Button
                  size="sm"
                  variant="ghost"
                  disabled={busy}
                  onClick={() => void remove(t, i)}
                  aria-label="Remove"
                >
                  ×
                </Button>
              </div>
            </div>
          </div>
        ))}
        {rows.length === 0 ? (
          <p className="text-muted-foreground text-xs">
            No templates: a draft starts from the last invoice to this client.
          </p>
        ) : null}
      </div>
    </div>
  );
}
