"use client";

// Linked external accounts the system pulls documents from. One flat list,
// drawn from the server's connectors feed: a pull's progress and outcome,
// a relink, a removal all arrive as events, so nothing here is stale and
// nothing polls while the stream is up. The kinds come from the server's
// registry; a kind the server is not configured for is shown, greyed, with
// what is missing -- not hidden.
import { History, KeyRound, Loader2, Plus, RefreshCw, Trash2, Wrench } from "lucide-react";
import { useCallback, useEffect, useMemo, useState } from "react";
import { toast } from "sonner";

import { useFinance } from "@/app/providers";
import { PageTitle } from "@/components/kit";
import { Dot, StatusBadge } from "@/components/status-badge";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Skeleton } from "@/components/ui/skeleton";
import { api } from "@/lib/api/client";
import { describe, useEvents, useFetch } from "@/lib/api/hooks";
import {
  type Connector,
  type ConnectorKind,
  type ConnectorRun,
  WatchConnectorsResponse,
} from "@/lib/api/schema";
import { ago, when } from "@/lib/format";
import { useT } from "@/lib/i18n";

type Entry = { connector: Connector; run: ConnectorRun | null };

type T = ReturnType<typeof useT>;

/** A kind's text in the page's language when the dictionary has it, else the server's. */
function kindText(t: T, k: ConnectorKind, field: "label" | "description" | "consent_note"): string {
  const key = `kinds.${k.name}.${field}`;
  const v = t(key);
  return v === key ? k[field] : v;
}

export default function ConnectorsPage() {
  const t = useT();
  const { parties, partyIds, partyName, multi } = useFinance();
  const key = partyIds.join(",");
  const kinds = useFetch(() => api.connectorKinds(), 0);

  // The feed is the source of truth once it is up. Until then, and whenever
  // the browser is between reconnects, the plain list fills in (every 10 s).
  const [entries, setEntries] = useState<Record<string, Entry>>({});
  const apply = useCallback((ev: WatchConnectorsResponse) => {
    setEntries((prev) => {
      const next = { ...prev };
      if (ev.deleted) delete next[ev.connector.id];
      else next[ev.connector.id] = { connector: ev.connector, run: ev.run ?? null };
      return next;
    });
  }, []);
  const feed = useEvents(
    partyIds.length ? api.connectorEventsUrl(partyIds) : null,
    WatchConnectorsResponse,
    apply,
  );
  const list = useFetch(() => api.connectors(partyIds), feed.live ? 0 : 10_000, [key, feed.live]);
  useEffect(() => {
    if (!list.data) return;
    setEntries((prev) => {
      const next: Record<string, Entry> = {};
      for (const c of list.data!.connectors) next[c.id] = { connector: c, run: prev[c.id]?.run ?? null };
      return next;
    });
  }, [list.data]);
  useEffect(() => setEntries({}), [key]);

  const rows = useMemo(
    () =>
      Object.values(entries).sort(
        (a, b) =>
          partyName(a.connector.party_id).localeCompare(partyName(b.connector.party_id)) ||
          (a.connector.label || a.connector.kind).localeCompare(b.connector.label || b.connector.kind),
      ),
    [entries, partyName],
  );
  const [linking, setLinking] = useState(false);

  return (
    <>
      <PageTitle title={t("connectors.title")} description={t("connectors.description")}>
        <div className="flex items-center gap-3">
          <Dot
            tone={feed.live ? "good" : "off"}
            label={feed.live ? t("connectors.live") : t("connectors.polling")}
          />
          <Button size="sm" onClick={() => setLinking(true)}>
            <Plus /> {t("connectors.link_mailbox")}
          </Button>
        </div>
      </PageTitle>

      <LinkDialog
        open={linking}
        onOpenChange={setLinking}
        kinds={kinds.data?.kinds ?? []}
        parties={parties}
        defaultParty={partyIds[0] ?? ""}
      />

      {list.error ? <p className="text-destructive text-sm">{list.error}</p> : null}
      {feed.error ? (
        <p className="text-destructive text-sm">{t("connectors.feed_error", { error: feed.error })}</p>
      ) : null}
      {list.loading && !list.data ? <Skeleton className="h-40 w-full" /> : null}
      <div className="space-y-3">
        {rows.map((e) => (
          <ConnectorRow
            key={e.connector.id}
            entry={e}
            parties={multi ? parties : []}
            partyName={partyName}
            onChanged={list.reload}
          />
        ))}
        {list.data && rows.length === 0 ? (
          <p className="text-muted-foreground text-sm">{t("connectors.nothing_linked")}</p>
        ) : null}
      </div>
    </>
  );
}

function LinkDialog({
  open,
  onOpenChange,
  kinds,
  parties,
  defaultParty,
}: {
  open: boolean;
  onOpenChange: (v: boolean) => void;
  kinds: ConnectorKind[];
  parties: { id: string; display_name: string }[];
  defaultParty: string;
}) {
  const t = useT();
  const [kind, setKind] = useState("");
  const [party, setParty] = useState(defaultParty);
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    if (open) {
      setParty(defaultParty);
      setKind(kinds.find((k) => k.configured)?.name ?? "");
    }
  }, [open, defaultParty, kinds]);
  const chosen = kinds.find((k) => k.name === kind);
  const start = async () => {
    setBusy(true);
    try {
      const r = await api.startConnector(party, kind);
      window.location.href = r.url;
    } catch (e) {
      toast.error(describe(e));
      setBusy(false);
    }
  };
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>{t("connectors.link_mailbox")}</DialogTitle>
          <DialogDescription>{t("connectors.link_dialog_desc")}</DialogDescription>
        </DialogHeader>
        <div className="space-y-4">
          <div className="space-y-2">
            <Label>{t("connectors.kind")}</Label>
            <div className="grid gap-2">
              {kinds.map((k) => (
                <button
                  key={k.name}
                  type="button"
                  disabled={!k.configured}
                  onClick={() => setKind(k.name)}
                  className={
                    "rounded-md border p-3 text-left text-sm disabled:opacity-60 " +
                    (kind === k.name ? "border-foreground" : "hover:bg-muted/50")
                  }
                >
                  <div className="font-medium">{kindText(t, k, "label")}</div>
                  <div className="text-muted-foreground text-xs">{kindText(t, k, "description")}</div>
                  {!k.configured ? (
                    <div className="text-xs text-amber-600">{t("connectors.not_configured")}</div>
                  ) : null}
                </button>
              ))}
            </div>
          </div>
          <div className="space-y-2">
            <Label>{t("connectors.belongs_to")}</Label>
            <Select value={party} onValueChange={setParty}>
              <SelectTrigger className="w-full">
                <SelectValue placeholder={t("common.party")} />
              </SelectTrigger>
              <SelectContent>
                {parties.map((p) => (
                  <SelectItem key={p.id} value={p.id}>
                    {p.display_name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          {chosen ? (
            <p className="text-muted-foreground text-xs">{kindText(t, chosen, "consent_note")}</p>
          ) : null}
        </div>
        <DialogFooter>
          <Button onClick={() => void start()} disabled={busy || !kind || !party}>
            {busy
              ? t("connectors.opening")
              : t("connectors.continue_to", {
                  provider: chosen ? kindText(t, chosen, "label").split(" /")[0] : t("connectors.provider"),
                })}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function ConnectorRow({
  entry: { connector: c, run },
  parties,
  partyName,
  onChanged,
}: {
  entry: Entry;
  parties: { id: string; display_name: string }[];
  partyName: (id: string) => string;
  onChanged: () => void;
}) {
  const t = useT();
  const [busy, setBusy] = useState("");
  const [open, setOpen] = useState<"" | "settings" | "history">("");
  const [query, setQuery] = useState(() => {
    try {
      return (JSON.parse(c.config || "{}") as { query?: string }).query ?? "";
    } catch {
      return "";
    }
  });
  const pulling = run !== null && run.outcome === "";
  const history = useFetch(() => api.connectorRuns(c.id), 0, [
    c.id,
    open === "history",
    run?.outcome,
    run?.id,
  ]);
  const act = async (what: string, f: () => Promise<string>) => {
    setBusy(what);
    try {
      const msg = await f();
      if (msg) toast.success(msg);
      onChanged();
    } catch (e) {
      toast.error(describe(e));
    } finally {
      setBusy("");
    }
  };
  return (
    <Card>
      <CardContent className="space-y-3 pt-4">
        <div className="flex flex-wrap items-start justify-between gap-3">
          <div className="min-w-0 space-y-1">
            <div className="flex flex-wrap items-center gap-2">
              <span className="font-medium">{c.label || c.kind}</span>
              <Badge variant="outline" className="text-[10px]">
                {c.kind}
              </Badge>
              {c.status === "linked" ? (
                <Badge
                  variant="outline"
                  className={
                    c.can_send
                      ? "border-transparent bg-emerald-600/12 text-[10px] text-emerald-700 dark:text-emerald-300"
                      : "text-muted-foreground border-dashed text-[10px]"
                  }
                  title={c.can_send ? t("connectors.can_send_hint") : t("connectors.cannot_send_hint")}
                >
                  {c.can_send ? t("connectors.can_send") : t("connectors.cannot_send")}
                </Badge>
              ) : null}
              {pulling ? (
                <Badge
                  variant="outline"
                  className="gap-1 border-transparent bg-amber-500/15 text-[10px] text-amber-700 dark:text-amber-300"
                >
                  <Loader2 className="size-3 animate-spin" /> {t("status.syncing")}
                </Badge>
              ) : (
                <StatusBadge status={c.status} className="text-[10px]" />
              )}
            </div>
            <RunLine run={run} c={c} />
            {c.failure ? <p className="text-destructive text-xs">{c.failure}</p> : null}
          </div>
          <div className="flex flex-wrap items-center gap-1">
            {parties.length > 1 ? (
              <Select
                value={c.party_id}
                onValueChange={(party_id) =>
                  void act("party", async () => {
                    await api.configureConnector(c.id, { party_id });
                    return t("connectors.moved_to", { party: partyName(party_id) });
                  })
                }
              >
                <SelectTrigger className="h-8 w-40 text-xs" disabled={busy !== ""}>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {parties.map((p) => (
                    <SelectItem key={p.id} value={p.id}>
                      {p.display_name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            ) : (
              <span className="text-muted-foreground mr-2 text-xs">{partyName(c.party_id)}</span>
            )}
            <Button
              // Consent again on Google's side for the same mailbox. This is
              // how a permission asked for later -- sending -- reaches a
              // mailbox linked before it; a refresh can renew a token, never
              // widen it. The callback merges into this row, so the history
              // and the documents stay. Loud while sending is not allowed.
              variant={c.can_send ? "ghost" : "outline"}
              size="sm"
              className={c.can_send ? undefined : "border-amber-500/50 text-amber-700 dark:text-amber-300"}
              disabled={busy !== "" || c.status === "pending"}
              onClick={() =>
                void act("relink", async () => {
                  const r = await api.startConnector(c.party_id, c.kind);
                  window.location.href = r.url;
                  return "";
                })
              }
              title={t("connectors.relink_title")}
            >
              <KeyRound />{" "}
              {busy === "relink"
                ? t("connectors.relink_opening")
                : c.can_send
                  ? t("connectors.relink")
                  : t("connectors.allow_sending")}
            </Button>
            <Button
              variant="outline"
              size="sm"
              disabled={busy !== "" || pulling || c.status !== "linked"}
              onClick={() =>
                void act("sync", async () => {
                  // Opens the run; what it does arrives on the feed.
                  await api.syncConnector(c.id);
                  return "";
                })
              }
            >
              <RefreshCw className={pulling ? "animate-spin" : undefined} />{" "}
              {pulling ? t("connectors.pulling") : t("connectors.pull_now")}
            </Button>
            <Button
              variant="ghost"
              size="sm"
              // An expired row is the one a test is for: the credential may
              // still work (a pull can misjudge a refused token), and a test
              // that succeeds links it again. A pull stays a linked row's.
              disabled={busy !== "" || !["linked", "expired"].includes(c.status)}
              onClick={() =>
                void act("test", async () =>
                  t("connectors.test_ok", { status: (await api.testConnector(c.id)).status }),
                )
              }
              title={t("connectors.test_title")}
            >
              <Wrench /> {busy === "test" ? t("connectors.testing") : t("connectors.test")}
            </Button>
            <Button variant="ghost" size="sm" onClick={() => setOpen(open === "history" ? "" : "history")}>
              <History /> {t("connectors.history")}
            </Button>
            {c.kind === "gmail" ? (
              <Button
                variant="ghost"
                size="sm"
                onClick={() => setOpen(open === "settings" ? "" : "settings")}
              >
                {t("connectors.filter")}
              </Button>
            ) : null}
            <Button
              variant="ghost"
              size="sm"
              disabled={busy !== ""}
              title={t("common.remove")}
              onClick={() => {
                if (window.confirm(t("connectors.remove_confirm")))
                  void act("delete", async () => {
                    await api.deleteConnector(c.id);
                    return t("common.removed");
                  });
              }}
            >
              <Trash2 />
            </Button>
          </div>
        </div>

        {open === "settings" ? (
          <div className="flex flex-wrap items-end gap-2 border-t pt-3">
            <div className="min-w-72 flex-1">
              <Label className="text-muted-foreground mb-1.5 block text-xs">
                {t("connectors.gmail_search")}
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
                  await api.configureConnector(c.id, { config: JSON.stringify({ query }) });
                  return t("common.saved");
                })
              }
            >
              {t("common.save")}
            </Button>
          </div>
        ) : null}

        {open === "history" ? (
          <div className="border-t pt-3 text-xs">
            {(history.data?.runs ?? []).length === 0 ? (
              <p className="text-muted-foreground">{t("connectors.no_runs")}</p>
            ) : null}
            {(history.data?.runs ?? []).map((r: ConnectorRun) => (
              <div key={r.id} className="flex flex-wrap items-center gap-3 border-b py-1 last:border-0">
                <span className="text-muted-foreground w-36">{when(r.started_at)}</span>
                <span className="w-16">{triggerWord(t, r.trigger)}</span>
                <StatusBadge status={r.outcome || "running"} className="text-[10px]" />
                <span>
                  {t("connectors.run_counts", { found: r.found, stored: r.stored, skipped: r.skipped })}
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

/** A run's trigger word in the page's language when the dictionary has it, else as sent. */
function triggerWord(t: T, trigger: string): string {
  const key = `status.${trigger}`;
  const v = t(key);
  return v === key ? trigger : v;
}

/** What the latest run is doing or did, in one line that moves while it pulls. */
function RunLine({ run, c }: { run: ConnectorRun | null; c: Connector }) {
  const t = useT();
  if (run && run.outcome === "") {
    return (
      <p className="text-muted-foreground text-xs">
        {t("connectors.pulling_since", {
          ago: ago(run.started_at),
          found: run.found,
          stored: run.stored,
          skipped: run.skipped,
        })}
      </p>
    );
  }
  if (run) {
    return (
      <p className="text-muted-foreground text-xs">
        {t("connectors.last_pull", { ago: ago(run.finished_at || run.started_at) })} ·{" "}
        <StatusBadge status={run.outcome} className="text-[10px]" /> ·{" "}
        {t("connectors.run_summary", { found: run.found, stored: run.stored, skipped: run.skipped })}
        {run.outcome === "partial" ? ` · ${t("connectors.more_remains")}` : ""}
        {run.error ? <span className="text-destructive"> · {run.error}</span> : null}
      </p>
    );
  }
  return (
    <p className="text-muted-foreground text-xs">
      {c.linked_at ? t("connectors.linked_at", { when: when(c.linked_at) }) : t("connectors.not_linked")} ·{" "}
      {t("connectors.never_pulled")}
    </p>
  );
}
