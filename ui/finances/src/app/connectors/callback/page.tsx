"use client";

// Where a provider sends the browser after authorization. Signed in behind
// Envoy, the page reads `state` and `code` from its own URL, POSTs them to
// CompleteConnector (the code in the body), then scrubs the URL.
import Link from "next/link";
import { useRouter, useSearchParams } from "next/navigation";
import { Suspense, useEffect, useState } from "react";
import { toast } from "sonner";

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

function Callback() {
  const t = useT();
  const params = useSearchParams();
  const router = useRouter();
  const state = params.get("state") ?? "";
  const code = params.get("code") ?? "";
  const providerError = params.get("error") ?? "";
  const [status, setStatus] = useState<"working" | "done" | "failed" | "idle">("working");
  const [detail, setDetail] = useState("");

  useEffect(() => {
    if (providerError) {
      setStatus("failed");
      setDetail("");
      return;
    }
    // No parameters: this is a revisit (back button, a session refresh
    // landing on the scrubbed URL), not a failed link. Say so, neutrally.
    if (!state || !code) {
      setStatus("idle");
      return;
    }
    let cancelled = false;
    api
      .completeConnector(state, code)
      .then((r) => {
        if (cancelled) return;
        setDetail(r.connector?.label ?? "");
        setStatus("done");
        // Leave this URL: the code is spent, and a reload here must not
        // look like a failure. The list shows the new link.
        toast.success(
          t("connectors.callback.toast_linked", {
            name: r.connector?.label ?? t("connectors.callback.mailbox"),
          }),
        );
        router.replace("/connectors/");
      })
      .catch((e: unknown) => {
        if (cancelled) return;
        setStatus("failed");
        setDetail(describe(e));
      });
    return () => {
      cancelled = true;
    };
  }, [state, code, providerError, router, t]);

  return (
    <>
      <PageTitle title={t("nav.authorization")} />
      <Card className="max-w-xl">
        <CardHeader>
          <CardTitle>
            {status === "working"
              ? t("connectors.callback.finishing")
              : status === "done"
                ? t("connectors.callback.linked")
                : t("connectors.callback.not_linked")}
          </CardTitle>
          <CardDescription>
            {status === "working"
              ? t("connectors.callback.exchanging")
              : status === "done"
                ? t("connectors.callback.connected", { name: detail })
                : status === "failed" && providerError
                  ? t("connectors.callback.refused", { error: providerError })
                  : detail}
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Button asChild variant={status === "done" ? "default" : "outline"}>
            <Link href="/connectors/">{t("nav.connectors")}</Link>
          </Button>
        </CardContent>
      </Card>
    </>
  );
}
