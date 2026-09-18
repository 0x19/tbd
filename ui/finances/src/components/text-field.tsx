import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

/** A labelled text input, the way every settings form here lays them out. */
export function TextField({
  label,
  value,
  onChange,
  mono,
  className,
  placeholder,
  hint,
}: {
  label: string;
  value: string;
  onChange: (v: string) => void;
  mono?: boolean;
  className?: string;
  placeholder?: string;
  hint?: string;
}) {
  return (
    <div className={className}>
      <Label className="text-muted-foreground mb-1.5 block text-xs">{label}</Label>
      <Input
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder}
        className={mono ? "font-mono" : undefined}
      />
      {hint ? <p className="text-muted-foreground mt-1 text-[11px]">{hint}</p> : null}
    </div>
  );
}
