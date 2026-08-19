import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import DataEditor, {
  CompactSelection,
  getDefaultTheme,
  GridCellKind,
  type CellClickedEventArgs,
  type EditableGridCell,
  type GridCell,
  type GridColumn,
  type GridSelection,
  type Item,
  type Theme,
} from "@glideapps/glide-data-grid";

import type { CellValue } from "@/src/features/tables/api";
import type { JsonValue } from "@/src/features/tables/components/JsonViewer/types";
import { RowContextMenu } from "@/src/features/tables/components/DataGrid/RowContextMenu";

export type JsonCellMode = "view" | "edit";

/** Pending, unsaved edits for a single row, keyed by column name. */
export interface RowEdit {
  rowIndex: number;
  row: CellValue[];
  values: Record<string, string | null>;
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
  /** Row currently unlocked for whole-row inline editing (via "Edit Row"). */
  editingRowIndex?: number | null;
  pendingEdit: RowEdit | null;
  onPendingEditChange: (edit: RowEdit | null) => void;
  saving?: boolean;
  selectable?: boolean;
  selectedRowIndices?: Set<number>;
  onSelectionChange?: (indices: Set<number>) => void;
  onDeleteRow?: (rowIndex: number) => void;
}

const ROW_HEIGHT = 32;
const DEFAULT_COLUMN_WIDTH = 180;
const MIN_COLUMN_WIDTH = 80;
const MAX_COLUMN_WIDTH = 400;
const MENU_COLUMN_ID = "__row_menu__";
const MENU_COLUMN_WIDTH = 36;

function cellValueEquals(original: CellValue, newValue: string | null): boolean {
  if (newValue === null) return original === null || original === undefined;
  if (original === null || original === undefined) return false;
  return String(original) === newValue;
}

function cellDisplayString(value: CellValue): string {
  if (value === null || value === undefined) return "";
  if (typeof value === "object") return JSON.stringify(value);
  if (typeof value === "boolean") return value ? "true" : "false";
  return String(value);
}

function estimateColumnWidth(columnName: string, sampleRows: CellValue[][], columnIndex: number): number {
  let maxLength = columnName.length;
  const sampleSize = Math.min(sampleRows.length, 30);
  for (let i = 0; i < sampleSize; i++) {
    const text = cellDisplayString(sampleRows[i]?.[columnIndex]);
    if (text.length > maxLength) maxLength = text.length;
  }
  const estimated = 16 + maxLength * 7;
  return Math.min(MAX_COLUMN_WIDTH, Math.max(MIN_COLUMN_WIDTH, estimated || DEFAULT_COLUMN_WIDTH));
}

const queryonLightTheme: Partial<Theme> = {
  accentColor: "#171717",
  accentFg: "#FAFAFA",
  accentLight: "rgba(23, 23, 23, 0.1)",
  textDark: "#0A0A0A",
  textMedium: "#737373",
  textLight: "#737373",
  textBubble: "#0A0A0A",
  bgIconHeader: "#737373",
  fgIconHeader: "#FFFFFF",
  textHeader: "#737373",
  textHeaderSelected: "#FFFFFF",
  bgCell: "#FFFFFF",
  bgCellMedium: "#F5F5F5",
  bgHeader: "#FFFFFF",
  bgHeaderHasFocus: "#F5F5F5",
  bgHeaderHovered: "#F5F5F5",
  bgBubble: "#F5F5F5",
  bgBubbleSelected: "#FFFFFF",
  bgSearchResult: "#fff3c4",
  borderColor: "#E5E5E5",
  drilldownBorder: "#E5E5E5",
  linkColor: "#171717",
};

const queryonDarkTheme: Partial<Theme> = {
  accentColor: "#E5E5E5",
  accentFg: "#171717",
  accentLight: "rgba(229, 229, 229, 0.15)",
  textDark: "#FAFAFA",
  textMedium: "#A1A1A1",
  textLight: "#A1A1A1",
  textBubble: "#FAFAFA",
  bgIconHeader: "#A1A1A1",
  fgIconHeader: "#0A0A0A",
  textHeader: "#A1A1A1",
  textHeaderSelected: "#0A0A0A",
  bgCell: "#0A0A0A",
  bgCellMedium: "#171717",
  bgHeader: "#0A0A0A",
  bgHeaderHasFocus: "#262626",
  bgHeaderHovered: "#262626",
  bgBubble: "#262626",
  bgBubbleSelected: "#171717",
  bgSearchResult: "#423c24",
  borderColor: "rgba(255, 255, 255, 0.1)",
  drilldownBorder: "rgba(255, 255, 255, 0.2)",
  linkColor: "#E5E5E5",
};

function usePrefersDark(): boolean {
  const [isDark, setIsDark] = useState(
    () => typeof document !== "undefined" && document.documentElement.classList.contains("dark")
  );

  useEffect(() => {
    const root = document.documentElement;
    const observer = new MutationObserver(() => setIsDark(root.classList.contains("dark")));
    observer.observe(root, { attributes: true, attributeFilter: ["class"] });
    return () => observer.disconnect();
  }, []);

  return isDark;
}

function useGlideTheme(): Partial<Theme> {
  const isDark = usePrefersDark();
  return useMemo(
    () => ({ ...getDefaultTheme(), ...(isDark ? queryonDarkTheme : queryonLightTheme) }),
    [isDark]
  );
}

export function DataGrid({
  columns,
  rows,
  onOpenJsonCell,
  editable,
  editingRowIndex = null,
  pendingEdit,
  onPendingEditChange,
  saving,
  selectable,
  selectedRowIndices,
  onSelectionChange,
  onDeleteRow,
}: DataGridProps) {
  const theme = useGlideTheme();
  const containerRef = useRef<HTMLDivElement>(null);
  const [columnWidths, setColumnWidths] = useState<Map<string, number>>(new Map());
  const [contextMenu, setContextMenu] = useState<{ rowIndex: number; x: number; y: number } | null>(null);

  const gridColumns: GridColumn[] = useMemo(() => {
    const menuColumn: GridColumn = {
      id: MENU_COLUMN_ID,
      title: "",
      width: MENU_COLUMN_WIDTH,
      hasMenu: false,
      themeOverride: { bgCell: theme.bgHeader },
    };
    const dataColumns = columns.map((name, index) => ({
      id: name,
      title: name,
      width: columnWidths.get(name) ?? estimateColumnWidth(name, rows.slice(0, 30), index),
      hasMenu: false,
    }));
    return [menuColumn, ...dataColumns];
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [columns, columnWidths, theme.bgHeader]);

  function rowValue(rowIndex: number, columnIndex: number): CellValue {
    const columnName = columns[columnIndex];
    if (pendingEdit !== null && pendingEdit.rowIndex === rowIndex && columnName in pendingEdit.values) {
      return pendingEdit.values[columnName];
    }
    return rows[rowIndex]?.[columnIndex];
  }

  const getCellContent = useCallback(
    ([gridCol, row]: Item): GridCell => {
      if (gridCol === 0) {
        return {
          kind: GridCellKind.Text,
          data: "",
          displayData: "⋮⋮",
          allowOverlay: false,
          readonly: true,
          themeOverride: { textDark: theme.textLight, baseFontStyle: "12px" },
        };
      }

      const col = gridCol - 1;
      const value = rowValue(row, col);
      const columnName = columns[col];
      const isJson = value !== null && value !== undefined && typeof value === "object";
      const isRowEditing =
        editingRowIndex === row || (pendingEdit !== null && pendingEdit.rowIndex === row);
      const isFieldPending =
        pendingEdit !== null && pendingEdit.rowIndex === row && columnName in pendingEdit.values;

      if (isJson) {
        return {
          kind: GridCellKind.Text,
          data: JSON.stringify(value),
          displayData: JSON.stringify(value),
          allowOverlay: false,
          readonly: true,
          themeOverride: { textDark: theme.linkColor, baseFontStyle: "12px monospace" },
        };
      }

      const isBoolean = typeof value === "boolean";
      const isNull = value === null || value === undefined;
      const display = isNull ? "" : isBoolean ? (value ? "true" : "false") : String(value);
      const activeRow = editingRowIndex ?? pendingEdit?.rowIndex ?? null;
      const canEditCell = !!editable && !saving && (activeRow === null || activeRow === row);

      return {
        kind: GridCellKind.Text,
        data: display,
        displayData: isNull ? "NULL" : display,
        allowOverlay: canEditCell,
        readonly: !canEditCell,
        themeOverride: isNull
          ? { textDark: theme.textLight, baseFontStyle: "italic 12px" }
          : isBoolean
            ? { textDark: value ? "#10b981" : theme.textLight }
            : isFieldPending
              ? { bgCell: "rgba(245, 158, 11, 0.15)", textDark: "#f59e0b" }
              : isRowEditing
                ? { bgCell: theme.bgCellMedium }
                : undefined,
      };
      // eslint-disable-next-line react-hooks/exhaustive-deps
    },
    [rows, columns, pendingEdit, editingRowIndex, editable, saving, theme]
  );

  function handleCellEdited([gridCol, row]: Item, newCell: EditableGridCell) {
    if (gridCol === 0 || newCell.kind !== GridCellKind.Text) return;

    const col = gridCol - 1;
    const columnName = columns[col];
    const originalValue = rows[row]?.[col];
    const newValue = newCell.data === "" ? null : newCell.data;

    const hasPendingForRow = pendingEdit !== null && pendingEdit.rowIndex === row;
    const baseValues = hasPendingForRow ? pendingEdit.values : {};

    if (cellValueEquals(originalValue, newValue)) {
      if (!hasPendingForRow) return;
      const { [columnName]: _removed, ...rest } = baseValues;
      onPendingEditChange(
        Object.keys(rest).length === 0 && editingRowIndex !== row
          ? null
          : { rowIndex: row, row: rows[row], values: rest }
      );
      return;
    }

    onPendingEditChange({
      rowIndex: row,
      row: rows[row],
      values: { ...baseValues, [columnName]: newValue },
    });
  }

  const lastClick = useRef<{ col: number; row: number; time: number } | null>(null);
  const DOUBLE_CLICK_MS = 400;

  function handleCellClicked([gridCol, row]: Item, event: CellClickedEventArgs) {
    if (gridCol === 0) {
      const containerRect = containerRef.current?.getBoundingClientRect();
      if (!containerRect) return;
      setContextMenu({
        rowIndex: row,
        x: event.bounds.x + 20,
        y: event.bounds.y + 20,
      });
      return;
    }

    const col = gridCol - 1;
    const value = rows[row]?.[col];
    if (value === null || value === undefined || typeof value !== "object") {
      lastClick.current = null;
      return; 
    }

    const now = Date.now();
    const isDoubleClick =
      lastClick.current !== null &&
      lastClick.current.col === col &&
      lastClick.current.row === row &&
      now - lastClick.current.time < DOUBLE_CLICK_MS;

    lastClick.current = isDoubleClick ? null : { col, row, time: now };
    onOpenJsonCell?.(columns[col], value as JsonValue, rows[row], isDoubleClick ? "edit" : "view");
  }

  function handleCellContextMenu([, row]: Item, event: CellClickedEventArgs) {
    event.preventDefault();
    const containerRect = containerRef.current?.getBoundingClientRect();
    if (!containerRect) return;

    setContextMenu({
      rowIndex: row,
      x:event.bounds.x + 20,
      y: event.bounds.y + 20,
    });
  }

  async function handleCopyRowAsJson(rowIndex: number) {
    const obj: Record<string, CellValue> = {};
    columns.forEach((col, i) => {
      obj[col] = rows[rowIndex]?.[i] ?? null;
    });
    await navigator.clipboard.writeText(JSON.stringify(obj, null, 2));
  }

  function handleColumnResize(column: GridColumn, newSize: number) {
    if (!column.id || column.id === MENU_COLUMN_ID) return;
    setColumnWidths((prev) => {
      const next = new Map(prev);
      next.set(column.id as string, newSize);
      return next;
    });
  }

  const [cellSelection, setCellSelection] = useState<GridSelection["current"]>(undefined);

  const gridSelection: GridSelection = useMemo(() => {
    let rowsSelection = CompactSelection.empty();
    if (selectedRowIndices) {
      for (const index of selectedRowIndices) {
        rowsSelection = rowsSelection.add(index);
      }
    }
    return { columns: CompactSelection.empty(), rows: rowsSelection, current: cellSelection };
  }, [selectedRowIndices, cellSelection]);

  function handleGridSelectionChange(newSelection: GridSelection) {
    setCellSelection(newSelection.current);
    if (!onSelectionChange) return;
    const indices = new Set<number>();
    for (const range of newSelection.rows) {
      indices.add(range);
    }
    onSelectionChange(indices);
  }

  return (
    <div ref={containerRef} className="relative z-0 h-full">
      <DataEditor
        columns={gridColumns}
        rows={rows.length}
        getCellContent={getCellContent}
        onCellEdited={editable ? handleCellEdited : undefined}
        onCellClicked={handleCellClicked}
        onCellContextMenu={handleCellContextMenu}
        onColumnResize={handleColumnResize}
        rowHeight={ROW_HEIGHT}
        headerHeight={ROW_HEIGHT}
        cellActivationBehavior="double-click"
        theme={theme}
        smoothScrollX
        smoothScrollY
        freezeColumns={1}
        rowMarkers={selectable ? "checkbox" : "none"}
        rowSelectionMode="multi"
        rowSelect={selectable ? "multi" : "none"}
        gridSelection={selectable ? gridSelection : undefined}
        onGridSelectionChange={selectable ? handleGridSelectionChange : undefined}
        getCellsForSelection
        width="100%"
        height="100%"
      />

      {contextMenu && (
        <RowContextMenu
          x={contextMenu.x}
          y={contextMenu.y}
          onClose={() => setContextMenu(null)}
          onCopyAsJson={() => {
            handleCopyRowAsJson(contextMenu.rowIndex);
            setContextMenu(null);
          }}
          onDelete={
            onDeleteRow
              ? () => {
                  onDeleteRow(contextMenu.rowIndex);
                  setContextMenu(null);
                }
              : undefined
          }
        />
      )}
    </div>
  );
}
