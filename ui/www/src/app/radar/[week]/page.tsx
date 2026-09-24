import type { Metadata } from "next";
import { notFound } from "next/navigation";

import { RadarContent } from "@/components/pages/radar";
import { issues } from "@/generated/radar/index";
import { issueLabel } from "@/lib/radar";

// A static export: one page per published week, rendered at build time from
// the published digests (`tool/radar-data.ts`), so search engines and link
// previews read the issue itself. An unknown week is the web server's 404.
export const dynamicParams = false;

export function generateStaticParams() {
  // A static export refuses a dynamic route with no params; before the first
  // issue is published one placeholder keeps the route and answers 404.
  const params = issues.map((i) => ({ week: i.slug }));
  return params.length ? params : [{ week: "none" }];
}

function issueOf(slug: string) {
  return issues.find((i) => i.slug === slug);
}

export async function generateMetadata({ params }: { params: Promise<{ week: string }> }): Promise<Metadata> {
  const { week } = await params;
  const issue = issueOf(week);
  if (!issue) return {};
  const n = Number(issue.week.split("-W")[1] ?? 0);
  const lead = issue.digests.find((d) => d.language === "go" && d.lang === "en") ?? issue.digests[0];
  const title = `Radar ${issueLabel(issue.number)} · Week ${n} · Go and Rust`;
  const description = lead?.summary || "What changed in Go and Rust this week, from the official sources.";
  return {
    title,
    description,
    alternates: { canonical: `/radar/${issue.slug}/` },
    openGraph: { title, description, type: "article" },
  };
}

export default async function RadarIssuePage({ params }: { params: Promise<{ week: string }> }) {
  const { week } = await params;
  const issue = issueOf(week);
  if (!issue) notFound();
  return <RadarContent fixedWeek={issue.week} />;
}
