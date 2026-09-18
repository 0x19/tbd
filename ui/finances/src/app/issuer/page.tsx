"use client";

// The issuer: who the invoices come from. The legal lines a Croatian
// invoice must carry, the bank it is paid into, and the numbering triple.
// Data, not template text: change an address here and every later invoice
// follows; issued ones keep what they were rendered with.
import { useEffect, useState } from "react";
import { toast } from "sonner";

import { useFinance } from "@/app/providers";
import { PageTitle, SectionTitle } from "@/components/kit";
import { TextField } from "@/components/text-field";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Skeleton } from "@/components/ui/skeleton";
import { Textarea } from "@/components/ui/textarea";
import { api } from "@/lib/api/client";
import { describe, useFetch } from "@/lib/api/hooks";
import type { IssuerProfile } from "@/lib/api/schema";

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

export default function IssuerPage() {
  const { parties, partyIds } = useFinance();
  const orgs = parties.filter((p) => p.kind === "org");
  const [party, setParty] = useState("");
  const chosen = party || orgs[0]?.id || partyIds[0] || "";
  return (
    <>
      <PageTitle
        title="Issuer"
        description="Who the invoices come from: the legal lines, the bank account, and the numbering. Printed on every invoice."
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
      {chosen ? <IssuerForm party={chosen} /> : null}
    </>
  );
}

function IssuerForm({ party }: { party: string }) {
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
  if (loaded.loading && !loaded.data) return <Skeleton className="h-96 w-full" />;
  if (loaded.error) return <p className="text-destructive text-sm">{loaded.error}</p>;
  const filled = !!loaded.data?.issuer?.legal_name;
  return (
    <div className="max-w-4xl space-y-6">
      {!filled ? (
        <p className="rounded-md border border-amber-500/40 bg-amber-500/10 p-3 text-sm">
          No issuer yet for this party. Invoices cannot be approved until the legal name, OIB, IBAN and
          address are set.
        </p>
      ) : null}

      <section>
        <SectionTitle title="Identity" description="The header block, and the tax identity beside it." />
        <Card>
          <CardContent className="grid gap-3 pt-4 sm:grid-cols-2">
            <TextField
              label="Legal name"
              value={form.legal_name}
              onChange={set("legal_name")}
              className="sm:col-span-2"
            />
            <div className="sm:col-span-2">
              <Label className="text-muted-foreground mb-1.5 block text-xs">Address, one line per row</Label>
              <Textarea
                rows={2}
                value={form.address_lines.join("\n")}
                onChange={(e) => setForm({ ...form, address_lines: e.target.value.split("\n") })}
              />
            </div>
            <TextField label="OIB" value={form.oib} onChange={set("oib")} mono />
            <TextField
              label="VAT ID"
              value={form.vat_id}
              onChange={set("vat_id")}
              mono
              hint="HR + OIB for EU trade."
            />
          </CardContent>
        </Card>
      </section>

      <section>
        <SectionTitle
          title="Payment"
          description="Where the client pays. The invoice number is the payment reference."
        />
        <Card>
          <CardContent className="grid gap-3 pt-4 sm:grid-cols-2">
            <TextField label="IBAN" value={form.iban} onChange={set("iban")} mono />
            <TextField label="SWIFT / BIC" value={form.swift} onChange={set("swift")} mono />
            <TextField
              label="Bank"
              value={form.bank_name}
              onChange={set("bank_name")}
              className="sm:col-span-2"
            />
            <TextField
              label="Due days"
              value={String(form.due_days)}
              onChange={(v) => setForm({ ...form, due_days: Number(v) || 0 })}
              mono
              hint="Due date = issue date + this."
            />
          </CardContent>
        </Card>
      </section>

      <section>
        <SectionTitle
          title="Legal footer"
          description="What a Croatian company invoice must state: court, registration, capital, who is responsible."
        />
        <Card>
          <CardContent className="grid gap-3 pt-4 sm:grid-cols-2">
            <TextField label="Competent court" value={form.court} onChange={set("court")} />
            <TextField
              label="Registration no. (MBS)"
              value={form.registration_no}
              onChange={set("registration_no")}
              mono
            />
            <TextField
              label="Share capital"
              value={form.share_capital}
              onChange={set("share_capital")}
              className="sm:col-span-2"
              placeholder="2,640.00 EUR, uplaćen u cijelosti / fully paid"
            />
            <TextField
              label="Management board member"
              value={form.board_member}
              onChange={set("board_member")}
            />
            <TextField
              label="Issued by (responsible person)"
              value={form.issued_by}
              onChange={set("issued_by")}
            />
            <TextField label="Place of issue" value={form.place_of_issue} onChange={set("place_of_issue")} />
          </CardContent>
        </Card>
      </section>

      <section>
        <SectionTitle
          title="Numbering"
          description="The legal triple: ordinal-premises-device-year. The ordinal is allocated at approval and never skips."
        />
        <Card>
          <CardContent className="grid gap-3 pt-4 sm:grid-cols-3">
            <TextField
              label="Premises (poslovni prostor)"
              value={form.premises}
              onChange={set("premises")}
              mono
            />
            <TextField label="Device (naplatni uređaj)" value={form.device} onChange={set("device")} mono />
            <TextField label="Operator ID" value={form.operator_id} onChange={set("operator_id")} mono />
          </CardContent>
        </Card>
      </section>

      <div className="flex items-center gap-3">
        <Button onClick={() => void save()} disabled={busy || !form.legal_name}>
          {busy ? "Saving…" : "Save issuer"}
        </Button>
        <span className="text-muted-foreground text-xs">
          Applies to drafts and future invoices; issued ones keep their print.
        </span>
      </div>
    </div>
  );
}
