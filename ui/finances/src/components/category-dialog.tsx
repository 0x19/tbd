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
import { useT } from "@/lib/i18n";

/** The wire kinds; the label and hint are `categories.kind_label.<kind>` and
 *  `categories.kind_hint.<kind>`, looked up where rendered. */
export const KINDS = ["expense", "income", "transfer", "tax", "capital"] as const;

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
  const t = useT();
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
          ? t("categories.cat.archived_toast", { n: done.unmatched })
          : t("categories.cat.saved", { name: done.category?.name ?? name }),
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
          <DialogTitle>{category ? t("categories.cat.edit") : t("categories.new_category")}</DialogTitle>
          <DialogDescription>{t("categories.cat.description")}</DialogDescription>
        </DialogHeader>
        <div className="grid gap-4">
          {!category && parties.length > 1 ? (
            <Field label={t("common.party")}>
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
          <Field label={t("common.name")}>
            <Input
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder={t("categories.cat.name_placeholder")}
              autoFocus
            />
          </Field>
          <Field label={t("categories.cat.kind")}>
            <Select value={kind} onValueChange={setKind}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {KINDS.map((k) => (
                  <SelectItem key={k} value={k}>
                    {t(`categories.kind_label.${k}`)}{" "}
                    <span className="text-muted-foreground">· {t(`categories.kind_hint.${k}`)}</span>
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </Field>
          <div className="flex items-center gap-2">
            <Switch checked={deductible} onCheckedChange={setDeductible} id="deductible" />
            <Label htmlFor="deductible">{t("categories.cat.deductible")}</Label>
          </div>
          {category ? (
            <div className="flex items-center gap-2">
              <Switch checked={archived} onCheckedChange={setArchived} id="archived" />
              <Label htmlFor="archived">{t("categories.cat.archived")}</Label>
            </div>
          ) : null}
          {category ? (
            <p className="text-muted-foreground text-xs">
              {t("categories.cat.slug_before")} <code>{category.slug}</code> {t("categories.cat.slug_after")}
            </p>
          ) : null}
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={onClose}>
            {t("common.cancel")}
          </Button>
          <Button onClick={() => void save()} disabled={busy || !name.trim() || !party}>
            {busy ? t("common.saving") : t("common.save")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
