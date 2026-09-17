"use client";

// Where the bank sends the browser after authorization. This host is behind
// Envoy's browser login, so the person is signed in; the page reads `state`
// and `code` from its own URL and POSTs them to CompleteConnection -- the code
// in the body, never a query string on our API, never stored, never logged.
import Link from "next/link";
import { useSearchParams } from "next/navigation";
import { Suspense, useEffect, useState } from "react";

import { PageTitle } from "@/components/kit";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { api } from "@/lib/api/client";
import { describe } from "@/lib/api/hooks";

export default function CallbackPage() {
  return (
    <Suspense fallback={null}>
      <Callback />
    </Suspense>
  );
}

function Callback() {
  const params = useSearchParams();
  const state = params.get("state") ?? "";
  const code = params.get("code") ?? "";
  const bankError = params.get("error") ?? "";
  const [status, setStatus] = useState<"working" | "done" | "failed">("working");
  const [detail, setDetail] = useState("");
  const [accounts, setAccounts] = useState(0);

  useEffect(() => {
    if (bankError) {
      setStatus("failed");
      setDetail(`The bank refused: ${bankError}`);
      return;
    }
    if (!state || !code) {
      setStatus("failed");
      setDetail("The redirect carried no state and code; start the link again.");
      return;
    }
    let cancelled = false;
    api
      .completeConnection(state, code)
      .then((r) => {
        if (cancelled) return;
        setAccounts(r.account_ids.length);
        setStatus("done");
        // The code is gone from the bank's side now; take it out of the
        // address bar too, so a bookmark or a screenshot does not carry it.
        window.history.replaceState(null, "", "/connect/callback/");
      })
      .catch((e: unknown) => {
        if (cancelled) return;
        setStatus("failed");
        setDetail(describe(e));
      });
    return () => {
      cancelled = true;
    };
  }, [state, code, bankError]);

  return (
    <>
      <PageTitle title="Bank authorization" />
      <Card className="max-w-xl">
        <CardHeader>
          <CardTitle>
            {status === "working" ? "Finishing the link…" : status === "done" ? "Linked" : "Not linked"}
          </CardTitle>
          <CardDescription>
            {status === "working"
              ? "Exchanging the bank's code for a session. This takes a moment."
              : status === "done"
                ? `${accounts} ${accounts === 1 ? "account is" : "accounts are"} now connected. The first fetch runs within minutes.`
                : detail}
          </CardDescription>
        </CardHeader>
        <CardContent className="flex gap-2">
          <Button asChild variant={status === "done" ? "default" : "outline"}>
            <Link href="/accounts/">Accounts</Link>
          </Button>
          <Button asChild variant="outline">
            <Link href="/connections/">Connections</Link>
          </Button>
        </CardContent>
      </Card>
    </>
  );
}
