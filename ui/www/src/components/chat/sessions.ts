"use client";

import { type ChatTurn, type CodeTurn, newId } from "@/components/chat/turn";

/**
 * The workbench's sessions live in this browser's `localStorage` under one key;
 * the dock hands a conversation over by putting it first in that list, which is
 * the one the workbench opens on arrival. Nothing here leaves the browser.
 */
export const WORKBENCH_STORE = "inorbit.workbench.sessions";
export const WORKBENCH_HREF = "/lab/llm/workbench/";
/** The most sessions kept; the oldest drop. */
export const MAX_SESSIONS = 30;

/** Put a conversation first in the workbench's sessions; false when the store is full or blocked. */
export function handOver(title: string, agent: string, turns: (ChatTurn | CodeTurn)[]): boolean {
  try {
    const raw = window.localStorage.getItem(WORKBENCH_STORE);
    const all = raw ? (JSON.parse(raw) as unknown[]) : [];
    const session = { id: newId(), title: title.slice(0, 60), createdAt: Date.now(), agent, turns };
    window.localStorage.setItem(WORKBENCH_STORE, JSON.stringify([session, ...all].slice(0, MAX_SESSIONS)));
    return true;
  } catch {
    return false;
  }
}
