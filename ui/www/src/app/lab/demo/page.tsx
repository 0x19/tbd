import type { Metadata } from "next";

import { LabDemoContent } from "@/components/pages/lab-demo";
import { lab } from "@/data/site";

export const metadata: Metadata = {
  title: "Lab demo",
  description: "The live demo of the lab's model service, behind a sign-in.",
  alternates: { canonical: "/lab/demo/" },
  ...(lab.public ? {} : { robots: { index: false, follow: false } }),
};

export default function LabDemoPage() {
  return <LabDemoContent />;
}
