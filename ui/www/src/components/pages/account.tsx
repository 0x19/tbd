"use client";

import { Frame, SectionHead, Tag } from "@/components/kit";
import { cv, lab } from "@/data/site";
import { useT } from "@/lib/i18n";
import { useMe } from "@/lib/me";

/**
 * Who is signed in, on this site. The gateway gates `/account/` behind the
 * sign-in, so reaching this page is how signing in starts (the header's
 * "Sign in" points here); once here, the page shows the person, the way out
 * (`/oauth2/signout`, which ends the session everywhere) and, for an admin, the
 * lab. `app/account/page.tsx` carries the metadata.
 */
export function AccountContent() {
  const t = useT();
  const me = useMe();
  const admin = me?.role === "admin";
  const label = me?.name || me?.email || me?.subject;
  return (
    <>
      <Frame className="pt-20 pb-10 sm:pt-28">
        <p className="text-muted-foreground font-mono text-[11px] tracking-[0.18em] uppercase">
          {t("account.eyebrow")}
        </p>
        <h1 className="mt-4 max-w-3xl text-4xl font-semibold tracking-[-0.035em] text-balance sm:text-6xl">
          {me ? t("account.title", { name: label ?? "" }) : t("account.pending")}
        </h1>
        {me?.email && me.name ? (
          <p className="text-muted-foreground mt-6 max-w-xl text-lg">{me.email}</p>
        ) : null}
        {me?.role ? (
          <p className="mt-4">
            <Tag>{me.role}</Tag>
          </p>
        ) : null}
      </Frame>
      <Frame className="pb-16">
        <SectionHead n="01" label={t("account.where.label")} />
        <ul className="mt-6 max-w-xl space-y-3 text-sm">
          {admin ? (
            <li>
              <a href={lab.href} className="text-foreground underline underline-offset-4">
                {t("account.lab")}
              </a>{" "}
              <span className="text-muted-foreground">{t("account.lab_note")}</span>
            </li>
          ) : null}
          <li>
            <a href={cv.fullUrl} className="text-foreground underline underline-offset-4">
              {t("account.cv")} ↗
            </a>{" "}
            <span className="text-muted-foreground">{t("account.cv_note")}</span>
          </li>
        </ul>
      </Frame>
      <Frame className="pb-24 sm:pb-32">
        <SectionHead n="02" label={t("account.out.label")} />
        <p className="text-muted-foreground mt-6 max-w-xl text-pretty">{t("account.out.text")}</p>
        <p className="mt-4">
          <a
            href="/oauth2/signout"
            className="text-foreground font-mono text-[11px] tracking-[0.14em] uppercase underline underline-offset-4"
          >
            {t("common.sign_out")}
          </a>
        </p>
      </Frame>
    </>
  );
}
