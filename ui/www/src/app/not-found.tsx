"use client";

import Link from "next/link";

import { Eyebrow, Frame } from "@/components/kit";
import { Button } from "@/components/ui/button";
import { useT } from "@/lib/i18n";

export default function NotFound() {
  const t = useT();
  return (
    <Frame className="pt-24 pb-32 sm:pt-32">
      <Eyebrow>404</Eyebrow>
      <h1 className="mt-6 max-w-2xl text-4xl font-semibold tracking-[-0.035em] text-balance sm:text-6xl">
        {t("common.404.title")}
      </h1>
      <Button className="mt-10" asChild>
        <Link href="/">{t("common.404.back")}</Link>
      </Button>
    </Frame>
  );
}
