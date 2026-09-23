"use client";

import Link from "next/link";
import { useEffect } from "react";

import { Frame } from "@/components/kit";

/**
 * Break it is the lab's first demo now (`/lab/break-it/`): it was never a
 * toy, it is four real services under load with measured margins. This route
 * stays for links that still point here; the browser is sent on at once, and
 * the sitemap does not list it.
 */
export default function BreakItMovedPage() {
  useEffect(() => {
    window.location.replace("/lab/break-it/");
  }, []);
  return (
    <Frame className="py-20">
      <p className="text-muted-foreground">
        Break it has moved to the{" "}
        <Link href="/lab/break-it/" className="text-foreground underline underline-offset-4">
          lab
        </Link>
        .
      </p>
    </Frame>
  );
}
