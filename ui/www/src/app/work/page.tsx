"use client";

import Link from "next/link";
import { useEffect } from "react";

import { Frame } from "@/components/kit";

/**
 * The open-source libraries were the work page (`/work/`) until the name said
 * something else to readers. This route stays for links that still point here; the browser is sent
 * on to `/open-source/` at once, and the sitemap does not list it.
 */
export default function WorkMovedPage() {
  useEffect(() => {
    window.location.replace("/open-source/");
  }, []);
  return (
    <Frame className="py-20">
      <p className="text-muted-foreground">
        The libraries are on the{" "}
        <Link href="/open-source/" className="text-foreground underline underline-offset-4">
          open source page
        </Link>
        .
      </p>
    </Frame>
  );
}
