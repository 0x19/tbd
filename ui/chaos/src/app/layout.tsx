import type { Metadata } from "next";
import { Inter, JetBrains_Mono } from "next/font/google";
import "./globals.css";
import { Providers } from "@/components/shell/providers";
import { AppSidebar } from "@/components/shell/app-sidebar";
import { SiteHeader } from "@/components/shell/site-header";
import { CommandMenu } from "@/components/shell/command-menu";
import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar";
import { Toaster } from "@/components/ui/sonner";

// The Admin Kit ships with Inter; the mono face is used for ids, addresses
// and TOML.
const inter = Inter({ variable: "--font-inter", subsets: ["latin"] });
const mono = JetBrains_Mono({ variable: "--font-jetbrains-mono", subsets: ["latin"] });

export const metadata: Metadata = {
  title: { default: "Chaos Admin", template: "%s · Chaos Admin" },
  description: "Validate, load-test and fault-test the stack.",
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html
      lang="en"
      suppressHydrationWarning
      className={`${inter.variable} ${mono.variable} h-full antialiased`}
    >
      <body className="min-h-full">
        <Providers>
          <SidebarProvider>
            <AppSidebar />
            <SidebarInset>
              <SiteHeader />
              <div className="flex flex-1 flex-col gap-6 px-4 py-6 md:px-6">{children}</div>
            </SidebarInset>
          </SidebarProvider>
          <CommandMenu />
          <Toaster richColors />
        </Providers>
      </body>
    </html>
  );
}
