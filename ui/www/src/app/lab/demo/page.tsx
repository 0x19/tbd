"use client";

import Link from "next/link";
import { useEffect } from "react";

import { Frame } from "@/components/kit";

/**
 * The demo became the model lab's workbench (`/lab/llm/workbench/`). This route
 * stays for links that still point here, sends the browser on at once, and is
 * not in the sitemap.
 */
export default function LabDemoMovedPage() {
  useEffect(() => {
    window.location.replace("/lab/llm/workbench/");
  }, []);
  return (
    <Frame className="py-20">
      <p className="text-muted-foreground">
        The demo is now the{" "}
        <Link href="/lab/llm/workbench/" className="text-foreground underline underline-offset-4">
          workbench
        </Link>
        .
      </p>
    </Frame>
  );
}
