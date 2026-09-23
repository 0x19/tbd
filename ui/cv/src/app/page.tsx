"use client";

import { useState } from "react";
import { toast } from "sonner";

import { useMe } from "@/app/providers";
import { StatusBadge } from "@/components/status-badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { Textarea } from "@/components/ui/textarea";
import { api, pdfUrl } from "@/lib/api/client";
import { describe, useFetch, when } from "@/lib/api/hooks";
import type { AccessState } from "@/lib/api/schema";
import { useT } from "@/lib/i18n";

const PUBLIC_CV = "https://inorbit.hr/cv/";

/** A visitor's standing, the request, and the download once approved. */
export default function AccessPage() {
  const t = useT();
  const me = useMe();
  const access = useFetch(() => api.access(), 30_000);
  const [note, setNote] = useState("");
  const [busy, setBusy] = useState<"ask" | "download" | null>(null);
  const state = access.data?.state ?? null;

  async function ask() {
    setBusy("ask");
    try {
      const r = await api.request(note);
      access.setData(r);
    } catch (e) {
      toast.error(t("cv.error", { error: describe(e) }));
    } finally {
      setBusy(null);
    }
  }

  async function download() {
    setBusy("download");
    try {
      const d = await api.download();
      const a = document.createElement("a");
      a.href = pdfUrl(d.pdf);
      a.download = d.filename || "cv.pdf";
      a.click();
      toast.success(t("cv.downloaded"));
      access.reload();
    } catch (e) {
      toast.error(t("cv.error", { error: describe(e) }));
    } finally {
      setBusy(null);
    }
  }

  return (
    <>
      <p className="text-muted-foreground max-w-2xl text-pretty">{t("cv.intro")}</p>

      {me.data ? (
        <p className="text-sm">
          {t("cv.you")}{" "}
          <span className="font-medium">{me.data.name || me.data.email || me.data.subject}</span>
          {me.data.email && me.data.name ? (
            <span className="text-muted-foreground"> · {me.data.email}</span>
          ) : null}
        </p>
      ) : null}
      {me.data && !me.data.email ? <p className="text-destructive text-sm">{t("cv.no_email")}</p> : null}

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            {t("cv.status")}
            {state ? <StatusBadge status={state.status} /> : null}
          </CardTitle>
        </CardHeader>
        <CardContent className="grid gap-4">
          {access.loading && !state ? <Skeleton className="h-6 w-2/3" /> : null}
          {access.error ? <p className="text-destructive text-sm">{access.error}</p> : null}
          {state ? <Standing state={state} /> : null}

          {state && state.status !== "approved" ? (
            <div className="grid gap-2">
              <label className="text-sm font-medium" htmlFor="note">
                {t("cv.note")}
              </label>
              <Textarea
                id="note"
                value={note}
                onChange={(e) => setNote(e.target.value)}
                placeholder={t("cv.note_hint")}
                rows={3}
                maxLength={2000}
              />
              <div>
                <Button onClick={ask} disabled={busy !== null || !me.data?.email}>
                  {busy === "ask"
                    ? t("cv.asking")
                    : state.status === "none"
                      ? t("cv.ask")
                      : t("cv.ask_again")}
                </Button>
              </div>
            </div>
          ) : null}

          {state?.status === "approved" ? (
            <div>
              <Button onClick={download} disabled={busy !== null}>
                {busy === "download" ? t("cv.downloading") : t("cv.download")}
              </Button>
            </div>
          ) : null}
        </CardContent>
      </Card>

      <p className="text-muted-foreground text-sm">
        {t("cv.public")}{" "}
        <a href={PUBLIC_CV} className="text-foreground underline underline-offset-4">
          inorbit.hr/cv
        </a>
      </p>
    </>
  );
}

function Standing({ state }: { state: AccessState }) {
  const t = useT();
  switch (state.status) {
    case "none":
      return <p className="text-sm">{t("cv.none")}</p>;
    case "requested":
      return (
        <div className="grid gap-1 text-sm">
          <p>{t("cv.requested")}</p>
          <p className="text-muted-foreground">
            {t("cv.requested_at", { when: when(state.requested_at) })}{" "}
            {state.notified_at
              ? t("cv.notified", { when: when(state.notified_at) })
              : state.notifications
                ? t("cv.not_notified")
                : t("cv.notifications_off")}
          </p>
          {state.note ? <p className="text-muted-foreground italic">&ldquo;{state.note}&rdquo;</p> : null}
        </div>
      );
    case "approved":
      return <p className="text-sm">{t("cv.approved", { when: when(state.decided_at) })}</p>;
    case "refused":
      return <p className="text-sm">{t("cv.refused", { when: when(state.decided_at) })}</p>;
    case "revoked":
      return <p className="text-sm">{t("cv.revoked", { when: when(state.decided_at) })}</p>;
    default:
      return <p className="text-sm">{state.status}</p>;
  }
}
