import { IconBrandGithub, IconBrandLinkedin, IconBrandX } from "@tabler/icons-react";
import { MapPin } from "lucide-react";
import type { Metadata } from "next";
import Link from "next/link";

import { Eyebrow, Frame, Reveal } from "@/components/kit";
import { company } from "@/data/site";

export const metadata: Metadata = {
  title: "Contact",
  description: `How to reach ${company.person}.`,
  alternates: { canonical: "/contact/" },
};

export default function ContactPage() {
  const elsewhere = [
    { icon: IconBrandGithub, label: "GitHub", value: "github.com/0x19", href: company.github },
    { icon: IconBrandX, label: "X", value: "@vesicnevio", href: company.x },
    { icon: IconBrandLinkedin, label: "LinkedIn", value: "in/neviovesic", href: company.linkedin },
  ];

  return (
    <>
      <Frame className="pt-20 pb-12 sm:pt-28">
        <Eyebrow>Contact</Eyebrow>
        <h1 className="mt-6 max-w-3xl text-4xl font-semibold tracking-[-0.035em] text-balance sm:text-6xl">
          Say hi.
        </h1>
        <p className="text-muted-foreground mt-6 max-w-xl text-lg text-pretty">
          A question about something here, a bug in one of the libraries, an idea worth a prototype, or work
          that needs doing — all fine. No form, no funnel.
        </p>
      </Frame>

      <Frame className="pb-16">
        <Reveal>
          <div className="border-t pt-10">
            <Eyebrow>Email</Eyebrow>
            <a
              href={`mailto:${company.email}`}
              className="mt-5 block text-3xl font-semibold tracking-[-0.03em] break-all transition-opacity hover:opacity-60 sm:text-6xl"
            >
              {company.email}
            </a>
          </div>
        </Reveal>
      </Frame>

      <Frame className="pb-24 sm:pb-32">
        <div className="grid gap-10 border-t pt-10 sm:grid-cols-2">
          <div>
            <Eyebrow>Elsewhere</Eyebrow>
            <ul className="mt-5 space-y-1">
              {elsewhere.map((e) => (
                <li key={e.label}>
                  <a
                    href={e.href}
                    target="_blank"
                    rel="noreferrer"
                    className="group flex items-center gap-3 py-2"
                  >
                    <e.icon className="text-muted-foreground size-4" />
                    <span className="font-medium">{e.label}</span>
                    <span className="text-muted-foreground font-mono text-xs">{e.value}</span>
                    <span className="text-muted-foreground ml-auto opacity-0 transition-opacity group-hover:opacity-100">
                      ↗
                    </span>
                  </a>
                </li>
              ))}
            </ul>
          </div>
          <div>
            <Eyebrow>Where</Eyebrow>
            <p className="mt-5 flex items-center gap-2 font-medium">
              <MapPin className="text-muted-foreground size-4" />
              {company.city}
            </p>
            <p className="text-muted-foreground mt-2 text-sm text-pretty">
              European hours. {company.legalName} is the company behind the invoices; its details are on the{" "}
              <Link className="text-foreground underline-offset-4 hover:underline" href="/legal/">
                legal page
              </Link>
              .
            </p>
          </div>
        </div>
      </Frame>
    </>
  );
}
