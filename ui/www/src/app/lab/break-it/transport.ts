/**
 * Three ways to watch the same thing.
 *
 * The gateway generates REST, server-sent events and a multiplexed WebSocket
 * from one set of proto descriptors, so the game does not change when the pipe
 * does — only this file does. Each transport reports what it costs, which is
 * the point of letting a visitor switch.
 */

export type Transport = "ws" | "sse" | "poll";

/** What a transport is costing right now, for the readout. */
export type Meter = {
  /** Milliseconds since the last frame arrived. */
  latency: number | null;
  /** Bytes received since the transport opened. */
  bytes: number;
  /** Frames or responses received. */
  frames: number;
  /** Times it had to reconnect or re-poll. */
  opens: number;
};

export const EMPTY_METER: Meter = { latency: null, bytes: 0, frames: 0, opens: 0 };

/** The shape the page reads. Only the fields it uses are named. */
export type World = {
  health: "HEALTH_HEALED" | "HEALTH_DEGRADED" | "HEALTH_BREACHED" | "HEALTH_UNSPECIFIED";
  objective?: {
    target?: number;
    current?: number;
    window_seconds?: number;
    latency_target_ms?: number;
    latency_current_ms?: number;
  };
  budget?: { tokens?: number; max_tokens?: number; refill_in_seconds?: number };
  traffic?: { requests_per_second?: number; error_rate?: number; p50_ms?: number; p99_ms?: number };
  instances?: {
    name: string;
    kind: string;
    running?: boolean;
    ailment?: string;
    requests_total?: string | number;
    requests_failed?: string | number;
    /** Whether a balancer sits in front of this kind at all. */
    balanced?: boolean;
    in_rotation?: boolean;
    /** Taken out by the balancer for failing or crawling; still up. */
    ejected?: boolean;
  }[];
  faults?: {
    id: string;
    move: string;
    instance: string;
    description: string;
    expires_in_seconds?: number;
    actor?: string;
  }[];
  held_for_seconds?: number;
  /** Seconds until the last bad second leaves the window. */
  healing_seconds?: number;
};

export type Score = {
  actor: string;
  seconds_to_breach?: number;
  faults_used?: number;
  at?: string;
};

type Sink = {
  world: (world: World) => void;
  meter: (update: (m: Meter) => Meter) => void;
  error: (message: string | null) => void;
};

/** Everything the page talks to, in one place. */
export const api = {
  world: "/v1/playground/world",
  events: "/v1/playground/events",
  faults: "/v1/playground/faults",
  scores: "/v1/playground/scores",
  socket: "/v1/ws",
} as const;

function measure(sink: Sink, bytes: number, since: number) {
  sink.meter((m) => ({
    latency: Math.round(performance.now() - since),
    bytes: m.bytes + bytes,
    frames: m.frames + 1,
    opens: m.opens,
  }));
}

/**
 * Open `transport` and stream worlds into `sink` until the returned function is
 * called. Each one is a few lines because the gateway serves all three from the
 * same RPC.
 */
export function open(transport: Transport, sink: Sink): () => void {
  switch (transport) {
    case "sse":
      return openSse(sink);
    case "poll":
      return openPoll(sink);
    default:
      return openSocket(sink);
  }
}

/** Server-sent events: the transcoded server-streaming RPC. */
function openSse(sink: Sink): () => void {
  const source = new EventSource(api.events);
  let last = performance.now();
  sink.meter((m) => ({ ...m, opens: m.opens + 1 }));

  source.onmessage = (event) => {
    try {
      const frame = JSON.parse(event.data) as { world?: World };
      if (frame.world) sink.world(frame.world);
      measure(sink, event.data.length, last);
      last = performance.now();
      sink.error(null);
    } catch {
      sink.error("could not read a frame");
    }
  };
  source.onerror = () => sink.error("the stream dropped; it will come back");

  return () => source.close();
}

/** The multiplexed socket: one frame to call, many frames back. */
function openSocket(sink: Sink): () => void {
  const url = new URL(api.socket, window.location.href);
  url.protocol = url.protocol.replace("http", "ws");
  const socket = new WebSocket(url);
  let last = performance.now();
  let closed = false;

  socket.onopen = () => {
    sink.meter((m) => ({ ...m, opens: m.opens + 1 }));
    sink.error(null);
    socket.send(
      JSON.stringify({
        type: "call",
        id: "watch",
        method: "tbd.playground.v1.PlaygroundService/Watch",
        body: {},
      }),
    );
  };
  socket.onmessage = (event) => {
    const text = String(event.data);
    try {
      const frame = JSON.parse(text) as { type: string; body?: { world?: World }; error?: string };
      if (frame.type === "data" && frame.body?.world) sink.world(frame.body.world);
      if (frame.type === "error") sink.error(frame.error ?? "the socket refused the call");
      measure(sink, text.length, last);
      last = performance.now();
    } catch {
      sink.error("could not read a frame");
    }
  };
  socket.onerror = () => sink.error("the socket dropped; it will come back");

  return () => {
    closed = true;
    if (socket.readyState === WebSocket.OPEN) socket.close();
    else socket.onopen = () => socket.close();
    void closed;
  };
}

/** Plain HTTP, asked again every second. The honest baseline. */
function openPoll(sink: Sink): () => void {
  let stop = false;
  const tick = async () => {
    while (!stop) {
      const since = performance.now();
      try {
        const response = await fetch(api.world, { cache: "no-store" });
        const text = await response.text();
        const body = JSON.parse(text) as { world?: World };
        if (body.world) sink.world(body.world);
        measure(sink, text.length, since);
        sink.meter((m) => ({ ...m, opens: m.opens + 1 }));
        sink.error(response.ok ? null : `the gateway answered ${response.status}`);
      } catch {
        sink.error("could not reach the gateway");
      }
      await new Promise((resolve) => setTimeout(resolve, 1000));
    }
  };
  void tick();
  return () => {
    stop = true;
  };
}

/** Spend budget on a move. The same RPC the socket carries. */
export async function inject(
  move: string,
  actor: string,
): Promise<{ accepted?: boolean; reason?: string; world?: World }> {
  const response = await fetch(api.faults, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ move, actor }),
  });
  if (response.status === 429) {
    return { accepted: false, reason: "too many moves from here; give it a moment" };
  }
  if (!response.ok) {
    return { accepted: false, reason: `the gateway answered ${response.status}` };
  }
  return (await response.json()) as { accepted?: boolean; reason?: string; world?: World };
}

/** The leaderboard. */
export async function scores(): Promise<Score[]> {
  try {
    const response = await fetch(api.scores, { cache: "no-store" });
    if (!response.ok) return [];
    const body = (await response.json()) as { scores?: Score[] };
    return body.scores ?? [];
  } catch {
    return [];
  }
}
