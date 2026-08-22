import { useEffect, useState } from "react";
import {
  AlertTriangle,
  Check,
  ChevronLeft,
  ChevronRight,
  Loader2,
  Plus,
  RefreshCw,
  Trash2,
  X,
  XCircle,
} from "lucide-react";

import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogMedia,
  AlertDialogTitle,
} from "@/src/app/components/ui/alert-dialog";
import { Button } from "@/src/app/components/ui/button";
import { toast } from "@/src/app/components/ui/toast";
import { Input } from "@/src/app/components/ui/input";
import { CopyButton } from "@/src/components/CopyButton";
import { ExportButton } from "@/src/components/ExportButton";
import { DataGrid, type JsonCellMode, type RowEdit } from "@/src/features/tables/components/DataGrid";
import { SchemaGraphView } from "@/src/features/schema/components/SchemaGraphView";
import { TableStructureView } from "@/src/features/schema/components/TableStructureView";
import { JsonInspectorSheet } from "@/src/features/tables/components/JsonViewer/JsonInspectorSheet";
import type { JsonValue } from "@/src/features/tables/components/JsonViewer/types";
import { TableFilterBar } from "@/src/features/tables/components/TableFilterBar";
import { TableSortBar } from "@/src/features/tables/components/TableSortBar";
import { TableToolbar } from "@/src/features/tables/components/TableToolbar";
import {
  countTableRows,
  deleteRows,
  fetchTableRows,
  insertRow,
  updateCellText,
  type CellValue,
  type TableFilter,
  type TableRowsResult,
  type TableSort,
} from "@/src/features/tables/api";
import type { TableTab } from "@/src/features/tables/types";
import { toErrorMessage } from "@/src/lib/tauri/errors";

interface TableViewProps {
  tab: TableTab;
}

interface JsonSheetState {
  connectionId: string;
  schema: string;
  table: string;
  row: Record<string, CellValue>;
  columnName: string;
  value: JsonValue;
  mode: JsonCellMode;
}

const DEFAULT_PAGE_SIZE = 200;
const MAX_PAGE_SIZE = 10000;

type ViewMode = "data" | "structure" | "schema";

export function TableView({ tab }: TableViewProps) {
  const [viewMode, setViewMode] = useState<ViewMode>("data");
  const [page, setPage] = useState(0);
  const [pageSize, setPageSize] = useState(DEFAULT_PAGE_SIZE);
  const [pageSizeInput, setPageSizeInput] = useState(String(DEFAULT_PAGE_SIZE));
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<TableRowsResult | null>(null);
  const [jsonSheet, setJsonSheet] = useState<JsonSheetState | null>(null);
  const [pendingEdit, setPendingEdit] = useState<RowEdit | null>(null);
  const [saving, setSaving] = useState(false);
  const [saveError, setSaveError] = useState<string | null>(null);
  const [selectedRowIndices, setSelectedRowIndices] = useState<Set<number>>(new Set());
  const [confirmDeleteOpen, setConfirmDeleteOpen] = useState(false);
  const [rowsPendingDelete, setRowsPendingDelete] = useState<number[]>([]);
  const [deleting, setDeleting] = useState(false);
  const [deleteError, setDeleteError] = useState<string | null>(null);
  const [filters, setFilters] = useState<TableFilter[]>([]);
  const [sort, setSort] = useState<TableSort[]>([]);
  const [isInsertingRow, setIsInsertingRow] = useState(false);
  const [hiddenColumns, setHiddenColumns] = useState<Set<string>>(new Set());
  const [showFilters, setShowFilters] = useState(false);
  const [showSort, setShowSort] = useState(false);
  const [totalRowCount, setTotalRowCount] = useState<number | null>(null);

  useEffect(() => {
    setPage(0);
    setFilters([]);
    setSort([]);
    setHiddenColumns(new Set());
    setShowFilters(false);
    setShowSort(false);
    setTotalRowCount(null);
    resetPendingState();
  }, [tab.id]);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(null);

    fetchTableRows(tab.connectionId, tab.schema, tab.table, pageSize, page * pageSize, filters, sort)
      .then((res) => {
        if (!cancelled) setResult(res);
      })
      .catch((err) => {
        if (!cancelled) setError(toErrorMessage(err));
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });

    return () => {
      cancelled = true;
    };
  }, [tab.connectionId, tab.schema, tab.table, page, pageSize, filters, sort]);

  useEffect(() => {
    let cancelled = false;

    countTableRows(tab.connectionId, tab.schema, tab.table, filters)
      .then((count) => {
        if (!cancelled) setTotalRowCount(count);
      })
      .catch(() => {
        if (!cancelled) setTotalRowCount(null);
      });

    return () => {
      cancelled = true;
    };
  }, [tab.connectionId, tab.schema, tab.table, filters]);

  function resetPendingState() {
    setPendingEdit(null);
    setSaveError(null);
    setSelectedRowIndices(new Set());
    setDeleteError(null);
    setIsInsertingRow(false);
  }

  function refresh() {
    setError(null);
    setLoading(true);
    resetPendingState();
    fetchTableRows(tab.connectionId, tab.schema, tab.table, pageSize, page * pageSize, filters, sort)
      .then(setResult)
      .catch((err) => setError(toErrorMessage(err)))
      .finally(() => setLoading(false));
  }

  function changePage(next: number) {
    resetPendingState();
    setPage(next);
  }

  function changePageSize(next: number) {
    resetPendingState();
    setPageSize(next);
    setPageSizeInput(String(next));
    setPage(0);
  }

  function commitPageSizeInput() {
    const next = Number(pageSizeInput);
    if (!Number.isFinite(next) || next < 1) {
      setPageSizeInput(String(pageSize));
      return;
    }
    const clamped = Math.min(Math.trunc(next), MAX_PAGE_SIZE);
    if (clamped === pageSize) {
      setPageSizeInput(String(pageSize));
      return;
    }
    changePageSize(clamped);
  }

  function applyFilters(next: TableFilter[]) {
    resetPendingState();
    setFilters(next);
    setPage(0);
  }

  function handleHeaderSortClick(column: string, direction: "asc" | "desc") {
    resetPendingState();
    setSort([{ column, direction }]);
    setPage(0);
  }

  function applySort(next: TableSort[]) {
    resetPendingState();
    setSort(next);
    setPage(0);
  }

  function clearFilters() {
    resetPendingState();
    setFilters([]);
    setShowFilters(false);
    setPage(0);
  }

  function clearSort() {
    resetPendingState();
    setSort([]);
    setShowSort(false);
    setPage(0);
  }

  function applyHiddenColumns(next: Set<string>) {
    setHiddenColumns(next);
  }

  function handleAddRow() {
    if (!result) return;
    resetPendingState();
    setIsInsertingRow(true);
    setPendingEdit({ rowIndex: 0, row: result.columns.map(() => null), values: {} });
  }

  async function handleSaveEdit() {
    if (!pendingEdit || !result) return;
    const changedColumns = Object.keys(pendingEdit.values);
    if (changedColumns.length === 0) {
      setPendingEdit(null);
      setIsInsertingRow(false);
      return;
    }

    setSaving(true);
    setSaveError(null);
    try {
      if (isInsertingRow) {
        const values: Record<string, CellValue> = {};
        for (const columnName of changedColumns) {
          values[columnName] = pendingEdit.values[columnName];
        }
        await insertRow(tab.connectionId, tab.schema, tab.table, values);
        toast.add({ type: "success", title: "Row inserted", description: `1 row added to ${tab.table}` });
      } else {
        const rowObject = rowToObject(result.columns, pendingEdit.row);
        for (const columnName of changedColumns) {
          await updateCellText(
            tab.connectionId,
            tab.schema,
            tab.table,
            rowObject,
            columnName,
            pendingEdit.values[columnName]
          );
        }
        toast.add({
          type: "success",
          title: "Row updated",
          description: `${changedColumns.length} field${changedColumns.length === 1 ? "" : "s"} updated in ${tab.table}`,
        });
      }
      setPendingEdit(null);
      setIsInsertingRow(false);
      refresh();
    } catch (err) {
      setSaveError(toErrorMessage(err));
    } finally {
      setSaving(false);
    }
  }

  function handleDiscardEdit() {
    setPendingEdit(null);
    setIsInsertingRow(false);
    setSaveError(null);
  }

  async function handleConfirmDelete() {
    if (!result || rowsPendingDelete.length === 0) return;
    setDeleting(true);
    setDeleteError(null);
    try {
      const rowsToDelete = rowsPendingDelete.map((i) => rowToObject(result.columns, result.rows[i]));
      const affected = await deleteRows(tab.connectionId, tab.schema, tab.table, rowsToDelete);
      setConfirmDeleteOpen(false);
      setRowsPendingDelete([]);
      toast.add({
        type: "success",
        title: "Rows deleted",
        description: `${affected} row${affected === 1 ? "" : "s"} deleted from ${tab.table}`,
      });
      refresh();
    } catch (err) {
      setDeleteError(toErrorMessage(err));
    } finally {
      setDeleting(false);
    }
  }

  function handleDeleteSelected() {
    setDeleteError(null);
    setRowsPendingDelete([...selectedRowIndices]);
    setConfirmDeleteOpen(true);
  }

  function handleDeleteRow(rowIndex: number) {
    setDeleteError(null);
    setRowsPendingDelete([rowIndex]);
    setConfirmDeleteOpen(true);
  }

  const selectedCount = selectedRowIndices.size;
  const pendingFieldCount = pendingEdit ? Object.keys(pendingEdit.values).length : 0;

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="flex shrink-0 items-center justify-between border-b px-3 py-1.5">
        <div className="flex items-center gap-3 text-xs text-muted-foreground">
          <span className="font-medium text-foreground">
            {tab.schema}.{tab.table}
          </span>
          <div className="flex items-center gap-1 rounded-lg border p-0.5">
            <Button
              variant={viewMode === "data" ? "secondary" : "ghost"}
              size="xs"
              onClick={() => setViewMode("data")}
            >
              Data
            </Button>
            <Button
              variant={viewMode === "structure" ? "secondary" : "ghost"}
              size="xs"
              onClick={() => setViewMode("structure")}
            >
              Structure
            </Button>
            <Button
              variant={viewMode === "schema" ? "secondary" : "ghost"}
              size="xs"
              onClick={() => setViewMode("schema")}
            >
              Schema
            </Button>
          </div>
        </div>
        {viewMode === "data" && (
        <div className="flex items-center gap-1">
          {selectedCount > 0 && (
            <Button
              variant="destructive"
              size="xs"
              className="mr-1 gap-1.5"
              onClick={handleDeleteSelected}
              disabled={loading}
            >
              <Trash2 className="size-3.5" />
              Delete {selectedCount}
            </Button>
          )}
          <Button
            variant="outline"
            size="xs"
            className="gap-1.5"
            onClick={handleAddRow}
            disabled={loading || !result || pendingEdit !== null}
          >
            <Plus className="size-3.5" />
            Add Row
          </Button>
          <Button variant="ghost" size="icon-sm" onClick={refresh} disabled={loading}>
            <RefreshCw className={`size-3.5 ${loading ? "animate-spin" : ""}`} />
          </Button>
        </div>
        )}
        {viewMode === "structure" && (
          <ExportButton
            target={{
              kind: "table",
              connectionId: tab.connectionId,
              schema: tab.schema,
              table: tab.table,
              filters: [],
              sort: [],
            }}
            fileBaseName={`${tab.schema}_${tab.table}`}
          />
        )}
      </div>

      {viewMode === "structure" && (
        <TableStructureView
          connectionId={tab.connectionId}
          engine={tab.engine}
          schema={tab.schema}
          table={tab.table}
        />
      )}

      {viewMode === "schema" && (
        <SchemaGraphView connectionId={tab.connectionId} schema={tab.schema} />
      )}

      {viewMode === "data" && (
      <>
      <TableToolbar
        columns={result?.columns ?? []}
        filters={filters}
        showFilters={showFilters}
        onToggleFilters={() => setShowFilters((prev) => !prev)}
        sort={sort}
        showSort={showSort}
        onToggleSort={() => setShowSort((prev) => !prev)}
        hiddenColumns={hiddenColumns}
        onHiddenColumnsChange={applyHiddenColumns}
      />
      {showFilters && (
        <TableFilterBar
          columns={result?.columns ?? []}
          filters={filters}
          onApply={applyFilters}
          onClear={clearFilters}
        />
      )}
      {showSort && (
        <TableSortBar
          columns={result?.columns ?? []}
          sort={sort}
          onApply={applySort}
          onClear={clearSort}
        />
      )}

      {pendingEdit !== null && (
        <div className="relative z-20 flex shrink-0 items-center justify-between gap-3 border-b bg-amber-500/10 px-3 py-1.5">
          <span className="text-xs text-amber-600 dark:text-amber-400">
            {isInsertingRow ? "Adding new row" : `Editing row ${pendingEdit.rowIndex + 1}`}
            {pendingFieldCount > 0 &&
              ` — ${pendingFieldCount} field${pendingFieldCount === 1 ? "" : "s"} ${isInsertingRow ? "set" : "changed"}`}
          </span>
          <div className="flex items-center gap-1.5">
            <Button variant="ghost" size="xs" className="gap-1" onClick={handleDiscardEdit} disabled={saving}>
              <X className="size-3.5" />
              Discard
            </Button>
            <Button
              size="xs"
              className="gap-1"
              onClick={handleSaveEdit}
              disabled={saving || pendingFieldCount === 0}
            >
              {saving ? <Loader2 className="size-3.5 animate-spin" /> : <Check className="size-3.5" />}
              Save
            </Button>
          </div>
        </div>
      )}

      {saveError && (
        <div className="mx-3 mt-2 flex items-start gap-2 rounded-lg border border-destructive/30 bg-destructive/10 px-3 py-2 text-xs text-destructive">
          <XCircle className="size-3.5 shrink-0 translate-y-0.5" />
          <span className="wrap-break-word">{saveError}</span>
        </div>
      )}

      <div className="relative min-h-0 flex-1">
        {loading && result && (
          <div className="absolute inset-x-0 top-0 z-10 h-0.5 overflow-hidden bg-primary/20">
            <div className="h-full w-1/3 animate-[loading-bar_1s_ease-in-out_infinite] bg-primary" />
          </div>
        )}

        {loading && !result && (
          <div className="flex h-full items-center justify-center text-muted-foreground">
            <Loader2 className="size-5 animate-spin" />
          </div>
        )}

        {error && (
          <div className="flex h-full flex-col items-center justify-center gap-2 p-6 text-center">
            <XCircle className="size-6 text-destructive" />
            <p className="max-w-md text-sm text-destructive">{error}</p>
            <Button variant="outline" size="sm" onClick={refresh}>
              Retry
            </Button>
          </div>
        )}

        {result && !error && result.rowCount === 0 && !isInsertingRow && (
          <div className="flex h-full items-center justify-center text-sm text-muted-foreground">
            This table has no rows.
          </div>
        )}

        {result && !error && (result.rowCount > 0 || isInsertingRow) && (
          <DataGrid
            columns={result.columns}
            rows={isInsertingRow ? [result.columns.map(() => null), ...result.rows] : result.rows}
            editable
            pendingEdit={pendingEdit}
            onPendingEditChange={(edit) => {
              setSaveError(null);
              setPendingEdit(edit);
            }}
            saving={saving}
            selectable
            selectedRowIndices={
              isInsertingRow ? new Set([...selectedRowIndices].map((i) => i + 1)) : selectedRowIndices
            }
            onSelectionChange={(indices) =>
              setSelectedRowIndices(
                isInsertingRow
                  ? new Set([...indices].filter((i) => i > 0).map((i) => i - 1))
                  : indices
              )
            }
            onOpenJsonCell={(columnName, value, row, mode) => {
              setJsonSheet({
                connectionId: tab.connectionId,
                schema: tab.schema,
                table: tab.table,
                row: rowToObject(result.columns, row),
                columnName,
                value,
                mode,
              });
            }}
            onDeleteRow={(rowIndex) => {
              if (isInsertingRow) {
                if (rowIndex === 0) return;
                handleDeleteRow(rowIndex - 1);
              } else {
                handleDeleteRow(rowIndex);
              }
            }}
            sortColumn={sort[0]?.column ?? null}
            sortDirection={sort[0]?.direction ?? null}
            onSortChange={handleHeaderSortClick}
            hiddenColumns={hiddenColumns}
          />
        )}
      </div>

      <div className="relative flex shrink-0 items-center justify-between gap-3 border-t px-3 py-1.5">
        <div className="flex items-center gap-2">
          <Button
            variant="ghost"
            size="icon-sm"
            onClick={() => changePage(Math.max(0, page - 1))}
            disabled={page === 0 || loading}
          >
            <ChevronLeft className="size-3.5" />
          </Button>
          <span className="text-xs text-muted-foreground">Page {page + 1}</span>
          <Button
            variant="ghost"
            size="icon-sm"
            onClick={() => changePage(page + 1)}
            disabled={!result?.hasMore || loading}
          >
            <ChevronRight className="size-3.5" />
          </Button>
          <div className="flex items-center gap-1.5">
            <Input
              type="number"
              draggable={false}
              min={1}
              max={MAX_PAGE_SIZE}
              value={pageSizeInput}
              disabled={loading}
              onChange={(e) => setPageSizeInput(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && commitPageSizeInput()}
              onBlur={commitPageSizeInput}
              className="h-7 w-16 text-xs [appearance:textfield] [&::-webkit-inner-spin-button]:appearance-none [&::-webkit-outer-spin-button]:appearance-none"
            />
            <span className="text-xs text-muted-foreground">rows / page</span>
          </div>
        </div>

        {result && (
          <div className="absolute left-1/2 flex -translate-x-1/2 items-center gap-2 text-xs text-muted-foreground">
            <span>
              {result.rowCount.toLocaleString()}
              {totalRowCount !== null && ` / ${totalRowCount.toLocaleString()}`} rows
            </span>
            <span>{result.durationMs}ms</span>
          </div>
        )}

        <div className="flex items-center gap-1.5">
          <CopyButton
            columns={result?.columns ?? []}
            rows={result?.rows ?? []}
            selectedRowIndices={selectedRowIndices}
            disabled={loading && !result}
          />
          <ExportButton
            target={{
              kind: "table",
              connectionId: tab.connectionId,
              schema: tab.schema,
              table: tab.table,
              filters,
              sort,
              page: result
                ? {
                    columns: result.columns,
                    rows: result.rows.map((row) => row.map((cell) => JSON.stringify(cell))),
                  }
                : undefined,
            }}
            fileBaseName={`${tab.schema}_${tab.table}`}
            disabled={loading && !result}
          />
        </div>
      </div>
      </>
      )}

      <JsonInspectorSheet
        open={jsonSheet !== null}
        onOpenChange={(open) => !open && setJsonSheet(null)}
        target={jsonSheet}
        onSaved={refresh}
      />

      <AlertDialog
        open={confirmDeleteOpen}
        onOpenChange={(open) => !deleting && setConfirmDeleteOpen(open)}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogMedia className="bg-destructive/10 text-destructive">
              <AlertTriangle />
            </AlertDialogMedia>
            <AlertDialogTitle>
              Delete {rowsPendingDelete.length} row{rowsPendingDelete.length === 1 ? "" : "s"}?
            </AlertDialogTitle>
            <AlertDialogDescription>
              This permanently deletes {rowsPendingDelete.length} row
              {rowsPendingDelete.length === 1 ? "" : "s"} from{" "}
              <strong>
                {tab.schema}.{tab.table}
              </strong>
              . This cannot be undone.
            </AlertDialogDescription>
          </AlertDialogHeader>

          {deleteError && (
            <div className="flex items-start gap-2 rounded-lg border border-destructive/30 bg-destructive/10 px-3 py-2 text-xs text-destructive">
              <XCircle className="size-3.5 shrink-0 translate-y-0.5" />
              <span className="wrap-break-word">{deleteError}</span>
            </div>
          )}

          <AlertDialogFooter>
            <AlertDialogCancel disabled={deleting}>Cancel</AlertDialogCancel>
            <AlertDialogAction
              variant="destructive"
              onClick={handleConfirmDelete}
              disabled={deleting}
              className="gap-1.5"
            >
              {deleting && <Loader2 className="size-3.5 animate-spin" />}
              Delete
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}

function rowToObject(columns: string[], row: CellValue[]): Record<string, CellValue> {
  const obj: Record<string, CellValue> = {};
  columns.forEach((col, i) => {
    obj[col] = row[i];
  });
  return obj;
}
