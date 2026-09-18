"use client";

// The issuer profile (what the footer of a Croatian invoice must carry) and
// the clients. Data, not template text: change an address here and every
// later invoice follows.
import { Plus } from "lucide-react";
import { useEffect, useState } from "react";
import { toast } from "sonner";

import { useFinance } from "@/app/providers";
import { PageTitle } from "@/components/kit";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";
import { api } from "@/lib/api/client";
import { describe, useFetch } from "@/lib/api/hooks";
import type { ClientProfile, IssuerProfile, LineTemplate } from "@/lib/api/schema";

const EMPTY_ISSUER = (party_id: string): IssuerProfile => ({
  party_id,
  legal_name: "",
  address_lines: [],
  oib: "",
  vat_id: "",
  iban: "",
  swift: "",
  bank_name: "",
  court: "",
  registration_no: "",
  share_capital: "",
  board_member: "",
  issued_by: "",
  place_of_issue: "",
  operator_id: "1",
  premises: "1",
  device: "1",
  due_days: 15,
});

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

export default function SettingsPage() {
  const { parties, partyIds } = useFinance();
  const orgs = parties.filter((p) => p.kind === "org");
  const [party, setParty] = useState("");
  const chosen = party || orgs[0]?.id || partyIds[0] || "";
  return (
    <>
      <PageTitle
        title="Issuer & clients"
        description="Who issues, and who is billed. Both print on every invoice."
      >
        {parties.length > 1 ? (
          <Select value={chosen} onValueChange={setParty}>
            <SelectTrigger className="w-48">
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
      </PageTitle>
      {chosen ? (
        <div className="grid gap-6 xl:grid-cols-2">
          <IssuerCard party={chosen} />
          <ClientsCard party={chosen} />
        </div>
      ) : null}
    </>
  );
}

function IssuerCard({ party }: { party: string }) {
  const loaded = useFetch(() => api.issuer(party), 0, [party]);
  const [form, setForm] = useState<IssuerProfile>(EMPTY_ISSUER(party));
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    setForm(loaded.data?.issuer ?? EMPTY_ISSUER(party));
  }, [loaded.data, party]);
  const set = (k: keyof IssuerProfile) => (v: string) => setForm({ ...form, [k]: v });
  const save = async () => {
    setBusy(true);
    try {
      const r = await api.upsertIssuer({ ...form, due_days: Number(form.due_days) || 15 });
      if (r.issuer) setForm(r.issuer);
      toast.success("Issuer saved.");
    } catch (e) {
      toast.error(describe(e));
    } finally {
      setBusy(false);
    }
  };
  return (
    <Card>
      <CardHeader>
        <CardTitle>Issuer</CardTitle>
        <CardDescription>The legal lines a Croatian invoice must carry.</CardDescription>
      </CardHeader>
      <CardContent className="grid gap-3 sm:grid-cols-2">
        <F label="Legal name" v={form.legal_name} on={set("legal_name")} className="sm:col-span-2" />
        <div className="sm:col-span-2">
          <Label className="text-muted-foreground mb-1.5 block text-xs">Address, one line per row</Label>
          <Textarea
            rows={2}
            value={form.address_lines.join("\n")}
            onChange={(e) => setForm({ ...form, address_lines: e.target.value.split("\n") })}
          />
        </div>
        <F label="OIB" v={form.oib} on={set("oib")} mono />
        <F label="VAT ID" v={form.vat_id} on={set("vat_id")} mono />
        <F label="IBAN" v={form.iban} on={set("iban")} mono />
        <F label="SWIFT / BIC" v={form.swift} on={set("swift")} mono />
        <F label="Bank" v={form.bank_name} on={set("bank_name")} className="sm:col-span-2" />
        <F label="Competent court" v={form.court} on={set("court")} />
        <F label="Registration no. (MBS)" v={form.registration_no} on={set("registration_no")} mono />
        <F label="Share capital" v={form.share_capital} on={set("share_capital")} className="sm:col-span-2" />
        <F label="Management board member" v={form.board_member} on={set("board_member")} />
        <F label="Issued by (responsible person)" v={form.issued_by} on={set("issued_by")} />
        <F label="Place of issue" v={form.place_of_issue} on={set("place_of_issue")} />
        <F label="Operator ID" v={form.operator_id} on={set("operator_id")} mono />
        <F label="Premises (poslovni prostor)" v={form.premises} on={set("premises")} mono />
        <F label="Device (naplatni uređaj)" v={form.device} on={set("device")} mono />
        <F
          label="Due days"
          v={String(form.due_days)}
          on={(v) => setForm({ ...form, due_days: Number(v) || 0 })}
          mono
        />
        <div className="sm:col-span-2">
          <Button onClick={() => void save()} disabled={busy || !form.legal_name}>
            {busy ? "Saving…" : "Save issuer"}
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}

function ClientsCard({ party }: { party: string }) {
  const loaded = useFetch(() => api.clients([party]), 0, [party]);
  const [editing, setEditing] = useState<ClientProfile | null>(null);
  const [busy, setBusy] = useState(false);
  const list = loaded.data?.clients ?? [];
  const save = async () => {
    if (!editing) return;
    setBusy(true);
    try {
      const { archived: _archived, ...input } = editing;
      await api.upsertClient(input);
      toast.success("Client saved.");
      setEditing(null);
      loaded.reload();
    } catch (e) {
      toast.error(describe(e));
    } finally {
      setBusy(false);
    }
  };
  const set = (k: keyof ClientProfile) => (v: string) => editing && setEditing({ ...editing, [k]: v });
  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between">
        <div>
          <CardTitle>Clients</CardTitle>
          <CardDescription>The VAT treatment decides the note the invoice carries.</CardDescription>
        </div>
        <Button variant="outline" size="sm" onClick={() => setEditing(EMPTY_CLIENT(party))}>
          <Plus /> Client
        </Button>
      </CardHeader>
      <CardContent className="space-y-3">
        {list.map((c) => (
          <button
            key={c.id}
            type="button"
            className="hover:bg-muted/50 flex w-full items-center justify-between rounded-md border px-3 py-2 text-left text-sm"
            onClick={() => setEditing(c)}
          >
            <span>
              <span className="font-medium">{c.name}</span>
              <span className="text-muted-foreground ml-2 text-xs">
                {c.address_lines.join(", ")} · {c.country_code}
              </span>
            </span>
            <Badge variant="outline" className="text-[10px]">
              {c.vat_treatment.replace(/_/g, " ")}
            </Badge>
          </button>
        ))}
        {editing ? (
          <div className="grid gap-3 rounded-md border p-3 sm:grid-cols-2">
            <F label="Name" v={editing.name} on={set("name")} className="sm:col-span-2" />
            <div className="sm:col-span-2">
              <Label className="text-muted-foreground mb-1.5 block text-xs">Address, one line per row</Label>
              <Textarea
                rows={2}
                value={editing.address_lines.join("\n")}
                onChange={(e) => setEditing({ ...editing, address_lines: e.target.value.split("\n") })}
              />
            </div>
            <F
              label="Country (ISO code)"
              v={editing.country_code}
              on={(v) => set("country_code")(v.toUpperCase())}
              mono
            />
            <F label="Tax ID (their number)" v={editing.tax_id} on={set("tax_id")} mono />
            <div>
              <Label className="text-muted-foreground mb-1.5 block text-xs">VAT treatment</Label>
              <Select value={editing.vat_treatment} onValueChange={set("vat_treatment")}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="outside_scope_non_eu">Outside EU · reverse charge, no VAT</SelectItem>
                  <SelectItem value="reverse_charge_eu">EU business · reverse charge</SelectItem>
                  <SelectItem value="standard_hr">Croatia · 25% VAT</SelectItem>
                  <SelectItem value="exempt_issuer">Issuer not in VAT system</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <F label="Currency" v={editing.currency} on={(v) => set("currency")(v.toUpperCase())} mono />
            <div className="sm:col-span-2">
              <Label className="text-muted-foreground mb-1.5 block text-xs">
                Recipients (emails, one per row) — for sending, later
              </Label>
              <Textarea
                rows={2}
                value={editing.recipients.join("\n")}
                onChange={(e) =>
                  setEditing({ ...editing, recipients: e.target.value.split("\n").filter(Boolean) })
                }
              />
            </div>
            <div className="flex gap-2 sm:col-span-2">
              <Button onClick={() => void save()} disabled={busy || !editing.name || !editing.country_code}>
                {busy ? "Saving…" : "Save client"}
              </Button>
              <Button variant="ghost" onClick={() => setEditing(null)}>
                Cancel
              </Button>
            </div>
            {editing.id ? <TemplatesEditor client={editing.id} /> : null}
          </div>
        ) : null}
      </CardContent>
    </Card>
  );
}

function F({
  label,
  v,
  on,
  mono,
  className,
}: {
  label: string;
  v: string;
  on: (v: string) => void;
  mono?: boolean;
  className?: string;
}) {
  return (
    <div className={className}>
      <Label className="text-muted-foreground mb-1.5 block text-xs">{label}</Label>
      <Input value={v} onChange={(e) => on(e.target.value)} className={mono ? "font-mono" : undefined} />
    </div>
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
    <div className="sm:col-span-2">
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
          <div
            key={t.id || `new-${i}`}
            className="grid gap-2 rounded-md border p-2 sm:grid-cols-[3rem_1fr_11rem_6rem_8rem_auto]"
          >
            <Input
              value={String(t.position)}
              inputMode="numeric"
              className="font-mono"
              onChange={(e) => set(i, { position: Number(e.target.value) || 0 })}
            />
            <Textarea
              rows={2}
              value={t.description}
              className="min-h-9 text-sm"
              placeholder="Description as printed"
              onChange={(e) => set(i, { description: e.target.value })}
            />
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
                  quantity_milli: String(Math.round(Number(e.target.value.replace(",", ".")) * 1000) || 1000),
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
