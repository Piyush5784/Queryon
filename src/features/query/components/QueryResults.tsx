import { useState } from "react";
import { AlertTriangle, CheckCircle2 } from "lucide-react";

import { DataGrid } from "@/src/features/tables/components/DataGrid";
import type { JsonValue } from "@/src/features/tables/components/JsonViewer/types";
import type { QueryResult } from "@/src/features/query/api";
import { QueryResultJsonSheet } from "@/src/features/query/components/QueryResultJsonSheet";

interface QueryResultsProps {
  result: QueryResult;
}

export function QueryResults({ result }: QueryResultsProps) {
  const [jsonSheet, setJsonSheet] = useState<{ columnName: string; value: JsonValue } | null>(null);

  if (result.kind === "affected") {
    return (
      <div className="flex h-full items-center justify-center p-6">
        <div className="flex items-center gap-2 text-sm text-muted-foreground">
          <CheckCircle2 className="size-4 text-emerald-500" />
          {result.rowCount} row{result.rowCount === 1 ? "" : "s"} affected
          <span className="text-muted-foreground/60">({result.durationMs}ms)</span>
        </div>
      </div>
    );
  }

  if (result.rowCount === 0) {
    return (
      <div className="flex h-full items-center justify-center text-sm text-muted-foreground">
        Query returned no rows.
      </div>
    );
  }

  return (
    <div className="flex h-full min-h-0 flex-col">
      {result.truncated && (
        <div className="flex shrink-0 items-center gap-2 border-b bg-amber-500/10 px-3 py-1.5 text-xs text-amber-600 dark:text-amber-400">
          <AlertTriangle className="size-3.5 shrink-0" />
          Showing the first {result.rows.length.toLocaleString()} of {result.rowCount.toLocaleString()} rows.
        </div>
      )}
      <div className="min-h-0 flex-1">
        <DataGrid
          columns={result.columns}
          rows={result.rows}
          pendingEdit={null}
          onPendingEditChange={() => {}}
          onOpenJsonCell={(columnName, value) => setJsonSheet({ columnName, value })}
        />
      </div>

      <QueryResultJsonSheet
        open={jsonSheet !== null}
        onOpenChange={(open) => !open && setJsonSheet(null)}
        columnName={jsonSheet?.columnName ?? null}
        value={jsonSheet?.value ?? null}
      />
    </div>
  );
}
