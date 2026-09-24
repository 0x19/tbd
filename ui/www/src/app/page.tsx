import type { Metadata } from "next";

import { HomeContent } from "@/components/pages/home";

export const metadata: Metadata = {
  alternates: { canonical: "/" },
};

export default function HomePage() {
  return <HomeContent />;
}
