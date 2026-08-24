import { useEffect, useRef, useState } from "react";
import { AlertCircle, CircleDot, Loader2, Play, Star } from "lucide-react";

import { Button } from "@queryon/ui/components/button";
import { toast } from "@queryon/ui/components/toast";
import { TooltipButton } from "@/src/components/TooltipButton";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@queryon/ui/components/dialog";
import { Input } from "@queryon/ui/components/input";
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from "@queryon/ui/components/resizable";
import {
  beginTransaction,
  cancelQuery,
  clearQueryResultCache,
  commitTransaction,
  executeQuery,
  rollbackTransaction,
  saveQuery,
  type QueryResult,
} from "@/src/features/query/api";
import { QueryResults } from "@/src/features/query/components/QueryResults";
import { RunningQueryOverlay } from "@/src/features/query/components/RunningQueryOverlay";
import { SqlEditor } from "@/src/features/query/components/SqlEditor";
import { markTabIdle, markTabRunning } from "@/src/features/query/runningTabs";
import type { QueryTab } from "@/src/features/query/types";
import { useSchemaNamespace } from "@/src/features/query/useSchemaNamespace";
import { toErrorMessage, toPrefixedErrorMessage } from "@/src/lib/tauri/errors";

type CommitMode = "auto" | "manual";

interface QueryTabViewProps {
  tab: QueryTab;
  onQueryActivity?: () => void;
}

export function QueryTabView({ tab, onQueryActivity }: QueryTabViewProps) {
  const [sql, setSql] = useState(tab.initialSql ?? "");
  const [running, setRunning] = useState(false);
  const [result, setResult] = useState<QueryResult | null>(null);
  const [executedSql, setExecutedSql] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [saveOpen, setSaveOpen] = useState(false);
  const [saveTitle, setSaveTitle] = useState(tab.title);
  const [saving, setSaving] = useState(false);
  const [saveError, setSaveError] = useState<string | null>(null);
  const [commitMode, setCommitMode] = useState<CommitMode>("auto");
  const [hasActiveTransaction, setHasActiveTransaction] = useState(false);
  const [transactionBusy, setTransactionBusy] = useState(false);
  const [cancelling, setCancelling] = useState(false);
  const schema = useSchemaNamespace(tab.connectionId);
  const activeTabIdRef = useRef(tab.id);
  activeTabIdRef.current = tab.id;
  const sqlRef = useRef(sql);
  sqlRef.current = sql;
  const handleCancelRef = useRef<() => void>(() => {});

  useEffect(() => {
    return () => {
      markTabIdle(activeTabIdRef.current);
      clearQueryResultCache(activeTabIdRef.current).catch(() => {});
    };
  }, []);

  useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      const isSaveShortcut = (event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "s";
      if (isSaveShortcut) {
        event.preventDefault();
        if (!sqlRef.current.trim()) return;
        setSaveTitle(tab.title);
        setSaveError(null);
        setSaveOpen(true);
        return;
      }

      if (event.key === "Escape") {
        handleCancelRef.current();
      }
    }

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [tab.id, tab.title]);

  async function handleExecute() {
    if (running || !sql.trim()) return;
    setRunning(true);
    setCancelling(false);
    setError(null);
    markTabRunning(tab.id);
    try {
      const res = await executeQuery(tab.connectionId, sql, tab.id);
      setResult(res);
      setExecutedSql(sql);
      if (res.kind === "rows") {
        const rowCount = res.totalRowCount ?? res.rows.length;
        toast.add({
          type: "success",
          title: "Query complete",
          description: `${rowCount.toLocaleString()} row${rowCount === 1 ? "" : "s"} returned in ${res.durationMs}ms`,
        });
      } else {
        toast.add({
          type: "success",
          title: "Query complete",
          description: `${res.rowCount.toLocaleString()} row${res.rowCount === 1 ? "" : "s"} affected in ${res.durationMs}ms`,
        });
      }
    } catch (err) {
      const message = toErrorMessage(err);
      setError(message);
      setResult(null);
      toast.add({ type: "error", title: "Query failed", description: message });
    } finally {
      setRunning(false);
      setCancelling(false);
      markTabIdle(tab.id);
      onQueryActivity?.();
    }
  }

  async function handleCancel() {
    if (!running || cancelling) return;
    setCancelling(true);
    try {
      await cancelQuery(tab.connectionId, tab.id);
    } catch (err) {
      toast.add({ type: "error", title: "Failed to cancel query", description: toErrorMessage(err) });
      setCancelling(false);
    }
  }
  handleCancelRef.current = handleCancel;

  async function handleToggleCommitMode(mode: CommitMode) {
    if (mode === commitMode) return;
    if (mode === "auto" && hasActiveTransaction) {
      setTransactionBusy(true);
      try {
        await rollbackTransaction(tab.connectionId, tab.id);
        setHasActiveTransaction(false);
        toast.add({
          type: "info",
          title: "Transaction rolled back",
          description: "Switched to auto commit — the open transaction was rolled back.",
        });
      } catch (err) {
        toast.add({ type: "error", title: "Failed to switch commit mode", description: toErrorMessage(err) });
        return;
      } finally {
        setTransactionBusy(false);
      }
    }
    setCommitMode(mode);
  }

  async function handleBeginTransaction() {
    setTransactionBusy(true);
    try {
      await beginTransaction(tab.connectionId, tab.id);
      setHasActiveTransaction(true);
    } catch (err) {
      toast.add({ type: "error", title: "Failed to begin transaction", description: toErrorMessage(err) });
    } finally {
      setTransactionBusy(false);
    }
  }

  async function handleCommitTransaction() {
    setTransactionBusy(true);
    try {
      await commitTransaction(tab.connectionId, tab.id);
      setHasActiveTransaction(false);
      toast.add({ type: "success", title: "Transaction committed" });
    } catch (err) {
      toast.add({ type: "error", title: "Failed to commit transaction", description: toErrorMessage(err) });
    } finally {
      setTransactionBusy(false);
    }
  }

  async function handleRollbackTransaction() {
    setTransactionBusy(true);
    try {
      await rollbackTransaction(tab.connectionId, tab.id);
      setHasActiveTransaction(false);
      toast.add({ type: "info", title: "Transaction rolled back" });
    } catch (err) {
      toast.add({ type: "error", title: "Failed to roll back transaction", description: toErrorMessage(err) });
    } finally {
      setTransactionBusy(false);
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
        <div className="flex items-center gap-3">
          <span className="text-xs text-muted-foreground">{tab.connectionName}</span>

          {!hasActiveTransaction ? (
            <div className="flex items-center gap-1 rounded-lg border p-0.5">
              <Button
                variant={commitMode === "auto" ? "secondary" : "ghost"}
                size="xs"
                onClick={() => handleToggleCommitMode("auto")}
                disabled={transactionBusy}
              >
                Auto
              </Button>
              <Button
                variant={commitMode === "manual" ? "secondary" : "ghost"}
                size="xs"
                onClick={() => handleToggleCommitMode("manual")}
                disabled={transactionBusy}
              >
                Manual Commit
              </Button>
            </div>
          ) : (
            <span className="flex items-center gap-1 rounded-lg border border-amber-500/30 bg-amber-500/10 px-2 py-1 text-xs text-amber-600 dark:text-amber-400">
              <CircleDot className="size-3 animate-pulse" />
              Transaction active
            </span>
          )}

          {commitMode === "manual" && (
            <div className="flex items-center gap-1">
              {!hasActiveTransaction ? (
                <Button
                  size="xs"
                  variant="outline"
                  onClick={handleBeginTransaction}
                  disabled={transactionBusy}
                >
                  Begin
                </Button>
              ) : (
                <>
                  <Button
                    size="xs"
                    variant="outline"
                    onClick={handleCommitTransaction}
                    disabled={transactionBusy}
                  >
                    Commit
                  </Button>
                  <Button
                    size="xs"
                    variant="outline"
                    className="text-destructive"
                    onClick={handleRollbackTransaction}
                    disabled={transactionBusy}
                  >
                    Rollback
                  </Button>
                </>
              )}
            </div>
          )}
        </div>
        <div className="flex items-center gap-2">
          {result?.kind === "rows" && (
            <span className="text-xs text-muted-foreground">
              {(result.totalRowCount ?? result.rows.length).toLocaleString()} row
              {(result.totalRowCount ?? result.rows.length) === 1 ? "" : "s"} · {result.durationMs}ms
            </span>
          )}
          <TooltipButton
            size="xs"
            variant="outline"
            className="gap-1.5"
            onClick={() => {
              setSaveTitle(tab.title);
              setSaveError(null);
              setSaveOpen(true);
            }}
            disabled={!sql.trim()}
            tooltip="Save Query"
            shortcut={["⌘", "S"]}
          >
            <Star className="size-3.5" />
            Save
          </TooltipButton>
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
          {running ? (
            <RunningQueryOverlay cancelling={cancelling} onCancel={handleCancel} />
          ) : error ? (
            <div className="flex h-full flex-col items-center justify-center gap-2 p-6 text-center">
              <AlertCircle className="size-6 text-destructive" />
              <p className="max-w-md text-sm text-destructive">{error}</p>
            </div>
          ) : result ? (
            <QueryResults
              connectionId={tab.connectionId}
              tabId={tab.id}
              sql={executedSql ?? sql}
              result={result}
            />
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
