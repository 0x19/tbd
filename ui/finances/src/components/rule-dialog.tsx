"use client";

import { useEffect, useState } from "react";
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
import type { Category, Rule, UpsertRule, UpsertRuleResponse } from "@/lib/api/schema";
import { useT } from "@/lib/i18n";

/** The service's normalisation, near enough to preview a match: upper case,
 *  Croatian diacritics stripped. */
export function normalise(s: string): string {
  return s.normalize("NFD").replace(/[̀-ͯ]/g, "").replace(/đ/g, "d").replace(/Đ/g, "D").toUpperCase();
}

/** The pass's match: a substring, unless `^` pins it to the start or `$` to the end. */
export function matchesPattern(pattern: string, value: string): boolean {
  const p = normalise(pattern.trim());
  const v = normalise(value);
  const start = p.startsWith("^");
  const end = p.endsWith("$");
  const core = p.replace(/^\^/, "").replace(/\$$/, "");
  if (start && end) return v === core;
  if (start) return v.startsWith(core);
  if (end) return v.endsWith(core);
  return v.includes(core);
}

/** A rule name from a counterparty: `DUBRAVICA-AUTOBUSNI KO` → `dubravica`. */
export function suggestName(counterparty: string): string {
  const first = normalise(counterparty)
    .split(/[^A-Z0-9]+/)
    .filter((w) => w.length > 2)[0];
  return (first ?? "").toLowerCase();
}

/** The plural form a count takes, for a `<key>.one|few|many` lookup: English
 *  needs only one/many, Croatian bends 2-4 differently from 5+ (and 12-14). */
export function pluralForm(n: number): "one" | "few" | "many" {
  const ten = n % 10;
  const hundred = n % 100;
  if (ten === 1 && hundred !== 11) return "one";
  if (ten >= 2 && ten <= 4 && (hundred < 12 || hundred > 14)) return "few";
  return "many";
}

/** Create or edit one rule. Saving reapplies every rule for the party, and the
 *  toast says what that pass did, so a rule that claims nothing is noticed now.
 *  `initial` prefills a new rule, typically from a transaction. */
export function RuleDialog({
  rule,
  initial,
  categories,
  onClose,
  onSaved,
}: {
  rule: Rule | null;
  initial?: Partial<UpsertRule>;
  categories: Category[];
  onClose: () => void;
  onSaved: (done: UpsertRuleResponse) => void;
}) {
  const t = useT();
  const { parties, partyIds } = useFinance();
  const seed = rule ?? initial ?? {};
  const [party, setParty] = useState(seed.party_id ?? partyIds[0] ?? "");
  const [name, setName] = useState(seed.name ?? "");
  const [priority, setPriority] = useState(String(seed.priority ?? 50));
  const [category, setCategory] = useState(seed.category_id ?? "");
  const [cp, setCp] = useState(seed.match_counterparty_like ?? "");
  const [rem, setRem] = useState(seed.match_remittance_like ?? "");
  const [iban, setIban] = useState(seed.match_counterparty_iban ?? "");
  const [ccy, setCcy] = useState(seed.match_currency ?? "");
  const [cd, setCd] = useState(seed.match_credit_debit ?? "");
  const [enabled, setEnabled] = useState(rule?.enabled ?? true);
  const [busy, setBusy] = useState(false);
  const [preview, setPreview] = useState<{ n: number; more: boolean; sample: string[] } | null>(null);
  const mine = categories.filter((c) => c.party_id === party && !c.archived);

  // What the text conditions would claim, from the newest rows mentioning
  // them. Approximate on purpose: the pass is the truth, and it runs on save.
  useEffect(() => {
    // The server searches for a substring; the anchors are ours to apply.
    const needle = (cp.trim() || rem.trim()).replace(/^\^/, "").replace(/\$$/, "");
    if (!needle || !party) {
      setPreview(null);
      return;
    }
    const handle = setTimeout(() => {
      void api
        .transactions({ party_ids: [party], limit: 100, search: needle })
        .then((r) => {
          const hits = r.transactions.filter(
            (t) =>
              (!cp.trim() || matchesPattern(cp, t.counterparty_name)) &&
              (!rem.trim() || matchesPattern(rem, t.remittance)) &&
              (!iban.trim() || t.counterparty_iban === iban.trim()) &&
              (!ccy || t.currency === ccy) &&
              (!cd || (cd === "DBIT") === t.amount_minor.startsWith("-")),
          );
          const sample = [...new Set(hits.map((t) => t.counterparty_name || t.remittance))].slice(0, 4);
          setPreview({ n: hits.length, more: r.transactions.length === 100, sample });
        })
        .catch(() => setPreview(null));
    }, 350);
    return () => clearTimeout(handle);
  }, [cp, rem, iban, ccy, cd, party]);

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
      toast.success(t("categories.rule.saved", { n: done.rule?.hits ?? 0, m: done.unmatched }));
      onSaved(done);
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
          <DialogTitle>{rule ? t("categories.rule.edit") : t("categories.new_rule")}</DialogTitle>
          <DialogDescription>{t("categories.rule.description")}</DialogDescription>
        </DialogHeader>
        <div className="grid gap-4">
          {!rule && parties.length > 1 ? (
            <Field label={t("common.party")}>
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
            <Field label={t("common.name")} className="col-span-2">
              <Input value={name} onChange={(e) => setName(e.target.value)} placeholder="hetzner" />
            </Field>
            <Field label={t("categories.rule.priority")}>
              <Input value={priority} onChange={(e) => setPriority(e.target.value)} inputMode="numeric" />
            </Field>
          </div>
          <Field label={t("categories.rule.category")}>
            <Select value={category} onValueChange={setCategory}>
              <SelectTrigger>
                <SelectValue placeholder={t("categories.rule.choose")} />
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
          <Field label={t("categories.rule.counterparty_contains")}>
            <Input
              value={cp}
              onChange={(e) => setCp(e.target.value)}
              placeholder="HETZNER"
              className="font-mono"
            />
          </Field>
          <Field label={t("categories.rule.remittance_contains")}>
            <Input
              value={rem}
              onChange={(e) => setRem(e.target.value)}
              placeholder="ERSTE ATM"
              className="font-mono"
            />
          </Field>
          <div className="grid grid-cols-3 gap-3">
            <Field label={t("categories.rule.counterparty_iban")}>
              <Input value={iban} onChange={(e) => setIban(e.target.value)} className="font-mono" />
            </Field>
            <Field label={t("common.currency")}>
              <Input
                value={ccy}
                onChange={(e) => setCcy(e.target.value.toUpperCase())}
                placeholder="EUR"
                maxLength={3}
                className="font-mono"
              />
            </Field>
            <Field label={t("categories.rule.direction")}>
              <Select value={cd || "any"} onValueChange={(v) => setCd(v === "any" ? "" : v)}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="any">{t("categories.rule.dir_any")}</SelectItem>
                  <SelectItem value="DBIT">{t("categories.rule.dir_out")}</SelectItem>
                  <SelectItem value="CRDT">{t("categories.rule.dir_in")}</SelectItem>
                </SelectContent>
              </Select>
            </Field>
          </div>
          <div className="bg-muted/50 rounded-md px-3 py-2 text-xs">
            {preview ? (
              <>
                <span className="font-medium">
                  {t(`categories.rule.would_claim.${pluralForm(preview.n)}`, {
                    n: `${preview.n}${preview.more ? "+" : ""}`,
                  })}
                </span>
                {preview.more ? (
                  <span className="text-muted-foreground">{t("categories.rule.among_newest")}</span>
                ) : null}
                {preview.sample.length ? (
                  <span className="text-muted-foreground"> · {preview.sample.join(" · ")}</span>
                ) : null}
                <span className="text-muted-foreground">{t("categories.rule.already_claimed")}</span>
              </>
            ) : (
              <span className="text-muted-foreground">{t("categories.rule.type_hint")}</span>
            )}
          </div>
          <div className="flex items-center gap-2">
            <Switch checked={enabled} onCheckedChange={setEnabled} id="enabled" />
            <Label htmlFor="enabled">{t("categories.rule.enabled")}</Label>
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={onClose}>
            {t("common.cancel")}
          </Button>
          <Button onClick={() => void save()} disabled={busy || !name.trim() || !category}>
            {busy ? t("common.saving") : t("categories.rule.save_apply")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

export function Field({
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
