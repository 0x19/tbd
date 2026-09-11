"use client";

import { CampaignReference } from "@/components/campaign-reference";
import { PageTitle } from "@/components/kit";
import { ScenarioReference } from "@/components/scenario-reference";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";

/** The file references as a page: docs/chaos/scenarios.md and stress.md, bundled at build time. */
export default function ReferencePage() {
  return (
    <>
      <PageTitle
        title="Reference"
        description="Everything a scenario file and a stress campaign can say: every table, every key with its default, behaviours, assertions, operations and invariants. The same text as docs/chaos/scenarios.md and docs/chaos/stress.md in the repo."
      />
      <Tabs defaultValue="scenarios" className="max-w-3xl">
        <TabsList>
          <TabsTrigger value="scenarios">Scenarios</TabsTrigger>
          <TabsTrigger value="stress">Stress campaigns</TabsTrigger>
        </TabsList>
        <TabsContent value="scenarios">
          <ScenarioReference />
        </TabsContent>
        <TabsContent value="stress">
          <CampaignReference />
        </TabsContent>
      </Tabs>
    </>
  );
}
