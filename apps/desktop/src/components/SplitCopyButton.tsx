import { useState } from "react";
import { ChevronDown, Copy, Loader2 } from "lucide-react";

import { Button } from "@queryon/ui/components/button";
import { ButtonGroup } from "@queryon/ui/components/button-group";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@queryon/ui/components/dropdown-menu";
import { useCopiedFlash } from "@/src/hooks/use-copied-flash";
import { useHoverOpen } from "@/src/hooks/use-hover-open";

export interface SplitCopyButtonOption {
  label: string;
  onClick: () => void | Promise<void>;
  disabled?: boolean;
}

interface SplitCopyButtonProps {
  onDefaultClick: () => void | Promise<void>;
  options: SplitCopyButtonOption[];
  disabled?: boolean;
}

export function SplitCopyButton({ onDefaultClick, options, disabled }: SplitCopyButtonProps) {
  const { copied, flash } = useCopiedFlash();
  const [copying, setCopying] = useState(false);
  const menu = useHoverOpen();

  async function run(action: () => void | Promise<void>) {
    setCopying(true);
    try {
      await action();
      flash();
    } finally {
      setCopying(false);
    }
  }

  return (
    <ButtonGroup onMouseEnter={menu.onMouseEnter} onMouseLeave={menu.onMouseLeave}>
      <Button
        variant="outline"
        size="xs"
        className="gap-1.5"
        disabled={disabled || copying}
        onClick={() => run(onDefaultClick)}
      >
        {copying ? <Loader2 className="size-3.5 animate-spin" /> : <Copy className="size-3.5" />}
        {copied ? "Copied" : "Copy"}
      </Button>
      <DropdownMenu open={menu.open} onOpenChange={menu.setOpen}>
        <DropdownMenuTrigger
          render={
            <Button variant="outline" size="xs" disabled={disabled || copying}>
              <ChevronDown className="size-3.5" />
            </Button>
          }
        />
        <DropdownMenuContent
          align="end"
          side="top"
          onMouseEnter={menu.onMouseEnter}
          onMouseLeave={menu.onMouseLeave}
        >
          {options.map((option) => (
            <DropdownMenuItem
              key={option.label}
              disabled={option.disabled}
              onClick={() => run(option.onClick)}
            >
              {option.label}
            </DropdownMenuItem>
          ))}
        </DropdownMenuContent>
      </DropdownMenu>
    </ButtonGroup>
  );
}
