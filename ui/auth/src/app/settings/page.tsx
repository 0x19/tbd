"use client";

import { Settings } from "@ory/elements-react/theme";

import { AuthShell, Loading } from "@/components/auth-shell";
import { useFlow } from "@/components/flow";
import { oryComponents } from "@/components/ory/components";
import { oryConfig } from "@/lib/ory-config";

export default function SettingsPage() {
  const flow = useFlow("settings");
  if (!flow) return <Loading />;
  return (
    <AuthShell>
      <Settings flow={flow} config={oryConfig()} components={oryComponents} />
    </AuthShell>
  );
}
