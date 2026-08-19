import { useState } from "react";
import { AlertCircle, Loader2 } from "lucide-react";

import { Button } from "@/src/app/components/ui/button";
import { Textarea } from "@/src/app/components/ui/textarea";
import type { JsonValue } from "@/src/features/tables/components/JsonViewer/types";

interface JsonEditorProps {
  value: JsonValue;
  saving: boolean;
  onSave: (next: JsonValue) => void;
  onCancel: () => void;
}

export function JsonEditor({ value, saving, onSave, onCancel }: JsonEditorProps) {
  const [text, setText] = useState(() => JSON.stringify(value, null, 2));
  const [parseError, setParseError] = useState<string | null>(null);

  function handleChange(next: string) {
    setText(next);
    setParseError(null);
  }

  function handleSave() {
    try {
      const parsed = JSON.parse(text);
      onSave(parsed);
    } catch (err) {
      setParseError(err instanceof Error ? err.message : "Invalid JSON");
    }
  }

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="min-h-0 flex-1 p-2">
        <Textarea
          value={text}
          onChange={(e) => handleChange(e.target.value)}
          spellCheck={false}
          className="h-full resize-none overflow-auto font-mono text-xs"
          disabled={saving}
        />
      </div>

      {parseError && (
        <div className="mx-2 flex items-start gap-2 rounded-lg border border-destructive/30 bg-destructive/10 px-3 py-2 text-xs text-destructive">
          <AlertCircle className="size-3.5 shrink-0 translate-y-0.5" />
          <span className="wrap-break-word">{parseError}</span>
        </div>
      )}

      <div className="flex shrink-0 justify-end gap-2 border-t p-3">
        <Button variant="outline" size="sm" onClick={onCancel} disabled={saving}>
          Cancel
        </Button>
        <Button size="sm" onClick={handleSave} disabled={saving} className="gap-1.5">
          {saving && <Loader2 className="size-3.5 animate-spin" />}
          Save
        </Button>
      </div>
    </div>
  );
}
