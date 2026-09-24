"use client";

// Clients: who is billed. A table of them, and a panel per client with its
// details, the VAT treatment that decides the invoice's note, and the line
// templates a new draft for it starts from.
import { Plus, Star } from "lucide-react";
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
import { useT } from "@/lib/i18n";

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
  is_default: false,
});

// Wire value → message key; the label is looked up at render time.
const TREATMENTS: Record<string, string> = {
  outside_scope_non_eu: "parties.treatment.outside_scope_non_eu",
  reverse_charge_eu: "parties.treatment.reverse_charge_eu",
  standard_hr: "parties.treatment.standard_hr",
  exempt_issuer: "parties.treatment.exempt_issuer",
};

export default function ClientsPage() {
  const t = useT();
  const { parties, partyIds, partyName, multi } = useFinance();
  const orgs = parties.filter((p) => p.kind === "org");
  const [party, setParty] = useState("");
  const chosen = party || orgs[0]?.id || partyIds[0] || "";
  const loaded = useFetch(() => api.clients(chosen ? [chosen] : []), 0, [chosen]);
  const [editing, setEditing] = useState<ClientProfile | null>(null);
  const list = loaded.data?.clients ?? [];
  const makeDefault = async (c: ClientProfile) => {
    try {
      await api.setDefaultClient(c.id);
      toast.success(t("parties.default_set", { name: c.name }));
      loaded.reload();
    } catch (e) {
      toast.error(describe(e));
    }
  };
  return (
    <>
      <PageTitle title={t("parties.clients.title")} description={t("parties.clients.description")}>
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
            <Plus /> {t("parties.new_client")}
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
                  <TableHead>{t("parties.client")}</TableHead>
                  <TableHead className="hidden md:table-cell">{t("parties.col.address")}</TableHead>
                  <TableHead>{t("parties.col.country")}</TableHead>
                  <TableHead className="hidden lg:table-cell">{t("parties.col.tax_id")}</TableHead>
                  <TableHead>{t("parties.col.vat")}</TableHead>
                  <TableHead>{t("common.currency")}</TableHead>
                  {multi ? (
                    <TableHead className="hidden xl:table-cell">{t("parties.col.billed_by")}</TableHead>
                  ) : null}
                  <TableHead className="w-36" />
                </TableRow>
              </TableHeader>
              <TableBody>
                {list.map((c) => (
                  <TableRow key={c.id} className="cursor-pointer" onClick={() => setEditing(c)}>
                    <TableCell className="font-medium">
                      {c.name}
                      {c.is_default ? (
                        <Badge variant="secondary" className="ml-2 text-[10px]">
                          <Star className="mr-1 size-3 fill-current" />
                          {t("parties.default")}
                        </Badge>
                      ) : null}
                    </TableCell>
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
                    <TableCell onClick={(e) => e.stopPropagation()}>
                      {c.is_default ? null : (
                        <Button
                          variant="ghost"
                          size="sm"
                          className="h-7 text-xs"
                          onClick={() => void makeDefault(c)}
                        >
                          <Star /> {t("parties.set_default")}
                        </Button>
                      )}
                    </TableCell>
                  </TableRow>
                ))}
                {loaded.data && list.length === 0 ? (
                  <TableRow>
                    <TableCell colSpan={8} className="text-muted-foreground py-10 text-center text-sm">
                      {t("parties.no_clients")}
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
  const t = useT();
  const [form, setForm] = useState<ClientProfile | null>(client);
  const [busy, setBusy] = useState(false);
  useEffect(() => setForm(client), [client]);
  const set = (k: keyof ClientProfile) => (v: string) => form && setForm({ ...form, [k]: v });
  const save = async () => {
    if (!form) return;
    setBusy(true);
    try {
      const { archived: _archived, is_default: _default, ...input } = form;
      const r = await api.upsertClient(input);
      toast.success(t("parties.client_saved"));
      if (r.client) setForm(r.client);
      onSaved();
    } catch (e) {
      toast.error(describe(e));
    } finally {
      setBusy(false);
    }
  };
  const treatment = form
    ? TREATMENTS[form.vat_treatment]
      ? t(TREATMENTS[form.vat_treatment])
      : form.vat_treatment
    : "";
  return (
    <Sheet open={client !== null} onOpenChange={(o) => !o && onClose()}>
      <SheetContent className="flex w-full flex-col gap-4 overflow-y-auto sm:max-w-2xl">
        <SheetHeader className="pr-8">
          <SheetTitle>{form?.id ? form.name || t("parties.client") : t("parties.new_client")}</SheetTitle>
          <SheetDescription>
            {form?.id
              ? t("parties.billed_in", { treatment, currency: form.currency })
              : t("parties.new_client.description")}
          </SheetDescription>
        </SheetHeader>
        {form ? (
          <>
            <div className="grid gap-3 sm:grid-cols-2">
              <TextField
                label={t("common.name")}
                value={form.name}
                onChange={set("name")}
                className="sm:col-span-2"
              />
              <div className="sm:col-span-2">
                <Label className="text-muted-foreground mb-1.5 block text-xs">
                  {t("parties.address_lines")}
                </Label>
                <Textarea
                  rows={3}
                  value={form.address_lines.join("\n")}
                  onChange={(e) => setForm({ ...form, address_lines: e.target.value.split("\n") })}
                />
              </div>
              <TextField
                label={t("parties.country_iso")}
                value={form.country_code}
                onChange={(v) => set("country_code")(v.toUpperCase())}
                mono
              />
              <TextField
                label={t("parties.tax_id_theirs")}
                value={form.tax_id}
                onChange={set("tax_id")}
                mono
              />
              <div>
                <Label className="text-muted-foreground mb-1.5 block text-xs">
                  {t("parties.vat_treatment")}
                </Label>
                <Select value={form.vat_treatment} onValueChange={set("vat_treatment")}>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {Object.entries(TREATMENTS).map(([v, key]) => (
                      <SelectItem key={v} value={v}>
                        {t(key)}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <TextField
                label={t("common.currency")}
                value={form.currency}
                onChange={(v) => set("currency")(v.toUpperCase())}
                mono
              />
              <div className="sm:col-span-2">
                <Label className="text-muted-foreground mb-1.5 block text-xs">
                  {t("parties.recipients")}
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
                {busy ? t("common.saving") : form.id ? t("parties.save_client") : t("parties.create_client")}
              </Button>
              <Button variant="ghost" onClick={onClose}>
                {t("common.close")}
              </Button>
            </div>
            {form.id ? (
              <TemplatesEditor client={form.id} />
            ) : (
              <p className="text-muted-foreground text-xs">{t("parties.templates_after_create")}</p>
            )}
          </>
        ) : null}
      </SheetContent>
    </Sheet>
  );
}

// Wire value → message key; the label is looked up at render time.
const MODES = [
  ["fixed", "parties.mode.fixed"],
  ["variable", "parties.mode.variable"],
  ["optional", "parties.mode.optional"],
] as const;

/** The rows a draft for this client starts from. */
function TemplatesEditor({ client }: { client: string }) {
  const t = useT();
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
  const save = async (row: LineTemplate) => {
    setBusy(true);
    try {
      await api.upsertLineTemplate(client, row);
      toast.success(t("parties.template_saved"));
      loaded.reload();
    } catch (e) {
      toast.error(describe(e));
    } finally {
      setBusy(false);
    }
  };
  const remove = async (row: LineTemplate, i: number) => {
    if (!row.id) return setRows(rows.filter((_, j) => j !== i));
    setBusy(true);
    try {
      await api.deleteLineTemplate(client, row.id);
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
          <Label className="text-xs font-medium">{t("parties.line_templates")}</Label>
          <p className="text-muted-foreground text-xs">{t("parties.line_templates.description")}</p>
        </div>
        <Button variant="outline" size="sm" onClick={() => setRows([...rows, blank()])}>
          <Plus /> {t("parties.row")}
        </Button>
      </div>
      <div className="space-y-2">
        {rows.map((row, i) => (
          <div key={row.id || `new-${i}`} className="space-y-2 rounded-md border p-2">
            <div className="grid gap-2 sm:grid-cols-[3rem_1fr]">
              <Input
                value={String(row.position)}
                inputMode="numeric"
                className="font-mono"
                title={t("parties.position")}
                onChange={(e) => set(i, { position: Number(e.target.value) || 0 })}
              />
              <Textarea
                rows={2}
                value={row.description}
                className="min-h-9 text-sm"
                placeholder={t("parties.description_as_printed")}
                onChange={(e) => set(i, { description: e.target.value })}
              />
            </div>
            <div className="grid gap-2 sm:grid-cols-[1fr_6rem_8rem_auto]">
              <Select value={row.mode} onValueChange={(v) => set(i, { mode: v })}>
                <SelectTrigger className="text-xs">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {MODES.map(([v, key]) => (
                    <SelectItem key={v} value={v}>
                      {t(key)}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              <Input
                value={(Number(row.quantity_milli) / 1000).toString()}
                inputMode="decimal"
                className="text-right font-mono"
                title={t("parties.quantity")}
                onChange={(e) =>
                  set(i, {
                    quantity_milli: String(
                      Math.round(Number(e.target.value.replace(",", ".")) * 1000) || 1000,
                    ),
                  })
                }
              />
              <Input
                value={(Number(row.unit_price_minor) / 100).toFixed(2)}
                inputMode="decimal"
                className="text-right font-mono"
                title={
                  row.mode === "variable" ? t("parties.unit_price.variable_hint") : t("parties.unit_price")
                }
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
                  disabled={busy || !row.description.trim()}
                  onClick={() => void save(row)}
                >
                  {t("common.save")}
                </Button>
                <Button
                  size="sm"
                  variant="ghost"
                  disabled={busy}
                  onClick={() => void remove(row, i)}
                  aria-label={t("common.remove")}
                >
                  ×
                </Button>
              </div>
            </div>
          </div>
        ))}
        {rows.length === 0 ? (
          <p className="text-muted-foreground text-xs">{t("parties.no_templates")}</p>
        ) : null}
      </div>
    </div>
  );
}
