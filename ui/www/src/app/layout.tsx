import "./globals.css";
import "./site.css";

import type { Metadata } from "next";
import localFont from "next/font/local";

import { SiteFooter } from "@/components/site-footer";
import { SiteHeader } from "@/components/site-header";
import { SiteNotice } from "@/components/site-notice";
import { StructuredData } from "@/components/structured-data";
import { company, indexable, site, url } from "@/data/site";
import { cn } from "@/lib/utils";

import { Providers } from "./providers";

// The same faces as the rest of the surfaces: Inter for text, Geist Mono for
// identifiers -- bundled from `src/fonts/` (the files the mobile app and the
// invoice PDF ship), so a build needs no network and the page loads nothing
// from a third party.
const inter = localFont({
  src: [
    { path: "../fonts/Inter-Regular.otf", weight: "400", style: "normal" },
    { path: "../fonts/Inter-Medium.otf", weight: "500", style: "normal" },
    { path: "../fonts/Inter-SemiBold.otf", weight: "600", style: "normal" },
    { path: "../fonts/Inter-Bold.otf", weight: "700", style: "normal" },
  ],
  variable: "--font-sans",
  display: "swap",
});
const geistMono = localFont({
  src: [
    { path: "../fonts/GeistMono-Regular.ttf", weight: "400", style: "normal" },
    { path: "../fonts/GeistMono-Medium.ttf", weight: "500", style: "normal" },
  ],
  variable: "--font-mono",
  display: "swap",
});

export const metadata: Metadata = {
  metadataBase: new URL(url),
  title: { default: `${company.name} · ${company.tagline}`, template: `%s · ${company.name}` },
  description: site.description,
  alternates: { canonical: "/" },
  openGraph: {
    type: "website",
    siteName: company.name,
    title: `${company.name} · ${company.tagline}`,
    description: site.description,
    url: "/",
  },
  // Only the real domain is indexed; every other build asks robots to stay out.
  robots: indexable ? { index: true, follow: true } : { index: false, follow: false },
};

export default function RootLayout({ children }: Readonly<{ children: React.ReactNode }>) {
  return (
    <html lang="en" suppressHydrationWarning className={cn("font-sans", inter.variable, geistMono.variable)}>
      <body className="antialiased">
        <StructuredData />
        <Providers>
          <div className="flex min-h-dvh flex-col">
            <SiteNotice />
            <SiteHeader />
            <main className="flex-1">{children}</main>
            <SiteFooter />
          </div>
        </Providers>
      </body>
    </html>
  );
}
