import { useState } from "react";
import { Pencil, Plus, RotateCcw, X } from "lucide-react";

import { Button } from "@queryon/ui/components/button";
import {
  Combobox,
  ComboboxContent,
  ComboboxEmpty,
  ComboboxInput,
  ComboboxItem,
  ComboboxList,
} from "@queryon/ui/components/combobox";
import { Input } from "@queryon/ui/components/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@queryon/ui/components/select";
import { makeTempId, type StagedColumnEdit, type StagedNewColumn } from "@/src/features/schema/staging";
import type { ColumnInfo } from "@/src/features/tables/api";

const COMMON_TYPES = [
  "text",
  "varchar(255)",
  "int4",
  "int8",
  "numeric",
  "boolean",
  "timestamptz",
  "date",
  "uuid",
  "jsonb",
] as const;

export const AUTO_INCREMENT_TYPE = "auto-increment";

// A free-text combobox, not a strict dropdown: the type list is shown as
// suggestions, but whatever the user types is the value directly — no
// separate "Custom…" mode to switch into first (see Beekeeper Studio's
// own column-type field, which uses the same freetext-autocomplete
// pattern rather than a picklist-or-freeform toggle).
function DataTypeField({
  value,
  onChange,
  showAutoIncrement,
}: {
  value: string;
  onChange: (value: string) => void;
  showAutoIncrement: boolean;
}) {
  const items = showAutoIncrement ? [AUTO_INCREMENT_TYPE, ...COMMON_TYPES] : COMMON_TYPES;

  return (
    <Combobox
      items={items}
      inputValue={value}
      onInputValueChange={onChange}
      value={value}
      onValueChange={(next) => next && onChange(next)}
    >
      <ComboboxInput
        placeholder="text, varchar(255), int8, ..."
        className="h-7 font-mono text-xs"
        showTrigger={false}
      />
      <ComboboxContent>
        <ComboboxEmpty>Press Enter to use this type</ComboboxEmpty>
        <ComboboxList>
          {(type: string) => (
            <ComboboxItem key={type} value={type} className="font-mono text-xs">
              {type === AUTO_INCREMENT_TYPE ? "Auto Increment" : type}
            </ComboboxItem>
          )}
        </ComboboxList>
      </ComboboxContent>
    </Combobox>
  );
}

function NullableField({ value, onChange }: { value: boolean; onChange: (value: boolean) => void }) {
  return (
    <Select value={value ? "nullable" : "not-nullable"} onValueChange={(next) => onChange(next === "nullable")}>
      <SelectTrigger size="sm" className="h-7 w-full text-xs">
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        <SelectItem value="nullable" className="text-xs">
          Nullable
        </SelectItem>
        <SelectItem value="not-nullable" className="text-xs">
          Not nullable
        </SelectItem>
      </SelectContent>
    </Select>
  );
}

const COMMON_DEFAULTS = ["NULL", "now()", "''", "0", "true", "false"] as const;
const CUSTOM_DEFAULT = "__custom_default__";

function DefaultField({ value, onChange }: { value: string | null; onChange: (value: string | null) => void }) {
  const asSelectValue = value === null ? "NULL" : value;
  const isCustom = !COMMON_DEFAULTS.includes(asSelectValue as (typeof COMMON_DEFAULTS)[number]);
  const [customMode, setCustomMode] = useState(isCustom);

  if (customMode) {
    return (
      <Input
        value={value ?? ""}
        onChange={(e) => onChange(e.target.value || null)}
        placeholder="e.g. now(), 0, 'active'"
        className="h-7 font-mono text-xs"
        onBlur={() => {
          if (!value?.trim()) setCustomMode(false);
        }}
      />
    );
  }

  return (
    <Select
      value={asSelectValue}
      onValueChange={(next) => {
        if (!next) return;
        if (next === CUSTOM_DEFAULT) {
          setCustomMode(true);
          return;
        }
        onChange(next === "NULL" ? null : next);
      }}
    >
      <SelectTrigger size="sm" className="h-7 w-full font-mono text-xs">
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        {COMMON_DEFAULTS.map((def) => (
          <SelectItem key={def} value={def} className="font-mono text-xs">
            {def}
          </SelectItem>
        ))}
        <SelectItem value={CUSTOM_DEFAULT} className="text-xs">
          Custom…
        </SelectItem>
      </SelectContent>
    </Select>
  );
}

interface ColumnsTabProps {
  columns: ColumnInfo[];
  droppedColumns: string[];
  editedColumns: StagedColumnEdit[];
  newColumns: StagedNewColumn[];
  // Auto Increment only ever appears as an option on a brand-new column
  // of a brand-new table — see AUTO_INCREMENT_TYPE's doc comment in
  // domain/schema/models.rs for why: MySQL requires it declared as a key
  // in the same CREATE TABLE, so adding it to an existing table (via
  // ALTER TABLE ADD COLUMN) or offering it in the column-edit form would
  // just produce a statement MySQL rejects.
  showAutoIncrement: boolean;
  onToggleDrop: (column: string) => void;
  onStartEdit: (column: ColumnInfo) => void;
  onCancelEdit: (currentName: string) => void;
  onChangeEdit: (currentName: string, patch: Partial<NewColumnLike>) => void;
  onAddRow: () => void;
  onRemoveNewRow: (tempId: string) => void;
  onChangeNewRow: (tempId: string, patch: Partial<StagedNewColumn>) => void;
}

type NewColumnLike = StagedColumnEdit["column"];

export function ColumnsTab({
  columns,
  droppedColumns,
  editedColumns,
  newColumns,
  showAutoIncrement,
  onToggleDrop,
  onStartEdit,
  onCancelEdit,
  onChangeEdit,
  onAddRow,
  onRemoveNewRow,
  onChangeNewRow,
}: ColumnsTabProps) {
  return (
    <div className="overflow-hidden rounded-lg border">
      <div className="grid grid-cols-[1fr_1fr_110px_1fr_1fr_60px_52px] items-center gap-2 border-b bg-muted/40 px-3 py-1.5 text-xs font-semibold text-muted-foreground">
        <span>Name</span>
        <span>Type</span>
        <span>Nullable</span>
        <span>Default</span>
        <span>Comment</span>
        <span>Primary</span>
        <span />
      </div>

      <div className="divide-y">
        {columns.map((column) => {
          const dropped = droppedColumns.includes(column.name);
          const edit = editedColumns.find((e) => e.currentName === column.name);

          if (edit) {
            return (
              <div
                key={column.name}
                className="grid grid-cols-[1fr_1fr_110px_1fr_1fr_60px_52px] items-center gap-2 bg-amber-500/5 px-3 py-1.5"
              >
                <Input
                  value={edit.column.name}
                  onChange={(e) => onChangeEdit(column.name, { name: e.target.value })}
                  className="h-7 font-mono text-xs"
                  autoFocus
                />
                <DataTypeField
                  value={edit.column.dataType}
                  onChange={(dataType) => onChangeEdit(column.name, { dataType })}
                  showAutoIncrement={false}
                />
                <NullableField
                  value={edit.column.isNullable}
                  onChange={(isNullable) => onChangeEdit(column.name, { isNullable })}
                />
                <DefaultField
                  value={edit.column.default}
                  onChange={(value) => onChangeEdit(column.name, { default: value })}
                />
                <span className="truncate text-xs text-muted-foreground">—</span>
                <div>{column.isPrimaryKey && <span className="text-emerald-500">✓</span>}</div>
                <button
                  type="button"
                  title="Cancel edit"
                  onClick={() => onCancelEdit(column.name)}
                  className="flex size-5 items-center justify-center rounded text-muted-foreground hover:bg-destructive/10 hover:text-destructive"
                >
                  <X className="size-3.5" />
                </button>
              </div>
            );
          }

          return (
            <div
              key={column.name}
              className={`grid grid-cols-[1fr_1fr_110px_1fr_1fr_60px_52px] items-center gap-2 px-3 py-1.5 text-xs ${
                dropped ? "bg-destructive/5 text-muted-foreground line-through" : ""
              }`}
            >
              <span className="truncate font-mono font-medium">{column.name}</span>
              <span className="truncate font-mono text-muted-foreground">{column.dataType}</span>
              <span className="truncate text-muted-foreground">{column.isNullable ? "Nullable" : "Not nullable"}</span>
              <span className="truncate font-mono text-muted-foreground">{column.default ?? "NULL"}</span>
              <span className="truncate text-muted-foreground">—</span>
              <div>{column.isPrimaryKey && <span className="text-emerald-500">✓</span>}</div>
              <div className="flex items-center gap-1">
                {!dropped && (
                  <button
                    type="button"
                    title="Edit column"
                    onClick={() => onStartEdit(column)}
                    className="flex size-5 items-center justify-center rounded text-muted-foreground hover:bg-primary/10 hover:text-primary"
                  >
                    <Pencil className="size-3.5" />
                  </button>
                )}
                <button
                  type="button"
                  title={dropped ? "Undo drop" : "Drop column"}
                  onClick={() => onToggleDrop(column.name)}
                  className={`flex size-5 items-center justify-center rounded hover:bg-destructive/10 hover:text-destructive ${
                    dropped ? "text-destructive" : "text-muted-foreground"
                  }`}
                >
                  {dropped ? <RotateCcw className="size-3.5" /> : <X className="size-3.5" />}
                </button>
              </div>
            </div>
          );
        })}

        {newColumns.map((column) => (
          <div
            key={column.tempId}
            className="grid grid-cols-[1fr_1fr_110px_1fr_1fr_60px_52px] items-center gap-2 bg-primary/5 px-3 py-1.5"
          >
            <Input
              value={column.name}
              onChange={(e) => onChangeNewRow(column.tempId, { name: e.target.value })}
              placeholder="column_name"
              className="h-7 font-mono text-xs"
              autoFocus
            />
            <DataTypeField
              value={column.dataType}
              onChange={(dataType) => onChangeNewRow(column.tempId, { dataType })}
              showAutoIncrement={showAutoIncrement}
            />
            {column.dataType === AUTO_INCREMENT_TYPE ? (
              <>
                <span className="truncate text-xs text-muted-foreground">Not null</span>
                <span className="truncate font-mono text-xs text-muted-foreground">auto</span>
              </>
            ) : (
              <>
                <NullableField
                  value={column.isNullable}
                  onChange={(isNullable) => onChangeNewRow(column.tempId, { isNullable })}
                />
                <DefaultField
                  value={column.default}
                  onChange={(value) => onChangeNewRow(column.tempId, { default: value })}
                />
              </>
            )}
            <span className="truncate text-xs text-muted-foreground">—</span>
            <div />
            <button
              type="button"
              title="Remove"
              onClick={() => onRemoveNewRow(column.tempId)}
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
          Add column
        </Button>
      </div>
    </div>
  );
}

export function newColumnRow(): StagedNewColumn {
  return { tempId: makeTempId(), name: "", dataType: "", isNullable: true, default: null };
}

export function editRowFor(column: ColumnInfo): StagedColumnEdit {
  return {
    currentName: column.name,
    column: {
      name: column.name,
      dataType: column.dataType,
      isNullable: column.isNullable,
      default: column.default,
    },
  };
}
