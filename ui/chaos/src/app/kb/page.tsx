"use client";

import { ArrowLeft, BookOpen, ExternalLink, LifeBuoy, Search } from "lucide-react";
import Link from "next/link";
import { useSearchParams } from "next/navigation";
import { Suspense, useMemo, useState } from "react";

import { useChaos } from "@/app/providers";
import { CategoryIcon, DocRow } from "@/components/kb-bits";
import { PageTitle } from "@/components/kit";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "@/components/ui/empty";
import { InputGroup, InputGroupAddon, InputGroupInput } from "@/components/ui/input-group";
import { Skeleton } from "@/components/ui/skeleton";
import { byId, categories, docHref, highlight, type Hit, ordered, search } from "@/lib/kb";

export default function KnowledgeBasePage() {
  return (
    <Suspense fallback={<Skeleton className="h-40" />}>
      <KnowledgeBase />
    </Suspense>
  );
}

/**
 * The knowledge base home: a search over every document and section, the
 * categories as cards, and the environment's observability links. `?category=`
 * lists one category; `?q=` arrives with a query typed.
 */
function KnowledgeBase() {
  const params = useSearchParams();
  const categoryId = params.get("category");
  const [q, setQ] = useState(params.get("q") ?? "");
  const { overview } = useChaos();
  const links = overview?.config.links;
  const hits = useMemo(() => (q.trim().length >= 2 ? search(q, 30) : null), [q]);
  const category = categories.find((c) => c.id === categoryId) ?? null;
  const runbook = byId("docs/chaos/runbook");
  const start = categories.find((c) => c.id === "start")?.docs ?? [];

  const open = [
    { t: "Dashboards", h: links?.grafana ? `${links.grafana}/dashboards` : "" },
    { t: "Traces", h: links?.grafana ? `${links.grafana}/explore` : "" },
    { t: "Logs", h: links?.victorialogs ?? "" },
    { t: "Metrics", h: links?.metrics ?? "" },
    { t: "Profiles", h: links?.pyroscope ?? "" },
  ].filter((l) => l.h);

  return (
    <>
      <PageTitle
        title="Knowledge base"
        description={`Everything the repository documents about itself, ${ordered.length} pages, rendered from the same markdown the repo carries. Search a symptom, a flag or a table name.`}
      >
        {runbook ? (
          <Button variant="outline" asChild>
            <Link href={docHref(runbook.id)}>
              <LifeBuoy /> Runbook
            </Link>
          </Button>
        ) : null}
      </PageTitle>

      <div className="grid gap-6 xl:grid-cols-[minmax(0,1fr)_20rem]">
        <div className="grid content-start gap-6">
          <InputGroup className="max-w-2xl">
            <InputGroupAddon>
              <Search />
            </InputGroupAddon>
            <InputGroupInput
              aria-label="Search the knowledge base"
              placeholder="Search: http_readyz, max_in_flight, TCP_NODELAY, erasure…"
              value={q}
              onChange={(e) => setQ(e.target.value)}
              autoFocus
            />
          </InputGroup>

          {hits ? (
            <Results hits={hits} q={q} />
          ) : category ? (
            <section className="grid gap-3">
              <div className="flex items-center gap-3">
                <Button variant="outline" size="icon-sm" aria-label="All categories" asChild>
                  <Link href="/kb/">
                    <ArrowLeft />
                  </Link>
                </Button>
                <div className="bg-muted flex size-9 items-center justify-center rounded-lg">
                  <CategoryIcon id={category.id} />
                </div>
                <div>
                  <h2 className="text-lg font-semibold">{category.title}</h2>
                  <p className="text-muted-foreground text-sm">{category.description}</p>
                </div>
              </div>
              <div className="divide-y rounded-xl border">
                {category.docs.map((d) => (
                  <DocRow key={d.id} doc={d} />
                ))}
              </div>
            </section>
          ) : (
            <section className="grid gap-4 sm:grid-cols-2 2xl:grid-cols-3">
              {categories.map((c) => (
                <Link
                  key={c.id}
                  href={`/kb/?category=${c.id}`}
                  className="bg-card hover:border-foreground/30 group rounded-2xl border p-5 transition-all hover:-translate-y-0.5 hover:shadow-sm"
                >
                  <div className="flex items-start justify-between gap-3">
                    <div className="bg-muted flex size-9 items-center justify-center rounded-lg">
                      <CategoryIcon id={c.id} />
                    </div>
                    <Badge variant="outline" className="text-[10px]">
                      {c.docs.length} page{c.docs.length === 1 ? "" : "s"}
                    </Badge>
                  </div>
                  <h3 className="mt-3 font-semibold">{c.title}</h3>
                  <p className="text-muted-foreground mt-1 text-sm leading-6">{c.description}</p>
                  <ul className="text-muted-foreground mt-3 grid gap-1 text-xs">
                    {c.docs.slice(0, 4).map((d) => (
                      <li key={d.id} className="truncate">
                        {d.title}
                      </li>
                    ))}
                    {c.docs.length > 4 ? <li>and {c.docs.length - 4} more</li> : null}
                  </ul>
                </Link>
              ))}
            </section>
          )}
        </div>

        <div className="grid content-start gap-4">
          <Card>
            <CardHeader>
              <CardTitle>Start here</CardTitle>
              <CardDescription>The five pages that place everything else.</CardDescription>
            </CardHeader>
            <CardContent className="grid gap-1">
              {start.map((d) => (
                <Button key={d.id} variant="ghost" size="sm" className="justify-start" asChild>
                  <Link href={docHref(d.id)}>
                    <BookOpen /> {d.title}
                  </Link>
                </Button>
              ))}
            </CardContent>
          </Card>
          <Card>
            <CardHeader>
              <CardTitle>Where to look</CardTitle>
              <CardDescription>
                {overview ? `This environment's signals (${overview.env}).` : "This environment's signals."}
              </CardDescription>
            </CardHeader>
            <CardContent className="grid gap-1">
              {open.length ? (
                open.map((l) => (
                  <Button key={l.t} variant="ghost" size="sm" className="justify-start" asChild>
                    <a href={l.h} target="_blank" rel="noreferrer">
                      <ExternalLink /> {l.t}
                    </a>
                  </Button>
                ))
              ) : (
                <p className="text-muted-foreground text-sm">
                  No observability links configured ([links] in the chaos config).
                </p>
              )}
              {runbook ? (
                <Button variant="ghost" size="sm" className="justify-start" asChild>
                  <Link href={docHref(runbook.id)}>
                    <LifeBuoy /> What a failure means
                  </Link>
                </Button>
              ) : null}
            </CardContent>
          </Card>
        </div>
      </div>
    </>
  );
}

/** Search results, one row per matching section, the two-line shape of the kit's search dialog. */
function Results({ hits, q }: { hits: Hit[]; q: string }) {
  if (!hits.length) {
    return (
      <Empty>
        <EmptyHeader>
          <EmptyMedia variant="icon">
            <Search />
          </EmptyMedia>
          <EmptyTitle>Nothing matches</EmptyTitle>
          <EmptyDescription>
            Every word has to appear in one page. Try a shorter query, a flag name or an error class.
          </EmptyDescription>
        </EmptyHeader>
      </Empty>
    );
  }
  return (
    <div className="divide-y rounded-xl border" data-testid="kb-results">
      {hits.map((h) => {
        const cat = categories.find((c) => c.id === h.doc.category);
        return (
          <Link
            key={`${h.doc.id}#${h.heading?.id ?? ""}`}
            href={docHref(h.doc.id, h.heading ? `#${h.heading.id}` : "")}
            className="hover:bg-muted/40 block px-4 py-3 transition-colors"
          >
            <div className="flex flex-wrap items-baseline gap-x-2 text-sm">
              <span className="font-medium">{h.doc.title}</span>
              {h.heading ? (
                <>
                  <span className="text-muted-foreground/60">›</span>
                  <span>{h.heading.title}</span>
                </>
              ) : null}
              <span className="text-muted-foreground ml-auto text-xs">{cat?.title}</span>
            </div>
            <p className="text-muted-foreground mt-1 text-xs leading-5">
              {highlight(h.snippet, q).map((s, i) =>
                s.hit ? <mark key={i}>{s.text}</mark> : <span key={i}>{s.text}</span>,
              )}
            </p>
          </Link>
        );
      })}
    </div>
  );
}
