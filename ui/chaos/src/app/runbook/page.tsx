"use client";

import { Activity, ExternalLink, FlaskConical, ListChecks, type LucideIcon, MonitorCog } from "lucide-react";
import { useState } from "react";

import { useChaos } from "@/app/providers";
import { PageTitle } from "@/components/kit";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

type Entry = { symptom: string; means: string; look: string; then: string };
type Section = {
  id: string;
  title: string;
  icon: LucideIcon;
  blurb: string;
  entries: Entry[];
};

const SECTIONS: Section[] = [
  {
    id: "validate",
    title: "Validate",
    icon: ListChecks,
    blurb: "One check per surface. A failing check names the surface that is broken.",
    entries: [
      {
        symptom: "http_readyz fails, http_healthz passes",
        means: "The protocol is up but cannot reach its engine. Through Envoy: no healthy engine endpoint.",
        look: "Stack page (is the engine running?), Envoy admin /clusters for the engine cluster, engine pod logs.",
        then: "Start the engine, or fix the engine URL (must be Envoy's engine LB).",
      },
      {
        symptom: "Every gRPC check fails with transport error",
        means: "Nothing listens on the engine URL, or it is HTTP/1.1 only (gRPC needs h2c or TLS).",
        look: "The engine URL in Targets; grpcurl -plaintext <host:port> list.",
        then: "Point at the engine LB (:15051 locally) not the edge; check the port map in docs/local-cluster.md.",
      },
      {
        symptom: "sse_events or grpc_engine_subscribe time out",
        means:
          "Streams are cut before two events arrive: a proxy with a stream timeout, or a heartbeat longer than the check timeout.",
        look: "Envoy route timeouts (SSE and gRPC routes must be 0s), engine heartbeat setting.",
        then: "Raise the timeout, or fix the route. The two streaming checks take about one heartbeat interval.",
      },
      {
        symptom: "ws_echo fails, everything else passes",
        means: "WebSocket upgrade is not forwarded, or the first frame stalls.",
        look: "Envoy upgrade_configs on the edge; a 40 ms stall points at Nagle (TCP_NODELAY).",
        then: "Route /ws with the websocket upgrade; clients and servers set TCP_NODELAY.",
      },
    ],
  },
  {
    id: "scenarios",
    title: "Scenarios",
    icon: FlaskConical,
    blurb: "A run is setup, load with a timeline, assertions, teardown. Each part fails differently.",
    entries: [
      {
        symptom: "Run status error, message starts with setup:",
        means:
          "The stack did not start: a fixed port is taken, or a protocol references an engine that never became ready.",
        look: "The error text; ss -ltnp for the port; the scenario's [stack] names.",
        then: "Scenarios should not use fixed ports. Free the port or drop listen.",
      },
      {
        symptom: "max_error_rate fails, errors are all http 503",
        means: "The engine was down or returning UNAVAILABLE for longer than the scenario allows.",
        look: "The timeline: a stop without a start, or a start that errored. Per-service counters.",
        then: "Check the timeline offsets against the load duration; widen the bound only if the scenario proves something else.",
      },
      {
        symptom: "max_p99_ms fails on CI, passes locally",
        means: "CI runners are slower and noisier; the bound came from the fastest machine.",
        look: "p99 in the run record versus the bound; the baseline scenario's p99 on the same runner.",
        then: "Set bounds from what the scenario proves. Loopback p99 here is about 3 ms; baseline allows 50.",
      },
      {
        symptom: "Timeline event has an error",
        means: "The action could not be applied: unknown instance, already running, or no fault injection.",
        look: "The action text; only engines have fault injection.",
        then: "Fix the instance name or the action; chaos check catches references before running.",
      },
      {
        symptom: "Throughput is far below rate",
        means: "The pacer stalled at max_in_flight: latency times rate exceeds the concurrency cap.",
        look: "p99 during the run; requests_total versus rate × duration.",
        then: "Raise max_in_flight, or accept that a slow engine limits throughput, which is what the run shows.",
      },
    ],
  },
  {
    id: "serve",
    title: "Serve and UI",
    icon: MonitorCog,
    blurb: "The API behind this page and the pod it runs in.",
    entries: [
      {
        symptom: "chaos API unreachable in the header",
        means:
          "The browser cannot reach /api/chaos: serve is down, or next dev is talking to the wrong port.",
        look: "curl <api>/healthz; NEXT_PUBLIC_CHAOS_API in dev; Envoy's chaos cluster in the cluster.",
        then: "Start mise run chaos:serve, or set the API URL.",
      },
      {
        symptom: "Live feed off in the footer",
        means: "The SSE connection dropped: a proxy timed the stream out, or serve restarted.",
        look: "Envoy route for /api/chaos/ must have timeout 0s; serve logs.",
        then: "The page reconnects on its own; reload if it does not.",
      },
      {
        symptom: "409 a run is already active",
        means: "One run at a time, so numbers are not mixed.",
        look: "The Runs page shows the active run.",
        then: "Wait, or cancel it.",
      },
      {
        symptom: "Saving a scenario fails with read-only file system",
        means: "The scenarios directory is inside the image.",
        look: "CHAOS_SCENARIOS_DIR and CHAOS_SCENARIOS_SEED on the pod.",
        then: "Point the directory at a volume; the seed fills it on start.",
      },
    ],
  },
  {
    id: "observability",
    title: "Observability",
    icon: Activity,
    blurb:
      "Every request carries a trace id: logs, traces and profiles for the same second are one click apart.",
    entries: [
      {
        symptom: "A run failed and you want the server side",
        means: "The engine and protocol logged every failure with the trace id of the request.",
        look: "Logs, filtered by time of the run and level error; Grafana's tbd/protocol and tbd/engine dashboards.",
        then: "Take a trace_id from a log line into Explore (Tempo) to see the whole request across Envoy, protocol and engine.",
      },
      {
        symptom: "Latency is high and nothing errors",
        means: "CPU or lock contention, not failures.",
        look: "Profiles (Pyroscope) for the run's minute; Envoy upstream time in the tbd/envoy dashboard.",
        then: "Compare the flame graph against a healthy minute; the hot frame names the code.",
      },
    ],
  },
];

/** The kit's Settings page: a vertical nav on the left, titled rows with dividers on the right. */
export default function RunbookPage() {
  const { overview } = useChaos();
  const links = overview?.config.links;
  const [active, setActive] = useState(SECTIONS[0].id);
  const section = SECTIONS.find((s) => s.id === active) ?? SECTIONS[0];

  return (
    <>
      <PageTitle
        title="Runbook"
        description="What a failure means, where to look, what to do. Add an entry whenever a failure taught something."
      />
      <div className="grid gap-8 lg:grid-cols-[14rem_1fr]">
        <nav className="grid content-start gap-1">
          {SECTIONS.map((s) => (
            <button
              key={s.id}
              type="button"
              onClick={() => setActive(s.id)}
              className={cn(
                "hover:bg-muted flex items-center gap-2 rounded-lg px-3 py-2 text-left text-sm",
                s.id === active && "bg-muted font-medium",
              )}
            >
              <s.icon className="size-4" />
              {s.title}
            </button>
          ))}
          {links && (links.grafana || links.victorialogs || links.pyroscope) ? (
            <div className="mt-4 grid gap-1 border-t pt-4">
              <div className="text-muted-foreground px-3 text-xs font-medium uppercase">Open</div>
              {[
                {
                  t: "Dashboards",
                  h: links.grafana ? `${links.grafana}/dashboards` : "",
                },
                {
                  t: "Traces",
                  h: links.grafana ? `${links.grafana}/explore` : "",
                },
                { t: "Logs", h: links.victorialogs },
                { t: "Metrics", h: links.metrics },
                { t: "Profiles", h: links.pyroscope },
              ]
                .filter((l) => l.h)
                .map((l) => (
                  <Button key={l.t} variant="ghost" size="sm" className="justify-start" asChild>
                    <a href={l.h} target="_blank" rel="noreferrer">
                      <ExternalLink /> {l.t}
                    </a>
                  </Button>
                ))}
            </div>
          ) : null}
        </nav>
        <div>
          <h2 className="text-lg font-semibold">{section.title}</h2>
          <p className="text-muted-foreground text-sm">{section.blurb}</p>
          <div className="mt-4 divide-y border-t">
            {section.entries.map((e) => (
              <div key={e.symptom} className="grid gap-4 py-5 md:grid-cols-[1fr_2fr]">
                <div>
                  <div className="font-medium">{e.symptom}</div>
                  <div className="text-muted-foreground mt-1 text-sm">{e.means}</div>
                </div>
                <dl className="grid gap-2 text-sm md:grid-cols-[5rem_1fr]">
                  <dt className="text-muted-foreground">Look at</dt>
                  <dd>{e.look}</dd>
                  <dt className="text-muted-foreground">Then</dt>
                  <dd>{e.then}</dd>
                </dl>
              </div>
            ))}
          </div>
        </div>
      </div>
    </>
  );
}
