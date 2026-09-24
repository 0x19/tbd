import "./globals.css";
import "./site.css";

import type { Metadata } from "next";
import localFont from "next/font/local";

import { SiteFooter } from "@/components/site-footer";
import { SiteHeader } from "@/components/site-header";
import { Toaster } from "@/components/ui/sonner";
import { cn } from "@/lib/utils";

import { Providers } from "./providers";

// The public site's faces, bundled the same way (`ui/www/src/fonts`): a build
// needs no network and the page loads nothing from a third party.
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
  title: { default: "The full CV · Nevio Vesic", template: "%s · Nevio Vesic" },
  description: "The full CV, for people the owner approved.",
  robots: { index: false, follow: false },
};

export default function RootLayout({ children }: Readonly<{ children: React.ReactNode }>) {
  return (
    <html lang="en" suppressHydrationWarning className={cn("font-sans", inter.variable, geistMono.variable)}>
      <body className="antialiased">
        <Providers>
          <div className="flex min-h-dvh flex-col">
            <SiteHeader />
            <main className="flex-1">{children}</main>
            <SiteFooter />
          </div>
        </Providers>
        <Toaster richColors />
      </body>
    </html>
  );
}
