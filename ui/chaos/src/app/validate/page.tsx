"use client";

import { Play, Search } from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";

import { useChaos } from "@/app/providers";
import { ChecksTable } from "@/app/runs/view/page";
import { FilterRail, PageTitle, StatRow } from "@/components/kit";
import { RunsTable } from "@/components/runs-table";
import { Button } from "@/components/ui/button";
import { InputGroup, InputGroupAddon, InputGroupInput } from "@/components/ui/input-group";
import { api } from "@/lib/api/client";
import { describe, useFetch } from "@/lib/api/hooks";
import type { RunRecord } from "@/lib/api/schema";
import { seconds, when } from "@/lib/format";

/** Events & Logs layout: filter rail for surfaces and results, a table with level chips and a totals row. */
export default function ValidatePage() {
  const { overview, reload, lastEvent } = useChaos();
  const history = useFetch(() => api.runs(100), 5000, [lastEvent]);
  const [protocol, setProtocol] = useState("");
  const [engine, setEngine] = useState("");
  const [timeout, setTimeoutValue] = useState("");
  const [busy, setBusy] = useState(false);
  const [result, setResult] = useState<RunRecord | null>(null);
  const [selected, setSelected] = useState<Record<string, string[]>>({});
  const [q, setQ] = useState("");

  const run = async () => {
    setBusy(true);
    try {
      const record = await api.validate({
        protocol: protocol || undefined,
        engine: engine || undefined,
        timeout: timeout || undefined,
      });
      setResult(record);
      reload();
      history.reload();
      if (record.status === "passed") toast.success("every check passed");
      else toast.error(`${record.validate?.failed} checks failed`);
    } catch (e) {
      toast.error(describe(e));
    } finally {
      setBusy(false);
    }
  };

  const checks = result?.validate?.checks ?? [];
  const surfaces = [...new Set(checks.map((c) => c.surface))].map((s) => ({
    value: s,
    label: s.toUpperCase(),
    count: checks.filter((c) => c.surface === s).length,
  }));
  const shown = checks.filter((c) => {
    if (selected.Surface?.length && !selected.Surface.includes(c.surface)) return false;
    if (selected.Result?.length && !selected.Result.includes(c.passed ? "pass" : "fail")) return false;
    if (q && !`${c.name} ${c.detail}`.toLowerCase().includes(q.toLowerCase())) return false;
    return true;
  });

  return (
    <>
      <PageTitle
        title="Validate"
        description="Eleven checks, one per surface, run concurrently with a timeout each. Green means the stack answers on every protocol the way the contract says."
      >
        <Button onClick={run} disabled={busy}>
          <Play /> {busy ? "Running…" : "Run validate"}
        </Button>
      </PageTitle>

      <div className="grid gap-3 rounded-xl border p-4 md:grid-cols-3">
        <label className="grid gap-1.5 text-sm">
          <span className="font-medium">Protocol base URL</span>
          <InputGroup>
            <InputGroupInput
              value={protocol}
              onChange={(e) => setProtocol(e.target.value)}
              placeholder={overview?.config.targets.protocol}
            />
          </InputGroup>
        </label>
        <label className="grid gap-1.5 text-sm">
          <span className="font-medium">Engine gRPC URL</span>
          <InputGroup>
            <InputGroupInput
              value={engine}
              onChange={(e) => setEngine(e.target.value)}
              placeholder={overview?.config.targets.engine}
            />
          </InputGroup>
        </label>
        <label className="grid gap-1.5 text-sm">
          <span className="font-medium">Per-check timeout</span>
          <InputGroup>
            <InputGroupInput
              value={timeout}
              onChange={(e) => setTimeoutValue(e.target.value)}
              placeholder={overview?.config.validate.timeout}
            />
          </InputGroup>
        </label>
        <p className="text-muted-foreground text-xs md:col-span-3">
          Empty fields use the config for <b>{overview?.env}</b>, as seen from chaos serve. Targets may be
          http:// (h2c) or https://; a private root goes in <code>[validate] ca_cert</code>.
        </p>
      </div>

      {result?.validate ? (
        <>
          <StatRow
            items={[
              {
                label: "Passed",
                value: result.validate.passed,
                tone: "good",
              },
              {
                label: "Failed",
                value: result.validate.failed,
                tone: result.validate.failed ? "bad" : undefined,
              },
              {
                label: "Took",
                value: seconds(result.duration_s),
              },
              {
                label: "Ran",
                value: <span className="text-base font-normal">{when(result.started_at)}</span>,
              },
            ]}
          />
          <div className="grid gap-6 lg:grid-cols-[16rem_1fr]">
            <FilterRail
              groups={[
                { title: "Surface", options: surfaces },
                {
                  title: "Result",
                  options: [
                    {
                      value: "pass",
                      label: "Pass",
                      count: checks.filter((c) => c.passed).length,
                    },
                    {
                      value: "fail",
                      label: "Fail",
                      count: checks.filter((c) => !c.passed).length,
                    },
                  ],
                },
              ]}
              selected={selected}
              onChange={(g, v) => setSelected({ ...selected, [g]: v })}
              onReset={() => setSelected({})}
            />
            <div className="grid content-start gap-3">
              <InputGroup className="max-w-sm">
                <InputGroupAddon>
                  <Search />
                </InputGroupAddon>
                <InputGroupInput
                  placeholder="Search checks"
                  value={q}
                  onChange={(e) => setQ(e.target.value)}
                />
              </InputGroup>
              <ChecksTable checks={shown} />
            </div>
          </div>
        </>
      ) : (
        <div className="text-muted-foreground rounded-xl border border-dashed p-10 text-center text-sm">
          Run validate to see each check with its latency and what it observed.
        </div>
      )}

      <section>
        <h2 className="mb-3 text-base font-semibold">Previous validate runs</h2>
        <div className="rounded-xl border">
          <RunsTable runs={(history.data ?? []).filter((r) => r.kind === "validate").slice(0, 10)} />
        </div>
      </section>
    </>
  );
}
