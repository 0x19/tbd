import type { Metadata } from "next";

import { TermsContent } from "@/components/pages/terms";

export const metadata: Metadata = {
  title: "Terms",
  description: `Terms of use for this site and the playgrounds on it.`,
  alternates: { canonical: "/terms/" },
};

export default function TermsPage() {
  return <TermsContent />;
}
