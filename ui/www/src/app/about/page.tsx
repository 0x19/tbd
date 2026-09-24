import type { Metadata } from "next";

import { AboutContent } from "@/components/pages/about";
import { company } from "@/data/site";

export const metadata: Metadata = {
  title: "About",
  description: `${company.person} — ${company.title}. Who I am, the path in chapters, and the CV as a PDF.`,
  alternates: { canonical: "/about/" },
};

export default function AboutPage() {
  return <AboutContent />;
}
