"use client";

import { useState } from "react";
import { toast } from "sonner";

import { useFinance } from "@/app/providers";
import { Field } from "@/components/rule-dialog";
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
import type { Category, UpsertCategoryResponse } from "@/lib/api/schema";

export const KINDS: { value: string; label: string; hint: string }[] = [
  { value: "expense", label: "Expense", hint: "Money spent on something." },
  { value: "income", label: "Income", hint: "Money earned." },
  { value: "transfer", label: "Transfer", hint: "Own money moving; not spending." },
  { value: "tax", label: "Tax", hint: "The state's share." },
  { value: "capital", label: "Capital", hint: "Loans, draws, investments." },
];

/** Create or edit one category. A rename keeps the slug; archiving disables
 *  the rules pointing at it and reapplies the rest. */
export function CategoryDialog({
  category,
  partyId,
  onClose,
  onSaved,
}: {
  category: Category | null;
  partyId?: string;
  onClose: () => void;
  onSaved: (done: UpsertCategoryResponse) => void;
}) {
  const { parties, partyIds } = useFinance();
  const [party, setParty] = useState(category?.party_id ?? partyId ?? partyIds[0] ?? "");
  const [name, setName] = useState(category?.name ?? "");
  const [kind, setKind] = useState(category?.kind ?? "expense");
  const [deductible, setDeductible] = useState(category?.deductible ?? false);
  const [archived, setArchived] = useState(category?.archived ?? false);
  const [busy, setBusy] = useState(false);

  const save = async () => {
    setBusy(true);
    try {
      const done = await api.upsertCategory({
        id: category?.id,
        party_id: party,
        name,
        kind,
        deductible,
        archived,
      });
      toast.success(
        archived
          ? `Archived. Its rules are off; ${done.unmatched} rows are uncategorised now.`
          : `Saved ${done.category?.name ?? name}.`,
      );
      onSaved(done);
    } catch (e) {
      toast.error(describe(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <Dialog open onOpenChange={(o) => !o && onClose()}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>{category ? "Edit category" : "New category"}</DialogTitle>
          <DialogDescription>
            What it does to the books matters more than what it is called: the kind decides whether the
            overview counts it as spending.
          </DialogDescription>
        </DialogHeader>
        <div className="grid gap-4">
          {!category && parties.length > 1 ? (
            <Field label="Party">
              <Select value={party} onValueChange={setParty}>
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
          <Field label="Name">
            <Input
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="Kiosks & newsstands"
              autoFocus
            />
          </Field>
          <Field label="Kind">
            <Select value={kind} onValueChange={setKind}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {KINDS.map((k) => (
                  <SelectItem key={k.value} value={k.value}>
                    {k.label} <span className="text-muted-foreground">· {k.hint}</span>
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </Field>
          <div className="flex items-center gap-2">
            <Switch checked={deductible} onCheckedChange={setDeductible} id="deductible" />
            <Label htmlFor="deductible">Deductible business cost</Label>
          </div>
          {category ? (
            <div className="flex items-center gap-2">
              <Switch checked={archived} onCheckedChange={setArchived} id="archived" />
              <Label htmlFor="archived">Archived: hidden from pickers, its rules turned off</Label>
            </div>
          ) : null}
          {category ? (
            <p className="text-muted-foreground text-xs">
              Slug <code>{category.slug}</code> stays as it is; rules and seeds key on it.
            </p>
          ) : null}
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={onClose}>
            Cancel
          </Button>
          <Button onClick={() => void save()} disabled={busy || !name.trim() || !party}>
            {busy ? "Saving…" : "Save"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
