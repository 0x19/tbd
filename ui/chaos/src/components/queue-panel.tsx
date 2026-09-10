"use client";

import { ListOrdered, X } from "lucide-react";
import Link from "next/link";
import { toast } from "sonner";

import { useChaos } from "@/app/providers";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { api } from "@/lib/api/client";
import { describe } from "@/lib/api/hooks";
import { ago } from "@/lib/format";

/** What waits for the active slot, front first. Renders nothing when empty. */
export function QueuePanel() {
  const { queue } = useChaos();
  if (!queue.length) return null;

  const remove = async (id: string) => {
    try {
      await api.queueRemove(id);
    } catch (e) {
      toast.error(describe(e));
    }
  };
  const clear = async () => {
    try {
      await api.queueClear();
      toast.success("queue cleared");
    } catch (e) {
      toast.error(describe(e));
    }
  };

  return (
    <div className="rounded-xl border" data-testid="queue-panel">
      <div className="flex items-center gap-2 border-b px-4 py-2.5">
        <ListOrdered className="text-muted-foreground size-4" />
        <span className="text-sm font-semibold">Queued</span>
        <Badge variant="secondary" className="tabular-nums">
          {queue.length}
        </Badge>
        <span className="text-muted-foreground text-xs">
          one run at a time; next starts when the active one ends
        </span>
        <Button variant="ghost" size="sm" className="ml-auto" onClick={clear}>
          Clear all
        </Button>
      </div>
      <ul className="divide-y">
        {queue.map((q, i) => (
          <li key={q.id} className="flex items-center gap-3 px-4 py-2 text-sm">
            <span className="text-muted-foreground w-5 text-right text-xs tabular-nums">{i + 1}</span>
            <span className="font-medium">
              {q.scenario_id ? (
                <Link
                  href={`/scenarios/view/?id=${encodeURIComponent(q.scenario_id)}`}
                  className="hover:underline"
                >
                  {q.name}
                </Link>
              ) : (
                q.name
              )}
            </span>
            <span className="text-muted-foreground capitalize">{q.kind}</span>
            {q.schedule_id ? (
              <Link href="/schedules/">
                <Badge variant="outline" className="text-[10px]">
                  scheduled
                </Badge>
              </Link>
            ) : null}
            <span className="text-muted-foreground ml-auto text-xs">{ago(q.queued_at)}</span>
            <Button
              variant="ghost"
              size="icon-sm"
              aria-label={`Remove ${q.name} from the queue`}
              onClick={() => remove(q.id)}
            >
              <X />
            </Button>
          </li>
        ))}
      </ul>
    </div>
  );
}
