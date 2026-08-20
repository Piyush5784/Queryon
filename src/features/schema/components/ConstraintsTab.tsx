import { useEffect, useState } from "react";
import { Plus, RotateCcw, X } from "lucide-react";

import { Button } from "@/src/app/components/ui/button";
import { Checkbox } from "@/src/app/components/ui/checkbox";
import { Input } from "@/src/app/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/src/app/components/ui/select";
import type { ConstraintInfo, ConstraintKind } from "@/src/features/schema/api";
import { makeTempId, type StagedNewConstraint } from "@/src/features/schema/staging";
import { getTableColumns, listTables } from "@/src/features/tables/api";

interface ConstraintsTabProps {
  connectionId: string;
  schema: string;
  table: string;
  availableColumns: string[];
  constraints: ConstraintInfo[];
  droppedConstraints: string[];
  newConstraints: StagedNewConstraint[];
  onToggleDrop: (constraint: string) => void;
  onAddRow: () => void;
  onRemoveNewRow: (tempId: string) => void;
  onChangeNewRow: (tempId: string, patch: Partial<StagedNewConstraint>) => void;
}

const KIND_LABELS: Record<ConstraintKind, string> = {
  "primary-key": "Primary Key",
  "foreign-key": "Foreign Key",
  unique: "Unique",
  check: "Check",
};

const KINDS = Object.keys(KIND_LABELS) as ConstraintKind[];

function constraintText(constraint: ConstraintInfo): string {
  const cols = constraint.columns.join(", ");
  switch (constraint.kind) {
    case "primary-key":
      return `PRIMARY KEY (${cols})`;
    case "unique":
      return `UNIQUE (${cols})`;
    case "foreign-key":
      return `FOREIGN KEY (${cols}) REFERENCES ${constraint.referencedTable} (${constraint.referencedColumns.join(", ")})`;
    case "check":
      return constraint.checkExpression ?? "";
  }
}

export function ConstraintsTab({
  connectionId,
  schema,
  table,
  availableColumns,
  constraints,
  droppedConstraints,
  newConstraints,
  onToggleDrop,
  onAddRow,
  onRemoveNewRow,
  onChangeNewRow,
}: ConstraintsTabProps) {
  return (
    <div className="overflow-hidden rounded-lg border">
      <div className="grid grid-cols-[1fr_110px_2fr_28px] items-center gap-2 border-b bg-muted/40 px-3 py-1.5 text-xs font-semibold text-muted-foreground">
        <span>Name</span>
        <span>Type</span>
        <span>Definition</span>
        <span />
      </div>

      <div className="divide-y">
        {constraints.map((constraint) => {
          const dropped = droppedConstraints.includes(constraint.name);
          return (
            <div
              key={constraint.name}
              className={`grid grid-cols-[1fr_110px_2fr_28px] items-center gap-2 px-3 py-1.5 text-xs ${
                dropped ? "bg-destructive/5 text-muted-foreground line-through" : ""
              }`}
            >
              <span className="truncate font-mono font-medium">{constraint.name}</span>
              <span className="text-muted-foreground">{KIND_LABELS[constraint.kind]}</span>
              <span className="truncate font-mono text-muted-foreground">{constraintText(constraint)}</span>
              <button
                type="button"
                title={dropped ? "Undo drop" : "Drop constraint"}
                onClick={() => onToggleDrop(constraint.name)}
                className={`flex size-5 items-center justify-center rounded hover:bg-destructive/10 hover:text-destructive ${
                  dropped ? "text-destructive" : "text-muted-foreground"
                }`}
              >
                {dropped ? <RotateCcw className="size-3.5" /> : <X className="size-3.5" />}
              </button>
            </div>
          );
        })}

        {newConstraints.map((constraint) => (
          <NewConstraintRow
            key={constraint.tempId}
            connectionId={connectionId}
            schema={schema}
            table={table}
            availableColumns={availableColumns}
            constraint={constraint}
            onRemove={() => onRemoveNewRow(constraint.tempId)}
            onChange={(patch) => onChangeNewRow(constraint.tempId, patch)}
          />
        ))}
      </div>

      <div className="border-t px-3 py-1.5">
        <Button variant="ghost" size="xs" className="gap-1.5 text-primary" onClick={onAddRow}>
          <Plus className="size-3" />
          Add constraint
        </Button>
      </div>
    </div>
  );
}

interface NewConstraintRowProps {
  connectionId: string;
  schema: string;
  table: string;
  availableColumns: string[];
  constraint: StagedNewConstraint;
  onRemove: () => void;
  onChange: (patch: Partial<StagedNewConstraint>) => void;
}

function NewConstraintRow({
  connectionId,
  schema,
  table,
  availableColumns,
  constraint,
  onRemove,
  onChange,
}: NewConstraintRowProps) {
  const [otherTables, setOtherTables] = useState<string[]>([]);
  const [referencedTableColumns, setReferencedTableColumns] = useState<string[]>([]);

  useEffect(() => {
    if (constraint.kind !== "foreign-key") return;
    listTables(connectionId)
      .then((tables) =>
        setOtherTables(tables.filter((t) => t.kind === "table" && t.name !== table).map((t) => t.name))
      )
      .catch(() => setOtherTables([]));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [constraint.kind, connectionId, table]);

  useEffect(() => {
    if (!constraint.referencedTable) {
      setReferencedTableColumns([]);
      return;
    }
    getTableColumns(connectionId, schema, constraint.referencedTable)
      .then((cols) => setReferencedTableColumns(cols.map((c) => c.name)))
      .catch(() => setReferencedTableColumns([]));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [constraint.referencedTable, connectionId, schema]);

  function toggleColumn(column: string) {
    const has = constraint.columns.includes(column);
    onChange({ columns: has ? constraint.columns.filter((c) => c !== column) : [...constraint.columns, column] });
  }

  function toggleReferencedColumn(column: string) {
    const has = constraint.referencedColumns.includes(column);
    onChange({
      referencedColumns: has
        ? constraint.referencedColumns.filter((c) => c !== column)
        : [...constraint.referencedColumns, column],
    });
  }

  return (
    <div className="space-y-2 bg-primary/5 px-3 py-2">
      <div className="grid grid-cols-[1fr_110px_2fr_28px] items-start gap-2">
        <Input
          value={constraint.name}
          onChange={(e) => onChange({ name: e.target.value })}
          placeholder="constraint_name"
          className="h-7 font-mono text-xs"
          autoFocus
        />
        <Select
          value={constraint.kind}
          onValueChange={(value) => value && onChange({ kind: value as ConstraintKind })}
        >
          <SelectTrigger size="sm" className="h-7 w-full text-xs">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {KINDS.map((k) => (
              <SelectItem key={k} value={k}>
                {KIND_LABELS[k]}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>

        <div className="space-y-2 text-xs">
          {constraint.kind !== "check" && (
            <div className="flex flex-wrap gap-x-2 gap-y-1">
              {availableColumns.map((column) => (
                <label key={column} className="flex items-center gap-1">
                  <Checkbox checked={constraint.columns.includes(column)} onCheckedChange={() => toggleColumn(column)} />
                  <span className="font-mono">{column}</span>
                </label>
              ))}
            </div>
          )}

          {constraint.kind === "foreign-key" && (
            <div className="space-y-2">
              <Select
                value={constraint.referencedTable ?? ""}
                onValueChange={(value) => value && onChange({ referencedTable: value, referencedColumns: [] })}
              >
                <SelectTrigger size="sm" className="h-7 w-full text-xs">
                  <SelectValue placeholder="References table" />
                </SelectTrigger>
                <SelectContent>
                  {otherTables.map((t) => (
                    <SelectItem key={t} value={t}>
                      {t}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              {constraint.referencedTable && (
                <div className="flex flex-wrap gap-x-2 gap-y-1">
                  {referencedTableColumns.map((column) => (
                    <label key={column} className="flex items-center gap-1">
                      <Checkbox
                        checked={constraint.referencedColumns.includes(column)}
                        onCheckedChange={() => toggleReferencedColumn(column)}
                      />
                      <span className="font-mono">{column}</span>
                    </label>
                  ))}
                </div>
              )}
            </div>
          )}

          {constraint.kind === "check" && (
            <Input
              value={constraint.checkExpression ?? ""}
              onChange={(e) => onChange({ checkExpression: e.target.value })}
              placeholder="e.g. price >= 0"
              className="h-7 font-mono text-xs"
            />
          )}
        </div>

        <button
          type="button"
          title="Remove"
          onClick={onRemove}
          className="flex size-5 items-center justify-center rounded text-muted-foreground hover:bg-destructive/10 hover:text-destructive"
        >
          <X className="size-3.5" />
        </button>
      </div>
    </div>
  );
}

export function newConstraintRow(): StagedNewConstraint {
  return {
    tempId: makeTempId(),
    name: "",
    kind: "unique",
    columns: [],
    referencedTable: null,
    referencedColumns: [],
    checkExpression: null,
  };
}
