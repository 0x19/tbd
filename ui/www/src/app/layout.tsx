import "./globals.css";
import "./site.css";

import type { Metadata } from "next";
import { Geist_Mono, Inter } from "next/font/google";

import { SiteFooter } from "@/components/site-footer";
import { SiteHeader } from "@/components/site-header";
import { SiteNotice } from "@/components/site-notice";
import { StructuredData } from "@/components/structured-data";
import { company, indexable, site, url } from "@/data/site";
import { cn } from "@/lib/utils";

import { Providers } from "./providers";

// The same faces as the rest of the surfaces: Inter for text, Geist Mono for
// identifiers.
const inter = Inter({ subsets: ["latin"], variable: "--font-sans" });
const geistMono = Geist_Mono({ subsets: ["latin"], variable: "--font-mono" });

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
