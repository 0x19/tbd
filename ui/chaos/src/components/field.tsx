import { Label } from "@/components/ui/label";

/** A labelled control. The wrapping `<label>` associates the control for
 * screen readers and tests without ids. */
export function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <Label className="grid gap-1.5 font-normal">
      <span className="text-muted-foreground text-xs">{label}</span>
      {children}
    </Label>
  );
}
