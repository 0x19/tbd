"use client";

import Link from "next/link";
import { useEffect } from "react";

import { Frame } from "@/components/kit";

/**
 * Break it lived here while it was the lab's first demo. It is paused now and
 * listed under Play only; this route stays for links that still point here,
 * sends the browser on at once, and is not in the sitemap.
 */
export default function BreakItMovedPage() {
  useEffect(() => {
    window.location.replace("/playgrounds/break-it/");
  }, []);
  return (
    <Frame className="py-20">
      <p className="text-muted-foreground">
        Break it is{" "}
        <Link href="/playgrounds/break-it/" className="text-foreground underline underline-offset-4">
          under Play
        </Link>
        .
      </p>
    </Frame>
  );
}
