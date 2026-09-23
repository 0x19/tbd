import type { Metadata } from "next";
import { notFound } from "next/navigation";

import { LabDocContent } from "@/components/pages/lab-doc";
import { lab } from "@/data/site";
import { docs } from "@/generated/lab/docs";
import { studies } from "@/generated/lab/index";

// A static export: every public page is rendered at build time from the
// generator's output, and an unknown slug is a 404 from the web server.
export const dynamicParams = false;

export function generateStaticParams() {
  // A static export refuses a dynamic route with no params, and an empty list is
  // the normal state before the first page is public: one placeholder slug keeps
  // the route, and the page answers it with the 404.
  const params = studies.map((e) => ({ slug: e.slug }));
  return params.length ? params : [{ slug: "none" }];
}

export async function generateMetadata({ params }: { params: Promise<{ slug: string }> }): Promise<Metadata> {
  const { slug } = await params;
  const doc = docs[`study/${slug}`];
  if (!doc) return {};
  return {
    title: doc.title,
    description: doc.summary,
    alternates: { canonical: doc.href },
    ...(lab.public ? {} : { robots: { index: false, follow: false } }),
  };
}

export default async function LabDocPage({ params }: { params: Promise<{ slug: string }> }) {
  const { slug } = await params;
  const doc = docs[`study/${slug}`];
  if (!doc) notFound();
  return <LabDocContent doc={doc} />;
}
