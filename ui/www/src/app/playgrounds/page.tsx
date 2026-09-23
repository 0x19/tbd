import type { Metadata } from "next";

import { PlaygroundsContent } from "@/components/pages/playgrounds";

export const metadata: Metadata = {
  title: "Playgrounds",
  description: "Small things I build for fun, put up so anyone can try them.",
  alternates: { canonical: "/playgrounds/" },
};

export default function PlaygroundsPage() {
  return <PlaygroundsContent />;
}
