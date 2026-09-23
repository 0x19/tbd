import type { Metadata } from "next";

import { ProjectsContent } from "@/components/pages/projects";

export const metadata: Metadata = {
  title: "Work",
  description: "Libraries and tools I have written and left in the open.",
  alternates: { canonical: "/work/" },
};

export default function WorkPage() {
  return <ProjectsContent />;
}
