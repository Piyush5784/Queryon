import { useEffect, useState } from "react";

import { Button } from "@queryon/ui/components/button";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@queryon/ui/components/dialog";
import { Input } from "@queryon/ui/components/input";
import { DdlPreviewDialog } from "@/src/features/schema/components/DdlPreviewDialog";
import type { DdlStatement } from "@/src/features/schema/api";

interface RenameTableDialogProps {
  connectionId: string;
  schema: string;
  table: string;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onRenamed: () => void;
}

export function RenameTableDialog({
  connectionId,
  schema,
  table,
  open,
  onOpenChange,
  onRenamed,
}: RenameTableDialogProps) {
  const [name, setName] = useState(table);
  const [previewOpen, setPreviewOpen] = useState(false);

  useEffect(() => {
    if (open) setName(table);
  }, [open, table]);

  const trimmed = name.trim();
  const statements: DdlStatement[] = [{ op: "renameTable", table, newName: trimmed }];

  function handleRenamed() {
    setPreviewOpen(false);
    onOpenChange(false);
    onRenamed();
  }

  return (
    <>
      <Dialog open={open} onOpenChange={onOpenChange}>
        <DialogContent className="sm:max-w-sm">
          <DialogHeader>
            <DialogTitle>Rename Table</DialogTitle>
          </DialogHeader>

          <div className="space-y-1">
            <label className="text-xs text-muted-foreground">New name</label>
            <Input
              value={name}
              onChange={(e) => setName(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && trimmed && trimmed !== table && setPreviewOpen(true)}
              className="font-mono"
              autoFocus
            />
          </div>

          <DialogFooter>
            <Button variant="outline" size="sm" onClick={() => onOpenChange(false)}>
              Cancel
            </Button>
            <Button size="sm" disabled={!trimmed || trimmed === table} onClick={() => setPreviewOpen(true)}>
              Rename
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {previewOpen && (
        <DdlPreviewDialog
          connectionId={connectionId}
          schema={schema}
          statements={statements}
          open={previewOpen}
          onOpenChange={setPreviewOpen}
          onExecuted={handleRenamed}
        />
      )}
    </>
  );
}
