"use client";

import type { FlowError } from "@ory/client-fetch";
import { useSearchParams } from "next/navigation";
import { useEffect, useState } from "react";

import { AuthShell, Loading } from "@/components/auth-shell";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card";
import { frontend } from "@/lib/client";

/** Kratos and Hydra both land here: Kratos with ?id=, Hydra with ?error=&error_description=. */
export default function ErrorPage() {
  const params = useSearchParams();
  const id = params.get("id");
  const [error, setError] = useState<FlowError | null>(null);

  useEffect(() => {
    if (!id) return;
    frontend
      .getFlowError({ id })
      .then(setError)
      .catch(() => setError({ id, error: { message: "unknown error" } }));
  }, [id]);

  if (id && !error) return <Loading />;
  const details = error?.error as { message?: string; reason?: string } | undefined;
  const title = id
    ? (details?.message ?? "Something went wrong")
    : (params.get("error") ?? "Something went wrong");
  const description = id
    ? (details?.reason ?? "The request could not be completed.")
    : (params.get("error_description") ?? "The request could not be completed.");
  return (
    <AuthShell>
      <Card className="w-full max-w-sm">
        <CardHeader>
          <CardTitle className="text-xl">{title}</CardTitle>
          <CardDescription>{description}</CardDescription>
        </CardHeader>
        {id ? (
          <CardContent>
            <p className="text-muted-foreground font-mono text-xs">error {id}</p>
          </CardContent>
        ) : null}
        <CardFooter>
          <Button asChild variant="outline">
            <a href="/login">Back to sign in</a>
          </Button>
        </CardFooter>
      </Card>
    </AuthShell>
  );
}
