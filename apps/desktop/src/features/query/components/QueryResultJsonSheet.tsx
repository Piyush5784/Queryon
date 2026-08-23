import { useState } from "react";
import { Check, Copy } from "lucide-react";

import { Button } from "@/src/app/components/ui/button";
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
} from "@/src/app/components/ui/sheet";
import { JsonViewer } from "@/src/features/tables/components/JsonViewer";
import type { JsonValue } from "@/src/features/tables/components/JsonViewer/types";

interface QueryResultJsonSheetProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  columnName: string | null;
  value: JsonValue | null;
}

export function QueryResultJsonSheet({ open, onOpenChange, columnName, value }: QueryResultJsonSheetProps) {
  const [copied, setCopied] = useState(false);

  async function handleCopy() {
    if (value === null) return;
    await navigator.clipboard.writeText(JSON.stringify(value, null, 2));
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  }

  return (
    <Sheet open={open} onOpenChange={onOpenChange}>
      <SheetContent className="w-full overflow-hidden sm:max-w-xl">
        <SheetHeader className="flex-row items-start justify-between gap-2 pr-10">
          <div>
            <SheetTitle>{columnName ?? "JSON value"}</SheetTitle>
            <SheetDescription>Inspect this cell's JSON value</SheetDescription>
          </div>
          <Button variant="outline" size="sm" className="gap-1.5" onClick={handleCopy}>
            {copied ? <Check className="size-3.5" /> : <Copy className="size-3.5" />}
            {copied ? "Copied" : "Copy"}
          </Button>
        </SheetHeader>

        <div className="min-h-0 flex-1 overflow-hidden">{value !== null && <JsonViewer value={value} />}</div>
      </SheetContent>
    </Sheet>
  );
}
