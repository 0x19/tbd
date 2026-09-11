"use client";

import { ArrowLeft, ArrowRight, ChevronDown, ChevronRight, Search } from "lucide-react";
import Link from "next/link";
import { useRouter, useSearchParams } from "next/navigation";
import { Suspense, useEffect, useState } from "react";

import { CategoryIcon, DocMeta } from "@/components/kb-bits";
import { Markdown } from "@/components/markdown";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/components/ui/collapsible";
import { Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "@/components/ui/empty";
import { Skeleton } from "@/components/ui/skeleton";
import {
  byId,
  categories,
  categoryOf,
  docHref,
  type KbCategory,
  type KbDoc,
  neighbours,
  related,
  resolveLink,
} from "@/lib/kb";
import { cn } from "@/lib/utils";

export default function ArticlePage() {
  return (
    <Suspense fallback={<Skeleton className="h-40" />}>
      <Article />
    </Suspense>
  );
}

/**
 * One document: the category nav on the left (the kit's settings sidebar), the
 * article column in the middle (the kit's inbox article: chip row, title, lead,
 * body), the table of contents on the right with the section in view marked.
 */
function Article() {
  const params = useSearchParams();
  const router = useRouter();
  const id = params.get("doc");
  const doc = byId(id);

  // A `#heading` in the URL lands on the heading once the article is in the DOM.
  useEffect(() => {
    if (!doc) return;
    const hash = window.location.hash.slice(1);
    if (!hash) {
      window.scrollTo({ top: 0 });
      return;
    }
    const el = document.getElementById(decodeURIComponent(hash));
    el?.scrollIntoView({ block: "start" });
  }, [doc]);

  if (!doc) {
    return (
      <Empty>
        <EmptyHeader>
          <EmptyMedia variant="icon">
            <Search />
          </EmptyMedia>
          <EmptyTitle>No such page</EmptyTitle>
          <EmptyDescription>
            {id ? `${id} is not in the knowledge base.` : "No document was named."} Browse the categories or
            search.
          </EmptyDescription>
        </EmptyHeader>
        <Button variant="outline" asChild>
          <Link href="/kb/">Knowledge base</Link>
        </Button>
      </Empty>
    );
  }

  const category = categoryOf(doc);
  const { prev, next } = neighbours(doc);
  const rel = related(doc);
  const toc = doc.headings.filter((h) => h.level === 2 || h.level === 3);

  return (
    <div className="grid gap-8 lg:grid-cols-[14rem_minmax(0,1fr)] 2xl:grid-cols-[14rem_minmax(0,1fr)_14rem]">
      <aside className="hidden lg:block">
        <CategoryNav current={doc} />
      </aside>

      <article className="mx-auto w-full max-w-[820px] min-w-0">
        <div className="text-muted-foreground mb-3 flex items-center gap-2 text-xs">
          <Link href="/kb/" className="hover:text-foreground">
            Knowledge base
          </Link>
          <ChevronRight className="size-3" />
          <Link
            href={`/kb/?category=${category.id}`}
            className="hover:text-foreground inline-flex items-center gap-1.5"
          >
            <CategoryIcon id={category.id} className="size-3" /> {category.title}
          </Link>
        </div>
        <h1 className="text-2xl font-semibold tracking-tight sm:text-3xl">{doc.title}</h1>
        <div className="mt-3">
          <DocMeta doc={doc} advisory={category.advisory} />
        </div>

        {category.advisory ? (
          <Alert className="mt-6">
            <AlertTitle>Idea material, not a spec</AlertTitle>
            <AlertDescription>
              Earlier thinking about the product direction, kept for the reasoning trail. Where it disagrees
              with the code or with the pages above, the code wins.
            </AlertDescription>
          </Alert>
        ) : null}
        {doc.generated ? (
          <Alert className="mt-6">
            <AlertTitle>Generated from the code</AlertTitle>
            <AlertDescription>
              Written by <code className="font-mono text-xs">mise run chaos:docs</code>; CI fails when it is
              stale. Edit the registry, not this page.
            </AlertDescription>
          </Alert>
        ) : null}

        {toc.length ? (
          <Collapsible className="mt-6 2xl:hidden">
            <CollapsibleTrigger className="text-muted-foreground flex items-center gap-1 text-xs font-medium uppercase">
              On this page <ChevronDown className="size-3" />
            </CollapsibleTrigger>
            <CollapsibleContent>
              <Toc doc={doc} />
            </CollapsibleContent>
          </Collapsible>
        ) : null}

        <Markdown
          key={doc.id}
          text={doc.text}
          className="doc-article mt-8"
          hideTitle
          resolve={(href) => resolveLink(doc, href)}
        />

        <footer className="mt-12 grid gap-6 border-t pt-6">
          {rel.to.length || rel.from.length ? (
            <div className="grid gap-3 text-sm">
              {rel.to.length ? (
                <div className="flex flex-wrap items-baseline gap-2">
                  <span className="text-muted-foreground text-xs uppercase">Links to</span>
                  {rel.to.map((d) => (
                    <Link
                      key={d.id}
                      href={docHref(d.id)}
                      className="hover:bg-muted rounded-md border px-2 py-0.5 text-xs"
                    >
                      {d.title}
                    </Link>
                  ))}
                </div>
              ) : null}
              {rel.from.length ? (
                <div className="flex flex-wrap items-baseline gap-2">
                  <span className="text-muted-foreground text-xs uppercase">Linked from</span>
                  {rel.from.map((d) => (
                    <Link
                      key={d.id}
                      href={docHref(d.id)}
                      className="hover:bg-muted rounded-md border px-2 py-0.5 text-xs"
                    >
                      {d.title}
                    </Link>
                  ))}
                </div>
              ) : null}
            </div>
          ) : null}
          <div className="grid gap-3 sm:grid-cols-2">
            {prev ? (
              <Link
                href={docHref(prev.id)}
                className="hover:bg-muted/40 rounded-xl border p-4 transition-colors"
              >
                <div className="text-muted-foreground flex items-center gap-1 text-xs">
                  <ArrowLeft className="size-3" /> Previous
                </div>
                <div className="mt-1 font-medium">{prev.title}</div>
              </Link>
            ) : (
              <span />
            )}
            {next ? (
              <Link
                href={docHref(next.id)}
                className="hover:bg-muted/40 rounded-xl border p-4 text-right transition-colors"
              >
                <div className="text-muted-foreground flex items-center justify-end gap-1 text-xs">
                  Next <ArrowRight className="size-3" />
                </div>
                <div className="mt-1 font-medium">{next.title}</div>
              </Link>
            ) : null}
          </div>
          <Button
            variant="ghost"
            size="sm"
            className="justify-self-start"
            onClick={() => router.push(`/kb/?category=${category.id}`)}
          >
            <ArrowLeft /> All of {category.title}
          </Button>
        </footer>
      </article>

      <aside className="hidden 2xl:block">
        <div className="sticky top-6">
          {toc.length ? (
            <>
              <div className="text-muted-foreground mb-2 text-xs font-medium uppercase">On this page</div>
              <Toc doc={doc} spy />
            </>
          ) : null}
        </div>
      </aside>
    </div>
  );
}

/** The category sidebar: every page, grouped, current one marked; design notes folded unless open. */
function CategoryNav({ current }: { current: KbDoc }) {
  return (
    <nav className="sticky top-6 grid max-h-[calc(100vh-5rem)] content-start gap-4 overflow-y-auto pr-2 text-sm">
      {categories.map((c) => (
        <CategoryGroup key={c.id} category={c} current={current} />
      ))}
    </nav>
  );
}

function CategoryGroup({ category, current }: { category: KbCategory; current: KbDoc }) {
  const holds = category.docs.some((d) => d.id === current.id);
  const [open, setOpen] = useState(holds || !category.advisory);
  useEffect(() => {
    if (holds) setOpen(true);
  }, [holds]);
  return (
    <Collapsible open={open} onOpenChange={setOpen}>
      <CollapsibleTrigger className="text-muted-foreground hover:text-foreground flex w-full items-center gap-1.5 px-2 text-[11px] font-medium tracking-wide uppercase">
        <CategoryIcon id={category.id} className="size-3" />
        {category.title}
        <ChevronDown className={cn("ml-auto size-3 transition-transform", !open && "-rotate-90")} />
      </CollapsibleTrigger>
      <CollapsibleContent>
        <ul className="mt-1 grid gap-0.5">
          {category.docs.map((d) => (
            <li key={d.id}>
              <Link
                href={docHref(d.id)}
                aria-current={d.id === current.id ? "page" : undefined}
                className={cn(
                  "hover:bg-muted block truncate rounded-lg px-2 py-1.5 text-[13px]",
                  d.id === current.id && "bg-muted font-medium",
                )}
                title={d.path}
              >
                {d.title}
              </Link>
            </li>
          ))}
        </ul>
      </CollapsibleContent>
    </Collapsible>
  );
}

/** The table of contents; with `spy` the heading in view is marked as the page scrolls. */
function Toc({ doc, spy = false }: { doc: KbDoc; spy?: boolean }) {
  const items = doc.headings.filter((h) => h.level === 2 || h.level === 3);
  const [active, setActive] = useState<string | null>(null);

  useEffect(() => {
    if (!spy) return;
    let frame = 0;
    const update = () => {
      frame = 0;
      let cur: string | null = null;
      for (const h of items) {
        const el = document.getElementById(h.id);
        if (!el) continue;
        if (el.getBoundingClientRect().top <= 96) cur = h.id;
        else break;
      }
      setActive(cur ?? items[0]?.id ?? null);
    };
    const onScroll = () => {
      if (!frame) frame = requestAnimationFrame(update);
    };
    update();
    window.addEventListener("scroll", onScroll, { passive: true });
    return () => {
      window.removeEventListener("scroll", onScroll);
      if (frame) cancelAnimationFrame(frame);
    };
  }, [spy, items]);

  return (
    <ul className="grid gap-0.5 border-l text-[13px]">
      {items.map((h) => (
        <li key={h.id}>
          <a
            href={`#${h.id}`}
            data-active={spy && active === h.id ? "true" : undefined}
            className={cn(
              "text-muted-foreground hover:text-foreground -ml-px block border-l border-transparent py-1 pl-3 transition-colors",
              h.level === 3 && "pl-6",
              spy && active === h.id && "border-foreground text-foreground font-medium",
            )}
            onClick={(e) => {
              e.preventDefault();
              document.getElementById(h.id)?.scrollIntoView({ block: "start", behavior: "smooth" });
              history.replaceState(null, "", `#${h.id}`);
            }}
          >
            {h.title}
          </a>
        </li>
      ))}
    </ul>
  );
}
