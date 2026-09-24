"use client";

import { usePathname } from "next/navigation";

import { Frame } from "@/components/kit";
import { MeshFabric } from "@/components/mesh-fabric";
import { cn } from "@/lib/utils";

const fade = (to: "top" | "bottom") =>
  `linear-gradient(to ${to}, black 0%, black 30%, rgba(0, 0, 0, 0.55) 62%, transparent 97%)`;

/**
 * The fabric behind the site footer, on every page: hung from the bottom of
 * the page and fading upward. On the home page it is the end of the story,
 * the reply leaving by the last strip, so there is a small step under the
 * strip and the footer's own top rule goes, leaving the strip's bottom edge
 * as the one line; elsewhere the footer keeps its rule and the fabric simply
 * sits behind it.
 */
export function Closing({ children }: { children: React.ReactNode }) {
  const home = usePathname() === "/";
  return (
    <div className={cn("relative", home && "[&>footer]:border-t-0")}>
      {home ? <div aria-hidden className="h-8 sm:h-12" /> : null}
      {/* Taller than the footer on purpose: the fabric rises behind whatever
          ends the page, clipped sideways only, and fades out on its way up. */}
      <div
        aria-hidden
        className="pointer-events-none absolute inset-x-0 bottom-0 -z-10 h-[45rem] overflow-x-clip"
        style={{ maskImage: fade("top"), WebkitMaskImage: fade("top") }}
      >
        <Frame className="relative h-full">
          <MeshFabric anchor="bottom" />
        </Frame>
      </div>
      {children}
    </div>
  );
}

/**
 * The same fabric behind the hero, hung from the top of the page and fading
 * down into the facts strip: where the request comes in from. Its hub sits
 * on the spine's line, two rows down, so the ripples leave from where the
 * strip's corner will pick the path up.
 */
export function Opening() {
  return (
    <div
      aria-hidden
      className="pointer-events-none absolute inset-x-0 top-0 -z-10 h-[45rem] overflow-hidden"
      style={{ maskImage: fade("bottom"), WebkitMaskImage: fade("bottom") }}
    >
      <Frame className="relative h-full">
        <MeshFabric anchor="top" />
      </Frame>
    </div>
  );
}
