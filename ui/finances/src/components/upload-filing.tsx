"use client";

import { FileUp } from "lucide-react";
import { useRef, useState } from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { api } from "@/lib/api/client";
import { describe } from "@/lib/api/hooks";
import type { Document } from "@/lib/api/schema";
import { useT } from "@/lib/i18n";

/** The gateway takes a 2 MiB body; base64 costs a third. A form is a few
 *  tens of kilobytes, so this is never reached in practice. */
const MAX_BYTES = 1_400_000;

function base64(buf: ArrayBuffer): string {
  const bytes = new Uint8Array(buf);
  let s = "";
  for (let i = 0; i < bytes.length; i += 0x8000) {
    s += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
  }
  return btoa(s);
}

/** A button that takes one or more ePorezna XML files and uploads each as a
 *  filing of `partyId`. Each file gets its own toast: the server's sentence
 *  when it is refused (it names the OIBs), the form's name when it is in. */
export function UploadFiling({
  partyId,
  onUploaded,
  size = "sm",
  variant = "outline",
  className,
  disabled,
}: {
  partyId: string;
  onUploaded: (docs: Document[]) => void;
  size?: "sm" | "default";
  variant?: "outline" | "ghost" | "default";
  className?: string;
  disabled?: boolean;
}) {
  const t = useT();
  const input = useRef<HTMLInputElement>(null);
  const [busy, setBusy] = useState(0);
  const pick = async (files: FileList | null) => {
    if (!files || files.length === 0 || !partyId) return;
    const list = Array.from(files);
    setBusy(list.length);
    const done: Document[] = [];
    for (const file of list) {
      try {
        if (file.size > MAX_BYTES) throw new Error(`too large: ${(file.size / 1_000_000).toFixed(1)} MB`);
        const bytes = base64(await file.arrayBuffer());
        const r = await api.uploadDocument(partyId, file.name, "text/xml", bytes);
        if (!r.document) throw new Error("no document came back");
        done.push(r.document);
        toast.success(t("filings.uploaded", { name: file.name }));
      } catch (e) {
        toast.error(t("filings.refused", { name: file.name, reason: describe(e) }));
      } finally {
        setBusy((n) => n - 1);
      }
    }
    if (input.current) input.current.value = "";
    if (done.length) onUploaded(done);
  };
  return (
    <>
      <input
        ref={input}
        type="file"
        accept=".xml,text/xml,application/xml"
        multiple
        className="hidden"
        onChange={(e) => void pick(e.target.files)}
      />
      <Button
        size={size}
        variant={variant}
        className={className}
        disabled={disabled || busy > 0 || !partyId}
        onClick={() => input.current?.click()}
        title={t("filings.upload_hint")}
      >
        <FileUp className={size === "sm" ? "size-3" : undefined} />{" "}
        {busy > 0 ? t("filings.uploading", { n: busy }) : t("filings.upload")}
      </Button>
    </>
  );
}
