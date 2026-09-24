import type { Metadata } from "next";
import { Suspense } from "react";

import { LabDraftContent } from "@/components/pages/lab-draft";

// Never indexed: the page is an empty shell until an admin's browser fetches a
// draft into it, and for anyone else it says there is nothing here.
export const metadata: Metadata = {
  title: "Draft",
  robots: { index: false, follow: false },
};

export default function LabDraftPage() {
  return (
    <Suspense>
      <LabDraftContent />
    </Suspense>
  );
}
