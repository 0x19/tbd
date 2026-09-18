"use client";

// Linked external accounts the system pulls documents from. The kinds come
// from the server's registry; a kind the server is not configured for is
// shown, greyed, with what is missing -- not hidden.
import { Plus, RefreshCw, Trash2, Wrench } from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";

import { useFinance } from "@/app/providers";
import { PageTitle } from "@/components/kit";
import { StatusBadge } from "@/components/status-badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Skeleton } from "@/components/ui/skeleton";
import { api } from "@/lib/api/client";
import { describe, useFetch } from "@/lib/api/hooks";
import type { Connector, ConnectorKind, ConnectorRun } from "@/lib/api/schema";
import { ago, when } from "@/lib/format";

export default function ConnectorsPage() {
  const { parties, partyIds, partyName, multi } = useFinance();
  const kinds = useFetch(() => api.connectorKinds(), 0);
  const list = useFetch(() => api.connectors(partyIds), 30_000, [partyIds.join(",")]);
  const [party, setParty] = useState("");
  const chosen = party || partyIds[0] || "";
  const [busy, setBusy] = useState("");

  const start = async (kind: string) => {
    setBusy(kind);
    try {
      const r = await api.startConnector(chosen, kind);
      window.location.href = r.url;
    } catch (e) {
      toast.error(describe(e));
      setBusy("");
    }
  };

  return (
    <>
      <PageTitle
        title="Connectors"
        description="Mailboxes and portals the system pulls receipts from. Read-only, and only what you link."
      >
        {parties.length > 1 ? (
          <Select value={chosen} onValueChange={setParty}>
            <SelectTrigger className="w-44">
              <SelectValue placeholder="Party" />
            </SelectTrigger>
            <SelectContent>
              {parties.map((p) => (
                <SelectItem key={p.id} value={p.id}>
                  {p.display_name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        ) : null}
      </PageTitle>

      <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
        {(kinds.data?.kinds ?? []).map((k: ConnectorKind) => (
          <Card key={k.name} className={k.configured ? undefined : "opacity-70"}>
            <CardHeader>
              <CardTitle>{k.label}</CardTitle>
              <CardDescription>{k.description}</CardDescription>
            </CardHeader>
            <CardContent className="space-y-3">
              <p className="text-muted-foreground text-xs">{k.consent_note}</p>
              {k.configured ? (
                <Button size="sm" onClick={() => void start(k.name)} disabled={busy !== "" || !chosen}>
                  <Plus /> {busy === k.name ? "Opening…" : `Link ${k.label.split(" /")[0]}`}
                </Button>
              ) : (
                <p className="text-xs text-amber-600">Not configured on the server yet.</p>
              )}
            </CardContent>
          </Card>
        ))}
      </div>

      {list.error ? <p className="text-destructive text-sm">{list.error}</p> : null}
      {list.loading && !list.data ? <Skeleton className="h-40 w-full" /> : null}
      <div className="space-y-4">
        {(list.data?.connectors ?? []).map((c) => (
          <ConnectorCard key={c.id} c={c} partyName={multi ? partyName : undefined} onChanged={list.reload} />
        ))}
        {list.data && list.data.connectors.length === 0 ? (
          <p className="text-muted-foreground text-sm">Nothing linked yet.</p>
        ) : null}
      </div>
    </>
  );
}

function ConnectorCard({
  c,
  partyName,
  onChanged,
}: {
  c: Connector;
  partyName?: (id: string) => string;
  onChanged: () => void;
}) {
  const [busy, setBusy] = useState("");
  const [showRuns, setShowRuns] = useState(false);
  const [query, setQuery] = useState(() => {
    try {
      return (JSON.parse(c.config || "{}") as { query?: string }).query ?? "";
    } catch {
      return "";
    }
  });
  const runs = useFetch(() => api.connectorRuns(c.id), 0, [c.id, showRuns, c.last_sync_at]);
  const act = async (what: string, f: () => Promise<string>) => {
    setBusy(what);
    try {
      toast.success(await f());
      onChanged();
    } catch (e) {
      toast.error(describe(e));
      onChanged();
    } finally {
      setBusy("");
    }
  };
  return (
    <Card>
      <CardHeader className="flex flex-row flex-wrap items-start justify-between gap-2">
        <div>
          <CardTitle className="flex items-center gap-2">
            {c.label || c.kind}
            <StatusBadge status={c.status} className="text-[10px]" />
          </CardTitle>
          <CardDescription>
            {c.kind}
            {partyName ? ` · ${partyName(c.party_id)}` : ""} · linked {c.linked_at ? when(c.linked_at) : "—"}{" "}
            · last sync {c.last_sync_at ? `${ago(c.last_sync_at)} (${c.last_sync_status})` : "never"}
            {c.last_sync_error ? <span className="text-destructive"> · {c.last_sync_error}</span> : null}
            {c.failure ? <span className="text-destructive"> · {c.failure}</span> : null}
          </CardDescription>
        </div>
        <div className="flex flex-wrap gap-1">
          <Button
            variant="outline"
            size="sm"
            disabled={busy !== "" || c.status !== "linked"}
            onClick={() => void act("test", async () => `OK: ${(await api.testConnector(c.id)).status}`)}
          >
            <Wrench /> {busy === "test" ? "Testing…" : "Test"}
          </Button>
          <Button
            variant="outline"
            size="sm"
            disabled={busy !== "" || c.status !== "linked"}
            onClick={() =>
              void act("sync", async () => {
                const r = await api.syncConnector(c.id);
                return `Found ${r.found}, stored ${r.stored} new, ${r.skipped} already known.`;
              })
            }
          >
            <RefreshCw className={busy === "sync" ? "animate-spin" : undefined} />{" "}
            {busy === "sync" ? "Pulling…" : "Pull now"}
          </Button>
          <Button variant="ghost" size="sm" onClick={() => setShowRuns((v) => !v)}>
            Runs
          </Button>
          <Button
            variant="ghost"
            size="sm"
            disabled={busy !== ""}
            onClick={() => {
              if (window.confirm("Remove this connector? Documents already pulled stay."))
                void act("delete", async () => {
                  await api.deleteConnector(c.id);
                  return "Removed.";
                });
            }}
          >
            <Trash2 />
          </Button>
        </div>
      </CardHeader>
      <CardContent className="space-y-3">
        {c.kind === "gmail" ? (
          <div className="flex flex-wrap items-end gap-2">
            <div className="min-w-72 flex-1">
              <Label className="text-muted-foreground mb-1.5 block text-xs">
                Gmail search (empty: any PDF attachment)
              </Label>
              <Input
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                placeholder="has:attachment filename:pdf (from:hetzner.com OR from:anthropic.com)"
                className="font-mono text-xs"
              />
            </div>
            <Button
              variant="outline"
              size="sm"
              disabled={busy !== ""}
              onClick={() =>
                void act("config", async () => {
                  await api.configureConnector(c.id, JSON.stringify({ query }));
                  return "Saved.";
                })
              }
            >
              Save
            </Button>
          </div>
        ) : null}
        {showRuns ? (
          <div className="text-xs">
            {(runs.data?.runs ?? []).length === 0 ? (
              <p className="text-muted-foreground">No runs yet.</p>
            ) : null}
            {(runs.data?.runs ?? []).map((r: ConnectorRun) => (
              <div key={r.id} className="flex flex-wrap gap-3 border-b py-1 last:border-0">
                <span className="text-muted-foreground w-36">{when(r.started_at)}</span>
                <span>{r.trigger}</span>
                <StatusBadge status={r.outcome || "running"} className="text-[10px]" />
                <span>
                  found {r.found} · stored {r.stored} · skipped {r.skipped}
                </span>
                {r.error ? <span className="text-destructive">{r.error}</span> : null}
              </div>
            ))}
          </div>
        ) : null}
      </CardContent>
    </Card>
  );
}
