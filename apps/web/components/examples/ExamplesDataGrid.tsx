"use client";

import { useEffect, useMemo, useState } from "react";
import DataEditor, {
  getDefaultTheme,
  GridCellKind,
  type GridCell,
  type GridColumn,
  type Item,
  type Theme,
} from "@glideapps/glide-data-grid";
import "@glideapps/glide-data-grid/dist/index.css";

const MIN_COLUMN_WIDTH = 90;
const MAX_COLUMN_WIDTH = 320;
const DEFAULT_COLUMN_WIDTH = 160;

const queryonLightTheme: Partial<Theme> = {
  accentColor: "#15803d",
  accentFg: "#FAFAFA",
  accentLight: "rgba(34, 197, 94, 0.12)",
  textDark: "#0b100d",
  textMedium: "#5f6b64",
  textLight: "#5f6b64",
  textBubble: "#0A0A0A",
  bgIconHeader: "#5f6b64",
  fgIconHeader: "#ffffff",
  textHeader: "#5f6b64",
  textHeaderSelected: "#ffffff",
  bgCell: "#FFFFFF",
  bgCellMedium: "#f3f6f4",
  bgHeader: "#f7faf8",
  bgHeaderHasFocus: "#eef3f0",
  bgHeaderHovered: "#eef3f0",
  bgBubble: "#eef3f0",
  bgBubbleSelected: "#FFFFFF",
  bgSearchResult: "#fff3c4",
  borderColor: "#e3e9e5",
  drilldownBorder: "#e3e9e5",
  linkColor: "#15803d",
};

const queryonDarkTheme: Partial<Theme> = {
  accentColor: "#4ade80",
  accentFg: "#04140a",
  accentLight: "rgba(74, 222, 128, 0.14)",
  textDark: "#FAFAFA",
  textMedium: "#A1A1A1",
  textLight: "#A1A1A1",
  textBubble: "#FAFAFA",
  bgIconHeader: "#A1A1A1",
  fgIconHeader: "#0A0A0A",
  textHeader: "#A1A1A1",
  textHeaderSelected: "#04140a",
  bgCell: "#070a08",
  bgCellMedium: "#0d1310",
  bgHeader: "#0d1310",
  bgHeaderHasFocus: "#16201a",
  bgHeaderHovered: "#16201a",
  bgBubble: "#16201a",
  bgBubbleSelected: "#0d1310",
  bgSearchResult: "#423c24",
  borderColor: "rgba(255, 255, 255, 0.1)",
  drilldownBorder: "rgba(255, 255, 255, 0.2)",
  linkColor: "#4ade80",
};

function usePrefersDark(): boolean {
  const [isDark, setIsDark] = useState(
    () =>
      typeof document !== "undefined" &&
      document.documentElement.classList.contains("dark"),
  );

  useEffect(() => {
    const root = document.documentElement;
    const observer = new MutationObserver(() =>
      setIsDark(root.classList.contains("dark")),
    );
    observer.observe(root, { attributes: true, attributeFilter: ["class"] });
    return () => observer.disconnect();
  }, []);

  return isDark;
}

function estimateColumnWidth(
  columnName: string,
  sampleRows: (string | number)[][],
  columnIndex: number,
) {
  let maxLength = columnName.length;
  const sampleSize = Math.min(sampleRows.length, 30);
  for (let i = 0; i < sampleSize; i++) {
    const text = String(sampleRows[i]?.[columnIndex] ?? "");
    if (text.length > maxLength) maxLength = text.length;
  }
  const estimated = 16 + maxLength * 7;
  return Math.min(
    MAX_COLUMN_WIDTH,
    Math.max(MIN_COLUMN_WIDTH, estimated || DEFAULT_COLUMN_WIDTH),
  );
}

export interface ExamplesDataGridProps {
  columns: string[];
  rows: (string | number)[][];
  height?: number;
}

export function ExamplesDataGrid({
  columns,
  rows,
  height = 460,
}: ExamplesDataGridProps) {
  const isDark = usePrefersDark();
  const theme = useMemo(
    () => ({
      ...getDefaultTheme(),
      ...(isDark ? queryonDarkTheme : queryonLightTheme),
    }),
    [isDark],
  );

  const gridColumns: GridColumn[] = useMemo(
    () =>
      columns.map((name, index) => ({
        id: name,
        title: name,
        width: estimateColumnWidth(name, rows, index),
      })),
    [columns, rows],
  );

  const getCellContent = ([col, row]: Item): GridCell => {
    const value = rows[row]?.[col] ?? "";
    const display = String(value);
    return {
      kind: GridCellKind.Text,
      data: display,
      displayData: display,
      allowOverlay: false,
      readonly: true,
    };
  };

  return (
    <div style={{ height }} className="w-full overflow-hidden">
      <DataEditor
        columns={gridColumns}
        rows={rows.length}
        getCellContent={getCellContent}
        theme={theme}
        rowHeight={32}
        headerHeight={34}
        smoothScrollX
        smoothScrollY
        width="100%"
        height={height}
      />
    </div>
  );
}
