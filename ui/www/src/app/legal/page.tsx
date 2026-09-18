import type { Metadata } from "next";
import Link from "next/link";

import { Eyebrow, Frame } from "@/components/kit";
import { company, orDash } from "@/data/site";

export const metadata: Metadata = {
  title: "Legal",
  description: `Company details and privacy notice for ${company.legalName}.`,
  alternates: { canonical: "/legal/" },
};

export default function LegalPage() {
  const rows = [
    { k: "Legal name", v: company.legalName },
    { k: "Represented by", v: company.person },
    { k: "Founded", v: company.founded ? String(company.founded) : "—" },
    { k: "Registered address", v: orDash(company.address) },
    { k: "OIB", v: company.oib },
    { k: "Court register (MBS)", v: orDash(company.registration) },
    { k: "Email", v: company.email },
  ];

  return (
    <>
      <Frame className="pt-20 pb-10 sm:pt-28">
        <Eyebrow>Legal</Eyebrow>
        <h1 className="mt-6 max-w-3xl text-4xl font-semibold tracking-[-0.035em] text-balance sm:text-6xl">
          Who you are dealing with.
        </h1>
      </Frame>

      <Frame className="pb-16">
        <Eyebrow className="border-t pt-6">Company details</Eyebrow>
        <dl className="mt-6 max-w-2xl">
          {rows.map((r) => (
            <div key={r.k} className="flex items-baseline justify-between gap-6 border-b py-4 text-sm">
              <dt className="text-muted-foreground">{r.k}</dt>
              <dd className="text-right font-medium">{r.v}</dd>
            </div>
          ))}
        </dl>
      </Frame>

      <Frame className="pb-24 sm:pb-32">
        <Eyebrow className="border-t pt-6">Privacy</Eyebrow>
        <div className="text-muted-foreground mt-6 max-w-2xl space-y-4 text-sm text-pretty">
          <p>
            This site sets no cookies, runs no analytics and embeds nothing from a third party. Pages are
            static files; nothing you do here is recorded beyond the web server&apos;s own access log, which
            keeps the request line, the response status and the time, and is rotated.
          </p>
          <p>
            If you email me, I keep that email and my reply for as long as the conversation is useful, and I
            do not pass it to anyone. To have it deleted, write to{" "}
            <a
              className="text-foreground underline-offset-4 hover:underline"
              href={`mailto:${company.email}`}
            >
              {company.email}
            </a>
            .
          </p>
          <p>
            A playground may need to process what you type into it to answer you. Where one does, it says so
            on its own page, and the rule is the same: nothing is kept that does not have to be. The{" "}
            <Link className="text-foreground underline-offset-4 hover:underline" href="/terms/">
              terms
            </Link>{" "}
            cover what you can expect from them.
          </p>
        </div>
      </Frame>
    </>
  );
}
