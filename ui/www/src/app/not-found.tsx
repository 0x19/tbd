import Link from "next/link";

import { Eyebrow, Frame } from "@/components/kit";
import { Button } from "@/components/ui/button";

export default function NotFound() {
  return (
    <Frame className="pt-24 pb-32 sm:pt-32">
      <Eyebrow>404</Eyebrow>
      <h1 className="mt-6 max-w-2xl text-4xl font-semibold tracking-[-0.035em] text-balance sm:text-6xl">
        There is nothing at this address.
      </h1>
      <Button className="mt-10" asChild>
        <Link href="/">Back to the start</Link>
      </Button>
    </Frame>
  );
}
