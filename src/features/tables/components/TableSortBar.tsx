import { useEffect, useState } from "react";
import { Plus, Search, X } from "lucide-react";

import { Button } from "@/src/app/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/src/app/components/ui/select";
import type { SortDirection, TableSort } from "@/src/features/tables/api";

interface TableSortBarProps {
  columns: string[];
  sort: TableSort[];
  onApply: (sort: TableSort[]) => void;
  onClear: () => void;
}

const DIRECTION_LABELS: Record<SortDirection, string> = {
  asc: "ascending",
  desc: "descending",
};

const DIRECTIONS = Object.keys(DIRECTION_LABELS) as SortDirection[];

function emptyRow(defaultColumn: string): TableSort {
  return { column: defaultColumn, direction: "asc" };
}

export function TableSortBar({ columns, sort, onApply, onClear }: TableSortBarProps) {
  const [rows, setRows] = useState<TableSort[]>(() =>
    sort.length > 0 ? sort : [emptyRow(columns[0] ?? "")]
  );

  useEffect(() => {
    if (sort.length > 0) setRows(sort);
  }, [sort]);

  function applyRows(next: TableSort[]) {
    onApply(next.filter((row) => !!row.column));
  }

  function updateRow(index: number, patch: Partial<TableSort>) {
    setRows((prev) => prev.map((row, i) => (i === index ? { ...row, ...patch } : row)));
  }

  function addRow() {
    setRows((prev) => [...prev, emptyRow(columns[0] ?? "")]);
  }

  function removeRow(index: number) {
    if (rows.length === 1) {
      onClear();
      return;
    }
    setRows((prev) => prev.filter((_, i) => i !== index));
  }

  return (
    <div className="flex shrink-0 flex-col gap-1.5 border-b bg-muted/30 px-3 py-1.5">
      {rows.map((row, index) => (
        <div key={index} className="flex items-center gap-1.5">
          <button
            type="button"
            onClick={() => removeRow(index)}
            className="flex size-6 shrink-0 items-center justify-center rounded-md text-muted-foreground hover:bg-muted"
            title="Remove sort"
          >
            <X className="size-3.5" />
          </button>

          <span className="w-14 shrink-0 text-xs text-muted-foreground">
            {index === 0 ? "sort by" : "then by"}
          </span>

          <Select
            value={row.column || columns[0] || ""}
            onValueChange={(value) => updateRow(index, { column: value ?? "" })}
          >
            <SelectTrigger size="sm" className="h-7 w-[140px] text-xs">
              <SelectValue placeholder="Column" />
            </SelectTrigger>
            <SelectContent>
              {columns.map((col) => (
                <SelectItem key={col} value={col}>
                  {col}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>

          <Select
            value={row.direction}
            onValueChange={(value) => updateRow(index, { direction: value as SortDirection })}
          >
            <SelectTrigger size="sm" className="h-7 w-[130px] text-xs">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {DIRECTIONS.map((dir) => (
                <SelectItem key={dir} value={dir}>
                  {DIRECTION_LABELS[dir]}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>

          <button
            type="button"
            onClick={() => applyRows(rows)}
            className="flex size-7 shrink-0 items-center justify-center rounded-md text-muted-foreground hover:bg-muted"
            title="Apply sort"
          >
            <Search className="size-3.5" />
          </button>

          {index === rows.length - 1 && (
            <button
              type="button"
              onClick={addRow}
              className="flex size-7 items-center justify-center rounded-md text-muted-foreground hover:bg-muted"
              title="Add sort"
            >
              <Plus className="size-3.5" />
            </button>
          )}
        </div>
      ))}

      <div>
        <Button variant="ghost" size="xs" onClick={onClear}>
          Clear sort
        </Button>
      </div>
    </div>
  );
}
