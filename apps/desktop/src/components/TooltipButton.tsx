import { Button, type buttonVariants } from "@queryon/ui/components/button";
import { Kbd, KbdGroup } from "@queryon/ui/components/kbd";
import { Tooltip, TooltipContent, TooltipTrigger } from "@queryon/ui/components/tooltip";
import type { VariantProps } from "class-variance-authority";
import type { ComponentProps } from "react";

interface TooltipButtonProps extends ComponentProps<typeof Button>, VariantProps<typeof buttonVariants> {
  tooltip: string;
  shortcut?: string[];
}

export function TooltipButton({ tooltip, shortcut, ...buttonProps }: TooltipButtonProps) {
  return (
    <Tooltip>
      <TooltipTrigger render={<Button {...buttonProps} />} />
      <TooltipContent>
        {tooltip}
        {shortcut && shortcut.length > 0 && (
          <KbdGroup>
            {shortcut.map((key) => (
              <Kbd key={key}>{key}</Kbd>
            ))}
          </KbdGroup>
        )}
      </TooltipContent>
    </Tooltip>
  );
}
