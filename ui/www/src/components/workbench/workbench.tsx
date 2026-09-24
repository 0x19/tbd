"use client";

import Link from "next/link";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { MAX_SESSIONS, WORKBENCH_STORE as STORE } from "@/components/chat/sessions";
import {
  type ChatTurn,
  ChatView,
  CodeRun,
  type CodeTurn,
  contextOf,
  GO_START,
  newId,
  RUST_START,
  startTurn,
  streamTurn,
  type Tier,
  TIER_NAME,
} from "@/components/chat/turn";
import { Button } from "@/components/ui/button";
import { LivePanel } from "@/components/workbench/live-panel";
import { type Agent, useAgents, useBudget } from "@/lib/agents";
import { useArena } from "@/lib/arena";
import { useT } from "@/lib/i18n";
import { listTools, mcp, type Tool, toolAnswer, type Transport, TRANSPORTS } from "@/lib/llm";
import { useMeState } from "@/lib/me";
import { runCode, type RunLanguage } from "@/lib/runner";
import { cn } from "@/lib/utils";

/** One entry of the transcript. */
type Turn =
  | ChatTurn
  | CodeTurn
  | { kind: "tools"; id: string; tools: Tool[] | null; error?: string }
  | { kind: "tool"; id: string; name: string; args: unknown; result?: unknown; error?: boolean; ms?: number }
  | { kind: "connect"; id: string }
  | { kind: "help"; id: string };

/** A session talks to the bare model, or to one agent (`agent`, RFC 0011). */
type Session = { id: string; title: string; createdAt: number; agent?: string; turns: Turn[] };

const MAX_TOKENS = 1024;

const blank = (agent?: string): Session => ({
  id: newId(),
  title: "",
  createdAt: Date.now(),
  agent,
  turns: [],
});

function load(): Session[] {
  try {
    const raw = window.localStorage.getItem(STORE);
    const parsed = raw ? (JSON.parse(raw) as Session[]) : [];
    // A turn that was running when the page closed did not finish.
    return parsed.map((s) => ({
      ...s,
      turns: s.turns.map((t) =>
        t.kind === "chat" && t.status === "running" ? { ...t, status: "stopped" } : t,
      ),
    }));
  } catch {
    return [];
  }
}

function save(sessions: Session[]) {
  try {
    window.localStorage.setItem(STORE, JSON.stringify(sessions.slice(0, MAX_SESSIONS)));
  } catch {
    /* a full or blocked store only loses the history, never the page */
  }
}

const COMMANDS = [
  { name: "/tier fast", help: "wb.cmd.tier_fast" },
  { name: "/tier deep", help: "wb.cmd.tier_deep" },
  { name: "/reason on", help: "wb.cmd.reason_on" },
  { name: "/reason off", help: "wb.cmd.reason_off" },
  { name: "/transport sse", help: "wb.cmd.sse" },
  { name: "/transport websocket", help: "wb.cmd.websocket" },
  { name: "/transport mcp", help: "wb.cmd.mcp" },
  { name: "/agent site", help: "wb.cmd.agent" },
  { name: "/agent none", help: "wb.cmd.agent_none" },
  { name: "/tools", help: "wb.cmd.tools" },
  { name: "/run go", help: "wb.cmd.run_go" },
  { name: "/run rust", help: "wb.cmd.run_rust" },
  { name: "/connect", help: "wb.cmd.connect" },
  { name: "/clear", help: "wb.cmd.clear" },
  { name: "/new", help: "wb.cmd.new" },
  { name: "/help", help: "wb.cmd.help" },
] as const;

/**
 * The model lab's workbench: a keyboard-first page in the manner of a
 * terminal agent. Sessions kept in this browser, a transcript that streams
 * (the model's reasoning shown while it thinks and folded once it answers),
 * a prompt with a slash menu and a status line, three transports for the same
 * turn, the platform's MCP tools callable as cards, the agents the service
 * speaks as (a session talks to one, or to the bare model), and the arena's
 * live view beside it all. `app/lab/llm/workbench/page.tsx` carries the metadata.
 */
export function Workbench() {
  const t = useT();
  const { me, known } = useMeState();
  const admin = me?.role === "admin";
  const arena = useArena(admin);
  const agents = useAgents(admin);
  const { budget, refresh: refreshBudget } = useBudget(admin);

  const [sessions, setSessions] = useState<Session[]>([]);
  const [current, setCurrent] = useState<string>("");
  const [tier, setTier] = useState<Tier>("TIER_FAST");
  const [reasoning, setReasoning] = useState(false);
  const [transport, setTransport] = useState<Transport>("sse");
  const [prompt, setPrompt] = useState("");
  const [menu, setMenu] = useState(0);
  const [panel, setPanel] = useState(true);
  const [now, setNow] = useState(() => Date.now());
  const abort = useRef<AbortController | null>(null);
  const scrollRef = useRef<HTMLDivElement | null>(null);
  const inputRef = useRef<HTMLTextAreaElement | null>(null);

  useEffect(() => {
    const loaded = load();
    const first = loaded[0] ?? blank();
    setSessions(loaded.length ? loaded : [first]);
    setCurrent(first.id);
  }, []);
  useEffect(() => {
    if (sessions.length) save(sessions);
  }, [sessions]);

  const session = sessions.find((s) => s.id === current);
  const turns = useMemo(() => session?.turns ?? [], [session]);
  const running = turns.some((x) => x.kind === "chat" && x.status === "running");
  const agentId = session?.agent ?? "";
  const agent: Agent | undefined = agentId ? agents?.find((a) => a.id === agentId) : undefined;

  // A clock for the live rate while a turn runs.
  useEffect(() => {
    if (!running) return;
    const timer = window.setInterval(() => setNow(Date.now()), 250);
    return () => window.clearInterval(timer);
  }, [running]);
  // Follow the newest turn by scrolling the transcript's own box, never the
  // window. (A block body on purpose: an effect that returns anything but a
  // function, such as the promise `scrollIntoView` now returns, crashes the
  // page when React calls it as the cleanup.)
  useEffect(() => {
    const box = scrollRef.current;
    if (box) box.scrollTop = box.scrollHeight;
  }, [turns]);

  // Ready to type on arrival, without the jump `autoFocus` makes by scrolling the
  // window to the prompt.
  useEffect(() => {
    if (admin) inputRef.current?.focus({ preventScroll: true });
  }, [admin]);

  // A turn's updates go to the session it started in, whichever one is open now.
  const edit = useCallback(
    (sid: string, id: string, f: (turn: Turn) => Turn) =>
      setSessions((all) =>
        all.map((s) => (s.id === sid ? { ...s, turns: s.turns.map((x) => (x.id === id ? f(x) : x)) } : s)),
      ),
    [],
  );
  const append = useCallback(
    (turn: Turn, title?: string) =>
      setSessions((all) =>
        all.map((s) =>
          s.id === current ? { ...s, title: s.title || title || s.title, turns: [...s.turns, turn] } : s,
        ),
      ),
    [current],
  );

  const ask = useCallback(
    async (
      question: string,
      over: Transport = transport,
      withTier: Tier = tier,
      withReasoning = reasoning,
    ) => {
      const sid = current;
      // With an agent its own tier and bounds apply: the request leaves them out.
      const turn = agent
        ? startTurn(question, over, agent.tier, false, agent.id)
        : startTurn(question, over, withTier, withReasoning);
      append(turn, question.slice(0, 60));
      abort.current?.abort();
      const controller = new AbortController();
      abort.current = controller;
      const messages = [...contextOf(turns), { role: "user" as const, content: question }];
      const req = agent
        ? { messages, agent: agent.id }
        : { messages, tier: withTier, max_tokens: MAX_TOKENS, reasoning: withReasoning };
      await streamTurn(turn, req, controller.signal, (f) =>
        edit(sid, turn.id, (x) => (x.kind === "chat" ? f(x) : x)),
      );
      void refreshBudget();
    },
    [agent, append, current, edit, reasoning, refreshBudget, tier, transport, turns],
  );

  /** Run a program in the sandbox and show what it printed (RFC 0010). */
  const runProgram = useCallback(
    async (language: RunLanguage, source: string) => {
      const sid = current;
      const id = newId();
      append({ kind: "code", id, language, source, status: "running" });
      const r = await runCode(language, source);
      edit(sid, id, (x) =>
        x.kind !== "code"
          ? x
          : r.ok
            ? { ...x, status: "done", result: r.result }
            : { ...x, status: "error", error: r.error, code: r.code },
      );
    },
    [append, current, edit],
  );

  const runTool = useCallback(
    async (name: string, args: Record<string, unknown>) => {
      const sid = current;
      const id = newId();
      append({ kind: "tool", id, name, args });
      const started = performance.now();
      const r = await mcp("tools/call", { name, arguments: args }).catch((e: unknown) => ({
        ok: false as const,
        code: "error",
        error: String(e),
      }));
      const ms = performance.now() - started;
      if (!r.ok)
        edit(sid, id, (x) => ({ ...x, result: { code: r.code, error: r.error }, error: true, ms }) as Turn);
      else {
        const { value, error } = toolAnswer(r.result);
        edit(sid, id, (x) => ({ ...x, result: value, error, ms }) as Turn);
      }
    },
    [append, current, edit],
  );

  const command = useCallback(
    async (line: string) => {
      const [name, arg] = line.trim().split(/\s+/, 2);
      switch (name) {
        case "/tier":
          setTier(arg === "deep" ? "TIER_DEEP" : "TIER_FAST");
          return;
        case "/reason":
          setReasoning(arg !== "off");
          return;
        case "/transport":
          if (TRANSPORTS.includes(arg as Transport)) setTransport(arg as Transport);
          return;
        case "/agent": {
          // Only an agent this caller may use; anything else is the bare model.
          const next = agents?.find((a) => a.id === arg && a.available)?.id;
          setSessions((all) => all.map((s) => (s.id === current ? { ...s, agent: next } : s)));
          return;
        }
        case "/clear":
          setSessions((all) => all.map((s) => (s.id === current ? { ...s, turns: [] } : s)));
          return;
        case "/new": {
          const s = blank(agentId || undefined);
          setSessions((all) => [s, ...all]);
          setCurrent(s.id);
          return;
        }
        case "/connect":
          append({ kind: "connect", id: newId() });
          return;
        case "/tools": {
          const sid = current;
          const id = newId();
          append({ kind: "tools", id, tools: null });
          const r = await listTools();
          edit(
            sid,
            id,
            (x) => ({ ...x, ...(r.ok ? { tools: r.tools } : { tools: [], error: r.error }) }) as Turn,
          );
          return;
        }
        default:
          append({ kind: "help", id: newId() });
      }
    },
    [agentId, agents, append, current, edit],
  );

  // `/agent <id>` for every agent this caller may use, beside the fixed commands.
  const commands = useMemo(
    () => [
      ...COMMANDS.filter((c) => !c.name.startsWith("/agent ") || c.name === "/agent none"),
      ...(agents ?? [])
        .filter((a) => a.available)
        .map((a) => ({ name: `/agent ${a.id}`, help: "wb.cmd.agent" as const })),
    ],
    [agents],
  );
  const matches = prompt.startsWith("/") ? commands.filter((c) => c.name.startsWith(prompt.trim())) : [];

  const submit = () => {
    const line = prompt.trim();
    if (!line) return;
    // `/run go` or `/run rust` with the program on the lines under it; alone, it
    // puts a starting program in the prompt to edit.
    const run = /^\/run\s+(go|rust)\s*(?:\n([\s\S]*))?$/.exec(line);
    if (run) {
      const language = run[1] as RunLanguage;
      const code = (run[2] ?? "").trim();
      if (code) {
        setPrompt("");
        void runProgram(language, code);
      } else {
        setPrompt(`/run ${language}\n${language === "go" ? GO_START : RUST_START}`);
      }
      setMenu(0);
      return;
    }
    if (line.startsWith("/")) {
      const chosen = matches[menu]?.name ?? line;
      if (/^\/run (go|rust)$/.test(chosen)) {
        const language = chosen.slice(5) as RunLanguage;
        setPrompt(`/run ${language}\n${language === "go" ? GO_START : RUST_START}`);
        setMenu(0);
        return;
      }
      void command(chosen);
      setPrompt("");
      setMenu(0);
      return;
    }
    if (running) return;
    setPrompt("");
    void ask(line);
  };

  const onKey = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (matches.length && (e.key === "ArrowDown" || e.key === "ArrowUp")) {
      e.preventDefault();
      setMenu((m) => (m + (e.key === "ArrowDown" ? 1 : matches.length - 1)) % matches.length);
      return;
    }
    if (matches.length && e.key === "Tab") {
      e.preventDefault();
      setPrompt(matches[menu].name + " ");
      return;
    }
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      submit();
      return;
    }
    if (e.key === "Escape") {
      abort.current?.abort();
      return;
    }
    if (e.key === "ArrowUp" && !prompt) {
      const last = [...turns].reverse().find((x) => x.kind === "chat");
      if (last?.kind === "chat") {
        e.preventDefault();
        setPrompt(last.prompt);
      }
    }
  };

  if (!known) return <div className="min-h-[70dvh] rounded-lg border" aria-busy />;
  if (me === null) {
    return (
      <div className="rounded-lg border p-8">
        <p className="text-muted-foreground text-sm">{t("lab.demo.gated")}</p>
        <Button className="mt-6" asChild>
          <a href="/account/">{t("lab.demo.signin")}</a>
        </Button>
      </div>
    );
  }
  if (!admin) {
    return <p className="text-muted-foreground rounded-lg border p-8 text-sm">{t("wb.not_admin")}</p>;
  }

  const shownTier: Tier = agent?.tier ?? tier;
  const liveTier = arena.snapshot?.tiers.find((x) => x.tier === TIER_NAME[shownTier]);

  return (
    <div
      className={cn(
        "grid min-h-[70dvh] overflow-hidden rounded-lg border lg:h-[calc(100dvh-13rem)]",
        panel ? "lg:grid-cols-[13rem_minmax(0,1fr)_22rem]" : "lg:grid-cols-[13rem_minmax(0,1fr)]",
      )}
    >
      {/* Sessions */}
      <aside className="bg-muted/20 hidden flex-col border-r lg:flex">
        <div className="flex items-center justify-between border-b px-3 py-2">
          <span className="font-mono text-[11px] tracking-[0.12em] uppercase">{t("wb.sessions")}</span>
          <button
            type="button"
            className="text-muted-foreground hover:text-foreground text-xs"
            onClick={() => void command("/new")}
          >
            + {t("wb.new")}
          </button>
        </div>
        <ul className="flex-1 overflow-y-auto p-1.5">
          {sessions.map((s) => (
            <li key={s.id} className="group flex items-center">
              <button
                type="button"
                onClick={() => setCurrent(s.id)}
                className={cn(
                  "min-w-0 flex-1 truncate rounded px-2 py-1.5 text-left text-xs",
                  s.id === current ? "bg-background border" : "text-muted-foreground hover:text-foreground",
                )}
              >
                {s.title || t("wb.untitled")}
              </button>
              <button
                type="button"
                aria-label={t("wb.delete")}
                className="text-muted-foreground hover:text-destructive px-1.5 text-xs opacity-0 group-hover:opacity-100"
                onClick={() =>
                  setSessions((all) => {
                    const left = all.filter((x) => x.id !== s.id);
                    const next = left.length ? left : [blank()];
                    if (s.id === current) setCurrent(next[0].id);
                    return next;
                  })
                }
              >
                ×
              </button>
            </li>
          ))}
        </ul>
        <p className="text-muted-foreground border-t p-3 text-[11px] leading-snug">
          {t("wb.sessions_local")}
        </p>
      </aside>

      {/* Transcript and prompt */}
      <section className="flex min-h-0 flex-col">
        <div className="flex items-center gap-3 border-b px-4 py-2 font-mono text-[11px]">
          <span className="tracking-[0.12em] uppercase">{t("lab.demo.title")}</span>
          <span className="text-muted-foreground truncate">{session?.title || t("wb.untitled")}</span>
          <label className="text-muted-foreground ml-auto flex items-center gap-1.5">
            <span className="hidden sm:inline">{t("wb.agent")}</span>
            <select
              value={agentId}
              onChange={(e) => void command(`/agent ${e.target.value || "none"}`)}
              className="bg-background rounded border px-1.5 py-0.5 font-mono text-[11px]"
              aria-label={t("wb.agent")}
            >
              <option value="">{t("wb.agent.none")}</option>
              {(agents ?? []).map((a) => (
                <option key={a.id} value={a.id} disabled={!a.available}>
                  {a.name}
                </option>
              ))}
            </select>
          </label>
          <button
            type="button"
            className="text-muted-foreground hover:text-foreground hidden lg:inline"
            onClick={() => setPanel((p) => !p)}
          >
            {panel ? t("wb.hide_live") : t("wb.show_live")}
          </button>
        </div>

        <div ref={scrollRef} className="min-h-0 flex-1 overflow-y-auto px-4 py-5 sm:px-6" aria-live="polite">
          {turns.length === 0 ? (
            agent ? (
              <Persona agent={agent} onPick={(q) => setPrompt(q)} />
            ) : (
              <Intro onPick={(q) => setPrompt(q)} />
            )
          ) : null}
          <ol className="grid gap-6">
            {turns.map((turn) => (
              <li key={turn.id}>
                <TurnView
                  turn={turn}
                  now={now}
                  onRerun={(tr) =>
                    turn.kind === "chat" && !running && void ask(turn.prompt, tr, turn.tier, turn.reasoning)
                  }
                  onRun={(name, args) => void runTool(name, args)}
                  onRunCode={(language, source) => void runProgram(language, source)}
                />
              </li>
            ))}
          </ol>
        </div>

        <div className="relative border-t p-3">
          {matches.length ? (
            <ul className="bg-background absolute right-3 bottom-full left-3 mb-1 max-h-72 overflow-y-auto rounded-md border p-1 shadow-sm">
              {matches.map((c, i) => (
                <li key={c.name}>
                  <button
                    type="button"
                    onMouseDown={(e) => {
                      e.preventDefault();
                      void command(c.name);
                      setPrompt("");
                    }}
                    className={cn(
                      "flex w-full items-baseline gap-3 rounded px-2 py-1.5 text-left text-xs",
                      i === menu ? "bg-muted" : "",
                    )}
                  >
                    <span className="font-mono">{c.name}</span>
                    <span className="text-muted-foreground">{t(c.help)}</span>
                  </button>
                </li>
              ))}
            </ul>
          ) : null}
          <div className="focus-within:ring-ring flex items-end gap-2 rounded-md border px-3 py-2 focus-within:ring-1">
            <span className="text-muted-foreground pb-1 font-mono text-sm select-none">›</span>
            <textarea
              ref={inputRef}
              value={prompt}
              onChange={(e) => {
                setPrompt(e.target.value);
                setMenu(0);
              }}
              onKeyDown={onKey}
              rows={Math.min(8, Math.max(1, prompt.split("\n").length))}
              placeholder={t("wb.placeholder")}
              aria-label={t("wb.placeholder")}
              className="min-h-6 flex-1 resize-none bg-transparent text-sm outline-none"
            />
            {running ? (
              <Button size="sm" variant="outline" onClick={() => abort.current?.abort()}>
                {t("lab.demo.ask.stop")} <kbd className="text-muted-foreground ml-1 text-[10px]">esc</kbd>
              </Button>
            ) : (
              <Button size="sm" onClick={submit} disabled={!prompt.trim()}>
                {t("lab.demo.ask.button")}
              </Button>
            )}
          </div>
          <p className="text-muted-foreground mt-2 flex flex-wrap gap-x-3 gap-y-1 font-mono text-[11px] tabular-nums">
            {agent ? <span className="text-foreground">{agent.name}</span> : null}
            <span>
              {TIER_NAME[shownTier]} · {liveTier?.model ?? "—"}
            </span>
            <span>
              {t("wb.status.reason")} {agent ? t("wb.agent.its") : reasoning ? t("wb.on") : t("wb.off")}
            </span>
            <span>{transport}</span>
            <span>
              {liveTier
                ? t("wb.status.queue", { running: liveTier.in_flight, waiting: liveTier.waiting })
                : "—"}
            </span>
            <span>
              {budget
                ? budget.unlimited
                  ? t("wb.status.unlimited")
                  : t("wb.status.budget", {
                      used: budget.used.toLocaleString(),
                      limit: budget.limit.toLocaleString(),
                    })
                : "—"}
            </span>
            <span className="ml-auto hidden sm:inline">{t("wb.status.keys")}</span>
          </p>
        </div>
      </section>

      {/* Behind the scenes */}
      {panel ? (
        <aside className="bg-muted/10 min-h-0 overflow-y-auto border-t p-3 lg:border-t-0 lg:border-l">
          <p className="mb-2 font-mono text-[11px] tracking-[0.12em] uppercase">{t("wb.behind")}</p>
          <LivePanel arena={arena} />
          <p className="text-muted-foreground mt-4 text-[11px] leading-snug">{t("wb.sends")}</p>
        </aside>
      ) : null}
    </div>
  );
}

function Intro({ onPick }: { onPick: (q: string) => void }) {
  const t = useT();
  const examples = [t("wb.example.1"), t("wb.example.2"), t("wb.example.3")];
  return (
    <div className="text-muted-foreground mb-8 max-w-xl text-sm">
      <p>{t("wb.intro")}</p>
      <ul className="mt-4 grid gap-2">
        {examples.map((q) => (
          <li key={q}>
            <button
              type="button"
              className="hover:text-foreground text-left underline-offset-4 hover:underline"
              onClick={() => onPick(q)}
            >
              › {q}
            </button>
          </li>
        ))}
      </ul>
      <p className="mt-4 font-mono text-xs">{t("wb.intro.keys")}</p>
    </div>
  );
}

/** A session with an agent opens with who it is (RFC 0011), and a question to start from. */
function Persona({ agent, onPick }: { agent: Agent; onPick: (q: string) => void }) {
  const t = useT();
  const examples = [t("wb.agent.example.1"), t("wb.agent.example.2")];
  return (
    <div className="mb-8 max-w-xl rounded-md border p-4 text-sm">
      <p className="font-mono text-[11px] tracking-[0.12em] uppercase">
        {agent.name} · {TIER_NAME[agent.tier]}
      </p>
      <p className="text-muted-foreground mt-2">{agent.persona}</p>
      <ul className="text-muted-foreground mt-4 grid gap-2">
        {examples.map((q) => (
          <li key={q}>
            <button
              type="button"
              className="hover:text-foreground text-left underline-offset-4 hover:underline"
              onClick={() => onPick(q)}
            >
              › {q}
            </button>
          </li>
        ))}
      </ul>
      <p className="text-muted-foreground mt-4 text-xs">{t("wb.agent.note")}</p>
    </div>
  );
}

function Json({ value }: { value: unknown }) {
  return (
    <pre className="bg-muted/40 max-h-72 overflow-auto rounded border p-3 font-mono text-[11px] leading-relaxed">
      {JSON.stringify(value, null, 2)}
    </pre>
  );
}

function TurnView({
  turn,
  now,
  onRerun,
  onRun,
  onRunCode,
}: {
  turn: Turn;
  now: number;
  onRerun: (transport: Transport) => void;
  onRun: (name: string, args: Record<string, unknown>) => void;
  onRunCode: (language: RunLanguage, source: string) => void;
}) {
  const t = useT();
  if (turn.kind === "code") return <CodeRun turn={turn} onRunCode={onRunCode} />;
  if (turn.kind === "help") {
    return (
      <div className="rounded-md border p-4 text-xs">
        <p className="mb-2 font-mono tracking-[0.12em] uppercase">{t("wb.help")}</p>
        <ul className="grid gap-1">
          {COMMANDS.map((c) => (
            <li key={c.name} className="flex gap-3">
              <span className="w-40 shrink-0 font-mono">{c.name}</span>
              <span className="text-muted-foreground">{t(c.help)}</span>
            </li>
          ))}
        </ul>
      </div>
    );
  }
  if (turn.kind === "connect") {
    const line = `claude mcp add --transport http tbd https://api.<domain>/mcp \\\n  --header "Authorization: Bearer <token>"`;
    return (
      <div className="rounded-md border p-4 text-sm">
        <p className="mb-2 font-mono text-xs tracking-[0.12em] uppercase">{t("wb.connect.title")}</p>
        <p className="text-muted-foreground mb-3 text-xs">{t("wb.connect.lead")}</p>
        <pre className="bg-muted/40 overflow-x-auto rounded border p-3 font-mono text-[11px]">{line}</pre>
        <p className="text-muted-foreground mt-3 text-xs">{t("wb.connect.token")}</p>
      </div>
    );
  }
  if (turn.kind === "tools") {
    return (
      <div className="rounded-md border p-4">
        <p className="mb-3 font-mono text-xs tracking-[0.12em] uppercase">
          {t("wb.tools.title")} {turn.tools ? `· ${turn.tools.length}` : ""}
        </p>
        {turn.error ? <p className="text-destructive text-xs">{turn.error}</p> : null}
        {turn.tools === null ? (
          <p className="text-muted-foreground text-xs">{t("wb.tools.loading")}</p>
        ) : null}
        <ul className="grid gap-2">
          {turn.tools?.map((tool) => {
            const props = Object.keys(tool.inputSchema?.properties ?? {});
            const noArgs = props.length === 0 || tool.name.endsWith("_ping");
            return (
              <li
                key={tool.name}
                className="grid gap-1 border-t pt-2 sm:grid-cols-[12rem_minmax(0,1fr)_auto] sm:items-baseline sm:gap-3"
              >
                <span className="font-mono text-xs">{tool.name}</span>
                <span className="text-muted-foreground text-xs text-pretty">{tool.description}</span>
                {noArgs ? (
                  <Button
                    size="sm"
                    variant="outline"
                    onClick={() => onRun(tool.name, tool.name.endsWith("_ping") ? { message: "hello" } : {})}
                  >
                    {t("wb.tools.run")}
                  </Button>
                ) : (
                  <span className="text-muted-foreground font-mono text-[10px]">{props.join(", ")}</span>
                )}
              </li>
            );
          })}
        </ul>
      </div>
    );
  }
  if (turn.kind === "tool") {
    return (
      <details open className="rounded-md border">
        <summary className="flex cursor-pointer items-center gap-2 px-4 py-2 font-mono text-xs">
          <span
            className={cn(
              "size-2 rounded-full",
              turn.result === undefined
                ? "bg-muted-foreground/40"
                : turn.error
                  ? "bg-destructive"
                  : "bg-emerald-500",
            )}
          />
          {t("wb.tool.call")} {turn.name}
          <span className="text-muted-foreground ml-auto tabular-nums">
            {turn.ms !== undefined ? `${turn.ms.toFixed(0)} ms` : "…"}
          </span>
        </summary>
        <div className="grid gap-2 border-t p-3">
          <p className="text-muted-foreground font-mono text-[10px] tracking-[0.12em] uppercase">
            {t("wb.tool.args")}
          </p>
          <Json value={turn.args} />
          {turn.result !== undefined ? (
            <>
              <p className="text-muted-foreground font-mono text-[10px] tracking-[0.12em] uppercase">
                {t("wb.tool.result")}
              </p>
              <Json value={turn.result} />
            </>
          ) : null}
        </div>
      </details>
    );
  }

  return <ChatView turn={turn} now={now} onRerun={onRerun} onRunCode={onRunCode} />;
}

/** The page around the workbench: the heading, and a link back to the lab. */
export function WorkbenchPage() {
  const t = useT();
  return (
    <div className="mx-auto w-full max-w-[96rem] px-4 pt-8 pb-10 sm:px-6">
      <div className="mb-4 flex flex-wrap items-baseline gap-x-4 gap-y-1">
        <Link
          href="/lab/llm/"
          className="text-muted-foreground hover:text-foreground font-mono text-[11px] tracking-[0.14em] uppercase"
        >
          ← {t("wb.back")}
        </Link>
        <h1 className="text-xl font-semibold tracking-tight">{t("lab.demo.title")}</h1>
        <p className="text-muted-foreground text-sm">{t("lab.demo.lead")}</p>
      </div>
      <Workbench />
    </div>
  );
}
