import type { Metadata } from "next";

import { ContactContent } from "@/components/pages/contact";
import { company } from "@/data/site";

export const metadata: Metadata = {
  title: "Contact",
  description: `How to reach ${company.person}.`,
  alternates: { canonical: "/contact/" },
};

export default function ContactPage() {
  return <ContactContent />;
}
