import "./globals.css";
import "./editor.css";

import type { Metadata } from "next";
import { Geist_Mono, Inter } from "next/font/google";

import { AppSidebar } from "@/components/layout/app-sidebar";
import { Header } from "@/components/layout/header";
import { SubHeader } from "@/components/layout/sub-header";
import { SidebarProvider } from "@/components/ui/sidebar";
import { Toaster } from "@/components/ui/sonner";
import { site } from "@/data/site";
import { cn } from "@/lib/utils";

import { Providers } from "./providers";

// The kit's faces: Inter for text, Geist Mono for ids, addresses and TOML.
// The variables are the ones globals.css maps to --font-sans / --font-mono.
const inter = Inter({ subsets: ["latin"], variable: "--font-sans" });
const geistMono = Geist_Mono({ subsets: ["latin"], variable: "--font-mono" });

export const metadata: Metadata = {
  title: { default: site.title, template: `%s · ${site.title}` },
  description: site.description,
};

export default function RootLayout({ children }: Readonly<{ children: React.ReactNode }>) {
  return (
    <html lang="en" suppressHydrationWarning className={cn("font-sans", inter.variable, geistMono.variable)}>
      <body className="group/body antialiased">
        <Providers>
          <div className="border-grid flex flex-1 flex-col">
            <SidebarProvider defaultOpen>
              <AppSidebar />
              <div
                id="content"
                className={cn(
                  "flex h-full w-full min-w-0 flex-col",
                  "has-[div[data-layout=fixed]]:h-svh",
                  "group-data-[scroll-locked=1]/body:h-full",
                  "has-[data-layout=fixed]:group-data-[scroll-locked=1]/body:h-svh",
                )}
              >
                <Header title={site.title} />
                <SubHeader />
                <main id="main-content" className="flex flex-1 flex-col gap-6 p-4 sm:p-6">
                  {children}
                </main>
              </div>
            </SidebarProvider>
          </div>
        </Providers>
        <Toaster richColors />
      </body>
    </html>
  );
}
