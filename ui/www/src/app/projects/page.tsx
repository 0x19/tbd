import type { Metadata } from "next";

import { ProjectsContent } from "@/components/pages/projects";

export const metadata: Metadata = {
  title: "Projects",
  description: "Libraries and tools I have written and left in the open.",
  alternates: { canonical: "/projects/" },
};

export default function ProjectsPage() {
  return <ProjectsContent />;
}
