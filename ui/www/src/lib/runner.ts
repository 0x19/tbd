"use client";

/**
 * The runner (RFC 0010) from the page: `POST /v1/runner/run` on this origin, as
 * the signed-in visitor, gated like the lab. The code goes to the platform to
 * be compiled and run once in a throwaway sandbox with no network, and nothing
 * of it is kept: the runner's record names the run, never its code.
 */

export type RunLanguage = "go" | "rust";

/** One step, as the gateway renders `tbd.runner.v1.Step`. */
export type RunStep = {
  exit_code?: number | null;
  stdout: string;
  stderr: string;
  truncated: boolean;
  /** 64-bit on the wire, so a string. */
  wall_ms: string;
  killed: string;
};

export type RunResult = {
  id: string;
  language: string;
  outcome: "ok" | "exit" | "compile_error" | "killed" | string;
  compile?: RunStep | null;
  run?: RunStep | null;
  total_ms: string;
  stub: boolean;
  runs_left_today: number;
};

const WIRE: Record<RunLanguage, string> = { go: "LANGUAGE_GO", rust: "LANGUAGE_RUST" };

/** A code fence's language as the runner names it, or nothing it can run. */
export function runnable(lang: string): RunLanguage | null {
  const l = lang.trim().toLowerCase();
  if (l === "go" || l === "golang") return "go";
  if (l === "rust" || l === "rs") return "rust";
  return null;
}

/** Compile and run once; the result, or the gateway's problem. */
export async function runCode(
  language: RunLanguage,
  source: string,
  stdin = "",
): Promise<{ ok: true; result: RunResult } | { ok: false; code: string; error: string }> {
  try {
    const res = await fetch("/v1/runner/run", {
      method: "POST",
      credentials: "include",
      headers: { "content-type": "application/json", accept: "application/json" },
      body: JSON.stringify({ language: WIRE[language], source, stdin }),
    });
    const text = await res.text();
    if (!res.ok) {
      try {
        const p = JSON.parse(text) as { code?: string; error?: string };
        return { ok: false, code: p.code ?? String(res.status), error: p.error ?? text };
      } catch {
        return { ok: false, code: String(res.status), error: text.slice(0, 300) || `HTTP ${res.status}` };
      }
    }
    return { ok: true, result: JSON.parse(text) as RunResult };
  } catch (e) {
    return { ok: false, code: "unavailable", error: String(e) };
  }
}
