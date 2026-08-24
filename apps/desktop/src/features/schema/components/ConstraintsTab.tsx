import { useEffect, useState } from "react";
import { Plus, RotateCcw, X } from "lucide-react";

import { Button } from "@queryon/ui/components/button";
import { Checkbox } from "@queryon/ui/components/checkbox";
import { Input } from "@queryon/ui/components/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@queryon/ui/components/select";
import type { ConstraintInfo, ConstraintKind, ForeignKeyAction } from "@/src/features/schema/api";
import { type SchemaCapabilities } from "@/src/features/schema/capabilities";
import { makeTempId, type StagedNewConstraint } from "@/src/features/schema/staging";
import { getTableColumns, listTables } from "@/src/features/tables/api";

interface ConstraintsTabProps {
  connectionId: string;
  schema: string;
  table: string;
  capabilities: SchemaCapabilities;
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

const FK_ACTION_LABELS: Record<ForeignKeyAction, string> = {
  "no-action": "No Action",
  restrict: "Restrict",
  cascade: "Cascade",
  "set-null": "Set Null",
  "set-default": "Set Default",
};

const FK_ACTIONS = Object.keys(FK_ACTION_LABELS) as ForeignKeyAction[];

function constraintText(constraint: ConstraintInfo): string {
  const cols = constraint.columns.join(", ");
  switch (constraint.kind) {
    case "primary-key":
      return `PRIMARY KEY (${cols})`;
    case "unique":
      return `UNIQUE (${cols})`;
    case "foreign-key": {
      const onUpdate = constraint.onUpdate && constraint.onUpdate !== "no-action"
        ? ` ON UPDATE ${FK_ACTION_LABELS[constraint.onUpdate].toUpperCase()}`
        : "";
      const onDelete = constraint.onDelete && constraint.onDelete !== "no-action"
        ? ` ON DELETE ${FK_ACTION_LABELS[constraint.onDelete].toUpperCase()}`
        : "";
      return `FOREIGN KEY (${cols}) REFERENCES ${constraint.referencedTable} (${constraint.referencedColumns.join(", ")})${onUpdate}${onDelete}`;
    }
    case "check":
      return constraint.checkExpression ?? "";
  }
}

export function ConstraintsTab({
  connectionId,
  schema,
  table,
  capabilities,
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
            capabilities={capabilities}
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
  capabilities: SchemaCapabilities;
  availableColumns: string[];
  constraint: StagedNewConstraint;
  onRemove: () => void;
  onChange: (patch: Partial<StagedNewConstraint>) => void;
}

function NewConstraintRow({
  connectionId,
  schema,
  table,
  capabilities,
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

  const nameDisabled = constraint.kind === "primary-key" && !capabilities.primaryKeyHasCustomName;

  return (
    <div className="space-y-2 bg-primary/5 px-3 py-2">
      <div className="grid grid-cols-[1fr_110px_2fr_28px] items-start gap-2">
        <div>
          <Input
            value={constraint.name}
            onChange={(e) => onChange({ name: e.target.value })}
            placeholder="constraint_name"
            className="h-7 font-mono text-xs"
            disabled={nameDisabled}
            autoFocus
          />
          {nameDisabled && (
            <p className="mt-1 text-[0.65rem] text-muted-foreground">
              MySQL always names a primary key &ldquo;PRIMARY&rdquo;
            </p>
          )}
        </div>
        <Select
          value={constraint.kind}
          onValueChange={(value) => {
            if (!value) return;
            const kind = value as ConstraintKind;
            const forceName = kind === "primary-key" && !capabilities.primaryKeyHasCustomName;
            onChange({ kind, ...(forceName ? { name: "PRIMARY" } : {}) });
          }}
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
              <div className="flex gap-2">
                <div className="flex-1 space-y-1">
                  <label className="text-[0.65rem] text-muted-foreground">On update</label>
                  <Select
                    value={constraint.onUpdate ?? "no-action"}
                    onValueChange={(value) =>
                      value && onChange({ onUpdate: value === "no-action" ? null : (value as ForeignKeyAction) })
                    }
                  >
                    <SelectTrigger size="sm" className="h-7 w-full text-xs">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      {FK_ACTIONS.map((action) => (
                        <SelectItem key={action} value={action}>
                          {FK_ACTION_LABELS[action]}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
                <div className="flex-1 space-y-1">
                  <label className="text-[0.65rem] text-muted-foreground">On delete</label>
                  <Select
                    value={constraint.onDelete ?? "no-action"}
                    onValueChange={(value) =>
                      value && onChange({ onDelete: value === "no-action" ? null : (value as ForeignKeyAction) })
                    }
                  >
                    <SelectTrigger size="sm" className="h-7 w-full text-xs">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      {FK_ACTIONS.map((action) => (
                        <SelectItem key={action} value={action}>
                          {FK_ACTION_LABELS[action]}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
              </div>
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
    onUpdate: null,
    onDelete: null,
    checkExpression: null,
  };
}
