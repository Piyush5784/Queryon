import { useState } from "react";
import { ChevronRight } from "lucide-react";

import type { JsonValue } from "@/src/features/tables/components/JsonViewer/types";

interface JsonNodeProps {
  label: string | null;
  value: JsonValue;
  depth: number;
  defaultExpanded?: boolean;
}

export function JsonNode({ label, value, depth, defaultExpanded }: JsonNodeProps) {
  const isContainer = value !== null && typeof value === "object";
  const [expanded, setExpanded] = useState(defaultExpanded ?? depth < 2);

  if (!isContainer) {
    return (
      <div className="flex items-start gap-1.5 py-0.5 pl-4 font-mono text-xs" style={{ paddingLeft: depth * 16 + 16 }}>
        {label !== null && <span className="shrink-0 text-sky-500">{JSON.stringify(label)}:</span>}
        <PrimitiveValue value={value} />
      </div>
    );
  }

  const isArray = Array.isArray(value);
  const entries = isArray ? value.map((v, i) => [String(i), v] as const) : Object.entries(value);
  const isEmpty = entries.length === 0;
  const bracket = isArray ? ["[", "]"] : ["{", "}"];
  const summary = isArray ? `${entries.length} items` : `${entries.length} keys`;

  return (
    <div>
      <button
        type="button"
        onClick={() => !isEmpty && setExpanded((prev) => !prev)}
        className="flex w-full items-center gap-1.5 py-0.5 text-left font-mono text-xs hover:bg-muted/50"
        style={{ paddingLeft: depth * 16 }}
      >
        {!isEmpty ? (
          <ChevronRight className={`size-3 shrink-0 text-muted-foreground transition-transform ${expanded ? "rotate-90" : ""}`} />
        ) : (
          <span className="inline-block size-3 shrink-0" />
        )}
        {label !== null && <span className="text-sky-500">{JSON.stringify(label)}:</span>}
        <span className="text-muted-foreground">{bracket[0]}</span>
        {!expanded && !isEmpty && (
          <span className="text-muted-foreground/70">
            {summary} {bracket[1]}
          </span>
        )}
        {isEmpty && <span className="text-muted-foreground">{bracket[1]}</span>}
      </button>

      {expanded && !isEmpty && (
        <div>
          {entries.map(([key, childValue]) => (
            <JsonNode key={key} label={isArray ? null : key} value={childValue} depth={depth + 1} />
          ))}
          <div className="font-mono text-xs text-muted-foreground" style={{ paddingLeft: depth * 16 + 16 }}>
            {bracket[1]}
          </div>
        </div>
      )}
    </div>
  );
}

function PrimitiveValue({ value }: { value: string | number | boolean | null }) {
  if (value === null) {
    return <span className="italic text-muted-foreground">null</span>;
  }
  if (typeof value === "boolean") {
    return <span className={value ? "text-emerald-500" : "text-rose-500"}>{String(value)}</span>;
  }
  if (typeof value === "number") {
    return <span className="text-amber-500">{value}</span>;
  }
  return <span className="text-foreground">{JSON.stringify(value)}</span>;
}
