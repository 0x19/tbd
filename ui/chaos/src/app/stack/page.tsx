"use client";

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { InstancesTable } from "@/components/instances-table";
import { ErrorNote, PageHeader } from "@/components/page-header";
import { useStack } from "@/lib/api/hooks";

export default function StackPage() {
  const stack = useStack();
  return (
    <>
      <PageHeader
        title="Stack"
        description="The topology this serve runs in-process. Stop an instance, start it on the same port, or change an engine's fault behaviour, and watch the protocol react."
      />
      <ErrorNote message={stack.error} />
      {stack.data ? (
        <InstancesTable instances={stack.data} onChange={stack.setData} />
      ) : (
        <Skeleton className="h-40" />
      )}
      <div className="grid gap-4 md:grid-cols-3">
        <Hint title="Stop and start">
          Stopping an engine makes every protocol that depends on it report <code>/readyz</code> 503 and REST
          calls fail fast with 503. Start brings it back on the same port; the protocol reconnects on its own.
        </Hint>
        <Hint title="Faults">
          Behaviours are the same objects a scenario&apos;s <code>[[timeline]]</code> uses. <em>error</em> at
          rate 0.5 is what <code>error_injection.toml</code> does; <em>slow</em> is what{" "}
          <code>latency.toml</code> does.
        </Hint>
        <Hint title="Counters">
          Served and failed are the engine&apos;s own counters. They reset on restart and they count every
          caller, including validate and load runs pointed at this stack.
        </Hint>
      </div>
    </>
  );
}

function Hint({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <Card size="sm">
      <CardHeader>
        <CardTitle>{title}</CardTitle>
        <CardDescription>{children}</CardDescription>
      </CardHeader>
      <CardContent />
    </Card>
  );
}
