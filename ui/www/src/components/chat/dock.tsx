"use client";

import { usePathname } from "next/navigation";
import { useCallback, useEffect, useRef, useState } from "react";

import { handOver, WORKBENCH_HREF } from "@/components/chat/sessions";
import {
  type ChatTurn,
  ChatView,
  CodeRun,
  type CodeTurn,
  contextOf,
  newId,
  startTurn,
  streamTurn,
  TIER_NAME,
} from "@/components/chat/turn";
import { Button } from "@/components/ui/button";
import { lab } from "@/data/site";
import { type Agent, useAgents, useBudget } from "@/lib/agents";
import { useArena } from "@/lib/arena";
import { useT } from "@/lib/i18n";
import { messages } from "@/lib/i18n/messages";
import { useMeState } from "@/lib/me";
import { runCode, type RunLanguage } from "@/lib/runner";
import { cn } from "@/lib/utils";

type Turn = ChatTurn | CodeTurn;

/** The agent the dock speaks to: the first one, the site guide (RFC 0011). */
const AGENT = "site";
const STORE = "inorbit.dock.turns";
const OPEN = "inorbit.dock.open";
const MAX_TURNS = 40;
/** A page asks the dock to open (the home page's card): `window.dispatchEvent(new Event(DOCK_OPEN))`. */
export const DOCK_OPEN = "inorbit:dock-open";

function load(): Turn[] {
  try {
    const raw = window.localStorage.getItem(STORE);
    const parsed = raw ? (JSON.parse(raw) as Turn[]) : [];
    // A turn that was running when the page closed did not finish.
    return parsed.map((t) => (t.status === "running" ? ({ ...t, status: "stopped" } as Turn) : t));
  } catch {
    return [];
  }
}

function save(turns: Turn[]) {
  try {
    window.localStorage.setItem(STORE, JSON.stringify(turns.slice(-MAX_TURNS)));
  } catch {
    /* a full or blocked store only loses the history, never the page */
  }
}

const editable = (el: EventTarget | null) =>
  el instanceof HTMLElement && (el.isContentEditable || ["INPUT", "TEXTAREA", "SELECT"].includes(el.tagName));

/**
 * The chat at the bottom of every page: the site guide (RFC 0011), with the
 * workbench's detail under every answer (tier, model, time to the first token,
 * tokens a second, tokens in and out, the model's reasoning folded above) and
 * its status line. It sends the path of the page it is on, so a question about
 * "this page" is answered from that page. Shown to whoever may use the agent,
 * which while the lab is private is an admin; never on the workbench, which is
 * the same conversation with every control. The conversation lives in this
 * browser's `localStorage`; "continue in the workbench" hands it over.
 */
export function ChatDock() {
  const pathname = usePathname() ?? "/";
  const { me, known } = useMeState();
  // Asking anyone else would only meet the gate: the lab's paths answer admins.
  const may = known && (lab.public || me?.role === "admin");
  const agents = useAgents(may);
  const agent = agents?.find((a) => a.id === AGENT && a.available);
  const onWorkbench = pathname.startsWith(WORKBENCH_HREF.slice(0, -1));
  if (!agent || onWorkbench) return null;
  return <Dock agent={agent} page={pathname} />;
}

function Dock({ agent, page }: { agent: Agent; page: string }) {
  const t = useT();
  const [open, setOpen] = useState(false);
  const [turns, setTurns] = useState<Turn[]>([]);
  const [loaded, setLoaded] = useState(false);
  const [prompt, setPrompt] = useState("");
  const [now, setNow] = useState(() => Date.now());
  const arena = useArena(open);
  const { budget, refresh } = useBudget(open);
  const abort = useRef<AbortController | null>(null);
  const scrollRef = useRef<HTMLDivElement | null>(null);
  const inputRef = useRef<HTMLTextAreaElement | null>(null);

  const name = messages.en[`dock.name.${agent.id}`] ? t(`dock.name.${agent.id}`) : agent.name;
  const running = turns.some((x) => x.kind === "chat" && x.status === "running");

  // The conversation, and whether this tab had the dock open (so following a
  // link the guide gave lands with it still open).
  useEffect(() => {
    setTurns(load());
    try {
      setOpen(window.sessionStorage.getItem(OPEN) === "1");
    } catch {
      /* closed */
    }
    setLoaded(true);
  }, []);
  useEffect(() => {
    if (loaded) save(turns);
  }, [loaded, turns]);
  useEffect(() => {
    try {
      window.sessionStorage.setItem(OPEN, open ? "1" : "0");
    } catch {
      /* the next page opens closed */
    }
    if (open) inputRef.current?.focus({ preventScroll: true });
  }, [open]);

  useEffect(() => {
    if (!running) return;
    const timer = window.setInterval(() => setNow(Date.now()), 250);
    return () => window.clearInterval(timer);
  }, [running]);
  // Follow the newest turn inside the dock's own box, never the window.
  useEffect(() => {
    const box = scrollRef.current;
    if (box) box.scrollTop = box.scrollHeight;
  }, [turns, open]);

  // `/` or ctrl+k opens it from anywhere that is not a field; esc closes it from
  // anywhere at all, wherever the focus is (an answer, a button, the fold). A
  // turn that is running keeps running; the stop button stops it.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape" && open) {
        e.preventDefault();
        setOpen(false);
        return;
      }
      if ((e.key === "k" || e.key === "K") && (e.ctrlKey || e.metaKey)) {
        e.preventDefault();
        setOpen((o) => !o);
        return;
      }
      if (e.key === "/" && !open && !editable(e.target) && !e.altKey && !e.ctrlKey && !e.metaKey) {
        e.preventDefault();
        setOpen(true);
      }
    };
    const onOpen = () => setOpen(true);
    window.addEventListener("keydown", onKey);
    window.addEventListener(DOCK_OPEN, onOpen);
    return () => {
      window.removeEventListener("keydown", onKey);
      window.removeEventListener(DOCK_OPEN, onOpen);
    };
  }, [open]);

  const patch = useCallback(
    (id: string, f: (x: Turn) => Turn) => setTurns((all) => all.map((x) => (x.id === id ? f(x) : x))),
    [],
  );

  const ask = useCallback(
    async (question: string) => {
      const turn = startTurn(question, "sse", agent.tier, false, agent.id, page);
      const messages = [...contextOf(turns), { role: "user" as const, content: question }];
      setTurns((all) => [...all, turn]);
      abort.current?.abort();
      const controller = new AbortController();
      abort.current = controller;
      // The agent's tier and bounds apply: the request names only the agent and the page.
      await streamTurn(turn, { messages, agent: agent.id, page }, controller.signal, (f) =>
        patch(turn.id, (x) => (x.kind === "chat" ? f(x) : x)),
      );
      void refresh();
    },
    [agent, page, patch, refresh, turns],
  );

  const runProgram = useCallback(
    async (language: RunLanguage, source: string) => {
      const id = newId();
      setTurns((all) => [...all, { kind: "code", id, language, source, status: "running" }]);
      const r = await runCode(language, source);
      patch(id, (x) =>
        x.kind !== "code"
          ? x
          : r.ok
            ? { ...x, status: "done", result: r.result }
            : { ...x, status: "error", error: r.error, code: r.code },
      );
    },
    [patch],
  );

  const submit = () => {
    const line = prompt.trim();
    if (!line || running) return;
    setPrompt("");
    void ask(line);
  };

  const onKey = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      submit();
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

  const toWorkbench = () => {
    const first = turns.find((x): x is ChatTurn => x.kind === "chat");
    if (handOver(first?.prompt ?? name, agent.id, turns)) window.location.assign(WORKBENCH_HREF);
  };

  const liveTier = arena.snapshot?.tiers.find((x) => x.tier === TIER_NAME[agent.tier]);

  return (
    <>
      {/* Room under the footer, so the bar never covers the page's last line. */}
      <div aria-hidden className="h-16" />
      <div className="pointer-events-none fixed inset-x-0 bottom-0 z-40 px-3 pb-3 sm:px-4">
        {open ? (
          <section
            aria-label={name}
            className="bg-background pointer-events-auto mx-auto flex max-h-[min(80dvh,48rem)] w-full max-w-3xl flex-col overflow-hidden rounded-lg border shadow-xl"
          >
            <header className="flex flex-wrap items-center gap-x-3 gap-y-1 border-b px-4 py-2">
              <span className="size-2 rounded-full bg-emerald-500" aria-hidden />
              <span className="font-mono text-[11px] tracking-[0.12em] uppercase">{name}</span>
              <span className="text-muted-foreground hidden min-w-0 flex-1 truncate text-xs sm:inline">
                {agent.persona}
              </span>
              <span className="ml-auto flex items-center gap-3 font-mono text-[11px]">
                {turns.length ? (
                  <>
                    <button
                      type="button"
                      className="text-muted-foreground hover:text-foreground"
                      onClick={toWorkbench}
                    >
                      {t("dock.continue")}
                    </button>
                    <button
                      type="button"
                      className="text-muted-foreground hover:text-foreground"
                      onClick={() => {
                        abort.current?.abort();
                        setTurns([]);
                      }}
                    >
                      {t("dock.clear")}
                    </button>
                  </>
                ) : null}
                <button
                  type="button"
                  className="text-muted-foreground hover:text-foreground"
                  onClick={() => setOpen(false)}
                  aria-label={t("dock.close")}
                >
                  {t("dock.close")} <kbd className="text-[10px]">esc</kbd>
                </button>
              </span>
            </header>

            <div ref={scrollRef} className="min-h-40 flex-1 overflow-y-auto px-4 py-4" aria-live="polite">
              {turns.length === 0 ? (
                <div className="text-muted-foreground text-sm">
                  <p className="sm:hidden">{agent.persona}</p>
                  <ul className="mt-2 grid gap-2 sm:mt-0">
                    {[t("dock.example.1"), t("dock.example.2"), t("dock.example.3")].map((q) => (
                      <li key={q}>
                        <button
                          type="button"
                          className="hover:text-foreground text-left underline-offset-4 hover:underline"
                          onClick={() => void ask(q)}
                        >
                          › {q}
                        </button>
                      </li>
                    ))}
                  </ul>
                </div>
              ) : null}
              <ol className="grid gap-6">
                {turns.map((turn) => (
                  <li key={turn.id}>
                    {turn.kind === "code" ? (
                      <CodeRun turn={turn} onRunCode={(l, s) => void runProgram(l, s)} />
                    ) : (
                      <ChatView turn={turn} now={now} onRunCode={(l, s) => void runProgram(l, s)} />
                    )}
                  </li>
                ))}
              </ol>
            </div>

            <div className="border-t p-3">
              <div className="focus-within:ring-ring flex items-end gap-2 rounded-md border px-3 py-2 focus-within:ring-1">
                <span className="text-muted-foreground pb-1 font-mono text-sm select-none">›</span>
                <textarea
                  ref={inputRef}
                  value={prompt}
                  onChange={(e) => setPrompt(e.target.value)}
                  onKeyDown={onKey}
                  rows={Math.min(6, Math.max(1, prompt.split("\n").length))}
                  placeholder={t("dock.placeholder")}
                  aria-label={t("dock.placeholder")}
                  className="min-h-6 flex-1 resize-none bg-transparent text-sm outline-none"
                />
                {running ? (
                  <Button size="sm" variant="outline" onClick={() => abort.current?.abort()}>
                    {t("lab.demo.ask.stop")}
                  </Button>
                ) : (
                  <Button size="sm" onClick={submit} disabled={!prompt.trim()}>
                    {t("lab.demo.ask.button")}
                  </Button>
                )}
              </div>
              <p className="text-muted-foreground mt-2 flex flex-wrap gap-x-3 gap-y-1 font-mono text-[11px] tabular-nums">
                <span className="text-foreground">{agent.id}</span>
                <span>
                  {TIER_NAME[agent.tier]} · {liveTier?.model ?? "—"}
                </span>
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
                <span className="truncate">{t("dock.page", { path: page })}</span>
                <span className="ml-auto hidden sm:inline">{t("dock.keys")}</span>
              </p>
              <details className="text-muted-foreground mt-2 text-[11px] leading-snug">
                <summary className="cursor-pointer font-mono tracking-[0.12em] uppercase">
                  {t("dock.sends.title")}
                  {lab.public ? "" : ` · ${t("dock.admins")}`}
                </summary>
                <p className="mt-1 max-w-prose">{t("dock.sends")}</p>
              </details>
            </div>
          </section>
        ) : (
          <button
            type="button"
            onClick={() => setOpen(true)}
            aria-label={`${name}: ${t("dock.open")}`}
            className={cn(
              "bg-background/95 pointer-events-auto mx-auto flex w-full max-w-3xl items-center gap-3 rounded-lg border px-4 py-2.5 text-left shadow-lg backdrop-blur",
              "hover:border-foreground/30 transition-colors",
            )}
          >
            <span className="size-2 shrink-0 rounded-full bg-emerald-500" aria-hidden />
            <span className="shrink-0 font-mono text-[11px] tracking-[0.12em] uppercase">{name}</span>
            <span className="text-muted-foreground min-w-0 flex-1 truncate text-sm">
              {turns.length
                ? (turns.findLast((x) => x.kind === "chat") as ChatTurn | undefined)?.prompt
                : t("dock.ask_page")}
            </span>
            <kbd className="text-muted-foreground hidden rounded border px-1.5 font-mono text-[10px] sm:inline">
              /
            </kbd>
          </button>
        )}
      </div>
    </>
  );
}
