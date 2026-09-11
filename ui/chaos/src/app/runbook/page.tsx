"use client";

import { useRouter } from "next/navigation";
import { useEffect } from "react";

import { Skeleton } from "@/components/ui/skeleton";

/** The runbook moved into the knowledge base (docs/chaos/runbook.md); old links land there. */
export default function RunbookRedirect() {
  const router = useRouter();
  useEffect(() => {
    router.replace("/kb/view/?doc=docs%2Fchaos%2Frunbook");
  }, [router]);
  return <Skeleton className="h-40" />;
}
