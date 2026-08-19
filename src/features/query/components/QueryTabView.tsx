import { useState } from "react";
import { AlertCircle, Loader2, Play, Star } from "lucide-react";

import { Button } from "@/src/app/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/src/app/components/ui/dialog";
import { Input } from "@/src/app/components/ui/input";
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from "@/src/app/components/ui/resizable";
import { executeQuery, saveQuery, type QueryResult } from "@/src/features/query/api";
import { QueryResults } from "@/src/features/query/components/QueryResults";
import { SqlEditor } from "@/src/features/query/components/SqlEditor";
import type { QueryTab } from "@/src/features/query/types";
import { useSchemaNamespace } from "@/src/features/query/useSchemaNamespace";
import { toErrorMessage, toPrefixedErrorMessage } from "@/src/lib/tauri/errors";

interface QueryTabViewProps {
  tab: QueryTab;
  onQueryActivity?: () => void;
}

export function QueryTabView({ tab, onQueryActivity }: QueryTabViewProps) {
  const [sql, setSql] = useState(tab.initialSql ?? "");
  const [running, setRunning] = useState(false);
  const [result, setResult] = useState<QueryResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [saveOpen, setSaveOpen] = useState(false);
  const [saveTitle, setSaveTitle] = useState(tab.title);
  const [saving, setSaving] = useState(false);
  const [saveError, setSaveError] = useState<string | null>(null);
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
      onQueryActivity?.();
    }
  }

  async function handleSave() {
    if (!saveTitle.trim() || !sql.trim()) return;
    setSaving(true);
    setSaveError(null);
    try {
      const now = new Date().toISOString();
      await saveQuery({
        id: crypto.randomUUID(),
        connectionId: tab.connectionId,
        title: saveTitle.trim(),
        sql,
        createdAt: now,
        updatedAt: now,
      });
      setSaveOpen(false);
      onQueryActivity?.();
    } catch (err) {
      setSaveError(toPrefixedErrorMessage("Failed to save query", err));
    } finally {
      setSaving(false);
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
          <Button
            size="xs"
            variant="outline"
            className="gap-1.5"
            onClick={() => {
              setSaveTitle(tab.title);
              setSaveError(null);
              setSaveOpen(true);
            }}
            disabled={!sql.trim()}
          >
            <Star className="size-3.5" />
            Save
          </Button>
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

      <Dialog open={saveOpen} onOpenChange={setSaveOpen}>
        <DialogContent className="sm:max-w-sm">
          <DialogHeader>
            <DialogTitle>Save Query</DialogTitle>
            <DialogDescription>
              Save this query for {tab.connectionName} so you can run it again later.
            </DialogDescription>
          </DialogHeader>

          <Input
            autoFocus
            placeholder="Query name"
            value={saveTitle}
            onChange={(e) => setSaveTitle(e.target.value)}
          />

          {saveError && <p className="text-xs text-destructive">{saveError}</p>}

          <DialogFooter>
            <Button variant="outline" onClick={() => setSaveOpen(false)} disabled={saving}>
              Cancel
            </Button>
            <Button onClick={handleSave} disabled={saving || !saveTitle.trim()}>
              {saving && <Loader2 className="size-3.5 animate-spin" />}
              Save
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
