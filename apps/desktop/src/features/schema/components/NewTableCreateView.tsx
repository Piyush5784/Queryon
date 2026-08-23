import { useEffect, useState } from "react";
import { AlertTriangle, Check, Copy, Loader2, X } from "lucide-react";

import { Button } from "@/src/app/components/ui/button";
import { Input } from "@/src/app/components/ui/input";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/src/app/components/ui/tabs";
import { ColumnsTab, newColumnRow } from "@/src/features/schema/components/ColumnsTab";
import { ConstraintsTab, newConstraintRow } from "@/src/features/schema/components/ConstraintsTab";
import { DdlPreviewDialog } from "@/src/features/schema/components/DdlPreviewDialog";
import { IndexesTab, newIndexRow } from "@/src/features/schema/components/IndexesTab";
import { capabilitiesFor } from "@/src/features/schema/capabilities";
import { renderDdl, type DdlPreview, type DdlStatement } from "@/src/features/schema/api";
import {
  emptyNewTableChanges,
  isNewTableValid,
  toCreateTableStatements,
  type NewTableChanges,
} from "@/src/features/schema/staging";
import type { Engine } from "@/src/features/connections/types";
import { toErrorMessage } from "@/src/lib/tauri/errors";

interface NewTableCreateViewProps {
  connectionId: string;
  engine: Engine;
  schema: string;
  onCreated: (tableName: string) => void;
  onCancel: () => void;
}

export function NewTableCreateView({ connectionId, engine, schema, onCreated, onCancel }: NewTableCreateViewProps) {
  const capabilities = capabilitiesFor(engine);
  const [name, setName] = useState("");
  const [changes, setChanges] = useState<NewTableChanges>(emptyNewTableChanges());
  const [previewOpen, setPreviewOpen] = useState(false);
  const [sqlPreview, setSqlPreview] = useState<DdlPreview[] | null>(null);
  const [sqlLoading, setSqlLoading] = useState(false);
  const [sqlError, setSqlError] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  const valid = isNewTableValid(name, changes);
  const statements: DdlStatement[] = valid ? toCreateTableStatements(name.trim(), changes) : [];

  useEffect(() => {
    if (!valid) {
      setSqlPreview(null);
      setSqlError(name.trim() ? "Fill in every column to see its SQL." : null);
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
  }, [valid, connectionId, schema, JSON.stringify(statements)]);

  async function handleCopySql() {
    if (!sqlPreview) return;
    await navigator.clipboard.writeText(sqlPreview.map((p) => p.sql).join("\n"));
    setCopied(true);
    setTimeout(() => setCopied(false), 1200);
  }

  function handleCreated() {
    setPreviewOpen(false);
    onCreated(name.trim());
  }

  const columnNames = changes.columns.map((c) => c.name).filter(Boolean);

  return (
    <div className="flex h-full min-h-0">
      <div className="flex min-h-0 flex-7 flex-col overflow-auto p-3">
        <div className="mb-3 flex items-center justify-between gap-3">
          <div className="flex min-w-0 flex-1 items-center gap-2">
            <label className="shrink-0 text-xs text-muted-foreground">Table name</label>
            <Input
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="table_name"
              className="h-7 max-w-64 font-mono text-xs"
              autoFocus
            />
          </div>
          <div className="flex shrink-0 items-center gap-2">
            <Button variant="outline" size="xs" className="gap-1.5" onClick={onCancel}>
              <X className="size-3" />
              Cancel
            </Button>
            <Button size="xs" disabled={!valid} onClick={() => setPreviewOpen(true)}>
              Create Table
            </Button>
          </div>
        </div>

        <Tabs defaultValue="columns">
          <TabsList variant="line">
            <TabsTrigger value="columns">Columns</TabsTrigger>
            <TabsTrigger value="indexes">Indexes</TabsTrigger>
            <TabsTrigger value="constraints">Constraints</TabsTrigger>
          </TabsList>

          <TabsContent value="columns" className="mt-3">
            <ColumnsTab
              columns={[]}
              droppedColumns={[]}
              editedColumns={[]}
              newColumns={changes.columns}
              showAutoIncrement
              onToggleDrop={() => {}}
              onStartEdit={() => {}}
              onCancelEdit={() => {}}
              onChangeEdit={() => {}}
              onAddRow={() => setChanges((prev) => ({ ...prev, columns: [...prev.columns, newColumnRow()] }))}
              onRemoveNewRow={(tempId) =>
                setChanges((prev) => ({
                  ...prev,
                  columns: prev.columns.filter((c) => c.tempId !== tempId),
                }))
              }
              onChangeNewRow={(tempId, patch) =>
                setChanges((prev) => ({
                  ...prev,
                  columns: prev.columns.map((c) => (c.tempId === tempId ? { ...c, ...patch } : c)),
                }))
              }
            />
          </TabsContent>

          <TabsContent value="indexes" className="mt-3">
            <IndexesTab
              indexes={[]}
              availableColumns={columnNames}
              droppedIndexes={[]}
              newIndexes={changes.indexes}
              onToggleDrop={() => {}}
              onAddRow={() => setChanges((prev) => ({ ...prev, indexes: [...prev.indexes, newIndexRow()] }))}
              onRemoveNewRow={(tempId) =>
                setChanges((prev) => ({
                  ...prev,
                  indexes: prev.indexes.filter((i) => i.tempId !== tempId),
                }))
              }
              onChangeNewRow={(tempId, patch) =>
                setChanges((prev) => ({
                  ...prev,
                  indexes: prev.indexes.map((i) => (i.tempId === tempId ? { ...i, ...patch } : i)),
                }))
              }
            />
          </TabsContent>

          <TabsContent value="constraints" className="mt-3">
            <ConstraintsTab
              connectionId={connectionId}
              schema={schema}
              table={name.trim() || "__new_table__"}
              capabilities={capabilities}
              availableColumns={columnNames}
              constraints={[]}
              droppedConstraints={[]}
              newConstraints={changes.constraints}
              onToggleDrop={() => {}}
              onAddRow={() =>
                setChanges((prev) => ({ ...prev, constraints: [...prev.constraints, newConstraintRow()] }))
              }
              onRemoveNewRow={(tempId) =>
                setChanges((prev) => ({
                  ...prev,
                  constraints: prev.constraints.filter((c) => c.tempId !== tempId),
                }))
              }
              onChangeNewRow={(tempId, patch) =>
                setChanges((prev) => ({
                  ...prev,
                  constraints: prev.constraints.map((c) => (c.tempId === tempId ? { ...c, ...patch } : c)),
                }))
              }
            />
          </TabsContent>
        </Tabs>
      </div>

      <div className="flex min-h-0 flex-3 flex-col border-l">
        <div className="flex h-8 shrink-0 items-center justify-between border-b px-3">
          <span className="text-xs font-semibold tracking-wide text-muted-foreground uppercase">SQL</span>
          <Button variant="outline" size="xs" className="gap-1.5" onClick={handleCopySql} disabled={!sqlPreview}>
            {copied ? <Check className="size-3.5" /> : <Copy className="size-3.5" />}
            {copied ? "Copied" : "Copy"}
          </Button>
        </div>

        <div className="min-h-0 flex-1 overflow-auto p-3">
          {!name.trim() && !sqlError && (
            <p className="text-xs text-muted-foreground">Name the table and add at least one column to see its SQL.</p>
          )}
          {sqlLoading && (
            <div className="flex items-center gap-2 text-xs text-muted-foreground">
              <Loader2 className="size-3.5 animate-spin" />
              Rendering SQL…
            </div>
          )}
          {!sqlLoading && sqlError && (
            <div className="flex items-start gap-2 text-xs text-amber-600 dark:text-amber-400">
              <AlertTriangle className="size-3.5 shrink-0 translate-y-0.5" />
              <span>{sqlError}</span>
            </div>
          )}
          {!sqlLoading && !sqlError && sqlPreview && (
            <pre className="font-mono text-xs whitespace-pre-wrap">{sqlPreview.map((p) => p.sql).join("\n")}</pre>
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
          onExecuted={handleCreated}
        />
      )}
    </div>
  );
}
