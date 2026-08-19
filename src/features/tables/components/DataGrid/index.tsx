import { useRef, useState } from "react";
import { flexRender, getCoreRowModel, useReactTable, type ColumnDef } from "@tanstack/react-table";
import { useVirtualizer } from "@tanstack/react-virtual";
import { Braces } from "lucide-react";

import { Checkbox } from "@/src/app/components/ui/checkbox";
import type { CellValue } from "@/src/features/tables/api";
import type { JsonValue } from "@/src/features/tables/components/JsonViewer/types";

export type JsonCellMode = "view" | "edit";

export interface PendingEdit {
  rowIndex: number;
  columnName: string;
  newValue: string | null;
  row: CellValue[];
}

interface DataGridProps {
  columns: string[];
  rows: CellValue[][];
  onOpenJsonCell?: (
    columnName: string,
    value: JsonValue,
    row: CellValue[],
    mode: JsonCellMode
  ) => void;
  editable?: boolean;
  pendingEdit: PendingEdit | null;
  onPendingEditChange: (edit: PendingEdit | null) => void;
  saving?: boolean;
  selectable?: boolean;
  selectedRowIndices?: Set<number>;
  onSelectionChange?: (indices: Set<number>) => void;
}

const ROW_HEIGHT = 32;
const SELECT_COLUMN_ID = "__select__";

function cellValueEquals(original: CellValue, newValue: string | null): boolean {
  if (newValue === null) return original === null || original === undefined;
  if (original === null || original === undefined) return false;
  return String(original) === newValue;
}

export function DataGrid({
  columns,
  rows,
  onOpenJsonCell,
  editable,
  pendingEdit,
  onPendingEditChange,
  saving,
  selectable,
  selectedRowIndices,
  onSelectionChange,
}: DataGridProps) {
  const parentRef = useRef<HTMLDivElement>(null);
  const [editingCell, setEditingCell] = useState<{ rowIndex: number; columnName: string } | null>(
    null
  );

  function toggleRow(rowIndex: number, checked: boolean) {
    if (!onSelectionChange) return;
    const next = new Set(selectedRowIndices ?? []);
    if (checked) next.add(rowIndex);
    else next.delete(rowIndex);
    onSelectionChange(next);
  }

  function toggleAll(checked: boolean) {
    if (!onSelectionChange) return;
    onSelectionChange(checked ? new Set(rows.map((_, i) => i)) : new Set());
  }

  const columnDefs: ColumnDef<CellValue[]>[] = columns.map((name, colIndex) => ({
    id: name,
    header: name,
    accessorFn: (row) => row[colIndex],
    cell: (info) => {
      const isEditing =
        editingCell !== null &&
        editingCell.rowIndex === info.row.index &&
        editingCell.columnName === name;

      const isPending =
        pendingEdit !== null &&
        pendingEdit.rowIndex === info.row.index &&
        pendingEdit.columnName === name;

      if (isEditing) {
        const originalValue = info.getValue<CellValue>();
        const startingValue = isPending ? pendingEdit.newValue : originalValue;
        return (
          <InlineCellEditor
            value={startingValue}
            disabled={!!saving}
            onCommit={(newValue) => {
              setEditingCell(null);
              if (cellValueEquals(originalValue, newValue)) {
                if (isPending) onPendingEditChange(null);
                return;
              }
              onPendingEditChange({
                rowIndex: info.row.index,
                columnName: name,
                newValue,
                row: info.row.original,
              });
            }}
            onCancel={() => setEditingCell(null)}
          />
        );
      }

      return (
        <CellRenderer
          value={isPending ? pendingEdit.newValue : info.getValue<CellValue>()}
          columnName={name}
          row={info.row.original}
          onOpenJsonCell={onOpenJsonCell}
          isPending={isPending}
          onStartEdit={
            editable && !saving && (pendingEdit === null || isPending)
              ? () => setEditingCell({ rowIndex: info.row.index, columnName: name })
              : undefined
          }
        />
      );
    },
  }));

  const table = useReactTable({
    data: rows,
    columns: columnDefs,
    getCoreRowModel: getCoreRowModel(),
  });

  const { rows: tableRows } = table.getRowModel();

  const virtualizer = useVirtualizer({
    count: tableRows.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => ROW_HEIGHT,
    overscan: 12,
  });

  const virtualRows = virtualizer.getVirtualItems();
  const totalHeight = virtualizer.getTotalSize();
  const paddingTop = virtualRows.length > 0 ? virtualRows[0].start : 0;
  const paddingBottom = virtualRows.length > 0 ? totalHeight - virtualRows[virtualRows.length - 1].end : 0;

  const allSelected = selectable && rows.length > 0 && selectedRowIndices?.size === rows.length;
  const someSelected = selectable && !!selectedRowIndices?.size && !allSelected;

  return (
    <div ref={parentRef} className="h-full overflow-auto">
      <table className="w-full border-collapse text-sm">
        <thead className="sticky top-0 z-10 bg-background">
          {table.getHeaderGroups().map((headerGroup) => (
            <tr key={headerGroup.id}>
              {selectable && (
                <th
                  key={SELECT_COLUMN_ID}
                  className="w-8 border-b border-r border-border px-2 py-1.5"
                >
                  <Checkbox
                    checked={!!allSelected}
                    indeterminate={!!someSelected}
                    onCheckedChange={(checked) => toggleAll(checked === true)}
                  />
                </th>
              )}
              {headerGroup.headers.map((header) => (
                <th
                  key={header.id}
                  className="border-b border-r border-border px-3 py-1.5 text-left text-xs font-medium text-muted-foreground last:border-r-0"
                >
                  {flexRender(header.column.columnDef.header, header.getContext())}
                </th>
              ))}
            </tr>
          ))}
        </thead>
        <tbody>
          {paddingTop > 0 && (
            <tr>
              <td style={{ height: paddingTop }} colSpan={columns.length + (selectable ? 1 : 0)} />
            </tr>
          )}
          {virtualRows.map((virtualRow) => {
            const row = tableRows[virtualRow.index];
            const isSelected = selectedRowIndices?.has(row.index) ?? false;
            return (
              <tr
                key={row.id}
                className={`hover:bg-muted/40 ${isSelected ? "bg-primary/5" : ""}`}
                style={{ height: ROW_HEIGHT }}
              >
                {selectable && (
                  <td className="border-b border-r border-border px-2 py-1">
                    <Checkbox
                      checked={isSelected}
                      onCheckedChange={(checked) => toggleRow(row.index, checked === true)}
                    />
                  </td>
                )}
                {row.getVisibleCells().map((cell) => (
                  <td
                    key={cell.id}
                    className="relative border-b border-r border-border px-3 py-1 text-xs last:border-r-0"
                  >
                    {flexRender(cell.column.columnDef.cell, cell.getContext())}
                  </td>
                ))}
              </tr>
            );
          })}
          {paddingBottom > 0 && (
            <tr>
              <td style={{ height: paddingBottom }} colSpan={columns.length + (selectable ? 1 : 0)} />
            </tr>
          )}
        </tbody>
      </table>
    </div>
  );
}

function CellRenderer({
  value,
  columnName,
  row,
  onOpenJsonCell,
  onStartEdit,
  isPending,
}: {
  value: CellValue;
  columnName: string;
  row: CellValue[];
  onOpenJsonCell?: (
    columnName: string,
    value: JsonValue,
    row: CellValue[],
    mode: JsonCellMode
  ) => void;
  onStartEdit?: () => void;
  isPending?: boolean;
}) {
  if (value !== null && value !== undefined && typeof value === "object") {
    return (
      <button
        type="button"
        onClick={() => onOpenJsonCell?.(columnName, value as JsonValue, row, "view")}
        onDoubleClick={() => onOpenJsonCell?.(columnName, value as JsonValue, row, "edit")}
        className="flex max-w-[300px] items-center gap-1 font-mono text-xs text-sky-500 hover:underline"
      >
        <Braces className="size-3 shrink-0" />
        <span className="truncate">{JSON.stringify(value)}</span>
      </button>
    );
  }

  return (
    <div
      onDoubleClick={onStartEdit}
      className={`min-h-4 ${onStartEdit ? "cursor-text" : ""} ${
        isPending ? "-mx-1.5 -my-0.5 rounded bg-amber-500/15 px-1.5 py-0.5 ring-1 ring-amber-500/40" : ""
      }`}
    >
      {value === null || value === undefined ? (
        <span className="italic text-muted-foreground">NULL</span>
      ) : typeof value === "boolean" ? (
        <span className={value ? "text-emerald-500" : "text-muted-foreground"}>
          {value ? "true" : "false"}
        </span>
      ) : (
        <span className="whitespace-nowrap">{String(value)}</span>
      )}
    </div>
  );
}

function InlineCellEditor({
  value,
  disabled,
  onCommit,
  onCancel,
}: {
  value: CellValue;
  disabled?: boolean;
  onCommit: (newValue: string | null) => void;
  onCancel: () => void;
}) {
  const [text, setText] = useState(value === null || value === undefined ? "" : String(value));

  function commit() {
    onCommit(text === "" ? null : text);
  }

  return (
    <input
      autoFocus
      value={text}
      disabled={disabled}
      onChange={(e) => setText(e.target.value)}
      onBlur={commit}
      onKeyDown={(e) => {
        if (e.key === "Enter") {
          e.preventDefault();
          (e.target as HTMLInputElement).blur();
        } else if (e.key === "Escape") {
          e.preventDefault();
          onCancel();
        }
      }}
      className="absolute inset-0 z-10 w-full min-w-0 rounded-sm border border-ring bg-background px-3 py-1 text-xs outline-none"
    />
  );
}
