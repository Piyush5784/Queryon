import { useEffect, useState } from "react";
import { Plus, Search, X } from "lucide-react";

import { Button } from "@queryon/ui/components/button";
import { Input } from "@queryon/ui/components/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@queryon/ui/components/select";
import type { FilterOperator, TableFilter } from "@/src/features/tables/api";

interface TableFilterBarProps {
  columns: string[];
  filters: TableFilter[];
  onApply: (filters: TableFilter[]) => void;
  onClear: () => void;
}

const OPERATOR_LABELS: Record<FilterOperator, string> = {
  equals: "equals",
  "not-equals": "not equals",
  "greater-than": "greater",
  "greater-or-equals": "greater or equals",
  "less-than": "less",
  "less-or-equals": "less or equals",
  like: "like",
  ilike: "ilike",
  "not-like": "not like",
  in: "in",
  "is-null": "is null",
  "is-not-null": "is not null",
};

const OPERATORS = Object.keys(OPERATOR_LABELS) as FilterOperator[];

function needsValue(operator: FilterOperator): boolean {
  return operator !== "is-null" && operator !== "is-not-null";
}

function valuePlaceholder(operator: FilterOperator): string {
  if (operator === "in") return "value1, value2, ...";
  if (operator === "like" || operator === "ilike" || operator === "not-like") return "pattern (use %)";
  return "Enter Value";
}

function emptyRow(defaultColumn: string): TableFilter {
  return { column: defaultColumn, operator: "equals", value: "" };
}

export function TableFilterBar({ columns, filters, onApply, onClear }: TableFilterBarProps) {
  const [rows, setRows] = useState<TableFilter[]>(() =>
    filters.length > 0 ? filters : [emptyRow(columns[0] ?? "")]
  );

  useEffect(() => {
    if (filters.length > 0) setRows(filters);
  }, [filters]);

  function isValidRow(row: TableFilter): boolean {
    return !!row.column && (!needsValue(row.operator) || (row.value ?? "").trim() !== "");
  }

  function applyRows(next: TableFilter[]) {
    onApply(next.filter(isValidRow));
  }

  function updateRow(index: number, patch: Partial<TableFilter>) {
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
            title="Remove filter"
          >
            <X className="size-3.5" />
          </button>

          <span className="w-9 shrink-0 text-xs text-muted-foreground">
            {index === 0 ? "where" : "and"}
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
            value={row.operator}
            onValueChange={(value) => {
              const operator = value as FilterOperator;
              updateRow(index, { operator, value: needsValue(operator) ? row.value : null });
            }}
          >
            <SelectTrigger size="sm" className="h-7 w-[130px] text-xs">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {OPERATORS.map((op) => (
                <SelectItem key={op} value={op}>
                  {OPERATOR_LABELS[op]}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>

          <Input
            value={row.value ?? ""}
            onChange={(e) => updateRow(index, { value: e.target.value })}
            onKeyDown={(e) => e.key === "Enter" && applyRows(rows)}
            placeholder={valuePlaceholder(row.operator)}
            disabled={!needsValue(row.operator)}
            className="h-7 w-[180px] text-xs"
          />

          <button
            type="button"
            onClick={() => applyRows(rows)}
            className="flex size-7 shrink-0 items-center justify-center rounded-md text-muted-foreground hover:bg-muted"
            title="Apply filters"
          >
            <Search className="size-3.5" />
          </button>

          {index === rows.length - 1 && (
            <button
              type="button"
              onClick={addRow}
              className="flex size-7 items-center justify-center rounded-md text-muted-foreground hover:bg-muted"
              title="Add filter"
            >
              <Plus className="size-3.5" />
            </button>
          )}
        </div>
      ))}

      <div>
        <Button variant="ghost" size="xs" onClick={onClear}>
          Clear filters
        </Button>
      </div>
    </div>
  );
}
