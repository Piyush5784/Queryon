import { useEffect, useState } from "react";
import { AlertTriangle, Loader2, XCircle } from "lucide-react";

import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogMedia,
  AlertDialogTitle,
} from "@/src/app/components/ui/alert-dialog";
import { Input } from "@/src/app/components/ui/input";
import { executeDdl } from "@/src/features/schema/api";
import { toErrorMessage } from "@/src/lib/tauri/errors";

interface DropTableDialogProps {
  connectionId: string;
  schema: string;
  table: string;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onDropped: () => void;
}

export function DropTableDialog({
  connectionId,
  schema,
  table,
  open,
  onOpenChange,
  onDropped,
}: DropTableDialogProps) {
  const [confirmText, setConfirmText] = useState("");
  const [dropping, setDropping] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (open) {
      setConfirmText("");
      setError(null);
    }
  }, [open]);

  const canDrop = confirmText === table;

  async function handleDrop() {
    if (!canDrop) return;
    setDropping(true);
    setError(null);
    try {
      const batch = await executeDdl(connectionId, schema, [{ op: "dropTable", table }]);
      const result = batch.results[0];
      if (!result?.success) {
        setError(result?.error ?? "Drop table failed.");
        return;
      }
      onOpenChange(false);
      onDropped();
    } catch (err) {
      setError(toErrorMessage(err));
    } finally {
      setDropping(false);
    }
  }

  return (
    <AlertDialog open={open} onOpenChange={(next) => !dropping && onOpenChange(next)}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogMedia className="bg-destructive/10 text-destructive">
            <AlertTriangle />
          </AlertDialogMedia>
          <AlertDialogTitle>Drop table {table}?</AlertDialogTitle>
          <AlertDialogDescription>
            This permanently deletes <strong>{schema}.{table}</strong> and all of its data. This
            cannot be undone.
          </AlertDialogDescription>
        </AlertDialogHeader>

        <div className="space-y-1">
          <label className="text-xs text-muted-foreground">
            Type <span className="font-mono font-medium text-foreground">{table}</span> to confirm
          </label>
          <Input
            value={confirmText}
            onChange={(e) => setConfirmText(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && canDrop && handleDrop()}
            className="font-mono"
            autoFocus
          />
        </div>

        {error && (
          <div className="flex items-start gap-2 rounded-lg border border-destructive/30 bg-destructive/10 px-3 py-2 text-xs text-destructive">
            <XCircle className="size-3.5 shrink-0 translate-y-0.5" />
            <span className="wrap-break-word">{error}</span>
          </div>
        )}

        <AlertDialogFooter>
          <AlertDialogCancel disabled={dropping}>Cancel</AlertDialogCancel>
          <AlertDialogAction
            variant="destructive"
            onClick={handleDrop}
            disabled={dropping || !canDrop}
            className="gap-1.5"
          >
            {dropping && <Loader2 className="size-3.5 animate-spin" />}
            Drop Table
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
