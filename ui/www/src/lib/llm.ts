"use client";

/**
 * One turn with the model service, three ways, one shape out. Each transport
 * reaches the same RPC (`tbd.llm.v1.LlmService/Generate`) through the same
 * gateway on this origin, as the signed-in visitor:
 *
 * - `sse`: `POST /v1/llm/generate/events`, the chunks as server-sent events;
 * - `websocket`: the multiplexed socket `/v1/ws`, one `call` frame, the chunks
 *   as `data` frames;
 * - `mcp`: `POST /mcp`, the `llm_generate` tool, which collects the stream and
 *   answers once (a tool call does not stream), so its answer arrives whole.
 *
 * A refusal comes back as the gateway's problem (`code`, `error`); `busy` is
 * the model service's admission saying the tier is full.
 */

export type Transport = "sse" | "websocket" | "mcp";
export const TRANSPORTS: Transport[] = ["sse", "websocket", "mcp"];

export type Message = { role: "system" | "user" | "assistant"; content: string };
export type Usage = { prompt_tokens: number; completion_tokens: number };

export type Request = {
  messages: Message[];
  tier: "TIER_FAST" | "TIER_DEEP";
  max_tokens: number;
  reasoning: boolean;
};

/** What a turn yields, in order: chunks, then exactly one of done or problem. */
export type Event =
  | { kind: "chunk"; text: string; reasoning: boolean; engine?: string; model?: string; stub?: boolean }
  | { kind: "done"; usage: Usage | null; engine?: string; model?: string; stub?: boolean; cut?: string }
  | { kind: "problem"; code: string; error: string };

type Chunk = {
  text: string;
  done: boolean;
  reasoning?: boolean;
  usage?: Usage | null;
  engine: string;
  model: string;
  stub: boolean;
};

const fromChunk = (c: Chunk): Event[] => {
  const out: Event[] = [];
  if (c.text)
    out.push({
      kind: "chunk",
      text: c.text,
      reasoning: !!c.reasoning,
      engine: c.engine,
      model: c.model,
      stub: c.stub,
    });
  if (c.done)
    out.push({ kind: "done", usage: c.usage ?? null, engine: c.engine, model: c.model, stub: c.stub });
  return out;
};

/** A refusal as the page reads it, from whatever the gateway answered. */
function problem(status: number, body: string): Event {
  try {
    const p = JSON.parse(body) as { code?: string; error?: string };
    return { kind: "problem", code: p.code ?? String(status), error: p.error ?? body };
  } catch {
    return { kind: "problem", code: String(status), error: body.slice(0, 300) || `HTTP ${status}` };
  }
}

/** Run one turn; `signal` stops it (the socket and the fetch are closed). */
export function generate(transport: Transport, req: Request, signal: AbortSignal): AsyncGenerator<Event> {
  if (transport === "websocket") return viaSocket(req, signal);
  if (transport === "mcp") return viaMcp(req, signal);
  return viaSse(req, signal);
}

async function* viaSse(req: Request, signal: AbortSignal): AsyncGenerator<Event> {
  const res = await fetch("/v1/llm/generate/events", {
    method: "POST",
    credentials: "include",
    signal,
    headers: { "content-type": "application/json", accept: "text/event-stream" },
    body: JSON.stringify(req),
  });
  if (!res.ok || !res.body) {
    yield problem(res.status, await res.text().catch(() => ""));
    return;
  }
  const reader = res.body.getReader();
  const decoder = new TextDecoder();
  let buffer = "";
  for (;;) {
    const { value, done } = await reader.read();
    if (done) return;
    buffer += decoder.decode(value, { stream: true }).replace(/\r\n/g, "\n");
    let cut = buffer.indexOf("\n\n");
    while (cut >= 0) {
      const frame = buffer.slice(0, cut);
      buffer = buffer.slice(cut + 2);
      cut = buffer.indexOf("\n\n");
      let event = "message";
      const data: string[] = [];
      for (const line of frame.split("\n")) {
        if (line.startsWith("event:")) event = line.slice(6).trim();
        else if (line.startsWith("data:")) data.push(line.slice(5).replace(/^ /, ""));
      }
      if (!data.length) continue;
      const text = data.join("\n");
      if (event === "error") {
        yield problem(0, text);
        return;
      }
      for (const e of fromChunk(JSON.parse(text) as Chunk)) yield e;
    }
  }
}

/** Frames from the socket as an async queue. */
async function* viaSocket(req: Request, signal: AbortSignal): AsyncGenerator<Event> {
  // The socket takes its caller from the cookie at the upgrade: refresh first.
  await fetch("/v1/me", { credentials: "include", signal }).catch(() => null);
  const url = new URL("/v1/ws", window.location.href);
  url.protocol = url.protocol.replace("http", "ws");
  const ws = new WebSocket(url);
  const queue: Event[] = [];
  let wake: (() => void) | null = null;
  let over = false;
  const push = (e: Event) => {
    queue.push(e);
    wake?.();
  };
  const end = () => {
    over = true;
    wake?.();
  };
  signal.addEventListener("abort", () => {
    ws.close();
    end();
  });
  ws.onopen = () =>
    ws.send(
      JSON.stringify({ type: "call", id: "turn", method: "tbd.llm.v1.LlmService/Generate", body: req }),
    );
  ws.onmessage = (m) => {
    const f = JSON.parse(String(m.data)) as { type: string; body?: Chunk; code?: string; error?: string };
    if (f.type === "data" && f.body) fromChunk(f.body).forEach(push);
    else if (f.type === "error") {
      push({ kind: "problem", code: f.code ?? "error", error: f.error ?? "the socket refused the call" });
      ws.close();
    } else if (f.type === "end") ws.close();
  };
  ws.onerror = () => push({ kind: "problem", code: "unavailable", error: "the socket could not be opened" });
  ws.onclose = end;
  for (;;) {
    while (queue.length) {
      const e = queue.shift()!;
      yield e;
      if (e.kind !== "chunk") {
        ws.close();
        return;
      }
    }
    if (over) return;
    await new Promise<void>((r) => (wake = r));
    wake = null;
  }
}

const MCP_HEADERS = {
  "content-type": "application/json",
  accept: "application/json, text/event-stream",
  "mcp-protocol-version": "2025-11-25",
};

/** One JSON-RPC call to `/mcp`: the result, or the problem it carried. */
export async function mcp(
  method: string,
  params: unknown,
  signal?: AbortSignal,
): Promise<{ ok: true; result: unknown } | { ok: false; code: string; error: string }> {
  const res = await fetch("/mcp", {
    method: "POST",
    credentials: "include",
    signal,
    headers: MCP_HEADERS,
    body: JSON.stringify({ jsonrpc: "2.0", id: 1, method, params }),
  });
  const text = await res.text();
  if (!res.ok) {
    const p = problem(res.status, text) as Extract<Event, { kind: "problem" }>;
    return { ok: false, code: p.code, error: p.error };
  }
  // JSON, or one server-sent event carrying it.
  const data = text.includes("data:")
    ? text
        .split("\n")
        .filter((l) => l.startsWith("data:"))
        .map((l) => l.slice(5).trim())
        .pop()!
    : text;
  const body = JSON.parse(data) as { result?: unknown; error?: { code: number; message: string } };
  if (body.error) return { ok: false, code: String(body.error.code), error: body.error.message };
  return { ok: true, result: body.result };
}

/** A tool's answer: the JSON in its one text block, and whether it is an error. */
export function toolAnswer(result: unknown): { value: unknown; error: boolean } {
  const r = result as { content?: { text?: string }[]; isError?: boolean };
  const text = r.content?.[0]?.text ?? "null";
  let value: unknown = text;
  try {
    value = JSON.parse(text);
  } catch {
    /* plain text stays text */
  }
  return { value, error: !!r.isError };
}

async function* viaMcp(req: Request, signal: AbortSignal): AsyncGenerator<Event> {
  const r = await mcp("tools/call", { name: "llm_generate", arguments: req }, signal);
  if (!r.ok) {
    yield { kind: "problem", code: r.code, error: r.error };
    return;
  }
  const { value, error } = toolAnswer(r.result);
  const v = value as {
    text?: string;
    reasoning?: string;
    cut?: string;
    last?: Chunk;
    code?: string;
    error?: string;
  };
  if (error) {
    yield { kind: "problem", code: v.code ?? "error", error: v.error ?? JSON.stringify(value) };
    return;
  }
  const last = v.last;
  if (v.reasoning) yield { kind: "chunk", text: v.reasoning, reasoning: true };
  if (v.text)
    yield {
      kind: "chunk",
      text: v.text,
      reasoning: false,
      engine: last?.engine,
      model: last?.model,
      stub: last?.stub,
    };
  yield {
    kind: "done",
    usage: last?.usage ?? null,
    engine: last?.engine,
    model: last?.model,
    stub: last?.stub,
    cut: v.cut,
  };
}

export type Tool = {
  name: string;
  description?: string;
  inputSchema?: { properties?: Record<string, unknown> };
};

/** The tools this caller is offered, as an agent sees them. */
export async function listTools(): Promise<{ ok: true; tools: Tool[] } | { ok: false; error: string }> {
  const r = await mcp("tools/list", {});
  if (!r.ok) return { ok: false, error: `${r.code}: ${r.error}` };
  return { ok: true, tools: ((r.result as { tools?: Tool[] }).tools ?? []) as Tool[] };
}
