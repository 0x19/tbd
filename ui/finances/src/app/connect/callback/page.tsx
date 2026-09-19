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
import { useT } from "@/lib/i18n";

export default function CallbackPage() {
  return (
    <Suspense fallback={null}>
      <Callback />
    </Suspense>
  );
}

// What went wrong, kept as a key plus its variables so the text follows a
// language change; the API's own message (`describe`) is shown as it comes.
type Failure = { key: string; vars?: Record<string, string> } | { text: string };

function Callback() {
  const t = useT();
  const params = useSearchParams();
  const state = params.get("state") ?? "";
  const code = params.get("code") ?? "";
  const bankError = params.get("error") ?? "";
  const [status, setStatus] = useState<"working" | "done" | "failed">("working");
  const [failure, setFailure] = useState<Failure | null>(null);
  const [accounts, setAccounts] = useState(0);

  useEffect(() => {
    if (bankError) {
      setStatus("failed");
      setFailure({ key: "banking.bank_refused", vars: { error: bankError } });
      return;
    }
    if (!state || !code) {
      setStatus("failed");
      setFailure({ key: "banking.no_state_code" });
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
        setFailure({ text: describe(e) });
      });
    return () => {
      cancelled = true;
    };
  }, [state, code, bankError]);

  const detail = failure === null ? "" : "text" in failure ? failure.text : t(failure.key, failure.vars);

  return (
    <>
      <PageTitle title={t("banking.callback.title")} />
      <Card className="max-w-xl">
        <CardHeader>
          <CardTitle>
            {status === "working"
              ? t("banking.finishing_link")
              : status === "done"
                ? t("banking.linked")
                : t("banking.not_linked")}
          </CardTitle>
          <CardDescription>
            {status === "working"
              ? t("banking.exchanging_code")
              : status === "done"
                ? accounts === 1
                  ? t("banking.one_account_connected")
                  : t("banking.n_accounts_connected", { n: accounts })
                : detail}
          </CardDescription>
        </CardHeader>
        <CardContent className="flex gap-2">
          <Button asChild variant={status === "done" ? "default" : "outline"}>
            <Link href="/accounts/">{t("nav.accounts")}</Link>
          </Button>
          <Button asChild variant="outline">
            <Link href="/connections/">{t("nav.connections")}</Link>
          </Button>
        </CardContent>
      </Card>
    </>
  );
}
