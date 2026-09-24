import type { Metadata } from "next";

import { ProjectsContent } from "@/components/pages/projects";

export const metadata: Metadata = {
  title: "Work",
  description: "Open-source libraries and tools in Go: blockchain tooling, data and storage, telecom.",
  alternates: { canonical: "/work/" },
};

export default function WorkPage() {
  return <ProjectsContent />;
}
