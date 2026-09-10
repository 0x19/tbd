"use client";

import { Recovery } from "@ory/elements-react/theme";

import { AuthShell, Loading } from "@/components/auth-shell";
import { useFlow } from "@/components/flow";
import { oryComponents } from "@/components/ory/components";
import { oryConfig } from "@/lib/ory-config";

export default function RecoveryPage() {
  const flow = useFlow("recovery");
  if (!flow) return <Loading />;
  return (
    <AuthShell>
      <Recovery flow={flow} config={oryConfig()} components={oryComponents} />
    </AuthShell>
  );
}
