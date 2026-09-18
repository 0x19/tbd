"use client";

// The receipts pulled for the period, newest first, and how many the reader
// could not make sense of -- no vendor or no amount, and nobody has said
// otherwise -- so they get looked at before the accountant asks.
import { Receipt } from "lucide-react";
import Link from "next/link";

import type { Document } from "@/lib/api/schema";
import { day, money } from "@/lib/format";
import { useT } from "@/lib/i18n";

import { Empty, Rows, WidgetCard } from "./cards";

function needsLook(d: Document): boolean {
  if (d.declared) return false;
  return !d.vendor || !d.total_minor || d.total_minor === "0";
}

export function ReceiptsCard({
  documents,
  total,
  loading,
}: {
  documents: Document[];
  total: number;
  loading: boolean;
}) {
  const t = useT();
  const attention = documents.filter(needsLook).length;
  const shown = [...documents]
    .sort((a, b) => (b.doc_date || b.created_at).localeCompare(a.doc_date || a.created_at))
    .slice(0, 5);

  return (
    <WidgetCard
      icon={Receipt}
      title={t("overview.receipts.title")}
      description={
        total
          ? [
              t("overview.receipts.pulled_n", { n: total }),
              attention ? t("overview.receipts.need_look_n", { n: attention }) : null,
            ]
              .filter(Boolean)
              .join(" · ")
          : undefined
      }
      href="/documents/"
      hrefLabel={t("overview.receipts.open")}
    >
      {loading ? (
        <Rows />
      ) : shown.length === 0 ? (
        <Empty>{t("overview.receipts.empty")}</Empty>
      ) : (
        <ul className="divide-y">
          {shown.map((d) => (
            <li key={d.id} className="flex items-center gap-3 py-2 text-sm">
              <span className="text-muted-foreground w-14 shrink-0 text-xs tabular-nums">
                {day(d.doc_date || d.created_at.slice(0, 10))}
              </span>
              <Link
                href={`/documents/?q=${encodeURIComponent(d.vendor || d.filename)}`}
                className={
                  needsLook(d)
                    ? "text-muted-foreground min-w-0 flex-1 truncate italic hover:underline"
                    : "min-w-0 flex-1 truncate hover:underline"
                }
              >
                {d.vendor || d.filename}
              </Link>
              <span className="w-28 shrink-0 text-right font-mono tabular-nums">
                {d.total_minor && d.total_minor !== "0" ? money(d.total_minor, d.currency || "EUR") : "—"}
              </span>
            </li>
          ))}
        </ul>
      )}
    </WidgetCard>
  );
}
