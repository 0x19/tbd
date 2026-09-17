"use client";

import { useState } from "react";
import { toast } from "sonner";

import { useFinance } from "@/app/providers";
import { Button } from "@/components/ui/button";
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
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import { api } from "@/lib/api/client";
import { describe } from "@/lib/api/hooks";
import type { Category, Rule } from "@/lib/api/schema";

/** Create or edit one rule. Saving reapplies every rule for the party, and the
 *  toast says what that pass did, so a rule that claims nothing is noticed now. */
export function RuleDialog({
  rule,
  categories,
  onClose,
  onSaved,
}: {
  rule: Rule | null;
  categories: Category[];
  onClose: () => void;
  onSaved: () => void;
}) {
  const { parties, partyIds } = useFinance();
  const [party, setParty] = useState(rule?.party_id ?? partyIds[0] ?? "");
  const [name, setName] = useState(rule?.name ?? "");
  const [priority, setPriority] = useState(String(rule?.priority ?? 50));
  const [category, setCategory] = useState(rule?.category_id ?? "");
  const [cp, setCp] = useState(rule?.match_counterparty_like ?? "");
  const [rem, setRem] = useState(rule?.match_remittance_like ?? "");
  const [iban, setIban] = useState(rule?.match_counterparty_iban ?? "");
  const [ccy, setCcy] = useState(rule?.match_currency ?? "");
  const [cd, setCd] = useState(rule?.match_credit_debit ?? "");
  const [enabled, setEnabled] = useState(rule?.enabled ?? true);
  const [busy, setBusy] = useState(false);
  const mine = categories.filter((c) => c.party_id === party);

  const save = async () => {
    setBusy(true);
    try {
      const done = await api.upsertRule({
        id: rule?.id,
        party_id: party,
        priority: Number(priority) || 100,
        name,
        category_id: category,
        match_counterparty_like: cp,
        match_remittance_like: rem,
        match_counterparty_iban: iban,
        match_currency: ccy,
        match_credit_debit: cd,
        enabled,
      });
      toast.success(
        `Saved. ${done.rule?.hits ?? 0} rows claimed by this rule; ${done.unmatched} still uncategorised.`,
      );
      onSaved();
    } catch (e) {
      toast.error(describe(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <Dialog open onOpenChange={(o) => !o && onClose()}>
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>{rule ? "Edit rule" : "New rule"}</DialogTitle>
          <DialogDescription>
            Every condition is optional; those set must all match. Text matches are case-insensitive and
            ignore Croatian diacritics.
          </DialogDescription>
        </DialogHeader>
        <div className="grid gap-4">
          {!rule && parties.length > 1 ? (
            <Field label="Party">
              <Select
                value={party}
                onValueChange={(v) => {
                  setParty(v);
                  setCategory("");
                }}
              >
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
              </Select>
            </Field>
          ) : null}
          <div className="grid grid-cols-3 gap-3">
            <Field label="Name" className="col-span-2">
              <Input value={name} onChange={(e) => setName(e.target.value)} placeholder="hetzner" />
            </Field>
            <Field label="Priority">
              <Input value={priority} onChange={(e) => setPriority(e.target.value)} inputMode="numeric" />
            </Field>
          </div>
          <Field label="Category">
            <Select value={category} onValueChange={setCategory}>
              <SelectTrigger>
                <SelectValue placeholder="Choose…" />
              </SelectTrigger>
              <SelectContent>
                {mine.map((c) => (
                  <SelectItem key={c.id} value={c.id}>
                    {c.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </Field>
          <Field label="Counterparty contains">
            <Input
              value={cp}
              onChange={(e) => setCp(e.target.value)}
              placeholder="HETZNER"
              className="font-mono"
            />
          </Field>
          <Field label="Remittance contains">
            <Input
              value={rem}
              onChange={(e) => setRem(e.target.value)}
              placeholder="ERSTE ATM"
              className="font-mono"
            />
          </Field>
          <div className="grid grid-cols-3 gap-3">
            <Field label="Counterparty IBAN">
              <Input value={iban} onChange={(e) => setIban(e.target.value)} className="font-mono" />
            </Field>
            <Field label="Currency">
              <Input
                value={ccy}
                onChange={(e) => setCcy(e.target.value.toUpperCase())}
                placeholder="EUR"
                maxLength={3}
                className="font-mono"
              />
            </Field>
            <Field label="Direction">
              <Select value={cd || "any"} onValueChange={(v) => setCd(v === "any" ? "" : v)}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="any">Any</SelectItem>
                  <SelectItem value="DBIT">Out (DBIT)</SelectItem>
                  <SelectItem value="CRDT">In (CRDT)</SelectItem>
                </SelectContent>
              </Select>
            </Field>
          </div>
          <div className="flex items-center gap-2">
            <Switch checked={enabled} onCheckedChange={setEnabled} id="enabled" />
            <Label htmlFor="enabled">Enabled</Label>
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={onClose}>
            Cancel
          </Button>
          <Button onClick={() => void save()} disabled={busy || !name.trim() || !category}>
            {busy ? "Saving…" : "Save & apply"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function Field({
  label,
  children,
  className,
}: {
  label: string;
  children: React.ReactNode;
  className?: string;
}) {
  return (
    <div className={className}>
      <Label className="text-muted-foreground mb-1.5 block text-xs">{label}</Label>
      {children}
    </div>
  );
}
