"use client";

import { useMemo } from "react";

import { fig } from "@/lib/arena";
import { useT } from "@/lib/i18n";
import { generate, type Message, type Request, type Transport, TRANSPORTS, type Usage } from "@/lib/llm";
import { renderMarkdown } from "@/lib/markdown";
import type { RunLanguage, RunResult } from "@/lib/runner";
import { cn } from "@/lib/utils";

/**
 * What a conversation with the model service is made of, shared by the
 * workbench and the chat dock: a chat turn (the prompt, the model's reasoning,
 * its answer and the figures under it) and a sandbox run. Both surfaces render
 * a turn with the same components, so the technical detail a visitor sees in
 * the dock is the workbench's.
 */

export type Tier = "TIER_FAST" | "TIER_DEEP";
export const TIER_NAME: Record<Tier, string> = { TIER_FAST: "fast", TIER_DEEP: "deep" };

export type ChatTurn = {
  kind: "chat";
  id: string;
  prompt: string;
  tier: Tier;
  reasoning: boolean;
  transport: Transport;
  /** The agent it spoke to (RFC 0011); absent for the bare model. */
  agent?: string;
  /** The page the agent was told the visitor is reading. */
  page?: string;
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
};

export type CodeTurn = {
  kind: "code";
  id: string;
  language: RunLanguage;
  source: string;
  status: "running" | "done" | "error";
  result?: RunResult;
  error?: string;
  code?: string;
};

export const newId = () => `${Date.now().toString(36)}${Math.random().toString(36).slice(2, 7)}`;

/** Earlier turns sent with a question, so it is a conversation; the oldest drop. */
export const CONTEXT_TURNS = 8;

/** The finished chat turns as messages, newest last, at most `CONTEXT_TURNS` of them. */
export function contextOf(turns: { kind: string }[]): Message[] {
  const done = turns.filter(
    (x): x is ChatTurn => x.kind === "chat" && (x as ChatTurn).status === "done" && !!(x as ChatTurn).text,
  );
  return done.slice(-CONTEXT_TURNS).flatMap((x) => [
    { role: "user" as const, content: x.prompt },
    { role: "assistant" as const, content: x.text },
  ]);
}

/** A chat turn as it starts. */
export function startTurn(
  prompt: string,
  over: Transport,
  tier: Tier,
  reasoning: boolean,
  agent?: string,
  page?: string,
): ChatTurn {
  return {
    kind: "chat",
    id: newId(),
    prompt,
    tier,
    reasoning,
    transport: over,
    agent: agent || undefined,
    page: page || undefined,
    status: "running",
    thinking: "",
    text: "",
    chunks: 0,
    startedAt: Date.now(),
    firstMs: null,
    totalMs: null,
    usage: null,
  };
}

/**
 * Run one turn and fold every event into it through `patch`: reasoning and
 * answer text as they arrive, the first answer token's time, the engine's
 * counts at the end, and a refusal as `busy` (the tier is full) or `error`.
 */
export async function streamTurn(
  turn: ChatTurn,
  req: Request,
  signal: AbortSignal,
  patch: (f: (x: ChatTurn) => ChatTurn) => void,
) {
  const { startedAt } = turn;
  try {
    for await (const e of generate(turn.transport, req, signal)) {
      const at = Date.now() - startedAt;
      if (e.kind === "chunk") {
        patch((x) => ({
          ...x,
          thinking: e.reasoning ? x.thinking + e.text : x.thinking,
          text: e.reasoning ? x.text : x.text + e.text,
          chunks: e.reasoning ? x.chunks : x.chunks + 1,
          firstMs: !e.reasoning && x.firstMs === null ? at : x.firstMs,
          engine: e.engine ?? x.engine,
          model: e.model ?? x.model,
          stub: e.stub ?? x.stub,
        }));
      } else if (e.kind === "done") {
        patch((x) => ({
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
        }));
      } else {
        const busy = e.code === "rate_limited" || e.error.startsWith("busy:");
        patch((x) => ({ ...x, status: busy ? "busy" : "error", error: e.error, totalMs: at }));
      }
    }
    patch((x) => (x.status === "running" ? { ...x, status: "stopped" } : x));
  } catch (err) {
    const stopped = signal.aborted;
    patch((x) => ({ ...x, status: stopped ? "stopped" : "error", error: stopped ? undefined : String(err) }));
  }
}

/** Tokens a second, from the engine's count when it has one, else a live estimate from chunks. */
export function rate(turn: ChatTurn, now: number): number | null {
  if (turn.firstMs === null) return null;
  const end = turn.totalMs ?? now - turn.startedAt;
  const seconds = (end - turn.firstMs) / 1000;
  if (seconds <= 0.05) return null;
  const tokens = turn.usage?.completion_tokens ?? turn.chunks;
  return tokens / seconds;
}

/**
 * The model's answer, rendered from markdown by `renderMarkdown`, which shows
 * raw HTML as text, never loads an image, and keeps only http(s) links. Its
 * code blocks carry a copy button, and Go and Rust a run button, wired here
 * by delegation.
 */
export function Answer({
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

export const GO_START = `package main

import "fmt"

func main() {
\tfmt.Println("hello from the sandbox")
}`;

export const RUST_START = `fn main() {
    println!("hello from the sandbox");
}`;

/** What a sandbox run printed: its streams, its exit, why it was stopped, and the times. */
export function CodeRun({
  turn,
  onRunCode,
}: {
  turn: CodeTurn;
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

/**
 * A chat turn: the prompt, the model's reasoning (open while it thinks,
 * folded once it answers), the answer, and the figures under it: the agent,
 * tier, model, transport, time to the first token, tokens a second, tokens in
 * and out, the whole time. `onRerun` offers the same prompt over each
 * transport; without it the footer has no rerun.
 */
export function ChatView({
  turn,
  now,
  onRerun,
  onRunCode,
}: {
  turn: ChatTurn;
  now: number;
  onRerun?: (transport: Transport) => void;
  onRunCode: (language: RunLanguage, source: string) => void;
}) {
  const t = useT();
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
        {turn.agent ? <span className="text-foreground">{turn.agent}</span> : null}
        {turn.page ? <span title={turn.page}>@ {turn.page}</span> : null}
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
        {onRerun && turn.status !== "running" ? (
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

/** The site's own links in an answer (a path) stay in the tab; `renderMarkdown` keeps them. */
export const isChat = (x: { kind: string }): x is ChatTurn => x.kind === "chat";
