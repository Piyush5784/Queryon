import { LayoutPanelTop } from "lucide-react";

import { Button } from "@queryon/ui/components/button";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@queryon/ui/components/popover";
import { cn } from "@queryon/ui/lib/utils";
import type { DockLayoutPreset } from "@/src/components/DockLayout";

interface LayoutOption {
  id: DockLayoutPreset;
  label: string;
}

const LAYOUT_OPTIONS: LayoutOption[] = [
  { id: "stacked", label: "Stacked" },
  { id: "side-by-side", label: "Side by side" },
  { id: "tabbed", label: "Tabbed" },
];

function LayoutThumbnail({ preset }: { preset: DockLayoutPreset }) {
  if (preset === "side-by-side") {
    return (
      <div className="flex h-14 w-full gap-1">
        <div className="flex-1 rounded-sm bg-muted-foreground/20" />
        <div className="flex-1 rounded-sm bg-muted-foreground/20" />
      </div>
    );
  }

  if (preset === "tabbed") {
    return (
      <div className="flex h-14 w-full flex-col gap-1">
        <div className="flex h-3 gap-0.5">
          <div className="w-6 rounded-t-sm bg-muted-foreground/35" />
          <div className="w-6 rounded-t-sm bg-muted-foreground/15" />
        </div>
        <div className="flex-1 rounded-sm rounded-tl-none bg-muted-foreground/20" />
      </div>
    );
  }

  return (
    <div className="flex h-14 w-full flex-col gap-1">
      <div className="flex-1 rounded-sm bg-muted-foreground/20" />
      <div className="flex-1 rounded-sm bg-muted-foreground/20" />
    </div>
  );
}

interface LayoutPickerProps {
  onSelect: (preset: DockLayoutPreset) => void;
}

export function LayoutPicker({ onSelect }: LayoutPickerProps) {
  return (
    <Popover>
      <PopoverTrigger
        render={
          <Button size="icon-sm" variant="ghost">
            <LayoutPanelTop className="size-3.5" />
          </Button>
        }
      />
      <PopoverContent align="end" className="w-64">
        <p className="px-0.5 text-xs font-medium text-muted-foreground">Layout</p>
        <div className="grid grid-cols-2 gap-2">
          {LAYOUT_OPTIONS.map((option) => (
            <button
              key={option.id}
              type="button"
              onClick={() => onSelect(option.id)}
              className={cn(
                "flex flex-col items-center gap-1.5 rounded-lg border bg-muted/20 p-2 transition-colors",
                "hover:border-foreground/30 hover:bg-muted/40"
              )}
            >
              <LayoutThumbnail preset={option.id} />
              <span className="text-xs text-muted-foreground">{option.label}</span>
            </button>
          ))}
        </div>
      </PopoverContent>
    </Popover>
  );
}
