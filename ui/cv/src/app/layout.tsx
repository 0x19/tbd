import "./globals.css";

import type { Metadata } from "next";
import { Geist_Mono, Inter } from "next/font/google";

import { Header } from "@/components/layout/header";
import { Toaster } from "@/components/ui/sonner";
import { cn } from "@/lib/utils";

import { Providers } from "./providers";

// The kit's faces: Inter for text, Geist Mono for ids and addresses. The
// variables are the ones globals.css maps to --font-sans / --font-mono.
const inter = Inter({ subsets: ["latin"], variable: "--font-sans" });
const geistMono = Geist_Mono({ subsets: ["latin"], variable: "--font-mono" });

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
            <Header />
            <main className="mx-auto flex w-full max-w-3xl flex-1 flex-col gap-6 px-4 py-8 sm:px-6 sm:py-12">
              {children}
            </main>
          </div>
        </Providers>
        <Toaster richColors />
      </body>
    </html>
  );
}
