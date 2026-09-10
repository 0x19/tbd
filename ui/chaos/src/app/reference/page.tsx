"use client";

import { PageTitle } from "@/components/kit";
import { ScenarioReference } from "@/components/scenario-reference";

/** The scenario file reference as a page: docs/chaos/scenarios.md, bundled at build time. */
export default function ReferencePage() {
  return (
    <>
      <PageTitle
        title="Scenario reference"
        description="Everything a scenario file can say: the five tables, every key with its default, behaviours, assertions and how to choose bounds. The same text as docs/chaos/scenarios.md in the repo."
      />
      <div className="max-w-3xl">
        <ScenarioReference />
      </div>
    </>
  );
}
