import type { Metadata } from "next";
import Link from "next/link";

import { Eyebrow, Frame } from "@/components/kit";
import { company } from "@/data/site";

export const metadata: Metadata = {
  title: "Terms",
  description: `Terms of use for this site and the playgrounds on it.`,
  alternates: { canonical: "/terms/" },
};

export default function TermsPage() {
  const sections = [
    {
      title: "What this site is",
      body: [
        `This site belongs to ${company.legalName} (OIB ${company.oib}), a company registered in Croatia. It exists to show what I build and to host the playgrounds. It is not a shop: nothing is sold here and there is no account to create.`,
      ],
    },
    {
      title: "The playgrounds",
      body: [
        "The playgrounds are experiments. They are offered as they are, with no promise that they work, stay available, keep your input, or give a correct answer. Do not put anything secret, personal or valuable into one, and do not rely on its output for a decision that matters — least of all a financial or legal one.",
        "Anything touching a blockchain is illustration, not advice. A simulation is a simulation; a transaction you sign anywhere else is yours alone.",
        "I may change a playground, rate-limit it or take it down at any time, usually without warning, because it is a side project rather than a service.",
      ],
    },
    {
      title: "Code and content",
      body: [
        "The open-source projects linked from here are governed by their own licences in their own repositories — that licence wins over anything on this page. The writing and design of this site are mine; take what is useful, but do not republish it wholesale as your own.",
      ],
    },
    {
      title: "Liability",
      body: [
        "To the extent the law allows, I am not liable for loss or damage arising from using this site or a playground on it. Nothing here limits liability for intent or gross negligence, or any liability that cannot be limited under Croatian law.",
      ],
    },
    {
      title: "Links out",
      body: [
        "Links leave for GitHub, my employer and other people's sites. What happens there is theirs, not mine.",
      ],
    },
    {
      title: "Law and changes",
      body: [
        "Croatian law applies, and the courts of Croatia have jurisdiction. These terms can change; the version on this page at the time you read it is the one that applies.",
      ],
    },
  ];

  return (
    <>
      <Frame className="pt-20 pb-10 sm:pt-28">
        <Eyebrow>Terms</Eyebrow>
        <h1 className="mt-6 max-w-3xl text-4xl font-semibold tracking-[-0.035em] text-balance sm:text-6xl">
          The short version: enjoy it, do not rely on it.
        </h1>
        <p className="text-muted-foreground mt-6 max-w-xl text-lg text-pretty">
          Plain terms for a personal site with experiments on it. The{" "}
          <Link className="text-foreground underline-offset-4 hover:underline" href="/legal/">
            legal page
          </Link>{" "}
          has the company details and what happens to your data.
        </p>
      </Frame>

      <Frame className="pb-24 sm:pb-32">
        <dl className="max-w-3xl">
          {sections.map((s, i) => (
            <div
              key={s.title}
              className="grid gap-3 border-t py-8 sm:grid-cols-[3rem_minmax(0,1fr)] sm:gap-6"
            >
              <dt className="text-muted-foreground/50 font-mono text-[11px] tabular-nums">
                {String(i + 1).padStart(2, "0")}
              </dt>
              <dd>
                <h2 className="text-lg font-medium tracking-tight">{s.title}</h2>
                <div className="text-muted-foreground mt-3 space-y-3 text-sm text-pretty">
                  {s.body.map((p) => (
                    <p key={p.slice(0, 24)}>{p}</p>
                  ))}
                </div>
              </dd>
            </div>
          ))}
        </dl>
      </Frame>
    </>
  );
}
