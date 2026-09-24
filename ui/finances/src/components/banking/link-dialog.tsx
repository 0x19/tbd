"use client";

// Linking a bank: which company, which login, and what happens next, said
// before the browser leaves for the bank.
import { Briefcase, User } from "lucide-react";
import { useEffect, useState } from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { api } from "@/lib/api/client";
import { describe } from "@/lib/api/hooks";
import { useT } from "@/lib/i18n";
import { cn } from "@/lib/utils";

export type Psu = "business" | "personal";

export function LinkBankDialog({
  open,
  onOpenChange,
  parties,
  defaultParty,
  defaultPsu = "business",
}: {
  open: boolean;
  onOpenChange: (v: boolean) => void;
  parties: { id: string; display_name: string }[];
  defaultParty: string;
  defaultPsu?: Psu;
}) {
  const t = useT();
  const [party, setParty] = useState(defaultParty);
  const [psu, setPsu] = useState<Psu>(defaultPsu);
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    if (open) {
      setParty(defaultParty);
      setPsu(defaultPsu);
    }
  }, [open, defaultParty, defaultPsu]);
  const start = async () => {
    setBusy(true);
    try {
      const s = await api.startConnection(party, psu);
      window.location.href = s.url;
    } catch (e) {
      toast.error(describe(e));
      setBusy(false);
    }
  };
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>{t("banking.link.title")}</DialogTitle>
          <DialogDescription>{t("banking.link.desc")}</DialogDescription>
        </DialogHeader>
        <div className="space-y-4">
          {parties.length > 1 ? (
            <div className="space-y-2">
              <Label>{t("banking.link.for")}</Label>
              <Select value={party} onValueChange={setParty}>
                <SelectTrigger className="w-full">
                  <SelectValue placeholder={t("common.party")} />
                </SelectTrigger>
                <SelectContent>
                  {parties.map((p) => (
                    <SelectItem key={p.id} value={p.id}>
                      {p.display_name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          ) : null}
          <div className="space-y-2">
            <Label>{t("banking.link.login")}</Label>
            <div className="grid gap-2">
              {(["business", "personal"] as Psu[]).map((p) => (
                <button
                  key={p}
                  type="button"
                  onClick={() => setPsu(p)}
                  className={cn(
                    "flex items-start gap-3 rounded-md border p-3 text-left text-sm",
                    psu === p ? "border-foreground" : "hover:bg-muted/50",
                  )}
                >
                  <span className="text-muted-foreground mt-0.5">
                    {p === "business" ? <Briefcase className="size-4" /> : <User className="size-4" />}
                  </span>
                  <span>
                    <span className="block font-medium">{t(`banking.${p}_login`)}</span>
                    <span className="text-muted-foreground block text-xs">{t(`banking.link.${p}_hint`)}</span>
                  </span>
                </button>
              ))}
            </div>
          </div>
          <p className="text-muted-foreground text-xs">{t("banking.link.what_happens")}</p>
        </div>
        <DialogFooter>
          <Button onClick={() => void start()} disabled={busy || !party}>
            {busy ? t("banking.opening_bank") : t("banking.link_erste")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
