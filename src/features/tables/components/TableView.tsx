import { useEffect, useState } from "react";
import {
  AlertTriangle,
  Check,
  ChevronLeft,
  ChevronRight,
  Loader2,
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
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/src/app/components/ui/select";
import { ExportButton } from "@/src/components/ExportButton";
import { DataGrid, type JsonCellMode, type RowEdit } from "@/src/features/tables/components/DataGrid";
import { JsonInspectorSheet } from "@/src/features/tables/components/JsonViewer/JsonInspectorSheet";
import type { JsonValue } from "@/src/features/tables/components/JsonViewer/types";
import {
  deleteRows,
  fetchTableRows,
  updateCellText,
  type CellValue,
  type TableRowsResult,
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

const PAGE_SIZE_OPTIONS = [50, 100, 200, 500];
const DEFAULT_PAGE_SIZE = 200;

export function TableView({ tab }: TableViewProps) {
  const [page, setPage] = useState(0);
  const [pageSize, setPageSize] = useState(DEFAULT_PAGE_SIZE);
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

  useEffect(() => {
    setPage(0);
    resetPendingState();
  }, [tab.id]);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(null);

    fetchTableRows(tab.connectionId, tab.schema, tab.table, pageSize, page * pageSize)
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
  }, [tab.connectionId, tab.schema, tab.table, page, pageSize]);

  function resetPendingState() {
    setPendingEdit(null);
    setSaveError(null);
    setSelectedRowIndices(new Set());
    setDeleteError(null);
  }

  function refresh() {
    setError(null);
    setLoading(true);
    resetPendingState();
    fetchTableRows(tab.connectionId, tab.schema, tab.table, pageSize, page * pageSize)
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
    setPage(0);
  }

  async function handleSaveEdit() {
    if (!pendingEdit || !result) return;
    const changedColumns = Object.keys(pendingEdit.values);
    if (changedColumns.length === 0) {
      setPendingEdit(null);
      return;
    }

    setSaving(true);
    setSaveError(null);
    try {
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
      setPendingEdit(null);
      refresh();
    } catch (err) {
      setSaveError(toErrorMessage(err));
    } finally {
      setSaving(false);
    }
  }

  function handleDiscardEdit() {
    setPendingEdit(null);
    setSaveError(null);
  }

  async function handleConfirmDelete() {
    if (!result || rowsPendingDelete.length === 0) return;
    setDeleting(true);
    setDeleteError(null);
    try {
      const rowsToDelete = rowsPendingDelete.map((i) => rowToObject(result.columns, result.rows[i]));
      await deleteRows(tab.connectionId, tab.schema, tab.table, rowsToDelete);
      setConfirmDeleteOpen(false);
      setRowsPendingDelete([]);
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
        <div className="flex items-center gap-2 text-xs text-muted-foreground">
          <span className="font-medium text-foreground">
            {tab.schema}.{tab.table}
          </span>
        </div>
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
          <Button variant="ghost" size="icon-sm" onClick={refresh} disabled={loading}>
            <RefreshCw className={`size-3.5 ${loading ? "animate-spin" : ""}`} />
          </Button>
        </div>
      </div>

      {pendingEdit !== null && (
        <div className="relative z-20 flex shrink-0 items-center justify-between gap-3 border-b bg-amber-500/10 px-3 py-1.5">
          <span className="text-xs text-amber-600 dark:text-amber-400">
            Editing row {pendingEdit.rowIndex + 1}
            {pendingFieldCount > 0 &&
              ` — ${pendingFieldCount} field${pendingFieldCount === 1 ? "" : "s"} changed`}
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

        {result && !error && result.rowCount === 0 && (
          <div className="flex h-full items-center justify-center text-sm text-muted-foreground">
            This table has no rows.
          </div>
        )}

        {result && !error && result.rowCount > 0 && (
          <DataGrid
            columns={result.columns}
            rows={result.rows}
            editable
            pendingEdit={pendingEdit}
            onPendingEditChange={(edit) => {
              setSaveError(null);
              setPendingEdit(edit);
            }}
            saving={saving}
            selectable
            selectedRowIndices={selectedRowIndices}
            onSelectionChange={setSelectedRowIndices}
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
            onDeleteRow={handleDeleteRow}
          />
        )}
      </div>

      <div className="flex shrink-0 items-center justify-between gap-3 border-t px-3 py-1.5">
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
          <Select
            value={String(pageSize)}
            onValueChange={(value) => changePageSize(Number(value))}
            disabled={loading}
          >
            <SelectTrigger size="sm" className="h-7 w-[130px] text-xs">
              {loading ? (
                <span className="flex items-center gap-1.5 text-muted-foreground">
                  <Loader2 className="size-3 animate-spin" />
                  Loading…
                </span>
              ) : (
                <SelectValue />
              )}
            </SelectTrigger>
            <SelectContent>
              {PAGE_SIZE_OPTIONS.map((size) => (
                <SelectItem key={size} value={String(size)}>
                  {size} rows / page
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        {result && (
          <span className="text-xs text-muted-foreground">
            {result.rowCount} rows on this page · {result.durationMs}ms
          </span>
        )}

        <ExportButton
          target={{ kind: "table", connectionId: tab.connectionId, schema: tab.schema, table: tab.table }}
          fileBaseName={`${tab.schema}_${tab.table}`}
          disabled={loading && !result}
        />
      </div>

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
