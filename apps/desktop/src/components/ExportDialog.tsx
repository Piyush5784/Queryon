import { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { CheckCircle2, Loader2, XCircle } from "lucide-react";

import { Button } from "@queryon/ui/components/button";
import { Checkbox } from "@queryon/ui/components/checkbox";
import type { ExportTarget } from "@/src/components/exportTypes";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@queryon/ui/components/dialog";
import {
  Field,
  FieldContent,
  FieldGroup,
  FieldLabel,
} from "@queryon/ui/components/field";
import { Input } from "@queryon/ui/components/input";
import { RadioGroup, RadioGroupItem } from "@queryon/ui/components/radio-group";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@queryon/ui/components/select";
import { EXPORT_FORMATS, suggestedFileName, type ExportEvent, type ExportFormat } from "@/src/lib/export";
import {
  cancelExport,
  getExportDefaultDirectory,
  pickExportDirectory,
  runQueryExport,
  runRowsExport,
  runTableExport,
  type QueryExportRequest,
  type RowsExportRequest,
  type TableExportRequest,
} from "@/src/lib/tauri/commands";
import { toPrefixedErrorMessage } from "@/src/lib/tauri/errors";

interface ExportDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  target: ExportTarget | null;
  fileBaseName: string;
}

type TableScope = "whole" | "filtered" | "page";
type QueryScope = "page" | "whole";

type RunState =
  | { phase: "idle" }
  | { phase: "running"; jobId: string; rowsWritten: number; totalRows: number | null }
  | { phase: "done"; rowsWritten: number; path: string }
  | { phase: "cancelled" }
  | { phase: "error"; message: string };

export function ExportDialog({ open, onOpenChange, target, fileBaseName }: ExportDialogProps) {
  const [fileName, setFileName] = useState("");
  const [format, setFormat] = useState<ExportFormat>("csv");
  const [directory, setDirectory] = useState("");
  const [prettyPrint, setPrettyPrint] = useState(true);
  const [deleteOnAbort, setDeleteOnAbort] = useState(true);
  const [advancedOpen, setAdvancedOpen] = useState(false);
  const [chunkSize, setChunkSize] = useState(500);
  const [tableScope, setTableScope] = useState<TableScope>("whole");
  const [queryScope, setQueryScope] = useState<QueryScope>("whole");
  const [runState, setRunState] = useState<RunState>({ phase: "idle" });
  const unlistenRef = useRef<(() => void) | null>(null);

  const hasActiveFilter = target?.kind === "table" && (target.filters.length > 0 || target.sort.length > 0);

  useEffect(() => {
    if (!open) return;
    setFileName(suggestedFileName(fileBaseName, format));
    setRunState({ phase: "idle" });
    setTableScope("whole");
    setQueryScope("whole");
    getExportDefaultDirectory()
      .then(setDirectory)
      .catch(() => setDirectory(""));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open, fileBaseName]);

  useEffect(() => {
    if (!open) return;
    setFileName((prev) => {
      const withoutExt = prev.replace(/\.[a-z0-9]+$/i, "");
      const ext = EXPORT_FORMATS.find((f) => f.value === format)?.extension ?? "txt";
      return `${withoutExt}.${ext}`;
    });
  }, [format, open]);

  useEffect(() => {
    return () => {
      unlistenRef.current?.();
    };
  }, []);

  async function handleChooseDirectory() {
    try {
      const chosen = await pickExportDirectory(directory || null);
      if (chosen) setDirectory(chosen);
    } catch (err) {
      setRunState({ phase: "error", message: toPrefixedErrorMessage("Failed to open folder picker", err) });
    }
  }

  async function handleRun() {
    if (!target || !fileName.trim() || !directory.trim()) return;

    const jobId = crypto.randomUUID();
    setRunState({ phase: "running", jobId, rowsWritten: 0, totalRows: null });

    const unlisten = await listen<ExportEvent>("export:progress", (event) => applyEvent(jobId, event.payload));
    const unlistenDone = await listen<ExportEvent>("export:done", (event) => applyEvent(jobId, event.payload));
    const unlistenCancelled = await listen<ExportEvent>("export:cancelled", (event) =>
      applyEvent(jobId, event.payload)
    );
    const unlistenError = await listen<ExportEvent>("export:error", (event) => applyEvent(jobId, event.payload));
    unlistenRef.current = () => {
      unlisten();
      unlistenDone();
      unlistenCancelled();
      unlistenError();
    };

    try {
      if (target.kind === "table") {
        if (tableScope === "page" && target.page) {
          const request: RowsExportRequest = {
            columns: target.page.columns,
            rows: target.page.rows,
            directory,
            fileName: fileName.trim(),
            format,
            prettyPrint,
            deleteOnAbort,
          };
          await runRowsExport(jobId, request);
        } else {
          const request: TableExportRequest = {
            connectionId: target.connectionId,
            schema: target.schema,
            table: target.table,
            filters: tableScope === "filtered" ? target.filters : [],
            sort: tableScope === "filtered" ? target.sort : [],
            directory,
            fileName: fileName.trim(),
            format,
            prettyPrint,
            chunkSize,
            deleteOnAbort,
          };
          await runTableExport(jobId, request);
        }
      } else if (target.kind === "query") {
        if (queryScope === "page") {
          const request: RowsExportRequest = {
            columns: target.page.columns,
            rows: target.page.rows,
            directory,
            fileName: fileName.trim(),
            format,
            prettyPrint,
            deleteOnAbort,
          };
          await runRowsExport(jobId, request);
        } else {
          const request: QueryExportRequest = {
            connectionId: target.connectionId,
            tabId: target.tabId,
            sql: target.sql,
            directory,
            fileName: fileName.trim(),
            format,
            prettyPrint,
            chunkSize,
            deleteOnAbort,
          };
          await runQueryExport(jobId, request);
        }
      } else {
        const request: RowsExportRequest = {
          columns: target.columns,
          rows: target.rows,
          directory,
          fileName: fileName.trim(),
          format,
          prettyPrint,
          deleteOnAbort,
        };
        await runRowsExport(jobId, request);
      }
    } catch (err) {
      setRunState({ phase: "error", message: toPrefixedErrorMessage("Failed to start export", err) });
    }
  }

  function applyEvent(expectedJobId: string, event: ExportEvent) {
    if (event.jobId !== expectedJobId) return;
    if (event.kind === "progress") {
      setRunState({ phase: "running", jobId: event.jobId, rowsWritten: event.rowsWritten, totalRows: event.totalRows });
    } else if (event.kind === "done") {
      setRunState({ phase: "done", rowsWritten: event.rowsWritten, path: event.path });
    } else if (event.kind === "cancelled") {
      setRunState({ phase: "cancelled" });
    } else {
      setRunState({ phase: "error", message: event.message });
    }
  }

  async function handleCancel() {
    if (runState.phase !== "running") return;
    await cancelExport(runState.jobId);
  }

  function handleOpenChange(next: boolean) {
    if (runState.phase === "running") return;
    onOpenChange(next);
  }

  const running = runState.phase === "running";
  const canRun = !running && fileName.trim().length > 0 && directory.trim().length > 0;
  const showChunkSize =
    (target?.kind === "table" && tableScope !== "page") ||
    (target?.kind === "query" && queryScope === "whole");

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Export {targetLabel(target)}</DialogTitle>
          <DialogDescription>
            This will export rows directly to a file. You can choose the format and file name.
            For large exports, this runs in the background, so you can keep working.
          </DialogDescription>
        </DialogHeader>

        <FieldGroup>
          {target?.kind === "table" && (
            <Field>
              <FieldLabel>Scope</FieldLabel>
              <FieldContent>
                <RadioGroup
                  value={tableScope}
                  onValueChange={(v) => setTableScope(v as TableScope)}
                  disabled={running}
                >
                  <Field orientation="horizontal">
                    <RadioGroupItem value="whole" id="scope-table-whole" />
                    <FieldLabel htmlFor="scope-table-whole" className="font-normal">
                      Whole table
                    </FieldLabel>
                  </Field>
                  <Field orientation="horizontal">
                    <RadioGroupItem
                      value="filtered"
                      id="scope-table-filtered"
                      disabled={running || !hasActiveFilter}
                    />
                    <FieldLabel
                      htmlFor="scope-table-filtered"
                      className={hasActiveFilter ? "font-normal" : "font-normal text-muted-foreground"}
                    >
                      Filtered view{!hasActiveFilter ? " (no filter applied)" : ""}
                    </FieldLabel>
                  </Field>
                  {target.page && (
                    <Field orientation="horizontal">
                      <RadioGroupItem value="page" id="scope-table-page" />
                      <FieldLabel htmlFor="scope-table-page" className="font-normal">
                        This page ({target.page.rows.length.toLocaleString()} rows loaded)
                      </FieldLabel>
                    </Field>
                  )}
                </RadioGroup>
              </FieldContent>
            </Field>
          )}

          {target?.kind === "query" && (
            <Field>
              <FieldLabel>Scope</FieldLabel>
              <FieldContent>
                <RadioGroup
                  value={queryScope}
                  onValueChange={(v) => setQueryScope(v as QueryScope)}
                  disabled={running}
                >
                  <Field orientation="horizontal">
                    <RadioGroupItem value="whole" id="scope-query-whole" />
                    <FieldLabel htmlFor="scope-query-whole" className="font-normal">
                      Whole result (re-runs the query)
                    </FieldLabel>
                  </Field>
                  <Field orientation="horizontal">
                    <RadioGroupItem value="page" id="scope-query-page" />
                    <FieldLabel htmlFor="scope-query-page" className="font-normal">
                      This page ({target.page.rows.length.toLocaleString()} rows loaded)
                    </FieldLabel>
                  </Field>
                </RadioGroup>
              </FieldContent>
            </Field>
          )}

          <div className="grid grid-cols-3 gap-3">
            <Field className="col-span-2">
              <FieldLabel htmlFor="export-filename">File Name</FieldLabel>
              <FieldContent>
                <Input
                  id="export-filename"
                  value={fileName}
                  onChange={(e) => setFileName(e.target.value)}
                  disabled={running}
                  autoComplete="off"
                  spellCheck={false}
                />
              </FieldContent>
            </Field>
            <Field>
              <FieldLabel htmlFor="export-format">Format</FieldLabel>
              <FieldContent>
                <Select value={format} onValueChange={(v) => setFormat(v as ExportFormat)} disabled={running}>
                  <SelectTrigger id="export-format">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {EXPORT_FORMATS.map((f) => (
                      <SelectItem key={f.value} value={f.value}>
                        {f.label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </FieldContent>
            </Field>
          </div>

          <Field>
            <FieldLabel htmlFor="export-directory">Output Directory</FieldLabel>
            <FieldContent>
              <div className="flex gap-2">
                <Input
                  id="export-directory"
                  value={directory}
                  onChange={(e) => setDirectory(e.target.value)}
                  disabled={running}
                  autoComplete="off"
                  spellCheck={false}
                  className="flex-1"
                />
                <Button variant="outline" size="sm" onClick={handleChooseDirectory} disabled={running}>
                  Choose
                </Button>
              </div>
            </FieldContent>
          </Field>

          <button
            type="button"
            onClick={() => setAdvancedOpen((prev) => !prev)}
            className="text-left text-xs font-medium text-muted-foreground hover:text-foreground"
          >
            {advancedOpen ? "▾" : "▸"} Advanced Options
          </button>

          {advancedOpen && (
            <div className="flex flex-col gap-3 rounded-lg border p-3">
              {format === "json" && (
                <Field orientation="horizontal">
                  <Checkbox
                    id="export-pretty"
                    checked={prettyPrint}
                    onCheckedChange={(checked) => setPrettyPrint(checked === true)}
                    disabled={running}
                  />
                  <FieldLabel htmlFor="export-pretty" className="font-normal">
                    Pretty-print
                  </FieldLabel>
                </Field>
              )}

              {showChunkSize && (
                <Field>
                  <FieldLabel htmlFor="export-chunk-size">Chunk size</FieldLabel>
                  <FieldContent>
                    <Input
                      id="export-chunk-size"
                      type="number"
                      min={50}
                      max={5000}
                      value={chunkSize}
                      onChange={(e) => setChunkSize(Number(e.target.value) || 500)}
                      disabled={running}
                    />
                  </FieldContent>
                </Field>
              )}

              <Field orientation="horizontal">
                <Checkbox
                  id="export-delete-on-abort"
                  checked={deleteOnAbort}
                  onCheckedChange={(checked) => setDeleteOnAbort(checked === true)}
                  disabled={running}
                />
                <FieldLabel htmlFor="export-delete-on-abort" className="font-normal">
                  Delete file on abort/error
                </FieldLabel>
              </Field>
            </div>
          )}
        </FieldGroup>

        {runState.phase === "running" && (
          <div className="flex items-center gap-2 rounded-lg border px-3 py-2 text-sm text-muted-foreground">
            <Loader2 className="size-4 shrink-0 animate-spin" />
            {runState.rowsWritten.toLocaleString()}
            {runState.totalRows ? ` / ${runState.totalRows.toLocaleString()}` : ""} rows written…
          </div>
        )}
        {runState.phase === "done" && (
          <div className="flex items-center gap-2 rounded-lg border border-emerald-500/30 bg-emerald-500/10 px-3 py-2 text-sm text-emerald-600 dark:text-emerald-400">
            <CheckCircle2 className="size-4 shrink-0" />
            Exported {runState.rowsWritten.toLocaleString()} rows to {runState.path}
          </div>
        )}
        {runState.phase === "cancelled" && (
          <div className="flex items-center gap-2 rounded-lg border px-3 py-2 text-sm text-muted-foreground">
            <XCircle className="size-4 shrink-0" />
            Export cancelled.
          </div>
        )}
        {runState.phase === "error" && (
          <div className="flex items-start gap-2 rounded-lg border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive">
            <XCircle className="size-4 shrink-0 translate-y-0.5" />
            <span className="wrap-break-word">{runState.message}</span>
          </div>
        )}

        <DialogFooter>
          {running ? (
            <Button variant="outline" onClick={handleCancel}>
              Cancel Export
            </Button>
          ) : (
            <Button variant="outline" onClick={() => onOpenChange(false)}>
              Close
            </Button>
          )}
          <Button onClick={handleRun} disabled={!canRun}>
            {running && <Loader2 className="size-3.5 animate-spin" />}
            Run
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function targetLabel(target: ExportTarget | null): string {
  if (!target) return "";
  if (target.kind === "table") return target.table;
  return "query result";
}
