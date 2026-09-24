// What the bank pages say about a consent and an account, as pure functions
// of the wire rows and the clock, so the banks page, the accounts page and
// the numbers at the top agree.
import type { Account, Connection } from "@/lib/api/schema";

/** A consent at Erste lasts 180 days; the page warns this far ahead. */
export const RENEW_AHEAD_DAYS = 30;
/** The bank allows this many fetches per account per day. */
export const FETCHES_PER_DAY = 4;
/** Scheduled fetches are this far apart. */
export const FETCH_EVERY_MS = 8 * 3600_000;
/** A fetch older than this is stale on a syncing account. */
export const STALE_AFTER_MS = 9 * 3600_000;

export type ConsentState = "pending" | "active" | "ending" | "ended" | "removed" | "replaced" | "failed";

/** Whole days from now until `iso`, negative when past. */
export function daysUntil(iso: string, now = Date.now()): number {
  return Math.floor((new Date(iso).getTime() - now) / 86_400_000);
}

/** One word for the consent's life, worst first. */
export function consentState(c: Connection, now = Date.now()): ConsentState {
  if (c.status === "pending") return "pending";
  if (c.status === "failed") return "failed";
  if (c.status === "revoked") return c.replaced_by ? "replaced" : "removed";
  if (c.status === "expired") return "ended";
  if (c.valid_until && daysUntil(c.valid_until, now) < 0) return "ended";
  if (c.valid_until && daysUntil(c.valid_until, now) < RENEW_AHEAD_DAYS) return "ending";
  return "active";
}

/** Whether the consent is one the sync loop fetches under. */
export function consentLive(c: Connection, now = Date.now()): boolean {
  const s = consentState(c, now);
  return s === "active" || s === "ending";
}

export type NextFetch =
  | { kind: "off" }
  | { kind: "backing_off"; until: Date }
  | { kind: "budget_spent" }
  | { kind: "due" }
  | { kind: "at"; at: Date }
  | { kind: "consent_ended" };

/** When the scheduler will next fetch this account, or why it will not. */
export function nextFetch(a: Account, live: boolean, now = Date.now()): NextFetch {
  if (!live) return { kind: "consent_ended" };
  if (!a.sync_enabled) return { kind: "off" };
  const backoff = a.sync_backoff_until ? new Date(a.sync_backoff_until) : null;
  if (backoff && backoff.getTime() > now) return { kind: "backing_off", until: backoff };
  if (a.sync_budget_used >= FETCHES_PER_DAY - 1) return { kind: "budget_spent" };
  if (!a.last_synced_at) return { kind: "due" };
  const at = new Date(a.last_synced_at).getTime() + FETCH_EVERY_MS;
  return at <= now ? { kind: "due" } : { kind: "at", at: new Date(at) };
}

/** Fetches a person may still ask for today on this account. */
export function fetchesLeft(a: Account): number {
  return Math.max(0, FETCHES_PER_DAY - a.sync_budget_used);
}

export function isStale(a: Account, now = Date.now()): boolean {
  return a.last_synced_at ? now - new Date(a.last_synced_at).getTime() > STALE_AFTER_MS : true;
}
