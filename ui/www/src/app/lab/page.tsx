import type { Metadata } from "next";

import { LabContent } from "@/components/pages/lab";
import { lab } from "@/data/site";

export const metadata: Metadata = {
  title: "Lab",
  description: "RFCs, studies and demos of what is being built, as they are written and measured.",
  alternates: { canonical: "/lab/" },
  // Admins-only until the first page is published (`lab.public`): nothing to index.
  ...(lab.public ? {} : { robots: { index: false, follow: false } }),
};

export default function LabPage() {
  return <LabContent />;
}
