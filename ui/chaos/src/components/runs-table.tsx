"use client";

import Link from "next/link";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { StatusBadge } from "@/components/status-badge";
import { ago, ms, num, pct, seconds } from "@/lib/format";
import type { RunSummary } from "@/lib/api/schema";

/** The kit's list table: id in mono, name, chips, numbers right-aligned. */
export function RunsTable({
  runs,
  actions,
  footer,
}: {
  runs: RunSummary[];
  actions?: (r: RunSummary) => React.ReactNode;
  footer?: React.ReactNode;
}) {
  if (!runs.length) {
    return (
      <div className="rounded-lg border border-dashed p-8 text-center text-sm text-muted-foreground">
        No runs match. Start one from Scenarios, Load or Validate.
      </div>
    );
  }
  return (
    <div className="overflow-x-auto">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Run</TableHead>
            <TableHead>Kind</TableHead>
            <TableHead>Status</TableHead>
            <TableHead className="text-right">Requests</TableHead>
            <TableHead className="text-right">Errors</TableHead>
            <TableHead className="text-right">req/s</TableHead>
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
                <div className="font-mono text-[11px] text-muted-foreground">{r.id.slice(0, 13)}</div>
              </TableCell>
              <TableCell className="text-muted-foreground capitalize">{r.kind}</TableCell>
              <TableCell>
                <StatusBadge status={r.status} />
              </TableCell>
              <TableCell className="text-right tabular-nums">{num(r.requests_total)}</TableCell>
              <TableCell className={`text-right tabular-nums ${r.error_rate ? "text-destructive" : ""}`}>
                {pct(r.error_rate)}
              </TableCell>
              <TableCell className="text-right tabular-nums">
                {r.throughput_rps === null ? "–" : Math.round(r.throughput_rps)}
              </TableCell>
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
          {footer}
        </TableBody>
      </Table>
    </div>
  );
}
