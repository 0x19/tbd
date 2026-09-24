import type { Metadata } from "next";

import { LabDemoContent } from "@/components/pages/lab-demo";
import { lab } from "@/data/site";

export const metadata: Metadata = {
  title: "Workbench",
  description:
    "Talk to the two local models of the lab's platform through its own gateway, behind a sign-in.",
  alternates: { canonical: "/lab/llm/workbench/" },
  ...(lab.public ? {} : { robots: { index: false, follow: false } }),
};

export default function WorkbenchPage() {
  return <LabDemoContent />;
}
