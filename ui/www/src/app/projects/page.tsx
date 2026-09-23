"use client";

import Link from "next/link";
import { useEffect } from "react";

import { Frame } from "@/components/kit";

/**
 * The projects are the work page now (`/work/`): finished things with a date
 * and a link, next to the lab, which is what is in progress. This route stays
 * for links that still point here; the browser is sent on at once, and the
 * sitemap does not list it.
 */
export default function ProjectsMovedPage() {
  useEffect(() => {
    window.location.replace("/work/");
  }, []);
  return (
    <Frame className="py-20">
      <p className="text-muted-foreground">
        The projects are on the{" "}
        <Link href="/work/" className="text-foreground underline underline-offset-4">
          work page
        </Link>
        .
      </p>
    </Frame>
  );
}
