import { Plus, RotateCcw, X } from "lucide-react";

import { Button } from "@queryon/ui/components/button";
import { Checkbox } from "@queryon/ui/components/checkbox";
import { Input } from "@queryon/ui/components/input";
import type { IndexInfo } from "@/src/features/schema/api";
import { makeTempId, type StagedNewIndex } from "@/src/features/schema/staging";

interface IndexesTabProps {
  indexes: IndexInfo[];
  availableColumns: string[];
  droppedIndexes: string[];
  newIndexes: StagedNewIndex[];
  onToggleDrop: (index: string) => void;
  onAddRow: () => void;
  onRemoveNewRow: (tempId: string) => void;
  onChangeNewRow: (tempId: string, patch: Partial<StagedNewIndex>) => void;
}

export function IndexesTab({
  indexes,
  availableColumns,
  droppedIndexes,
  newIndexes,
  onToggleDrop,
  onAddRow,
  onRemoveNewRow,
  onChangeNewRow,
}: IndexesTabProps) {
  function toggleColumn(row: StagedNewIndex, column: string) {
    const has = row.columns.includes(column);
    onChangeNewRow(row.tempId, {
      columns: has ? row.columns.filter((c) => c !== column) : [...row.columns, column],
    });
  }

  return (
    <div className="overflow-hidden rounded-lg border">
      <div className="grid grid-cols-[1fr_60px_60px_1.5fr_28px] items-center gap-2 border-b bg-muted/40 px-3 py-1.5 text-xs font-semibold text-muted-foreground">
        <span>Name</span>
        <span>Unique</span>
        <span>Primary</span>
        <span>Columns</span>
        <span />
      </div>

      <div className="divide-y">
        {indexes.map((index) => {
          const dropped = droppedIndexes.includes(index.name);
          return (
            <div
              key={index.name}
              className={`grid grid-cols-[1fr_60px_60px_1.5fr_28px] items-center gap-2 px-3 py-1.5 text-xs ${
                dropped ? "bg-destructive/5 text-muted-foreground line-through" : ""
              }`}
            >
              <span className="truncate font-mono font-medium">{index.name}</span>
              <Checkbox checked={index.isUnique} disabled />
              <Checkbox checked={index.isPrimary} disabled />
              <span className="truncate font-mono text-muted-foreground">{index.columns.join(", ")}</span>
              {index.isPrimary ? (
                <div />
              ) : (
                <button
                  type="button"
                  title={dropped ? "Undo drop" : "Drop index"}
                  onClick={() => onToggleDrop(index.name)}
                  className={`flex size-5 items-center justify-center rounded hover:bg-destructive/10 hover:text-destructive ${
                    dropped ? "text-destructive" : "text-muted-foreground"
                  }`}
                >
                  {dropped ? <RotateCcw className="size-3.5" /> : <X className="size-3.5" />}
                </button>
              )}
            </div>
          );
        })}

        {newIndexes.map((index) => (
          <div
            key={index.tempId}
            className="grid grid-cols-[1fr_60px_60px_1.5fr_28px] items-center gap-2 bg-primary/5 px-3 py-1.5"
          >
            <Input
              value={index.name}
              onChange={(e) => onChangeNewRow(index.tempId, { name: e.target.value })}
              placeholder="index_name"
              className="h-7 font-mono text-xs"
              autoFocus
            />
            <Checkbox
              checked={index.isUnique}
              onCheckedChange={(checked) => onChangeNewRow(index.tempId, { isUnique: checked === true })}
            />
            <div />
            <div className="flex flex-wrap gap-x-2 gap-y-1">
              {availableColumns.map((column) => (
                <label key={column} className="flex items-center gap-1 text-xs">
                  <Checkbox
                    checked={index.columns.includes(column)}
                    onCheckedChange={() => toggleColumn(index, column)}
                  />
                  <span className="font-mono">{column}</span>
                </label>
              ))}
            </div>
            <button
              type="button"
              title="Remove"
              onClick={() => onRemoveNewRow(index.tempId)}
              className="flex size-5 items-center justify-center rounded text-muted-foreground hover:bg-destructive/10 hover:text-destructive"
            >
              <X className="size-3.5" />
            </button>
          </div>
        ))}
      </div>

      <div className="border-t px-3 py-1.5">
        <Button variant="ghost" size="xs" className="gap-1.5 text-primary" onClick={onAddRow}>
          <Plus className="size-3" />
          Add index
        </Button>
      </div>
    </div>
  );
}

export function newIndexRow(): StagedNewIndex {
  return { tempId: makeTempId(), name: "", columns: [], isUnique: false };
}
