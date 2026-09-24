"use client";

import { IconCheck, IconCopy } from "@tabler/icons-react";
import { useState } from "react";

import { Button, ButtonProps } from "@/components/ui/button";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip";
import { useT } from "@/lib/i18n";

interface Props extends ButtonProps {
  text: string;
  className?: string;
}

export function CopyButton({ text, className, ...rest }: Props) {
  const t = useT();
  const [isCopied, setIsCopied] = useState(false);

  const copyToClipboard = async () => {
    try {
      await navigator.clipboard.writeText(text);
      setIsCopied(true);
      setTimeout(() => setIsCopied(false), 2000);
    } catch (err) {
      console.error("Failed to copy text: ", err);
    }
  };

  return (
    <TooltipProvider>
      <Tooltip delayDuration={0}>
        <TooltipTrigger asChild>
          <Button
            variant="outline"
            size="icon"
            className={className}
            onClick={copyToClipboard}
            aria-label={isCopied ? t("nav.copied") : t("nav.copy_to_clipboard")}
            {...rest}
          >
            {isCopied ? (
              <IconCheck strokeWidth={1.5} className="m-auto" />
            ) : (
              <IconCopy strokeWidth={1.5} className="m-auto" />
            )}
          </Button>
        </TooltipTrigger>
        <TooltipContent>
          <p>{isCopied ? t("nav.copied_tip") : t("nav.copy")}</p>
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );
}
