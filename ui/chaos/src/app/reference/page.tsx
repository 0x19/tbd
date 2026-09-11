"use client";

import { useRouter } from "next/navigation";
import { useEffect } from "react";

import { Skeleton } from "@/components/ui/skeleton";

/** The reference page moved into the knowledge base; old links land on the scenario file reference. */
export default function ReferenceRedirect() {
  const router = useRouter();
  useEffect(() => {
    router.replace("/kb/view/?doc=docs%2Fchaos%2Fscenarios");
  }, [router]);
  return <Skeleton className="h-40" />;
}
