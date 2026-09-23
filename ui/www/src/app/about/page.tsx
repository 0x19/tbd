import type { Metadata } from "next";

import { AboutContent } from "@/components/pages/about";
import { company } from "@/data/site";

export const metadata: Metadata = {
  title: "About",
  description: `${company.person} — ${company.title}. The record, role by role, with a PDF to keep.`,
  alternates: { canonical: "/about/" },
};

export default function AboutPage() {
  return <AboutContent />;
}
