import { useEffect, useState } from "react";
import { AlertTriangle, Check, Copy, Loader2, RotateCcw } from "lucide-react";

import { Button } from "@/src/app/components/ui/button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/src/app/components/ui/tabs";
import { ColumnsTab, editRowFor, newColumnRow } from "@/src/features/schema/components/ColumnsTab";
import { ConstraintsTab, newConstraintRow } from "@/src/features/schema/components/ConstraintsTab";
import { DdlPreviewDialog } from "@/src/features/schema/components/DdlPreviewDialog";
import { IndexesTab, newIndexRow } from "@/src/features/schema/components/IndexesTab";
import {
  listConstraints,
  listIndexes,
  renderDdl,
  type ConstraintInfo,
  type DdlPreview,
  type IndexInfo,
} from "@/src/features/schema/api";
import {
  allValid,
  changeCount,
  emptyChanges,
  hasChanges,
  toStatements,
  type SchemaChanges,
  type StagedColumnEdit,
} from "@/src/features/schema/staging";
import { getTableColumns, type ColumnInfo } from "@/src/features/tables/api";
import { toErrorMessage } from "@/src/lib/tauri/errors";

interface TableStructureViewProps {
  connectionId: string;
  schema: string;
  table: string;
}

export function TableStructureView({ connectionId, schema, table }: TableStructureViewProps) {
  const [columns, setColumns] = useState<ColumnInfo[] | null>(null);
  const [constraints, setConstraints] = useState<ConstraintInfo[] | null>(null);
  const [indexes, setIndexes] = useState<IndexInfo[] | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const [changes, setChanges] = useState<SchemaChanges>(emptyChanges());
  const [previewOpen, setPreviewOpen] = useState(false);
  const [sqlPreview, setSqlPreview] = useState<DdlPreview[] | null>(null);
  const [sqlLoading, setSqlLoading] = useState(false);
  const [sqlError, setSqlError] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  function load() {
    let cancelled = false;
    setLoading(true);
    setError(null);

    Promise.all([
      getTableColumns(connectionId, schema, table),
      listConstraints(connectionId, schema, table),
      listIndexes(connectionId, schema, table),
    ])
      .then(([columnsResult, constraintsResult, indexesResult]) => {
        if (cancelled) return;
        setColumns(columnsResult);
        setConstraints(constraintsResult);
        setIndexes(indexesResult);
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
  }

  useEffect(() => {
    setColumns(null);
    setConstraints(null);
    setIndexes(null);
    setChanges(emptyChanges());
    return load();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [connectionId, schema, table]);

  const pendingChanges = hasChanges(changes);
  const changesValid = allValid(changes);
  const statements = toStatements(table, changes);

  useEffect(() => {
    if (!pendingChanges) {
      setSqlPreview(null);
      setSqlError(null);
      return;
    }
    if (!changesValid) {
      setSqlPreview(null);
      setSqlError("Fill in every staged row to see its SQL — some names, types, or columns are missing.");
      return;
    }

    let cancelled = false;
    setSqlLoading(true);
    setSqlError(null);
    renderDdl(connectionId, schema, statements)
      .then((previews) => {
        if (!cancelled) setSqlPreview(previews);
      })
      .catch((err) => {
        if (!cancelled) setSqlError(toErrorMessage(err));
      })
      .finally(() => {
        if (!cancelled) setSqlLoading(false);
      });

    return () => {
      cancelled = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [pendingChanges, changesValid, connectionId, schema, JSON.stringify(statements)]);

  function toggleDropColumn(column: string) {
    setChanges((prev) => ({
      ...prev,
      droppedColumns: prev.droppedColumns.includes(column)
        ? prev.droppedColumns.filter((c) => c !== column)
        : [...prev.droppedColumns, column],
      editedColumns: prev.editedColumns.filter((e) => e.currentName !== column),
    }));
  }

  function startEditColumn(column: ColumnInfo) {
    setChanges((prev) => ({
      ...prev,
      editedColumns: prev.editedColumns.some((e) => e.currentName === column.name)
        ? prev.editedColumns
        : [...prev.editedColumns, editRowFor(column)],
    }));
  }

  function cancelEditColumn(currentName: string) {
    setChanges((prev) => ({
      ...prev,
      editedColumns: prev.editedColumns.filter((e) => e.currentName !== currentName),
    }));
  }

  function changeEditColumn(currentName: string, patch: Partial<StagedColumnEdit["column"]>) {
    setChanges((prev) => ({
      ...prev,
      editedColumns: prev.editedColumns.map((e) =>
        e.currentName === currentName ? { ...e, column: { ...e.column, ...patch } } : e
      ),
    }));
  }

  function toggleDropIndex(index: string) {
    setChanges((prev) => ({
      ...prev,
      droppedIndexes: prev.droppedIndexes.includes(index)
        ? prev.droppedIndexes.filter((i) => i !== index)
        : [...prev.droppedIndexes, index],
    }));
  }

  function toggleDropConstraint(constraint: string) {
    setChanges((prev) => ({
      ...prev,
      droppedConstraints: prev.droppedConstraints.includes(constraint)
        ? prev.droppedConstraints.filter((c) => c !== constraint)
        : [...prev.droppedConstraints, constraint],
    }));
  }

  function discardAll() {
    setChanges(emptyChanges());
  }

  function handleExecuted() {
    discardAll();
    setPreviewOpen(false);
    load();
  }

  async function handleCopySql() {
    if (!sqlPreview) return;
    await navigator.clipboard.writeText(sqlPreview.map((p) => p.sql).join("\n"));
    setCopied(true);
    setTimeout(() => setCopied(false), 1200);
  }

  if (loading) {
    return (
      <div className="flex h-full items-center justify-center text-muted-foreground">
        <Loader2 className="size-5 animate-spin" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex h-full items-center justify-center p-6 text-center text-sm text-destructive">
        {error}
      </div>
    );
  }

  if (!columns || !constraints || !indexes) return null;

  const columnNames = columns.map((c) => c.name);

  return (
    <div className="flex h-full min-h-0">
      <div className="flex min-h-0 flex-7 flex-col overflow-auto p-3">
        <Tabs defaultValue="columns">
          <div className="flex items-center justify-between">
            <TabsList variant="line">
              <TabsTrigger value="columns">Columns</TabsTrigger>
              <TabsTrigger value="indexes">Indexes</TabsTrigger>
              <TabsTrigger value="constraints">Constraints</TabsTrigger>
            </TabsList>

            {pendingChanges && (
              <div className="flex items-center gap-2">
                <span className="text-xs text-muted-foreground">
                  {changeCount(changes)} staged change{changeCount(changes) === 1 ? "" : "s"}
                </span>
                <Button variant="outline" size="xs" className="gap-1.5" onClick={discardAll}>
                  <RotateCcw className="size-3" />
                  Discard
                </Button>
                <Button size="xs" disabled={!changesValid} onClick={() => setPreviewOpen(true)}>
                  Save
                </Button>
              </div>
            )}
          </div>

          <TabsContent value="columns" className="mt-3">
            <ColumnsTab
              columns={columns}
              droppedColumns={changes.droppedColumns}
              editedColumns={changes.editedColumns}
              newColumns={changes.newColumns}
              onToggleDrop={toggleDropColumn}
              onStartEdit={startEditColumn}
              onCancelEdit={cancelEditColumn}
              onChangeEdit={changeEditColumn}
              onAddRow={() =>
                setChanges((prev) => ({ ...prev, newColumns: [...prev.newColumns, newColumnRow()] }))
              }
              onRemoveNewRow={(tempId) =>
                setChanges((prev) => ({
                  ...prev,
                  newColumns: prev.newColumns.filter((c) => c.tempId !== tempId),
                }))
              }
              onChangeNewRow={(tempId, patch) =>
                setChanges((prev) => ({
                  ...prev,
                  newColumns: prev.newColumns.map((c) => (c.tempId === tempId ? { ...c, ...patch } : c)),
                }))
              }
            />
          </TabsContent>

          <TabsContent value="indexes" className="mt-3">
            <IndexesTab
              indexes={indexes}
              availableColumns={columnNames}
              droppedIndexes={changes.droppedIndexes}
              newIndexes={changes.newIndexes}
              onToggleDrop={toggleDropIndex}
              onAddRow={() =>
                setChanges((prev) => ({ ...prev, newIndexes: [...prev.newIndexes, newIndexRow()] }))
              }
              onRemoveNewRow={(tempId) =>
                setChanges((prev) => ({
                  ...prev,
                  newIndexes: prev.newIndexes.filter((i) => i.tempId !== tempId),
                }))
              }
              onChangeNewRow={(tempId, patch) =>
                setChanges((prev) => ({
                  ...prev,
                  newIndexes: prev.newIndexes.map((i) => (i.tempId === tempId ? { ...i, ...patch } : i)),
                }))
              }
            />
          </TabsContent>

          <TabsContent value="constraints" className="mt-3">
            <ConstraintsTab
              connectionId={connectionId}
              schema={schema}
              table={table}
              availableColumns={columnNames}
              constraints={constraints}
              droppedConstraints={changes.droppedConstraints}
              newConstraints={changes.newConstraints}
              onToggleDrop={toggleDropConstraint}
              onAddRow={() =>
                setChanges((prev) => ({ ...prev, newConstraints: [...prev.newConstraints, newConstraintRow()] }))
              }
              onRemoveNewRow={(tempId) =>
                setChanges((prev) => ({
                  ...prev,
                  newConstraints: prev.newConstraints.filter((c) => c.tempId !== tempId),
                }))
              }
              onChangeNewRow={(tempId, patch) =>
                setChanges((prev) => ({
                  ...prev,
                  newConstraints: prev.newConstraints.map((c) => (c.tempId === tempId ? { ...c, ...patch } : c)),
                }))
              }
            />
          </TabsContent>
        </Tabs>
      </div>

      <div className="flex min-h-0 flex-3 flex-col border-l">
        <div className="flex h-8 shrink-0 items-center justify-between border-b px-3">
          <span className="text-xs font-semibold tracking-wide text-muted-foreground uppercase">SQL</span>
          <Button
            variant="outline"
            size="xs"
            className="gap-1.5"
            onClick={handleCopySql}
            disabled={!sqlPreview}
          >
            {copied ? <Check className="size-3.5" /> : <Copy className="size-3.5" />}
            {copied ? "Copied" : "Copy"}
          </Button>
        </div>

        <div className="min-h-0 flex-1 overflow-auto p-3">
          {!pendingChanges && (
            <p className="text-xs text-muted-foreground">
              Stage a change (add or drop a column, index, or constraint) to see its SQL here.
            </p>
          )}
          {pendingChanges && sqlLoading && (
            <div className="flex items-center gap-2 text-xs text-muted-foreground">
              <Loader2 className="size-3.5 animate-spin" />
              Rendering SQL…
            </div>
          )}
          {pendingChanges && !sqlLoading && sqlError && (
            <div className="flex items-start gap-2 text-xs text-amber-600 dark:text-amber-400">
              <AlertTriangle className="size-3.5 shrink-0 translate-y-0.5" />
              <span>{sqlError}</span>
            </div>
          )}
          {pendingChanges && !sqlLoading && !sqlError && sqlPreview && (
            <pre className="font-mono text-xs whitespace-pre-wrap">
              {sqlPreview.map((p) => p.sql).join("\n")}
            </pre>
          )}
        </div>
      </div>

      {previewOpen && (
        <DdlPreviewDialog
          connectionId={connectionId}
          schema={schema}
          statements={statements}
          open={previewOpen}
          onOpenChange={setPreviewOpen}
          onExecuted={handleExecuted}
        />
      )}
    </div>
  );
}
