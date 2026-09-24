"use client";

import Link from "next/link";
import { useCallback, useEffect, useRef, useState } from "react";

import { Frame, SectionHead, Tag } from "@/components/kit";
import { Button } from "@/components/ui/button";
import { useT } from "@/lib/i18n";
import { useMe } from "@/lib/me";

/** `GET /v1/llm/models`, as the protocol renders `tbd.llm.v1`. */
type ModelInfo = { tier: string; engine: string; model: string; up: boolean; stub: boolean };
type Models = { models: ModelInfo[]; default_tier: string };
/** `GET /v1/llm/budget`. Counts are 64-bit on the wire, so strings. */
type Budget = {
  tokens_per_day: string;
  used_today: string;
  remaining: string;
  unlimited: boolean;
  recorded: boolean;
  day: string;
};
/** One chunk of `POST /v1/llm/generate/events`. */
type Chunk = {
  text: string;
  index: number;
  done: boolean;
  usage?: { prompt_tokens: number; completion_tokens: number } | null;
  engine: string;
  model: string;
  tier: string;
  stub: boolean;
  /** The chunk is the model's reasoning, not its answer. */
  reasoning?: boolean;
};

type Run = {
  text: string;
  /** What the model thought before it answered, when it was allowed to. */
  thinking: string;
  chunks: number;
  firstMs: number | null;
  totalMs: number | null;
  usage: Chunk["usage"];
  engine?: string;
  model?: string;
  stub?: boolean;
  error?: string;
  running: boolean;
};

const TIER_NAME: Record<string, string> = { TIER_FAST: "fast", TIER_DEEP: "deep" };

async function getJson<T>(path: string): Promise<T | null> {
  try {
    const res = await fetch(path, { credentials: "include", headers: { accept: "application/json" } });
    if (!res.ok) return null;
    return (await res.json()) as T;
  } catch {
    return null;
  }
}

/**
 * The lab's demo: the model service of this platform, called from this page on
 * this origin (`/v1/llm/*`, gated like the lab). The two tiers and whether each
 * engine is up, the caller's daily budget, and an ask box that streams the
 * answer as the service sends it, with the time to the first chunk and the
 * tokens per second computed here from what the engine reported. What the
 * page sends is said below, before it sends anything.
 * `app/lab/llm/workbench/page.tsx` carries the metadata.
 */
export function LabDemoContent() {
  const t = useT();
  const me = useMe();
  const [models, setModels] = useState<Models | null>(null);
  const [budget, setBudget] = useState<Budget | null>(null);
  const [tier, setTier] = useState("TIER_FAST");
  // Off by default: reasoning spends the token budget before any answer shows,
  // and a short budget can end with nothing to show (study 0001). On, the
  // thinking streams into its own fold above the answer.
  const [reasoning, setReasoning] = useState(false);
  const [prompt, setPrompt] = useState("");
  const [run, setRun] = useState<Run | null>(null);
  const abort = useRef<AbortController | null>(null);

  const refresh = useCallback(async () => {
    const [m, b] = await Promise.all([getJson<Models>("/v1/llm/models"), getJson<Budget>("/v1/llm/budget")]);
    setModels(m);
    setBudget(b);
  }, []);

  useEffect(() => {
    if (me) void refresh();
  }, [me, refresh]);

  const ask = async () => {
    const question = prompt.trim();
    if (!question || run?.running) return;
    abort.current?.abort();
    const controller = new AbortController();
    abort.current = controller;
    const started = performance.now();
    const state: Run = {
      text: "",
      thinking: "",
      chunks: 0,
      firstMs: null,
      totalMs: null,
      usage: null,
      running: true,
    };
    setRun({ ...state });
    try {
      const res = await fetch("/v1/llm/generate/events", {
        method: "POST",
        credentials: "include",
        signal: controller.signal,
        headers: { "content-type": "application/json", accept: "text/event-stream" },
        body: JSON.stringify({
          messages: [{ role: "user", content: question }],
          tier,
          max_tokens: 512,
          reasoning,
        }),
      });
      if (!res.ok || !res.body) {
        const body = await res.text().catch(() => "");
        throw new Error(`${res.status}: ${body.slice(0, 300)}`);
      }
      const reader = res.body.getReader();
      const decoder = new TextDecoder();
      let buffer = "";
      for (;;) {
        const { value, done } = await reader.read();
        if (done) break;
        buffer += decoder.decode(value, { stream: true });
        let cut = buffer.indexOf("\n\n");
        while (cut >= 0) {
          const frame = buffer.slice(0, cut);
          buffer = buffer.slice(cut + 2);
          cut = buffer.indexOf("\n\n");
          let event = "message";
          let data = "";
          for (const line of frame.split("\n")) {
            if (line.startsWith("event:")) event = line.slice(6).trim();
            else if (line.startsWith("data:")) data += line.slice(5).trim();
          }
          if (!data) continue;
          if (event === "error") {
            const problem = JSON.parse(data) as { error?: string; code?: string };
            throw new Error(problem.error ?? problem.code ?? data);
          }
          const chunk = JSON.parse(data) as Chunk;
          state.chunks += 1;
          if (chunk.reasoning) state.thinking += chunk.text;
          else state.text += chunk.text;
          state.engine = chunk.engine;
          state.model = chunk.model;
          state.stub = chunk.stub;
          if (chunk.text && state.firstMs === null) state.firstMs = performance.now() - started;
          if (chunk.done) {
            state.usage = chunk.usage ?? null;
            state.totalMs = performance.now() - started;
            state.running = false;
          }
          setRun({ ...state });
        }
      }
      state.running = false;
      state.totalMs ??= performance.now() - started;
      setRun({ ...state });
    } catch (e) {
      if ((e as Error).name === "AbortError") return;
      state.running = false;
      state.error = (e as Error).message;
      state.totalMs = performance.now() - started;
      setRun({ ...state });
    } finally {
      void refresh();
    }
  };

  const perSecond =
    run?.usage && run.totalMs && run.firstMs !== null && run.usage.completion_tokens > 0
      ? (run.usage.completion_tokens / Math.max(1, run.totalMs - run.firstMs)) * 1000
      : null;

  return (
    <>
      <Frame className="pt-20 pb-10 sm:pt-28">
        <p className="mb-6">
          <Link
            href="/lab/llm/"
            className="text-muted-foreground hover:text-foreground font-mono text-[11px] tracking-[0.14em] uppercase"
          >
            ← {t("lab.back")}
          </Link>
        </p>
        <p className="text-muted-foreground font-mono text-[11px] tracking-[0.18em] uppercase">
          {t("lab.demo.eyebrow")}
        </p>
        <h1 className="mt-4 max-w-3xl text-4xl font-semibold tracking-[-0.035em] text-balance sm:text-6xl">
          {t("lab.demo.title")}
        </h1>
        <p className="text-muted-foreground mt-6 max-w-xl text-lg text-pretty">{t("lab.demo.lead")}</p>
      </Frame>

      {!me ? (
        <Frame className="pb-16">
          <SectionHead n="01" label={t("lab.demo.gated.label")} />
          <p className="text-muted-foreground mt-6 max-w-xl text-pretty">{t("lab.demo.gated")}</p>
          <p className="mt-4 max-w-xl text-sm">
            <a href="/account/" className="text-foreground underline underline-offset-4">
              {t("lab.demo.signin")}
            </a>
          </p>
        </Frame>
      ) : (
        <>
          <Frame className="pb-12">
            <SectionHead n="01" label={t("lab.demo.tiers.label")} lead={t("lab.demo.tiers.lead")} />
            <div className="mt-8 grid gap-4 sm:grid-cols-2">
              {(models?.models ?? []).map((m) => (
                <div key={m.tier} className="rounded-lg border p-5">
                  <div className="flex items-center justify-between">
                    <span className="font-mono text-sm font-medium">{TIER_NAME[m.tier] ?? m.tier}</span>
                    <Tag>{m.up ? t("lab.demo.up") : t("lab.demo.down")}</Tag>
                  </div>
                  <p className="text-muted-foreground mt-3 font-mono text-xs">
                    {m.engine} · {m.model}
                    {m.stub ? ` · ${t("lab.demo.stub")}` : ""}
                  </p>
                </div>
              ))}
              {models === null ? (
                <p className="text-muted-foreground text-sm">{t("lab.demo.no_models")}</p>
              ) : null}
            </div>
            {budget ? (
              <p className="text-muted-foreground mt-6 font-mono text-xs">
                {budget.recorded
                  ? t("lab.demo.budget", {
                      used: budget.used_today,
                      limit: budget.unlimited ? "∞" : budget.tokens_per_day,
                      day: budget.day,
                    })
                  : t("lab.demo.budget_unrecorded")}
              </p>
            ) : null}
          </Frame>

          <Frame className="pb-12">
            <SectionHead n="02" label={t("lab.demo.ask.label")} lead={t("lab.demo.ask.lead")} />
            <div className="mt-8 grid max-w-2xl gap-3">
              <textarea
                value={prompt}
                onChange={(e) => setPrompt(e.target.value)}
                rows={3}
                placeholder={t("lab.demo.ask.placeholder")}
                className="bg-background ring-border w-full rounded-md px-3 py-2 text-sm ring-1 ring-inset focus:outline-none"
              />
              <div className="flex flex-wrap items-center gap-3">
                <label className="text-muted-foreground flex items-center gap-2 font-mono text-[11px] tracking-[0.14em] uppercase">
                  {t("lab.demo.ask.tier")}
                  <select
                    value={tier}
                    onChange={(e) => setTier(e.target.value)}
                    className="bg-background ring-border rounded-md px-2 py-1 text-xs ring-1 ring-inset"
                  >
                    <option value="TIER_FAST">fast</option>
                    <option value="TIER_DEEP">deep</option>
                  </select>
                </label>
                <label
                  className="text-muted-foreground flex items-center gap-2 font-mono text-[11px] tracking-[0.14em] uppercase"
                  title={t("lab.demo.ask.reasoning_help")}
                >
                  <input
                    type="checkbox"
                    checked={reasoning}
                    onChange={(e) => setReasoning(e.target.checked)}
                    className="accent-foreground"
                  />
                  {t("lab.demo.ask.reasoning")}
                </label>
                <Button
                  size="sm"
                  onClick={() => void ask()}
                  disabled={!prompt.trim() || run?.running === true}
                >
                  {run?.running ? t("lab.demo.ask.running") : t("lab.demo.ask.button")}
                </Button>
                {run?.running ? (
                  <Button size="sm" variant="ghost" onClick={() => abort.current?.abort()}>
                    {t("lab.demo.ask.stop")}
                  </Button>
                ) : null}
              </div>
              {run ? (
                <div className="mt-2 rounded-lg border p-5">
                  <p className="text-muted-foreground flex flex-wrap gap-x-5 gap-y-1 font-mono text-[11px] tabular-nums">
                    {run.engine ? (
                      <span>
                        {run.engine} · {run.model}
                        {run.stub ? ` · ${t("lab.demo.stub")}` : ""}
                      </span>
                    ) : null}
                    <span>
                      {t("lab.demo.first")} {run.firstMs === null ? "—" : `${Math.round(run.firstMs)} ms`}
                    </span>
                    <span>
                      {t("lab.demo.total")} {run.totalMs === null ? "…" : `${Math.round(run.totalMs)} ms`}
                    </span>
                    <span>
                      {t("lab.demo.tokens")}{" "}
                      {run.usage ? `${run.usage.prompt_tokens} + ${run.usage.completion_tokens}` : "—"}
                    </span>
                    <span>
                      {t("lab.demo.rate")} {perSecond === null ? "—" : `${perSecond.toFixed(1)} tok/s`}
                    </span>
                  </p>
                  {run.error ? (
                    <p className="mt-4 font-mono text-sm text-red-600 dark:text-red-400">{run.error}</p>
                  ) : (
                    <>
                      {run.thinking ? (
                        <details className="mt-4 text-sm">
                          <summary className="text-muted-foreground cursor-pointer font-mono text-[11px] tracking-[0.14em] uppercase">
                            {t("lab.demo.reasoning.fold")}
                          </summary>
                          <p className="text-muted-foreground mt-2 whitespace-pre-wrap">{run.thinking}</p>
                        </details>
                      ) : null}
                      <p className="mt-4 text-sm whitespace-pre-wrap">
                        {run.text ||
                          (run.running ? "…" : "") ||
                          (run.usage ? (
                            <span className="text-muted-foreground">{t("lab.demo.no_answer")}</span>
                          ) : (
                            ""
                          ))}
                      </p>
                    </>
                  )}
                </div>
              ) : null}
            </div>
          </Frame>
        </>
      )}

      <Frame className="pb-24 sm:pb-32">
        <SectionHead n={me ? "03" : "02"} label={t("lab.demo.sends.label")} />
        <p className="text-muted-foreground mt-6 max-w-xl text-pretty">{t("lab.demo.sends.text")}</p>
        <p className="mt-10">
          <Link
            href="/lab/llm/"
            className="text-muted-foreground hover:text-foreground font-mono text-[11px] tracking-[0.14em] uppercase"
          >
            ← {t("lab.back")}
          </Link>
        </p>
      </Frame>
    </>
  );
}
