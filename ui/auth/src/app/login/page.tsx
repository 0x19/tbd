"use client";

import { Login } from "@ory/elements-react/theme";

import { AuthShell, Loading } from "@/components/auth-shell";
import { useFlow } from "@/components/flow";
import { oryComponents } from "@/components/ory/components";
import { oryConfig } from "@/lib/ory-config";

export default function LoginPage() {
  const flow = useFlow("login");
  if (!flow) return <Loading />;
  return (
    <AuthShell>
      <Login flow={flow} config={oryConfig()} components={oryComponents} />
    </AuthShell>
  );
}
