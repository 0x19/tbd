"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { useState } from "react";
import { Play, Plus } from "lucide-react";
import { toast } from "sonner";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Empty, ErrorNote, PageHeader } from "@/components/page-header";
import { api } from "@/lib/api/client";
import { describe, useFetch } from "@/lib/api/hooks";

export default function ScenariosPage() {
  const router = useRouter();
  const list = useFetch(() => api.scenarios(), 5000);
  const [busy, setBusy] = useState<string | null>(null);

  const run = async (id: string) => {
    setBusy(id);
    try {
      const summary = await api.runScenario(id);
      toast.success(`started ${summary.name}`);
      router.push(`/runs/view/?id=${summary.id}`);
    } catch (e) {
      toast.error(describe(e));
    } finally {
      setBusy(null);
    }
  };

  return (
    <>
      <PageHeader
        title="Scenarios"
        description="TOML files under the scenarios directory: a stack, load, a fault timeline and assertions. Each run starts its own stack on free ports."
      >
        <Button size="sm" render={<Link href="/scenarios/view/?id=new" />}>
          <Plus /> New scenario
        </Button>
      </PageHeader>
      <ErrorNote message={list.error} />
      {!list.data ? (
        <Skeleton className="h-40" />
      ) : !list.data.length ? (
        <Empty>No scenario files yet.</Empty>
      ) : (
        <div className="overflow-x-auto rounded-lg border">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Id</TableHead>
                <TableHead>Name</TableHead>
                <TableHead>Description</TableHead>
                <TableHead>Check</TableHead>
                <TableHead className="text-right" />
              </TableRow>
            </TableHeader>
            <TableBody>
              {list.data.map((s) => (
                <TableRow key={s.id}>
                  <TableCell>
                    <Link
                      href={`/scenarios/view/?id=${encodeURIComponent(s.id)}`}
                      className="font-mono text-xs hover:underline"
                    >
                      {s.id}
                    </Link>
                  </TableCell>
                  <TableCell className="font-medium">
                    {s.name ?? <span className="text-muted-foreground">–</span>}
                    {s.skip ? (
                      <Badge variant="outline" className="ml-2">
                        skip
                      </Badge>
                    ) : null}
                  </TableCell>
                  <TableCell className="max-w-md truncate text-muted-foreground">{s.description}</TableCell>
                  <TableCell>
                    {s.ok ? (
                      <Badge variant="outline" className="text-emerald-600 dark:text-emerald-400">
                        ok
                      </Badge>
                    ) : (
                      <span className="text-xs text-destructive">{s.error}</span>
                    )}
                  </TableCell>
                  <TableCell className="text-right">
                    <Button
                      size="sm"
                      variant="outline"
                      disabled={!s.ok || busy === s.id}
                      onClick={() => run(s.id)}
                    >
                      <Play /> run
                    </Button>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
      )}
    </>
  );
}
