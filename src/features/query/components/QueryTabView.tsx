import { useState } from "react";
import { AlertCircle, Loader2, Play } from "lucide-react";

import { Button } from "@/src/app/components/ui/button";
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from "@/src/app/components/ui/resizable";
import { executeQuery, type QueryResult } from "@/src/features/query/api";
import { QueryResults } from "@/src/features/query/components/QueryResults";
import { SqlEditor } from "@/src/features/query/components/SqlEditor";
import type { QueryTab } from "@/src/features/query/types";
import { useSchemaNamespace } from "@/src/features/query/useSchemaNamespace";
import { toErrorMessage } from "@/src/lib/tauri/errors";

interface QueryTabViewProps {
  tab: QueryTab;
}

export function QueryTabView({ tab }: QueryTabViewProps) {
  const [sql, setSql] = useState("");
  const [running, setRunning] = useState(false);
  const [result, setResult] = useState<QueryResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const schema = useSchemaNamespace(tab.connectionId);

  async function handleExecute() {
    if (running || !sql.trim()) return;
    setRunning(true);
    setError(null);
    try {
      const res = await executeQuery(tab.connectionId, sql);
      setResult(res);
    } catch (err) {
      setError(toErrorMessage(err));
      setResult(null);
    } finally {
      setRunning(false);
    }
  }

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="flex shrink-0 items-center justify-between border-b px-3 py-1.5">
        <span className="text-xs text-muted-foreground">{tab.connectionName}</span>
        <div className="flex items-center gap-2">
          {result?.kind === "rows" && (
            <span className="text-xs text-muted-foreground">
              {result.rowCount.toLocaleString()} row{result.rowCount === 1 ? "" : "s"} ·{" "}
              {result.durationMs}ms
            </span>
          )}
          <Button size="xs" className="gap-1.5" onClick={handleExecute} disabled={running || !sql.trim()}>
            {running ? <Loader2 className="size-3.5 animate-spin" /> : <Play className="size-3.5" />}
            Run
            <kbd className="ml-1 rounded bg-primary-foreground/20 px-1 text-[0.65rem] font-normal">
              ⌘⏎
            </kbd>
          </Button>
        </div>
      </div>

      <ResizablePanelGroup orientation="vertical" className="min-h-0 flex-1">
        <ResizablePanel defaultSize={45} minSize={50}>
          <SqlEditor
            value={sql}
            onChange={setSql}
            onExecute={handleExecute}
            disabled={running}
            schema={schema}
          />
        </ResizablePanel>
        <ResizableHandle withHandle />
        <ResizablePanel defaultSize={55} minSize={15}>
          {error ? (
            <div className="flex h-full flex-col items-center justify-center gap-2 p-6 text-center">
              <AlertCircle className="size-6 text-destructive" />
              <p className="max-w-md text-sm text-destructive">{error}</p>
            </div>
          ) : result ? (
            <QueryResults result={result} />
          ) : (
            <div className="flex h-full items-center justify-center text-sm text-muted-foreground">
              Run a query to see results here.
            </div>
          )}
        </ResizablePanel>
      </ResizablePanelGroup>
    </div>
  );
}
