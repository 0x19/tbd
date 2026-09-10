"use client";

import { useState } from "react";
import { Play } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Field } from "@/components/field";
import { ChecksTable } from "@/app/runs/view/page";
import { Empty, PageHeader } from "@/components/page-header";
import { RunsTable } from "@/components/runs-table";
import { StatCard } from "@/components/stat-card";
import { useChaos } from "@/components/shell/providers";
import { api } from "@/lib/api/client";
import { describe, useFetch } from "@/lib/api/hooks";
import type { RunRecord } from "@/lib/api/schema";
import { seconds, when } from "@/lib/format";

/** `chaos validate` from the browser, against the config's targets or any pair of URLs. */
export default function ValidatePage() {
  const { overview, reload } = useChaos();
  const history = useFetch(() => api.runs(100), 5000);
  const [protocol, setProtocol] = useState("");
  const [engine, setEngine] = useState("");
  const [timeout, setTimeoutValue] = useState("");
  const [busy, setBusy] = useState(false);
  const [result, setResult] = useState<RunRecord | null>(null);

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

  const shown = result;
  return (
    <>
      <PageHeader
        title="Validate"
        description="Eleven checks, one per surface, run concurrently with a timeout each. Green means the stack answers on every protocol the way the contract says."
      >
        <Button size="sm" onClick={run} disabled={busy}>
          <Play /> {busy ? "running…" : "run validate"}
        </Button>
      </PageHeader>
      <Card size="sm">
        <CardHeader>
          <CardTitle>Targets</CardTitle>
          <CardDescription>
            Empty fields use the config: {overview?.config.targets.protocol} and{" "}
            {overview?.config.targets.engine}
            {overview?.env ? ` (${overview.env})` : ""}.
          </CardDescription>
        </CardHeader>
        <CardContent className="grid gap-3 md:grid-cols-3">
          <Field label="Protocol base URL">
            <Input
              value={protocol}
              onChange={(e) => setProtocol(e.target.value)}
              placeholder={overview?.config.targets.protocol}
            />
          </Field>
          <Field label="Engine gRPC URL">
            <Input
              value={engine}
              onChange={(e) => setEngine(e.target.value)}
              placeholder={overview?.config.targets.engine}
            />
          </Field>
          <Field label="Per-check timeout">
            <Input
              value={timeout}
              onChange={(e) => setTimeoutValue(e.target.value)}
              placeholder={overview?.config.validate.timeout}
            />
          </Field>
        </CardContent>
      </Card>
      {shown?.validate ? (
        <>
          <div className="grid gap-4 md:grid-cols-3">
            <StatCard label="Passed" value={shown.validate.passed} tone="good" />
            <StatCard
              label="Failed"
              value={shown.validate.failed}
              tone={shown.validate.failed ? "bad" : "good"}
            />
            <StatCard label="Took" value={seconds(shown.duration_s)} hint={when(shown.started_at)} />
          </div>
          <ChecksTable checks={shown.validate.checks} />
        </>
      ) : (
        <Empty>Run validate to see each check.</Empty>
      )}
      <Card>
        <CardHeader>
          <CardTitle>Previous validate runs</CardTitle>
        </CardHeader>
        <CardContent>
          <RunsTable runs={(history.data ?? []).filter((r) => r.kind === "validate")} />
        </CardContent>
      </Card>
    </>
  );
}
