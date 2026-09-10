"use client";

import type { Session } from "@ory/client-fetch";
import { useEffect, useState } from "react";

import { AuthShell, Loading } from "@/components/auth-shell";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card";
import { frontend } from "@/lib/client";

/** Where a person lands after signing in: who they are, and what they can do. */
export default function Home() {
  const [session, setSession] = useState<Session | null | undefined>(undefined);

  useEffect(() => {
    frontend
      .toSession()
      .then(setSession)
      .catch(() => setSession(null));
  }, []);

  useEffect(() => {
    if (session === null) window.location.replace("/login");
  }, [session]);

  if (!session) return <Loading />;
  const traits = (session.identity?.traits ?? {}) as { email?: string; name?: string };
  const methods = session.authentication_methods?.map((m) => m.provider ?? m.method).join(", ");
  return (
    <AuthShell>
      <Card className="w-full max-w-sm">
        <CardHeader>
          <CardTitle className="text-xl">Signed in</CardTitle>
          <CardDescription>
            {traits.name ? `${traits.name} · ` : ""}
            {traits.email}
          </CardDescription>
        </CardHeader>
        <CardContent>
          <dl className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1 text-sm">
            <dt className="text-muted-foreground">Session until</dt>
            <dd>{session.expires_at ? new Date(session.expires_at).toLocaleString() : "—"}</dd>
            <dt className="text-muted-foreground">Signed in with</dt>
            <dd className="font-mono text-xs">{methods}</dd>
          </dl>
        </CardContent>
        <CardFooter className="gap-2">
          <Button asChild variant="outline" className="flex-1">
            <a href="/settings">Account settings</a>
          </Button>
          <Button asChild variant="ghost" className="flex-1">
            <a href="/logout">Sign out everywhere</a>
          </Button>
        </CardFooter>
      </Card>
    </AuthShell>
  );
}
