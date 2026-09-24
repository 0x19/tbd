"use client";

import { Paperclip } from "lucide-react";
import { useRef, useState } from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { api } from "@/lib/api/client";
import { describe } from "@/lib/api/hooks";
import type { Document } from "@/lib/api/schema";
import { useT } from "@/lib/i18n";

/** The gateway takes a 2 MiB body; base64 costs a third, and the JSON
 *  around it a little. What is left for the file. */
const MAX_BYTES = 1_400_000;

/** A file as the API takes it: base64, with its type, and a photo shrunk to
 *  fit. A PDF that does not fit is refused with a reason. */
export async function prepare(
  file: File,
): Promise<{ bytes: string; content_type: string; filename: string }> {
  const type = file.type || (file.name.toLowerCase().endsWith(".pdf") ? "application/pdf" : "");
  if (type === "application/pdf") {
    if (file.size > MAX_BYTES)
      throw new Error(`too large: ${(file.size / 1_000_000).toFixed(1)} MB, at most 1.4 MB`);
    return { bytes: base64(await file.arrayBuffer()), content_type: type, filename: file.name };
  }
  if (type === "image/jpeg" || type === "image/png" || type === "image/webp" || type === "image/heic") {
    const blob = await shrink(file);
    return {
      bytes: base64(await blob.arrayBuffer()),
      content_type: "image/jpeg",
      filename: jpgName(file.name),
    };
  }
  throw new Error("want a PDF or a photo (JPEG, PNG)");
}

function jpgName(name: string): string {
  return name.replace(/\.[a-z0-9]+$/i, "") + ".jpg";
}

function base64(buf: ArrayBuffer): string {
  const bytes = new Uint8Array(buf);
  let s = "";
  for (let i = 0; i < bytes.length; i += 0x8000) {
    s += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
  }
  return btoa(s);
}

/** A photo re-encoded as JPEG, longest side 2000px, quality lowered until
 *  it fits. A receipt stays legible well below that. */
async function shrink(file: File): Promise<Blob> {
  const url = URL.createObjectURL(file);
  try {
    const img = await new Promise<HTMLImageElement>((resolve, reject) => {
      const i = new Image();
      i.onload = () => resolve(i);
      i.onerror = () => reject(new Error("the image could not be read"));
      i.src = url;
    });
    const scale = Math.min(1, 2000 / Math.max(img.width, img.height));
    const canvas = document.createElement("canvas");
    canvas.width = Math.round(img.width * scale);
    canvas.height = Math.round(img.height * scale);
    const ctx = canvas.getContext("2d");
    if (!ctx) throw new Error("no canvas");
    ctx.drawImage(img, 0, 0, canvas.width, canvas.height);
    for (const quality of [0.85, 0.7, 0.55, 0.4]) {
      const blob = await new Promise<Blob | null>((resolve) => canvas.toBlob(resolve, "image/jpeg", quality));
      if (blob && blob.size <= MAX_BYTES) return blob;
    }
    throw new Error("the photo is too large even shrunk");
  } finally {
    URL.revokeObjectURL(url);
  }
}

/** A button that takes a file and uploads it as a receipt of `partyId`. */
export function UploadReceipt({
  partyId,
  onUploaded,
  label,
  size = "sm",
  variant = "outline",
  className,
  disabled,
}: {
  partyId: string;
  onUploaded: (doc: Document) => void;
  label?: string;
  size?: "sm" | "default";
  variant?: "outline" | "ghost" | "default";
  className?: string;
  disabled?: boolean;
}) {
  const t = useT();
  const input = useRef<HTMLInputElement>(null);
  const [busy, setBusy] = useState(false);
  const pick = async (file: File | undefined) => {
    if (!file || !partyId) return;
    setBusy(true);
    try {
      const p = await prepare(file);
      const r = await api.uploadDocument(partyId, p.filename, p.content_type, p.bytes);
      if (!r.document) throw new Error("no document came back");
      toast.success(t("documents.uploaded", { name: p.filename }));
      onUploaded(r.document);
    } catch (e) {
      toast.error(describe(e));
    } finally {
      setBusy(false);
      if (input.current) input.current.value = "";
    }
  };
  return (
    <>
      <input
        ref={input}
        type="file"
        accept="application/pdf,image/jpeg,image/png,image/webp,image/heic"
        className="hidden"
        onChange={(e) => void pick(e.target.files?.[0])}
      />
      <Button
        size={size}
        variant={variant}
        className={className}
        disabled={disabled || busy || !partyId}
        onClick={() => input.current?.click()}
        title={t("documents.upload_hint")}
      >
        <Paperclip className={size === "sm" ? "size-3" : undefined} />{" "}
        {busy ? t("documents.uploading") : (label ?? t("documents.upload"))}
      </Button>
    </>
  );
}
