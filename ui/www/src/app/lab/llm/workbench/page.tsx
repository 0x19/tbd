import type { Metadata } from "next";

import { WorkbenchPage } from "@/components/workbench/workbench";
import { lab } from "@/data/site";

export const metadata: Metadata = {
  title: "Workbench",
  description:
    "Talk to the two local models of the lab's platform through its own gateway, over server-sent events, a WebSocket or MCP, and watch what it does behind the scenes.",
  alternates: { canonical: "/lab/llm/workbench/" },
  ...(lab.public ? {} : { robots: { index: false, follow: false } }),
};

export default function Page() {
  return <WorkbenchPage />;
}
