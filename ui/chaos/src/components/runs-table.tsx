"use client";

import Link from "next/link";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { StatusBadge } from "@/components/status-badge";
import { Empty } from "@/components/page-header";
import { ago, ms, num, pct, seconds } from "@/lib/format";
import type { RunSummary } from "@/lib/api/schema";

export function RunsTable({
  runs,
  actions,
}: {
  runs: RunSummary[];
  actions?: (r: RunSummary) => React.ReactNode;
}) {
  if (!runs.length) return <Empty>No runs yet. Start one from Scenarios, Load or Validate.</Empty>;
  return (
    <div className="overflow-x-auto rounded-lg border">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Run</TableHead>
            <TableHead>Kind</TableHead>
            <TableHead>Status</TableHead>
            <TableHead className="text-right">Requests</TableHead>
            <TableHead className="text-right">Errors</TableHead>
            <TableHead className="text-right">p99</TableHead>
            <TableHead className="text-right">Checks</TableHead>
            <TableHead className="text-right">Took</TableHead>
            <TableHead>Started</TableHead>
            {actions ? <TableHead /> : null}
          </TableRow>
        </TableHeader>
        <TableBody>
          {runs.map((r) => (
            <TableRow key={r.id}>
              <TableCell>
                <Link href={`/runs/view/?id=${r.id}`} className="font-medium hover:underline">
                  {r.name}
                </Link>
                {r.error ? <div className="max-w-64 truncate text-xs text-destructive">{r.error}</div> : null}
              </TableCell>
              <TableCell className="text-muted-foreground">{r.kind}</TableCell>
              <TableCell>
                <StatusBadge status={r.status} />
              </TableCell>
              <TableCell className="text-right tabular-nums">{num(r.requests_total)}</TableCell>
              <TableCell className="text-right tabular-nums">{pct(r.error_rate)}</TableCell>
              <TableCell className="text-right tabular-nums">{ms(r.p99_ms)}</TableCell>
              <TableCell className="text-right tabular-nums">
                {r.passed ? `${r.passed[0]}/${r.passed[1]}` : "–"}
              </TableCell>
              <TableCell className="text-right tabular-nums">{seconds(r.duration_s)}</TableCell>
              <TableCell className="text-muted-foreground" title={r.started_at}>
                {ago(r.started_at)}
              </TableCell>
              {actions ? <TableCell className="text-right">{actions(r)}</TableCell> : null}
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  );
}
