import { useEffect, useState } from "react";
import { Loader2, Square } from "lucide-react";

import { TooltipButton } from "@/src/components/TooltipButton";

interface RunningQueryOverlayProps {
  cancelling: boolean;
  onCancel: () => void;
}

function formatElapsed(ms: number): string {
  const totalSeconds = Math.floor(ms / 1000);
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  return `${minutes.toString().padStart(2, "0")}:${seconds.toString().padStart(2, "0")}`;
}

export function RunningQueryOverlay({ cancelling, onCancel }: RunningQueryOverlayProps) {
  const [elapsedMs, setElapsedMs] = useState(0);

  useEffect(() => {
    const start = Date.now();
    const interval = setInterval(() => setElapsedMs(Date.now() - start), 250);
    return () => clearInterval(interval);
  }, []);

  return (
    <div className="flex h-full flex-col items-center justify-center gap-3 p-6 text-center">
      <Loader2 className="size-6 animate-spin text-muted-foreground" />
      <div className="space-y-1">
        <p className="text-sm font-medium">Running query…</p>
        <p className="font-mono text-xs text-muted-foreground">{formatElapsed(elapsedMs)}</p>
      </div>
      <TooltipButton
        size="xs"
        variant="outline"
        className="gap-1.5"
        onClick={onCancel}
        disabled={cancelling}
        tooltip="Cancel"
        shortcut={["Esc"]}
      >
        {cancelling ? <Loader2 className="size-3.5 animate-spin" /> : <Square className="size-3" />}
        {cancelling ? "Cancelling…" : "Cancel"}
      </TooltipButton>
    </div>
  );
}
