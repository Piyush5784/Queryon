import { useEffect, useState } from "react";
import { Check, Copy, Pencil } from "lucide-react";

import { Button } from "@queryon/ui/components/button";
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
} from "@queryon/ui/components/sheet";
import { updateJsonCell, type CellValue } from "@/src/features/tables/api";
import { JsonEditor } from "@/src/features/tables/components/JsonViewer/JsonEditor";
import { JsonViewer } from "@/src/features/tables/components/JsonViewer";
import type { JsonValue } from "@/src/features/tables/components/JsonViewer/types";
import { toErrorMessage } from "@/src/lib/tauri/errors";

interface JsonCellTarget {
  connectionId: string;
  schema: string;
  table: string;
  row: Record<string, CellValue>;
  columnName: string;
  value: JsonValue;
  mode: "view" | "edit";
}

interface JsonInspectorSheetProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  target: JsonCellTarget | null;
  onSaved: () => void;
}

export function JsonInspectorSheet({ open, onOpenChange, target, onSaved }: JsonInspectorSheetProps) {
  const [mode, setMode] = useState<"view" | "edit">("view");
  const [copied, setCopied] = useState(false);
  const [saving, setSaving] = useState(false);
  const [saveError, setSaveError] = useState<string | null>(null);

  useEffect(() => {
    if (open && target) {
      setMode(target.mode);
      setSaveError(null);
    }
  }, [open, target]);

  async function handleCopy() {
    if (!target) return;
    await navigator.clipboard.writeText(JSON.stringify(target.value, null, 2));
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  }

  async function handleSave(next: JsonValue) {
    if (!target) return;
    setSaving(true);
    setSaveError(null);
    try {
      await updateJsonCell(
        target.connectionId,
        target.schema,
        target.table,
        target.row,
        target.columnName,
        next as CellValue
      );
      onSaved();
      onOpenChange(false);
    } catch (err) {
      setSaveError(toErrorMessage(err));
    } finally {
      setSaving(false);
    }
  }

  return (
    <Sheet open={open} onOpenChange={onOpenChange}>
      <SheetContent className="w-full overflow-hidden sm:max-w-xl">
        <SheetHeader className="flex-row items-start justify-between gap-2 pr-10">
          <div>
            <SheetTitle>{target?.columnName ?? "JSON value"}</SheetTitle>
            <SheetDescription>
              {mode === "edit" ? "Editing this cell's JSON value" : "Inspect this cell's JSON value"}
            </SheetDescription>
          </div>
          <div className="flex shrink-0 gap-1.5">
            {mode === "view" && (
              <Button variant="outline" size="sm" className="gap-1.5" onClick={() => setMode("edit")}>
                <Pencil className="size-3.5" />
                Edit
              </Button>
            )}
            <Button variant="outline" size="sm" className="gap-1.5" onClick={handleCopy}>
              {copied ? <Check className="size-3.5" /> : <Copy className="size-3.5" />}
              {copied ? "Copied" : "Copy"}
            </Button>
          </div>
        </SheetHeader>

        <div className="min-h-0 flex-1 overflow-hidden">
          {target &&
            (mode === "edit" ? (
              <JsonEditor
                value={target.value}
                saving={saving}
                onSave={handleSave}
                onCancel={() => setMode("view")}
              />
            ) : (
              <JsonViewer value={target.value} />
            ))}
        </div>

        {saveError && (
          <div className="mx-4 mb-4 rounded-lg border border-destructive/30 bg-destructive/10 px-3 py-2 text-xs text-destructive">
            {saveError}
          </div>
        )}
      </SheetContent>
    </Sheet>
  );
}
