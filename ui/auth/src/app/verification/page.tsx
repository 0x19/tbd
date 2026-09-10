"use client";

import { Verification } from "@ory/elements-react/theme";

import { AuthShell, Loading } from "@/components/auth-shell";
import { useFlow } from "@/components/flow";
import { oryComponents } from "@/components/ory/components";
import { oryConfig } from "@/lib/ory-config";

export default function VerificationPage() {
  const flow = useFlow("verification");
  if (!flow) return <Loading />;
  return (
    <AuthShell>
      <Verification flow={flow} config={oryConfig()} components={oryComponents} />
    </AuthShell>
  );
}
