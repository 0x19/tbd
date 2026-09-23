"use client";

import Link from "next/link";
import { useEffect } from "react";

import { Frame } from "@/components/kit";

/**
 * The CV lives on the about page now. This route stays for links that
 * still point here and for the PDF beside it (`/cv/nevio-vesic.pdf`);
 * the browser is sent on at once, and the sitemap does not list it.
 */
export default function CvMovedPage() {
  useEffect(() => {
    window.location.replace("/about/");
  }, []);
  return (
    <Frame className="py-20">
      <p className="text-muted-foreground">
        The CV is now on the{" "}
        <Link href="/about/" className="text-foreground underline underline-offset-4">
          about page
        </Link>
        .
      </p>
    </Frame>
  );
}
