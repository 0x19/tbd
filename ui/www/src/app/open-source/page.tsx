import type { Metadata } from "next";

import { ProjectsContent } from "@/components/pages/projects";

export const metadata: Metadata = {
  title: "Open source",
  description: "Open-source libraries and tools in Go: blockchain tooling, data and storage, telecom.",
  alternates: { canonical: "/open-source/" },
};

export default function OpenSourcePage() {
  return <ProjectsContent />;
}
