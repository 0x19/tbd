"use client";

import { useEffect, useState } from "react";

import type { LabDoc, LabEntry } from "@/lib/lab";
import { useMe } from "@/lib/me";

/**
 * The lab's drafts (`public: false` documents), for admins only.
 *
 * `tool/lab-data.ts` renders them into `public/lab-private/drafts.json`, never
 * into `src/generated/`, so nothing private is compiled into the page's scripts.
 * Envoy serves `/lab-private/` to the admin role alone, before and after the lab
 * is published (`devops/envoy/envoy.yaml`, `www.*` host), and Caddy marks it
 * `private, no-store` so no cache in front ever keeps a copy. The file is only
 * asked for once `/v1/me` says the visitor is an admin; for everyone else, and on
 * any error, there are no drafts.
 */
export type Drafts = { entries: LabEntry[]; docs: Record<string, LabDoc> };

const NONE: Drafts = { entries: [], docs: {} };
let cached: Promise<Drafts> | null = null;

function fetchDrafts(): Promise<Drafts> {
  cached ??= fetch("/lab-private/drafts.json", {
    credentials: "include",
    cache: "no-store",
    headers: { accept: "application/json" },
  })
    .then(async (res) => {
      if (!res.ok) return NONE;
      const body = (await res.json()) as Partial<Drafts>;
      return Array.isArray(body.entries) && body.docs && typeof body.docs === "object"
        ? (body as Drafts)
        : NONE;
    })
    .catch(() => NONE);
  return cached;
}

/** The drafts an admin may read, optionally only one lab's; empty for everyone else. */
export function useDrafts(lab?: string): Drafts & { ready: boolean } {
  const me = useMe();
  const admin = me?.role === "admin";
  const [drafts, setDrafts] = useState<Drafts>(NONE);
  const [ready, setReady] = useState(false);
  useEffect(() => {
    if (!admin) return;
    let live = true;
    void fetchDrafts().then((d) => {
      if (!live) return;
      setDrafts(d);
      setReady(true);
    });
    return () => {
      live = false;
    };
  }, [admin]);
  const entries = lab ? drafts.entries.filter((e) => e.lab === lab) : drafts.entries;
  return { entries, docs: drafts.docs, ready };
}
