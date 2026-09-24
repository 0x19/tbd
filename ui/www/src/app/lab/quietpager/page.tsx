import type { Metadata } from "next";

import { LabHomeContent } from "@/components/pages/lab-home";
import { lab } from "@/data/site";

export const metadata: Metadata = {
  title: "Quiet Pager",
  description:
    "Katas from production systems in Go and Rust, graded under injected faults by an open-source tool, and a weekly radar of both languages.",
  alternates: { canonical: "/lab/quietpager/" },
  ...(lab.public ? {} : { robots: { index: false, follow: false } }),
};

/** Quiet Pager's lab page: its documents are drafts, so for now only admins see any. */
export default function QuietPagerLabPage() {
  return <LabHomeContent id="quietpager" />;
}
