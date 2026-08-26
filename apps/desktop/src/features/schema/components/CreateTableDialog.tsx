import { Dialog, DialogContent } from "@queryon/ui/components/dialog";
import { NewTableCreateView } from "@/src/features/schema/components/NewTableCreateView";
import type { Engine } from "@/src/features/connections/types";

interface CreateTableDialogProps {
  connectionId: string;
  engine: Engine;
  schema: string;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onCreated: () => void;
}

export function CreateTableDialog({
  connectionId,
  engine,
  schema,
  open,
  onOpenChange,
  onCreated,
}: CreateTableDialogProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="h-[70vh] sm:max-w-4xl">
        <NewTableCreateView
          connectionId={connectionId}
          engine={engine}
          schema={schema}
          onCancel={() => onOpenChange(false)}
          onCreated={() => {
            onOpenChange(false);
            onCreated();
          }}
        />
      </DialogContent>
    </Dialog>
  );
}
