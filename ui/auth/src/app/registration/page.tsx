"use client";

import { Registration } from "@ory/elements-react/theme";

import { AuthShell, Loading } from "@/components/auth-shell";
import { useFlow } from "@/components/flow";
import { oryComponents } from "@/components/ory/components";
import { oryConfig } from "@/lib/ory-config";

export default function RegistrationPage() {
  const flow = useFlow("registration");
  if (!flow) return <Loading />;
  return (
    <AuthShell>
      <Registration flow={flow} config={oryConfig()} components={oryComponents} />
    </AuthShell>
  );
}
