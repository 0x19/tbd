"use client";

import Link from "next/link";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { Button } from "@/components/ui/button";
import { LivePanel } from "@/components/workbench/live-panel";
import { fig, useArena } from "@/lib/arena";
import { useT } from "@/lib/i18n";
import {
  generate,
  listTools,
  mcp,
  type Message,
  type Tool,
  toolAnswer,
  type Transport,
  TRANSPORTS,
  type Usage,
} from "@/lib/llm";
import { renderMarkdown } from "@/lib/markdown";
import { useMeState } from "@/lib/me";
import { runCode, type RunLanguage, type RunResult } from "@/lib/runner";
import { cn } from "@/lib/utils";

type Tier = "TIER_FAST" | "TIER_DEEP";

/** One entry of the transcript. */
type Turn =
  | {
      kind: "chat";
      id: string;
      prompt: string;
      tier: Tier;
      reasoning: boolean;
      transport: Transport;
      status: "running" | "done" | "busy" | "error" | "stopped";
      thinking: string;
      text: string;
      /** Chunks seen, for the live rate. */
      chunks: number;
      startedAt: number;
      firstMs: number | null;
      totalMs: number | null;
      usage: Usage | null;
      engine?: string;
      model?: string;
      stub?: boolean;
      error?: string;
      cut?: string;
    }
  | { kind: "tools"; id: string; tools: Tool[] | null; error?: string }
  | { kind: "tool"; id: string; name: string; args: unknown; result?: unknown; error?: boolean; ms?: number }
  | { kind: "connect"; id: string }
  | { kind: "help"; id: string }
  | {
      kind: "code";
      id: string;
      language: RunLanguage;
      source: string;
      status: "running" | "done" | "error";
      result?: RunResult;
      error?: string;
      code?: string;
    };

type Session = { id: string; title: string; createdAt: number; turns: Turn[] };

const STORE = "inorbit.workbench.sessions";
const MAX_TOKENS = 1024;
/** Earlier turns sent with a question, so it is a conversation; the oldest drop. */
const CONTEXT_TURNS = 8;

const newId = () => `${Date.now().toString(36)}${Math.random().toString(36).slice(2, 7)}`;
const blank = (): Session => ({ id: newId(), title: "", createdAt: Date.now(), turns: [] });

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
    window.localStorage.setItem(STORE, JSON.stringify(sessions.slice(0, 30)));
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
  { name: "/tools", help: "wb.cmd.tools" },
  { name: "/run go", help: "wb.cmd.run_go" },
  { name: "/run rust", help: "wb.cmd.run_rust" },
  { name: "/connect", help: "wb.cmd.connect" },
  { name: "/clear", help: "wb.cmd.clear" },
  { name: "/new", help: "wb.cmd.new" },
  { name: "/help", help: "wb.cmd.help" },
] as const;

const TIER_NAME: Record<Tier, string> = { TIER_FAST: "fast", TIER_DEEP: "deep" };

/** Tokens a second, from the engine's count when it has one, else a live estimate from chunks. */
function rate(turn: Extract<Turn, { kind: "chat" }>, now: number): number | null {
  if (turn.firstMs === null) return null;
  const end = turn.totalMs ?? now - turn.startedAt;
  const seconds = (end - turn.firstMs) / 1000;
  if (seconds <= 0.05) return null;
  const tokens = turn.usage?.completion_tokens ?? turn.chunks;
  return tokens / seconds;
}

/**
 * The model lab's workbench: a keyboard-first page in the manner of a
 * terminal agent. Sessions kept in this browser, a transcript that streams
 * (the model's reasoning shown while it thinks and folded once it answers),
 * a prompt with a slash menu and a status line, three transports for the same
 * turn, the platform's MCP tools callable as cards, and the arena's live view
 * beside it all. `app/lab/llm/workbench/page.tsx` carries the metadata.
 */
export function Workbench() {
  const t = useT();
  const { me, known } = useMeState();
  const admin = me?.role === "admin";
  const arena = useArena(admin);

  const [sessions, setSessions] = useState<Session[]>([]);
  const [current, setCurrent] = useState<string>("");
  const [tier, setTier] = useState<Tier>("TIER_FAST");
  const [reasoning, setReasoning] = useState(false);
  const [transport, setTransport] = useState<Transport>("sse");
  const [prompt, setPrompt] = useState("");
  const [menu, setMenu] = useState(0);
  const [panel, setPanel] = useState(true);
  const [budget, setBudget] = useState<{ used: number; limit: number; unlimited: boolean } | null>(null);
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

  const refreshBudget = useCallback(async () => {
    try {
      const res = await fetch("/v1/llm/budget", { credentials: "include" });
      if (!res.ok) return;
      const b = (await res.json()) as { used_today: string; tokens_per_day: string; unlimited: boolean };
      setBudget({ used: Number(b.used_today), limit: Number(b.tokens_per_day), unlimited: b.unlimited });
    } catch {
      /* the status line shows a dash */
    }
  }, []);
  useEffect(() => {
    if (admin) void refreshBudget();
  }, [admin, refreshBudget]);

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

  /** The conversation so far, as messages, newest last. */
  const context = useCallback((): Message[] => {
    const done = turns.filter(
      (x): x is Extract<Turn, { kind: "chat" }> => x.kind === "chat" && x.status === "done" && !!x.text,
    );
    return done.slice(-CONTEXT_TURNS).flatMap((x) => [
      { role: "user" as const, content: x.prompt },
      { role: "assistant" as const, content: x.text },
    ]);
  }, [turns]);

  const ask = useCallback(
    async (
      question: string,
      over: Transport = transport,
      withTier: Tier = tier,
      withReasoning = reasoning,
    ) => {
      const sid = current;
      const id = newId();
      const startedAt = Date.now();
      append(
        {
          kind: "chat",
          id,
          prompt: question,
          tier: withTier,
          reasoning: withReasoning,
          transport: over,
          status: "running",
          thinking: "",
          text: "",
          chunks: 0,
          startedAt,
          firstMs: null,
          totalMs: null,
          usage: null,
        },
        question.slice(0, 60),
      );
      abort.current?.abort();
      const controller = new AbortController();
      abort.current = controller;
      const req = {
        messages: [...context(), { role: "user" as const, content: question }],
        tier: withTier,
        max_tokens: MAX_TOKENS,
        reasoning: withReasoning,
      };
      try {
        for await (const e of generate(over, req, controller.signal)) {
          const at = Date.now() - startedAt;
          if (e.kind === "chunk") {
            edit(sid, id, (x) =>
              x.kind !== "chat"
                ? x
                : {
                    ...x,
                    thinking: e.reasoning ? x.thinking + e.text : x.thinking,
                    text: e.reasoning ? x.text : x.text + e.text,
                    chunks: e.reasoning ? x.chunks : x.chunks + 1,
                    firstMs: !e.reasoning && x.firstMs === null ? at : x.firstMs,
                    engine: e.engine ?? x.engine,
                    model: e.model ?? x.model,
                    stub: e.stub ?? x.stub,
                  },
            );
          } else if (e.kind === "done") {
            edit(sid, id, (x) =>
              x.kind !== "chat"
                ? x
                : {
                    ...x,
                    status: "done",
                    totalMs: at,
                    usage: e.usage,
                    engine: e.engine ?? x.engine,
                    model: e.model ?? x.model,
                    stub: e.stub ?? x.stub,
                    cut: e.cut,
                    // MCP answers whole: the first token is the answer's arrival.
                    firstMs: x.firstMs ?? at,
                  },
            );
          } else {
            const busy = e.code === "rate_limited" || e.error.startsWith("busy:");
            edit(sid, id, (x) =>
              x.kind !== "chat" ? x : { ...x, status: busy ? "busy" : "error", error: e.error, totalMs: at },
            );
          }
        }
        edit(sid, id, (x) => (x.kind === "chat" && x.status === "running" ? { ...x, status: "stopped" } : x));
      } catch (err) {
        const stopped = controller.signal.aborted;
        edit(sid, id, (x) =>
          x.kind !== "chat"
            ? x
            : { ...x, status: stopped ? "stopped" : "error", error: stopped ? undefined : String(err) },
        );
      } finally {
        void refreshBudget();
      }
    },
    [append, context, current, edit, reasoning, refreshBudget, tier, transport],
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
        case "/clear":
          setSessions((all) => all.map((s) => (s.id === current ? { ...s, turns: [] } : s)));
          return;
        case "/new": {
          const s = blank();
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
    [append, current, edit],
  );

  const matches = prompt.startsWith("/") ? COMMANDS.filter((c) => c.name.startsWith(prompt.trim())) : [];

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

  const liveTier = arena.snapshot?.tiers.find((x) => x.tier === TIER_NAME[tier]);

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
          <button
            type="button"
            className="text-muted-foreground hover:text-foreground ml-auto hidden lg:inline"
            onClick={() => setPanel((p) => !p)}
          >
            {panel ? t("wb.hide_live") : t("wb.show_live")}
          </button>
        </div>

        <div ref={scrollRef} className="min-h-0 flex-1 overflow-y-auto px-4 py-5 sm:px-6" aria-live="polite">
          {turns.length === 0 ? <Intro onPick={(q) => setPrompt(q)} /> : null}
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
            <span>
              {TIER_NAME[tier]} · {liveTier?.model ?? "—"}
            </span>
            <span>
              {t("wb.status.reason")} {reasoning ? t("wb.on") : t("wb.off")}
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

/**
 * The model's answer, rendered from markdown by `renderMarkdown`, which shows
 * raw HTML as text, never loads an image, and keeps only http(s) links. Its
 * code blocks carry a copy button, wired here by delegation.
 */
function Answer({
  markdown,
  onRunCode,
}: {
  markdown: string;
  onRunCode: (language: RunLanguage, source: string) => void;
}) {
  const html = useMemo(() => renderMarkdown(markdown), [markdown]);
  const onClick = (e: React.MouseEvent<HTMLDivElement>) => {
    const run = (e.target as HTMLElement).closest("button[data-run]");
    if (run) {
      const source = run.closest(".md-code")?.querySelector("code")?.textContent;
      const language = run.getAttribute("data-run");
      if (source && (language === "go" || language === "rust")) onRunCode(language, source);
      return;
    }
    const button = (e.target as HTMLElement).closest("button[data-copy]");
    const code = button?.closest(".md-code")?.querySelector("code")?.textContent;
    if (!button || code === undefined || code === null) return;
    void navigator.clipboard?.writeText(code).then(() => {
      button.textContent = "copied";
      window.setTimeout(() => (button.textContent = "copy"), 1200);
    });
  };
  return <div className="md" onClick={onClick} dangerouslySetInnerHTML={{ __html: html }} />;
}

const GO_START = `package main

import "fmt"

func main() {
\tfmt.Println("hello from the sandbox")
}`;

const RUST_START = `fn main() {
    println!("hello from the sandbox");
}`;

/** What a sandbox run printed: its streams, its exit, why it was stopped, and the times. */
function CodeRun({
  turn,
  onRunCode,
}: {
  turn: Extract<Turn, { kind: "code" }>;
  onRunCode: (language: RunLanguage, source: string) => void;
}) {
  const t = useT();
  const r = turn.result;
  const step = r?.run ?? r?.compile ?? null;
  const why = step && step.killed !== "none" ? t(`wb.run.killed.${step.killed}`) : null;
  const dot =
    turn.status === "running"
      ? "bg-muted-foreground/40 animate-pulse"
      : turn.status === "error"
        ? "bg-destructive"
        : r?.outcome === "ok"
          ? "bg-emerald-500"
          : "bg-amber-500";
  return (
    <article className="rounded-md border">
      <header className="flex flex-wrap items-center gap-2 border-b px-4 py-2 font-mono text-xs">
        <span className={cn("size-2 rounded-full", dot)} />
        <span>
          {t("wb.run.title")} · {turn.language}
        </span>
        <span className="text-muted-foreground">
          {turn.status === "running"
            ? t("wb.run.running")
            : turn.status === "error"
              ? turn.code === "rate_limited"
                ? t("wb.busy")
                : t("wb.run.failed")
              : t(`wb.run.outcome.${r?.outcome ?? "ok"}`)}
        </span>
        {r ? (
          <span className="text-muted-foreground ml-auto tabular-nums">
            {t("wb.run.times", {
              compile: r.compile?.wall_ms ?? "—",
              run: r.run?.wall_ms ?? "—",
              total: r.total_ms,
            })}
            {r.stub ? " · stub" : ""}
          </span>
        ) : null}
      </header>
      <details className="border-b px-4 py-2">
        <summary className="text-muted-foreground cursor-pointer font-mono text-[11px] tracking-[0.12em] uppercase">
          {t("wb.run.source")} · {turn.source.length.toLocaleString()} {t("wb.chars")}
        </summary>
        <pre className="bg-muted/40 mt-2 max-h-72 overflow-auto rounded border p-3 font-mono text-[11px]">
          {turn.source}
        </pre>
      </details>
      <div className="grid gap-2 p-4">
        {turn.status === "error" ? <p className="text-destructive text-sm">{turn.error}</p> : null}
        {r?.outcome === "compile_error" ? (
          <pre className="bg-muted/40 max-h-72 overflow-auto rounded border p-3 font-mono text-[11px] whitespace-pre-wrap text-amber-700 dark:text-amber-400">
            {r.compile?.stderr}
          </pre>
        ) : null}
        {r?.run ? (
          <>
            {r.run.stdout ? (
              <pre className="bg-muted/40 max-h-96 overflow-auto rounded border p-3 font-mono text-[12px] whitespace-pre-wrap">
                {r.run.stdout}
              </pre>
            ) : null}
            {r.run.stderr ? (
              <pre className="bg-muted/40 text-destructive max-h-60 overflow-auto rounded border p-3 font-mono text-[11px] whitespace-pre-wrap">
                {r.run.stderr}
              </pre>
            ) : null}
            {!r.run.stdout && !r.run.stderr ? (
              <p className="text-muted-foreground text-xs">{t("wb.run.silent")}</p>
            ) : null}
          </>
        ) : null}
        {why || step?.truncated ? (
          <p className="text-xs text-amber-700 dark:text-amber-400">
            {why}
            {step?.truncated ? ` ${t("wb.run.truncated")}` : ""}
          </p>
        ) : null}
        <footer className="text-muted-foreground flex flex-wrap items-center gap-x-3 gap-y-1 border-t pt-2 font-mono text-[11px] tabular-nums">
          {r?.run && r.run.exit_code !== undefined && r.run.exit_code !== null ? (
            <span>
              {t("wb.run.exit")} {r.run.exit_code}
            </span>
          ) : null}
          {r ? <span>{t("wb.run.left", { n: r.runs_left_today })}</span> : null}
          {turn.status !== "running" ? (
            <button
              type="button"
              className="hover:text-foreground ml-auto underline-offset-2 hover:underline"
              onClick={() => onRunCode(turn.language, turn.source)}
            >
              {t("wb.run.again")}
            </button>
          ) : null}
        </footer>
      </div>
    </article>
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

  const live = rate(turn, now);
  const thinkingOpen = turn.status === "running" && !turn.text;
  return (
    <article className="grid gap-3">
      <p className="font-mono text-sm">
        <span className="text-muted-foreground select-none">› </span>
        <span className="whitespace-pre-wrap">{turn.prompt}</span>
      </p>
      {turn.thinking ? (
        <details open={thinkingOpen} className="text-muted-foreground border-l-2 pl-3 text-xs">
          <summary className="cursor-pointer font-mono text-[11px] tracking-[0.12em] uppercase">
            {thinkingOpen ? t("wb.thinking") : t("wb.thought")} · {turn.thinking.length.toLocaleString()}{" "}
            {t("wb.chars")}
          </summary>
          <p className="mt-2 whitespace-pre-wrap">{turn.thinking}</p>
        </details>
      ) : null}
      {turn.status === "busy" ? (
        <div className="rounded-md border border-amber-500/50 bg-amber-500/5 p-3 text-sm">
          <p className="font-mono text-[11px] tracking-[0.12em] text-amber-700 uppercase dark:text-amber-400">
            {t("wb.busy")}
          </p>
          <p className="mt-1">{turn.error}</p>
        </div>
      ) : turn.status === "error" ? (
        <p className="text-destructive text-sm">{turn.error}</p>
      ) : (
        <div className="text-sm leading-relaxed">
          {turn.text ? <Answer markdown={turn.text} onRunCode={onRunCode} /> : null}
          {turn.status === "running" ? (
            <span className="bg-foreground ml-0.5 inline-block h-4 w-1.5 animate-pulse align-text-bottom" />
          ) : null}
          {turn.status === "done" && !turn.text ? (
            <span className="text-muted-foreground">{t("lab.demo.no_answer")}</span>
          ) : null}
          {turn.status === "stopped" ? (
            <span className="text-muted-foreground"> {t("wb.stopped")}</span>
          ) : null}
        </div>
      )}
      {turn.transport === "mcp" && turn.status === "running" ? (
        <p className="text-muted-foreground text-xs">{t("wb.mcp_whole")}</p>
      ) : null}
      <footer className="text-muted-foreground flex flex-wrap items-center gap-x-3 gap-y-1 border-t pt-2 font-mono text-[11px] tabular-nums">
        <span>{TIER_NAME[turn.tier]}</span>
        <span>
          {turn.model ?? "—"}
          {turn.stub ? " · stub" : ""}
        </span>
        <span>{turn.transport}</span>
        <span>
          {t("wb.ttft")} {fig(turn.firstMs, 0, " ms")}
        </span>
        <span>{fig(live, 1, " tok/s")}</span>
        <span>{turn.usage ? `${turn.usage.prompt_tokens} + ${turn.usage.completion_tokens} tok` : "—"}</span>
        <span>{fig(turn.totalMs === null ? null : turn.totalMs / 1000, 1, " s")}</span>
        {turn.cut ? <span className="text-amber-600">{turn.cut}</span> : null}
        {turn.status !== "running" ? (
          <span className="ml-auto flex gap-1">
            {t("wb.rerun")}
            {TRANSPORTS.map((tr) => (
              <button
                key={tr}
                type="button"
                className="hover:text-foreground underline-offset-2 hover:underline"
                onClick={() => onRerun(tr)}
              >
                {tr}
              </button>
            ))}
          </span>
        ) : null}
      </footer>
    </article>
  );
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
