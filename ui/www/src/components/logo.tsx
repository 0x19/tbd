import { cn } from "@/lib/utils";

/**
 * The InOrbit monogram: the tittle of the i is the body, the O is the orbit.
 * Inline rather than an <img> so it takes `currentColor` and needs no request;
 * the same geometry ships as files under /brand for documents and invoices.
 */
export function Logo({ className, size = 20 }: { className?: string; size?: number }) {
  return (
    <svg
      viewBox="0 0 32 32"
      width={size}
      height={size}
      fill="none"
      role="img"
      aria-label="InOrbit"
      className={cn("shrink-0", className)}
    >
      <circle cx="18.8" cy="17.3" r="8.7" stroke="currentColor" strokeWidth="2.6" />
      <rect x="3.7" y="9.8" width="2.8" height="17.5" rx="1.4" fill="currentColor" />
      <circle cx="5.1" cy="6.8" r="1.9" fill="currentColor" />
    </svg>
  );
}
