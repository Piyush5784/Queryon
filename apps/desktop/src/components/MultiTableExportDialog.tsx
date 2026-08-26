import { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { CheckCircle2, ChevronRight, Loader2, XCircle } from "lucide-react";

import { Button } from "@queryon/ui/components/button";
import { Checkbox } from "@queryon/ui/components/checkbox";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@queryon/ui/components/dialog";
import { Field, FieldContent, FieldGroup, FieldLabel } from "@queryon/ui/components/field";
import { Input } from "@queryon/ui/components/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@queryon/ui/components/select";
import { listTables, type TableRef } from "@/src/features/tables/api";
import { EXPORT_FORMATS, type ExportEvent, type ExportFormat } from "@/src/lib/export";
import { getExportDefaultDirectory, pickExportDirectory, runTableExport } from "@/src/lib/tauri/commands";
import { toErrorMessage, toPrefixedErrorMessage } from "@/src/lib/tauri/errors";

interface MultiTableExportDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  connectionId: string;
}

interface JobState {
  jobId: string;
  schema: string;
  table: string;
  rowsWritten: number;
  status: "running" | "done" | "error" | "cancelled";
  message?: string;
}

export function MultiTableExportDialog({ open, onOpenChange, connectionId }: MultiTableExportDialogProps) {
  const [tables, setTables] = useState<TableRef[]>([]);
  const [tablesLoading, setTablesLoading] = useState(false);
  const [tablesError, setTablesError] = useState<string | null>(null);
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [format, setFormat] = useState<ExportFormat>("csv");
  const [directory, setDirectory] = useState("");
  const [running, setRunning] = useState(false);
  const [jobs, setJobs] = useState<JobState[]>([]);
  const [startError, setStartError] = useState<string | null>(null);
  const unlistenRef = useRef<(() => void) | null>(null);

  const schemas = groupBySchema(tables);

  useEffect(() => {
    if (!open) return;
    setSelected(new Set());
    setRunning(false);
    setJobs([]);
    setStartError(null);
    setTablesError(null);
    setTablesLoading(true);
    listTables(connectionId)
      .then(setTables)
      .catch((err) => setTablesError(toErrorMessage(err)))
      .finally(() => setTablesLoading(false));
    getExportDefaultDirectory()
      .then(setDirectory)
      .catch(() => setDirectory(""));
  }, [open, connectionId]);

  useEffect(() => {
    return () => {
      unlistenRef.current?.();
    };
  }, []);

  function key(table: TableRef) {
    return `${table.schema}.${table.name}`;
  }

  function toggle(table: TableRef) {
    setSelected((prev) => {
      const next = new Set(prev);
      const k = key(table);
      if (next.has(k)) next.delete(k);
      else next.add(k);
      return next;
    });
  }

  function toggleSchema(schemaTables: TableRef[]) {
    const keys = schemaTables.map(key);
    const allSelected = keys.every((k) => selected.has(k));
    setSelected((prev) => {
      const next = new Set(prev);
      if (allSelected) {
        keys.forEach((k) => next.delete(k));
      } else {
        keys.forEach((k) => next.add(k));
      }
      return next;
    });
  }

  async function handleChooseDirectory() {
    try {
      const chosen = await pickExportDirectory(directory || null);
      if (chosen) setDirectory(chosen);
    } catch (err) {
      setStartError(toPrefixedErrorMessage("Failed to open folder picker", err));
    }
  }

  async function handleRun() {
    if (running || selected.size === 0 || !directory.trim()) return;

    const selectedTables = tables.filter((t) => selected.has(key(t)));
    const extension = EXPORT_FORMATS.find((f) => f.value === format)?.extension ?? "txt";

    const initialJobs: JobState[] = selectedTables.map((t) => ({
      jobId: crypto.randomUUID(),
      schema: t.schema,
      table: t.name,
      rowsWritten: 0,
      status: "running",
    }));

    setJobs(initialJobs);
    setRunning(true);
    setStartError(null);

    const unlisten = await listen<ExportEvent>("export:progress", (e) => applyEvent(e.payload));
    const unlistenDone = await listen<ExportEvent>("export:done", (e) => applyEvent(e.payload));
    const unlistenCancelled = await listen<ExportEvent>("export:cancelled", (e) => applyEvent(e.payload));
    const unlistenError = await listen<ExportEvent>("export:error", (e) => applyEvent(e.payload));
    unlistenRef.current = () => {
      unlisten();
      unlistenDone();
      unlistenCancelled();
      unlistenError();
    };

    try {
      await Promise.all(
        initialJobs.map((job, i) => {
          const table = selectedTables[i];
          return runTableExport(job.jobId, {
            connectionId,
            schema: table.schema,
            table: table.name,
            filters: [],
            sort: [],
            directory,
            fileName: `${table.schema}_${table.name}.${extension}`,
            format,
            prettyPrint: true,
            chunkSize: 500,
            deleteOnAbort: true,
          });
        })
      );
    } catch (err) {
      setStartError(toPrefixedErrorMessage("Failed to start one or more exports", err));
    }
  }

  function applyEvent(event: ExportEvent) {
    setJobs((prev) =>
      prev.map((job) => {
        if (job.jobId !== event.jobId) return job;
        if (event.kind === "progress") return { ...job, rowsWritten: event.rowsWritten };
        if (event.kind === "done") return { ...job, rowsWritten: event.rowsWritten, status: "done" };
        if (event.kind === "cancelled") return { ...job, status: "cancelled" };
        return { ...job, status: "error", message: event.message };
      })
    );
  }

  function handleOpenChange(next: boolean) {
    if (running && !allJobsFinished(jobs)) return;
    onOpenChange(next);
  }

  const allDone = running && allJobsFinished(jobs);
  const canRun = !running && selected.size > 0 && directory.trim().length > 0;

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent className="sm:max-w-2xl">
        <DialogHeader>
          <DialogTitle>Export Tables</DialogTitle>
          <DialogDescription>
            Pick one or more tables. Each is exported to its own file in the chosen directory.
          </DialogDescription>
        </DialogHeader>

        {!running ? (
          <>
            <div className="max-h-64 overflow-y-auto rounded-lg border">
              {tablesLoading && (
                <div className="flex items-center gap-2 px-3 py-4 text-sm text-muted-foreground">
                  <Loader2 className="size-3.5 animate-spin" />
                  Loading tables…
                </div>
              )}
              {tablesError && <p className="px-3 py-4 text-sm text-destructive">{tablesError}</p>}
              {!tablesLoading && !tablesError && Object.entries(schemas).map(([schemaName, schemaTables]) => {
                const allSelected = schemaTables.every((t) => selected.has(key(t)));
                return (
                  <div key={schemaName} className="border-b last:border-b-0">
                    <label className="flex items-center gap-2 bg-muted/40 px-3 py-1.5 text-xs font-medium">
                      <Checkbox
                        checked={allSelected}
                        onCheckedChange={() => toggleSchema(schemaTables)}
                      />
                      {schemaName}
                      <span className="text-muted-foreground">({schemaTables.length})</span>
                    </label>
                    {schemaTables.map((t) => (
                      <label
                        key={key(t)}
                        className="flex items-center gap-2 px-3 py-1.5 pl-7 text-sm hover:bg-accent/50"
                      >
                        <Checkbox checked={selected.has(key(t))} onCheckedChange={() => toggle(t)} />
                        {t.name}
                      </label>
                    ))}
                  </div>
                );
              })}
              {!tablesLoading && !tablesError && tables.length === 0 && (
                <p className="px-3 py-4 text-center text-sm text-muted-foreground">No tables found.</p>
              )}
            </div>

            <FieldGroup>
              <div className="grid grid-cols-3 gap-3">
                <Field className="col-span-2">
                  <FieldLabel htmlFor="multi-export-directory">Output Directory</FieldLabel>
                  <FieldContent>
                    <div className="flex gap-2">
                      <Input
                        id="multi-export-directory"
                        value={directory}
                        onChange={(e) => setDirectory(e.target.value)}
                        autoComplete="off"
                        spellCheck={false}
                        className="flex-1"
                      />
                      <Button variant="outline" size="sm" onClick={handleChooseDirectory}>
                        Choose
                      </Button>
                    </div>
                  </FieldContent>
                </Field>
                <Field>
                  <FieldLabel htmlFor="multi-export-format">Format</FieldLabel>
                  <FieldContent>
                    <Select value={format} onValueChange={(v) => setFormat(v as ExportFormat)}>
                      <SelectTrigger id="multi-export-format">
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
            </FieldGroup>

            {startError && (
              <div className="flex items-start gap-2 rounded-lg border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive">
                <XCircle className="size-4 shrink-0 translate-y-0.5" />
                <span className="wrap-break-word">{startError}</span>
              </div>
            )}
          </>
        ) : (
          <div className="max-h-80 space-y-1 overflow-y-auto rounded-lg border p-2">
            {jobs.map((job) => (
              <div key={job.jobId} className="flex items-center gap-2 rounded-md px-2 py-1.5 text-sm">
                {job.status === "running" && <Loader2 className="size-3.5 shrink-0 animate-spin text-muted-foreground" />}
                {job.status === "done" && <CheckCircle2 className="size-3.5 shrink-0 text-emerald-500" />}
                {job.status === "cancelled" && <XCircle className="size-3.5 shrink-0 text-muted-foreground" />}
                {job.status === "error" && <XCircle className="size-3.5 shrink-0 text-destructive" />}
                <ChevronRight className="size-3 shrink-0 text-muted-foreground" />
                <span className="truncate font-mono text-xs">
                  {job.schema}.{job.table}
                </span>
                <span className="ml-auto shrink-0 text-xs text-muted-foreground">
                  {job.status === "error" ? job.message : `${job.rowsWritten.toLocaleString()} rows`}
                </span>
              </div>
            ))}
          </div>
        )}

        <DialogFooter>
          {running ? (
            <Button variant="outline" onClick={() => onOpenChange(false)} disabled={!allDone}>
              {allDone ? "Close" : "Exporting…"}
            </Button>
          ) : (
            <>
              <Button variant="outline" onClick={() => onOpenChange(false)}>
                Cancel
              </Button>
              <Button onClick={handleRun} disabled={!canRun}>
                Export {selected.size > 0 ? `${selected.size} Table${selected.size === 1 ? "" : "s"}` : ""}
              </Button>
            </>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function allJobsFinished(jobs: JobState[]): boolean {
  return jobs.length > 0 && jobs.every((j) => j.status !== "running");
}

function groupBySchema(tables: TableRef[]): Record<string, TableRef[]> {
  const groups: Record<string, TableRef[]> = {};
  for (const t of tables) {
    (groups[t.schema] ??= []).push(t);
  }
  return groups;
}
