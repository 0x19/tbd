"use client";

// Personal / business / combined. The choices are the parties the server says
// the caller may read -- the toggle narrows within the grant and can never
// widen it, which is why it is built from `parties` and not from a list here.
import { useFinance } from "@/app/providers";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";

export function ScopeToggle({ className }: { className?: string }) {
  const { parties, scope, setScope, multi } = useFinance();
  if (!multi) return null;
  return (
    <Tabs value={scope} onValueChange={setScope} className={className}>
      <TabsList>
        <TabsTrigger value="all">Combined</TabsTrigger>
        {parties.map((p) => (
          <TabsTrigger key={p.id} value={p.id}>
            {p.display_name}
          </TabsTrigger>
        ))}
      </TabsList>
    </Tabs>
  );
}
