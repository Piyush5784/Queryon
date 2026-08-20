import { Dialog, DialogContent } from "@/src/app/components/ui/dialog";
import { NewTableCreateView } from "@/src/features/schema/components/NewTableCreateView";

interface CreateTableDialogProps {
  connectionId: string;
  schema: string;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onCreated: () => void;
}

export function CreateTableDialog({ connectionId, schema, open, onOpenChange, onCreated }: CreateTableDialogProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="h-[70vh] sm:max-w-4xl">
        <NewTableCreateView
          connectionId={connectionId}
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
