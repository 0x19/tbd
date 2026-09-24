"use client";

// The issuer: who the invoices come from. The legal lines a Croatian
// invoice must carry, the bank it is paid into, and the numbering triple.
// Data, not template text: change an address here and every later invoice
// follows; issued ones keep what they were rendered with.
import { Plus } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { toast } from "sonner";

import { useFinance } from "@/app/providers";
import { PageTitle, SectionTitle } from "@/components/kit";
import { TextField } from "@/components/text-field";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Label } from "@/components/ui/label";
import { Sheet, SheetContent, SheetDescription, SheetHeader, SheetTitle } from "@/components/ui/sheet";
import { Skeleton } from "@/components/ui/skeleton";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
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
  const { parties, partyName, reload } = useFinance();
  const orgs = useMemo(() => parties.filter((p) => p.kind === "org"), [parties]);
  const key = orgs.map((o) => o.id).join(",");
  const issuers = useFetch(() => api.issuers(orgs.map((o) => o.id)), 0, [key]);
  const [editing, setEditing] = useState<string | null>(null);
  const [adding, setAdding] = useState(false);
  const profileOf = (party: string) => issuers.data?.issuers.find((i) => i.party_id === party);
  return (
    <>
      <PageTitle title={t("parties.issuers.title")} description={t("parties.issuers.description")}>
        <Button size="sm" onClick={() => setAdding(true)}>
          <Plus /> {t("parties.add_company")}
        </Button>
      </PageTitle>

      <Card>
        <CardContent className="p-0">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>{t("parties.col.company")}</TableHead>
                <TableHead className="hidden sm:table-cell">{t("parties.col.oib")}</TableHead>
                <TableHead className="hidden md:table-cell">{t("parties.col.iban")}</TableHead>
                <TableHead className="hidden lg:table-cell">{t("parties.col.place")}</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {orgs.map((o) => {
                const p = profileOf(o.id);
                return (
                  <TableRow key={o.id} className="cursor-pointer" onClick={() => setEditing(o.id)}>
                    <TableCell className="font-medium">
                      {p?.legal_name || partyName(o.id)}
                      {issuers.data && !p?.legal_name ? (
                        <Badge variant="outline" className="ml-2 text-[10px]">
                          {t("parties.no_profile")}
                        </Badge>
                      ) : null}
                    </TableCell>
                    <TableCell className="hidden font-mono text-xs sm:table-cell">{p?.oib || "—"}</TableCell>
                    <TableCell className="hidden font-mono text-xs md:table-cell">{p?.iban || "—"}</TableCell>
                    <TableCell className="text-muted-foreground hidden text-xs lg:table-cell">
                      {p?.place_of_issue || "—"}
                    </TableCell>
                  </TableRow>
                );
              })}
              {orgs.length === 0 ? (
                <TableRow>
                  <TableCell colSpan={4} className="text-muted-foreground py-10 text-center text-sm">
                    {t("parties.no_companies")}
                  </TableCell>
                </TableRow>
              ) : null}
            </TableBody>
          </Table>
        </CardContent>
      </Card>

      <Sheet open={editing !== null} onOpenChange={(o) => !o && setEditing(null)}>
        <SheetContent className="w-full overflow-y-auto sm:max-w-3xl">
          <SheetHeader>
            <SheetTitle>{editing ? profileOf(editing)?.legal_name || partyName(editing) : ""}</SheetTitle>
            <SheetDescription>{t("parties.issuer.description")}</SheetDescription>
          </SheetHeader>
          {editing ? (
            <div className="mt-4">
              <IssuerForm party={editing} onSaved={issuers.reload} />
            </div>
          ) : null}
        </SheetContent>
      </Sheet>

      <AddCompanyDialog
        open={adding}
        onClose={() => setAdding(false)}
        onCreated={(partyId) => {
          setAdding(false);
          reload();
          issuers.reload();
          setEditing(partyId);
        }}
      />
    </>
  );
}

function AddCompanyDialog({
  open,
  onClose,
  onCreated,
}: {
  open: boolean;
  onClose: () => void;
  onCreated: (partyId: string) => void;
}) {
  const t = useT();
  const [form, setForm] = useState({ legal_name: "", oib: "", vat_id: "", country_code: "HR" });
  const [busy, setBusy] = useState(false);
  const oibOk = form.oib === "" || /^\d{11}$/.test(form.oib);
  const create = async () => {
    setBusy(true);
    try {
      const r = await api.createIssuer({ ...form, country_code: form.country_code.toUpperCase() });
      toast.success(t("parties.company_created", { name: form.legal_name }));
      setForm({ legal_name: "", oib: "", vat_id: "", country_code: "HR" });
      if (r.party) onCreated(r.party.id);
    } catch (e) {
      toast.error(describe(e));
    } finally {
      setBusy(false);
    }
  };
  return (
    <Dialog open={open} onOpenChange={(o) => !o && onClose()}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>{t("parties.add_company")}</DialogTitle>
          <DialogDescription>{t("parties.add_company_hint")}</DialogDescription>
        </DialogHeader>
        <div className="grid gap-3">
          <TextField
            label={t("parties.company.legal_name")}
            value={form.legal_name}
            onChange={(v) => setForm({ ...form, legal_name: v })}
          />
          <div className="grid grid-cols-2 gap-3">
            <TextField
              label={t("parties.company.oib")}
              value={form.oib}
              onChange={(v) => setForm({ ...form, oib: v })}
            />
            <TextField
              label={t("parties.company.vat_id")}
              value={form.vat_id}
              onChange={(v) => setForm({ ...form, vat_id: v })}
            />
          </div>
          <TextField
            label={t("parties.company.country")}
            value={form.country_code}
            onChange={(v) => setForm({ ...form, country_code: v })}
          />
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={onClose}>
            {t("common.cancel")}
          </Button>
          <Button onClick={() => void create()} disabled={busy || !form.legal_name.trim() || !oibOk}>
            {busy ? t("common.saving") : t("parties.add_company")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function IssuerForm({ party, onSaved }: { party: string; onSaved: () => void }) {
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
      onSaved();
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
