import { useEffect, useState } from "react";
import { Copy, Loader2 } from "lucide-react";

import { Button } from "@queryon/ui/components/button";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@queryon/ui/components/dialog";
import { getTableDdl } from "@/src/features/schema/api";
import { toErrorMessage } from "@/src/lib/tauri/errors";

interface TableDdlDialogProps {
  connectionId: string;
  schema: string;
  table: string;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function TableDdlDialog({ connectionId, schema, table, open, onOpenChange }: TableDdlDialogProps) {
  const [ddl, setDdl] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    if (!open) return;
    let cancelled = false;
    setLoading(true);
    setError(null);
    setDdl(null);

    getTableDdl(connectionId, schema, table)
      .then((result) => {
        if (!cancelled) setDdl(result);
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
  }, [open, connectionId, schema, table]);

  async function handleCopy() {
    if (!ddl) return;
    await navigator.clipboard.writeText(ddl);
    setCopied(true);
    setTimeout(() => setCopied(false), 1200);
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-2xl">
        <DialogHeader>
          <DialogTitle className="font-mono text-sm">
            {schema}.{table}
          </DialogTitle>
        </DialogHeader>

        {loading && (
          <div className="flex items-center justify-center py-10 text-muted-foreground">
            <Loader2 className="size-5 animate-spin" />
          </div>
        )}

        {error && <p className="text-xs text-destructive">{error}</p>}

        {ddl && (
          <pre className="max-h-[60vh] overflow-auto rounded-lg border bg-muted/30 p-3 font-mono text-xs whitespace-pre-wrap">
            {ddl}
          </pre>
        )}

        <DialogFooter>
          <Button variant="outline" size="sm" className="gap-1.5" onClick={handleCopy} disabled={!ddl}>
            <Copy className="size-3.5" />
            {copied ? "Copied" : "Copy"}
          </Button>
          <Button size="sm" onClick={() => onOpenChange(false)}>
            Close
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
