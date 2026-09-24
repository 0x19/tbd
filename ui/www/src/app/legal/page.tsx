import type { Metadata } from "next";

import { LegalContent } from "@/components/pages/legal";
import { company } from "@/data/site";

export const metadata: Metadata = {
  title: "Legal",
  description: `Company details and privacy notice for ${company.legalName}.`,
  alternates: { canonical: "/legal/" },
};

export default function LegalPage() {
  return <LegalContent />;
}
