import { useEffect, useState } from "react";
import { AlertTriangle, CheckCircle2, Loader2, XCircle } from "lucide-react";

import { Button } from "@queryon/ui/components/button";
import { toast } from "@queryon/ui/components/toast";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@queryon/ui/components/dialog";
import {
  executeDdl,
  renderDdl,
  type DdlExecutionResult,
  type DdlStatement,
} from "@/src/features/schema/api";
import { toErrorMessage } from "@/src/lib/tauri/errors";

interface DdlPreviewDialogProps {
  connectionId: string;
  schema: string;
  statements: DdlStatement[];
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onExecuted: () => void;
}

export function DdlPreviewDialog({
  connectionId,
  schema,
  statements,
  open,
  onOpenChange,
  onExecuted,
}: DdlPreviewDialogProps) {
  const [sqlLines, setSqlLines] = useState<string[] | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [running, setRunning] = useState(false);
  const [results, setResults] = useState<DdlExecutionResult[] | null>(null);
  const [rolledBack, setRolledBack] = useState(false);

  useEffect(() => {
    if (!open) return;
    let cancelled = false;
    setLoading(true);
    setError(null);
    setSqlLines(null);
    setResults(null);
    setRolledBack(false);

    renderDdl(connectionId, schema, statements)
      .then((previews) => {
        if (!cancelled) setSqlLines(previews.map((p) => p.sql));
      })
      .catch((err) => {
        if (!cancelled) setError(toErrorMessage(err));
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });

    return () => {
      cancelled = true;
    };
  }, [open, connectionId, schema, statements]);

  async function handleRun() {
    setRunning(true);
    setError(null);
    try {
      const batch = await executeDdl(connectionId, schema, statements);
      setResults(batch.results);
      setRolledBack(batch.rolledBack);
      if (batch.results.every((r) => r.success)) {
        const count = batch.results.length;
        toast.add({
          type: "success",
          title: "Schema updated",
          description: `${count} statement${count === 1 ? "" : "s"} applied successfully`,
        });
        onExecuted();
      } else if (batch.rolledBack) {
        toast.add({
          type: "error",
          title: "Schema change failed",
          description: "The batch was rolled back — nothing was applied.",
        });
      } else {
        const failed = batch.results.filter((r) => !r.success).length;
        toast.add({
          type: "error",
          title: "Schema change failed",
          description: `${failed} of ${batch.results.length} statements failed`,
        });
      }
    } catch (err) {
      setError(toErrorMessage(err));
    } finally {
      setRunning(false);
    }
  }

  const allSucceeded = results !== null && !rolledBack && results.every((r) => r.success);

  return (
    <Dialog open={open} onOpenChange={(next) => !running && onOpenChange(next)}>
      <DialogContent className="sm:max-w-xl">
        <DialogHeader>
          <DialogTitle>Review SQL</DialogTitle>
        </DialogHeader>

        {loading && (
          <div className="flex items-center justify-center py-10 text-muted-foreground">
            <Loader2 className="size-5 animate-spin" />
          </div>
        )}

        {error && (
          <div className="flex items-start gap-2 rounded-lg border border-destructive/30 bg-destructive/10 px-3 py-2 text-xs text-destructive">
            <XCircle className="size-3.5 shrink-0 translate-y-0.5" />
            <span className="wrap-break-word">{error}</span>
          </div>
        )}

        {!loading && sqlLines && !results && (
          <div className="space-y-2">
            <div className="flex items-start gap-2 rounded-lg border border-amber-500/30 bg-amber-500/10 px-3 py-2 text-xs text-amber-600 dark:text-amber-400">
              <AlertTriangle className="size-3.5 shrink-0 translate-y-0.5" />
              <span>
                This runs directly against your database. On Postgres the whole batch runs in one
                transaction — if any statement fails, everything here is rolled back. MySQL cannot
                offer that guarantee: DDL there auto-commits per statement, so execution stops at
                the first failure but earlier successful statements stay applied.
              </span>
            </div>
            <pre className="max-h-[40vh] overflow-auto rounded-lg border bg-muted/30 p-3 font-mono text-xs whitespace-pre-wrap">
              {sqlLines.join("\n")}
            </pre>
          </div>
        )}

        {results && rolledBack && (
          <div className="mb-2 flex items-start gap-2 rounded-lg border border-destructive/30 bg-destructive/10 px-3 py-2 text-xs text-destructive">
            <XCircle className="size-3.5 shrink-0 translate-y-0.5" />
            <span>This batch failed and was rolled back — none of these statements were applied.</span>
          </div>
        )}

        {results && (
          <div className="divide-y overflow-hidden rounded-lg border">
            {results.map((result, index) => (
              <div key={index} className="flex items-start gap-2 px-3 py-2 text-xs">
                {result.success ? (
                  <CheckCircle2 className="size-3.5 shrink-0 translate-y-0.5 text-emerald-500" />
                ) : (
                  <XCircle className="size-3.5 shrink-0 translate-y-0.5 text-destructive" />
                )}
                <div className="min-w-0 flex-1">
                  <div className="font-mono break-all">{result.sql}</div>
                  {result.error && <div className="mt-1 text-destructive">{result.error}</div>}
                </div>
              </div>
            ))}
          </div>
        )}

        <DialogFooter>
          {!results ? (
            <>
              <Button variant="outline" size="sm" onClick={() => onOpenChange(false)} disabled={running}>
                Cancel
              </Button>
              <Button
                size="sm"
                variant="destructive"
                className="gap-1.5"
                onClick={handleRun}
                disabled={running || loading || !sqlLines}
              >
                {running && <Loader2 className="size-3.5 animate-spin" />}
                Run
              </Button>
            </>
          ) : (
            <Button size="sm" onClick={() => onOpenChange(false)}>
              {allSucceeded ? "Done" : "Close"}
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
