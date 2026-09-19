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
import { useT } from "@/lib/i18n";

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
  const t = useT();
  const { parties, partyIds } = useFinance();
  const orgs = parties.filter((p) => p.kind === "org");
  const [party, setParty] = useState("");
  const chosen = party || orgs[0]?.id || partyIds[0] || "";
  return (
    <>
      <PageTitle title={t("parties.issuer.title")} description={t("parties.issuer.description")}>
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
  const t = useT();
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
      toast.success(t("parties.issuer_saved"));
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
          {t("parties.no_issuer")}
        </p>
      ) : null}

      <section>
        <SectionTitle title={t("parties.identity")} description={t("parties.identity.description")} />
        <Card>
          <CardContent className="grid gap-3 pt-4 sm:grid-cols-2">
            <TextField
              label={t("parties.legal_name")}
              value={form.legal_name}
              onChange={set("legal_name")}
              className="sm:col-span-2"
            />
            <div className="sm:col-span-2">
              <Label className="text-muted-foreground mb-1.5 block text-xs">
                {t("parties.address_lines")}
              </Label>
              <Textarea
                rows={2}
                value={form.address_lines.join("\n")}
                onChange={(e) => setForm({ ...form, address_lines: e.target.value.split("\n") })}
              />
            </div>
            <TextField label={t("parties.oib")} value={form.oib} onChange={set("oib")} mono />
            <TextField
              label={t("parties.vat_id")}
              value={form.vat_id}
              onChange={set("vat_id")}
              mono
              hint={t("parties.vat_id.hint")}
            />
          </CardContent>
        </Card>
      </section>

      <section>
        <SectionTitle title={t("parties.payment")} description={t("parties.payment.description")} />
        <Card>
          <CardContent className="grid gap-3 pt-4 sm:grid-cols-2">
            <TextField label={t("parties.iban")} value={form.iban} onChange={set("iban")} mono />
            <TextField label={t("parties.swift")} value={form.swift} onChange={set("swift")} mono />
            <TextField
              label={t("parties.bank")}
              value={form.bank_name}
              onChange={set("bank_name")}
              className="sm:col-span-2"
            />
            <TextField
              label={t("parties.due_days")}
              value={String(form.due_days)}
              onChange={(v) => setForm({ ...form, due_days: Number(v) || 0 })}
              mono
              hint={t("parties.due_days.hint")}
            />
          </CardContent>
        </Card>
      </section>

      <section>
        <SectionTitle title={t("parties.legal_footer")} description={t("parties.legal_footer.description")} />
        <Card>
          <CardContent className="grid gap-3 pt-4 sm:grid-cols-2">
            <TextField label={t("parties.competent_court")} value={form.court} onChange={set("court")} />
            <TextField
              label={t("parties.registration_no")}
              value={form.registration_no}
              onChange={set("registration_no")}
              mono
            />
            <TextField
              label={t("parties.share_capital")}
              value={form.share_capital}
              onChange={set("share_capital")}
              className="sm:col-span-2"
              placeholder={t("parties.share_capital.placeholder")}
            />
            <TextField
              label={t("parties.board_member")}
              value={form.board_member}
              onChange={set("board_member")}
            />
            <TextField label={t("parties.issued_by")} value={form.issued_by} onChange={set("issued_by")} />
            <TextField
              label={t("parties.place_of_issue")}
              value={form.place_of_issue}
              onChange={set("place_of_issue")}
            />
          </CardContent>
        </Card>
      </section>

      <section>
        <SectionTitle title={t("parties.numbering")} description={t("parties.numbering.description")} />
        <Card>
          <CardContent className="grid gap-3 pt-4 sm:grid-cols-3">
            <TextField label={t("parties.premises")} value={form.premises} onChange={set("premises")} mono />
            <TextField label={t("parties.device")} value={form.device} onChange={set("device")} mono />
            <TextField
              label={t("parties.operator_id")}
              value={form.operator_id}
              onChange={set("operator_id")}
              mono
            />
          </CardContent>
        </Card>
      </section>

      <div className="flex items-center gap-3">
        <Button onClick={() => void save()} disabled={busy || !form.legal_name}>
          {busy ? t("common.saving") : t("parties.save_issuer")}
        </Button>
        <span className="text-muted-foreground text-xs">{t("parties.applies_note")}</span>
      </div>
    </div>
  );
}
