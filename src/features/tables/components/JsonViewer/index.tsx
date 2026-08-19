import { useState } from "react";
import { Braces, Code2 } from "lucide-react";

import { Button } from "@/src/app/components/ui/button";
import { JsonNode } from "@/src/features/tables/components/JsonViewer/JsonNode";
import type { JsonValue } from "@/src/features/tables/components/JsonViewer/types";

interface JsonViewerProps {
  value: JsonValue;
}

export function JsonViewer({ value }: JsonViewerProps) {
  const [mode, setMode] = useState<"tree" | "raw">("tree");

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="flex shrink-0 items-center gap-1 border-b px-1 py-1.5">
        <Button
          variant={mode === "tree" ? "secondary" : "ghost"}
          size="xs"
          onClick={() => setMode("tree")}
          className="gap-1.5"
        >
          <Braces className="size-3.5" />
          Tree
        </Button>
        <Button
          variant={mode === "raw" ? "secondary" : "ghost"}
          size="xs"
          onClick={() => setMode("raw")}
          className="gap-1.5"
        >
          <Code2 className="size-3.5" />
          Raw
        </Button>
      </div>

      <div className="min-h-0 flex-1 overflow-auto p-2">
        {mode === "tree" ? (
          <JsonNode label={null} value={value} depth={0} defaultExpanded />
        ) : (
          <pre className="whitespace-pre-wrap break-words font-mono text-xs">
            {JSON.stringify(value, null, 2)}
          </pre>
        )}
      </div>
    </div>
  );
}
