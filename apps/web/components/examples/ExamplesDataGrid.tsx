"use client"

import { useEffect, useMemo, useState } from "react"
import DataEditor, {
  getDefaultTheme,
  GridCellKind,
  type GridCell,
  type GridColumn,
  type Item,
  type Theme,
} from "@glideapps/glide-data-grid"
import "@glideapps/glide-data-grid/dist/index.css"

const MIN_COLUMN_WIDTH = 90
const MAX_COLUMN_WIDTH = 320
const DEFAULT_COLUMN_WIDTH = 160

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
  bgHeader: "#FAFAFA",
  bgHeaderHasFocus: "#F5F5F5",
  bgHeaderHovered: "#F5F5F5",
  bgBubble: "#F5F5F5",
  bgBubbleSelected: "#FFFFFF",
  bgSearchResult: "#fff3c4",
  borderColor: "#E5E5E5",
  drilldownBorder: "#E5E5E5",
  linkColor: "#171717",
}

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
  bgHeader: "#171717",
  bgHeaderHasFocus: "#262626",
  bgHeaderHovered: "#262626",
  bgBubble: "#262626",
  bgBubbleSelected: "#171717",
  bgSearchResult: "#423c24",
  borderColor: "rgba(255, 255, 255, 0.1)",
  drilldownBorder: "rgba(255, 255, 255, 0.2)",
  linkColor: "#E5E5E5",
}

function usePrefersDark(): boolean {
  const [isDark, setIsDark] = useState(
    () => typeof document !== "undefined" && document.documentElement.classList.contains("dark")
  )

  useEffect(() => {
    const root = document.documentElement
    const observer = new MutationObserver(() => setIsDark(root.classList.contains("dark")))
    observer.observe(root, { attributes: true, attributeFilter: ["class"] })
    return () => observer.disconnect()
  }, [])

  return isDark
}

function estimateColumnWidth(columnName: string, sampleRows: (string | number)[][], columnIndex: number) {
  let maxLength = columnName.length
  const sampleSize = Math.min(sampleRows.length, 30)
  for (let i = 0; i < sampleSize; i++) {
    const text = String(sampleRows[i]?.[columnIndex] ?? "")
    if (text.length > maxLength) maxLength = text.length
  }
  const estimated = 16 + maxLength * 7
  return Math.min(MAX_COLUMN_WIDTH, Math.max(MIN_COLUMN_WIDTH, estimated || DEFAULT_COLUMN_WIDTH))
}

export interface ExamplesDataGridProps {
  columns: string[]
  rows: (string | number)[][]
  height?: number
}

export function ExamplesDataGrid({ columns, rows, height = 460 }: ExamplesDataGridProps) {
  const isDark = usePrefersDark()
  const theme = useMemo(
    () => ({ ...getDefaultTheme(), ...(isDark ? queryonDarkTheme : queryonLightTheme) }),
    [isDark]
  )

  const gridColumns: GridColumn[] = useMemo(
    () =>
      columns.map((name, index) => ({
        id: name,
        title: name,
        width: estimateColumnWidth(name, rows, index),
      })),
    [columns, rows]
  )

  const getCellContent = ([col, row]: Item): GridCell => {
    const value = rows[row]?.[col] ?? ""
    const display = String(value)
    return {
      kind: GridCellKind.Text,
      data: display,
      displayData: display,
      allowOverlay: false,
      readonly: true,
    }
  }

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
  )
}
