import { useEffect, useRef, useState } from "react";
import { AlertCircle, CheckCircle2, ChevronDown, ChevronLeft, ChevronRight, Copy, Loader2 } from "lucide-react";

import { Button } from "@queryon/ui/components/button";
import { ButtonGroup } from "@queryon/ui/components/button-group";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@queryon/ui/components/dropdown-menu";
import { ExportButton } from "@/src/components/ExportButton";
import { DataGrid } from "@/src/features/tables/components/DataGrid";
import type { JsonValue } from "@/src/features/tables/components/JsonViewer/types";
import { fetchQueryResultPage, type QueryResult } from "@/src/features/query/api";
import { QueryResultJsonSheet } from "@/src/features/query/components/QueryResultJsonSheet";
import { toErrorMessage } from "@/src/lib/tauri/errors";

export const QUERY_RESULT_PAGE_SIZE = 10_000;

function rowsToTsv(columns: string[], rows: unknown[][]): string {
  const cellToText = (cell: unknown) => {
    if (cell === null || cell === undefined) return "";
    const text = typeof cell === "string" ? cell : JSON.stringify(cell);
    return text.replace(/\t/g, " ").replace(/\r?\n/g, " ");
  };
  const lines = [columns.join("\t")];
  for (const row of rows) {
    lines.push(row.map(cellToText).join("\t"));
  }
  return lines.join("\n");
}

function rowsToJson(columns: string[], rows: unknown[][]): string {
  const objects = rows.map((row) => {
    const obj: Record<string, unknown> = {};
    columns.forEach((col, i) => {
      obj[col] = row[i] ?? null;
    });
    return obj;
  });
  return JSON.stringify(objects, null, 2);
}

interface CopyPageButtonProps {
  columns: string[];
  rows: unknown[][];
  disabled?: boolean;
}

function CopyPageButton({ columns, rows, disabled }: CopyPageButtonProps) {
  const [copied, setCopied] = useState(false);
  const [copying, setCopying] = useState(false);
  const [open, setOpen] = useState(false);
  const closeTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  function cancelClose() {
    if (closeTimeoutRef.current) {
      clearTimeout(closeTimeoutRef.current);
      closeTimeoutRef.current = null;
    }
  }

  function openOnHover() {
    cancelClose();
    setOpen(true);
  }

  function closeOnHoverOut() {
    cancelClose();
    closeTimeoutRef.current = setTimeout(() => setOpen(false), 150);
  }

  async function copyAs(format: "tsv" | "json") {
    setCopying(true);
    try {
      const text = format === "tsv" ? rowsToTsv(columns, rows) : rowsToJson(columns, rows);
      await navigator.clipboard.writeText(text);
      setCopied(true);
      setTimeout(() => setCopied(false), 1200);
    } finally {
      setCopying(false);
    }
  }

  return (
    <ButtonGroup onMouseEnter={openOnHover} onMouseLeave={closeOnHoverOut}>
      <Button
        variant="outline"
        size="xs"
        className="gap-1.5"
        disabled={disabled || copying}
        onClick={() => copyAs("tsv")}
      >
        {copying ? <Loader2 className="size-3.5 animate-spin" /> : <Copy className="size-3.5" />}
        {copied ? "Copied" : "Copy"}
      </Button>
      <DropdownMenu open={open} onOpenChange={setOpen}>
        <DropdownMenuTrigger
          render={
            <Button variant="outline" size="xs" disabled={disabled || copying}>
              <ChevronDown className="size-3.5" />
            </Button>
          }
        />
        <DropdownMenuContent align="end" side="top">
          <DropdownMenuItem onClick={() => copyAs("tsv")}>Copy as raw text (TSV)</DropdownMenuItem>
          <DropdownMenuItem onClick={() => copyAs("json")}>Copy as JSON</DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </ButtonGroup>
  );
}

interface QueryResultsProps {
  connectionId: string;
  tabId: string;
  sql: string;
  result: QueryResult;
}

export function QueryResults({ connectionId, tabId, sql, result }: QueryResultsProps) {
  const [jsonSheet, setJsonSheet] = useState<{ columnName: string; value: JsonValue } | null>(null);
  const [page, setPage] = useState(0);
  const [pageRows, setPageRows] = useState(result.kind === "rows" ? result.rows : []);
  const [pageError, setPageError] = useState<string | null>(null);
  const [pageLoading, setPageLoading] = useState(false);

  useEffect(() => {
    setPage(0);
    setPageRows(result.kind === "rows" ? result.rows : []);
    setPageError(null);
  }, [result]);

  if (result.kind === "affected") {
    return (
      <div className="flex h-full items-center justify-center p-6">
        <div className="flex items-center gap-2 text-sm text-muted-foreground">
          <CheckCircle2 className="size-4 text-emerald-500" />
          {result.rowCount} row{result.rowCount === 1 ? "" : "s"} affected
          <span className="text-muted-foreground/60">({result.durationMs}ms)</span>
        </div>
      </div>
    );
  }

  if (result.rows.length === 0 && result.totalRowCount === 0) {
    return (
      <div className="flex h-full items-center justify-center text-sm text-muted-foreground">
        Query returned no rows.
      </div>
    );
  }

  const totalRowCount = result.totalRowCount;
  const paginated = totalRowCount !== null;
  const totalPages = paginated ? Math.max(1, Math.ceil(totalRowCount / QUERY_RESULT_PAGE_SIZE)) : 1;

  async function goToPage(next: number) {
    if (!paginated || next < 0 || next >= totalPages || next === page || pageLoading) return;
    setPageLoading(true);
    setPageError(null);
    try {
      const rowsPage = await fetchQueryResultPage(
        connectionId,
        tabId,
        next * QUERY_RESULT_PAGE_SIZE,
        QUERY_RESULT_PAGE_SIZE
      );
      setPageRows(rowsPage.rows);
      setPage(next);
    } catch (err) {
      setPageError(toErrorMessage(err));
    } finally {
      setPageLoading(false);
    }
  }

  return (
    <div className="flex h-full min-h-0 flex-col">
      {!paginated && (
        <div className="flex shrink-0 items-center gap-2 border-b bg-amber-500/10 px-3 py-1.5 text-xs text-amber-600 dark:text-amber-400">
          <AlertCircle className="size-3.5 shrink-0" />
          This query can't be paginated — showing everything it returned.
        </div>
      )}
      <div className="min-h-0 flex-1">
        {pageError ? (
          <div className="flex h-full items-center justify-center p-6 text-center text-sm text-destructive">
            {pageError}
          </div>
        ) : (
          <DataGrid
            columns={result.columns}
            rows={pageRows}
            pendingEdit={null}
            onPendingEditChange={() => {}}
            onOpenJsonCell={(columnName, value) => setJsonSheet({ columnName, value })}
          />
        )}
      </div>

      {paginated && (
        <div className="relative flex shrink-0 items-center justify-between gap-3 border-t px-3 py-1.5">
          <div className="flex items-center gap-2">
            <Button
              variant="ghost"
              size="icon-sm"
              onClick={() => goToPage(page - 1)}
              disabled={page === 0 || pageLoading}
            >
              <ChevronLeft className="size-3.5" />
            </Button>
            <span className="text-xs text-muted-foreground">
              Page {page + 1} of {totalPages.toLocaleString()}
            </span>
            <Button
              variant="ghost"
              size="icon-sm"
              onClick={() => goToPage(page + 1)}
              disabled={page >= totalPages - 1 || pageLoading}
            >
              <ChevronRight className="size-3.5" />
            </Button>
            <span className="pl-1 text-xs text-muted-foreground">
              {QUERY_RESULT_PAGE_SIZE.toLocaleString()} rows / page
            </span>
          </div>

          <div className="absolute left-1/2 flex -translate-x-1/2 items-center gap-2 text-xs text-muted-foreground">
            {pageLoading && <Loader2 className="size-3 animate-spin" />}
            <span>{totalRowCount.toLocaleString()} rows</span>
            <span className="text-muted-foreground/60">·</span>
            <span>{result.durationMs}ms</span>
          </div>

          <div className="flex items-center gap-1.5">
            <CopyPageButton
              columns={result.columns}
              rows={pageRows}
              disabled={pageLoading}
            />
            <ExportButton
              target={{
                kind: "query",
                connectionId,
                tabId,
                sql,
                page: {
                  columns: result.columns,
                  rows: pageRows.map((row) => row.map((cell) => JSON.stringify(cell))),
                },
              }}
              fileBaseName="query_result"
              disabled={pageLoading}
            />
          </div>
        </div>
      )}

      {!paginated && (
        <div className="flex shrink-0 items-center justify-end gap-1.5 border-t px-3 py-1.5">
          <CopyPageButton columns={result.columns} rows={pageRows} />
          <ExportButton
            target={{
              kind: "query",
              connectionId,
              tabId,
              sql,
              page: {
                columns: result.columns,
                rows: pageRows.map((row) => row.map((cell) => JSON.stringify(cell))),
              },
            }}
            fileBaseName="query_result"
          />
        </div>
      )}

      <QueryResultJsonSheet
        open={jsonSheet !== null}
        onOpenChange={(open) => !open && setJsonSheet(null)}
        columnName={jsonSheet?.columnName ?? null}
        value={jsonSheet?.value ?? null}
      />
    </div>
  );
}
