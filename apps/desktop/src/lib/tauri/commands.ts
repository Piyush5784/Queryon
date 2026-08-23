import { commands } from "@/src/lib/tauri/bindings";
import type {
  ColumnInfo,
  ConnectionInfo,
  ConnectionProfile,
  ConstraintInfo,
  ConstraintKind,
  ColumnEdit,
  DdlBatchResult,
  DdlExecutionResult,
  DdlPreview,
  DdlStatement,
  ExportFormat,
  FilterOperator,
  ForeignKeyAction,
  IndexInfo,
  NewColumn,
  NewConstraint,
  NewIndex,
  QueryExportRequest,
  QueryHistoryEntry,
  RowsExportRequest,
  SavedConnectionProfile,
  SavedQuery,
  SortDirection,
  TableExportRequest,
  TableFilter,
  TableRef,
  TableRowsResult as RawTableRowsResult,
  TableSort,
  QueryResult as RawQueryResult,
  QueryResultPage as RawQueryResultPage,
} from "@/src/lib/tauri/bindings";

export type {
  ColumnInfo,
  ConnectionInfo,
  ConnectionProfile,
  ConstraintInfo,
  ConstraintKind,
  ColumnEdit,
  DdlBatchResult,
  DdlExecutionResult,
  DdlPreview,
  DdlStatement,
  ExportFormat,
  FilterOperator,
  ForeignKeyAction,
  IndexInfo,
  NewColumn,
  NewConstraint,
  NewIndex,
  QueryExportRequest,
  QueryHistoryEntry,
  RowsExportRequest,
  SavedConnectionProfile,
  SavedQuery,
  SortDirection,
  TableExportRequest,
  TableFilter,
  TableRef,
  TableSort,
};

export type CellValue = string | number | boolean | null | Record<string, unknown> | unknown[];

export interface TableRowsResult {
  columns: string[];
  rows: CellValue[][];
  rowCount: number;
  hasMore: boolean;
  durationMs: number;
}

export type QueryResult =
  | {
      kind: "rows";
      columns: string[];
      rows: CellValue[][];
      totalRowCount: number | null;
      durationMs: number;
    }
  | {
      kind: "affected";
      rowCount: number;
      durationMs: number;
    };

export interface QueryResultPage {
  columns: string[];
  rows: CellValue[][];
  totalRowCount: number;
}

function decodeCell(cell: string): CellValue {
  try {
    return JSON.parse(cell) as CellValue;
  } catch {
    return cell;
  }
}

function decodeRows(rows: string[][]): CellValue[][] {
  return rows.map((row) => row.map(decodeCell));
}

function encodeRow(row: Record<string, CellValue>): Record<string, string> {
  return Object.fromEntries(Object.entries(row).map(([key, value]) => [key, JSON.stringify(value)]));
}

async function unwrap<T>(result: { status: "ok"; data: T } | { status: "error"; error: string }): Promise<T> {
  if (result.status === "error") {
    throw result.error;
  }
  return result.data;
}

export async function connect(profile: ConnectionProfile): Promise<ConnectionInfo> {
  return unwrap(await commands.dbConnect(profile));
}

export async function testConnection(profile: ConnectionProfile): Promise<ConnectionInfo> {
  return unwrap(await commands.dbTestConnection(profile));
}

export async function disconnect(connectionId: string): Promise<void> {
  await commands.dbDisconnect(connectionId);
}

export async function listActiveConnections(): Promise<string[]> {
  return commands.dbListActiveConnections();
}

export async function saveConnection(profile: ConnectionProfile): Promise<void> {
  await unwrap(await commands.dbSaveConnection(profile));
}

export async function listSavedConnections(): Promise<SavedConnectionProfile[]> {
  return unwrap(await commands.dbListSavedConnections());
}

export async function connectSaved(connectionId: string): Promise<ConnectionInfo> {
  return unwrap(await commands.dbConnectSaved(connectionId));
}

export async function deleteSavedConnection(connectionId: string): Promise<void> {
  await unwrap(await commands.dbDeleteSavedConnection(connectionId));
}

export async function renameSavedConnection(connectionId: string, name: string): Promise<void> {
  await unwrap(await commands.dbRenameSavedConnection(connectionId, name));
}

export async function pickSshKeyFile(): Promise<string | null> {
  return unwrap(await commands.sshPickKeyFile());
}

export async function pickSqliteFile(): Promise<string | null> {
  return unwrap(await commands.dbPickSqliteFile());
}

export async function pickDuckdbFile(): Promise<string | null> {
  return unwrap(await commands.dbPickDuckdbFile());
}

export async function listTables(connectionId: string): Promise<TableRef[]> {
  return unwrap(await commands.dbListTables(connectionId));
}

export async function getTableColumns(
  connectionId: string,
  schema: string,
  table: string
): Promise<ColumnInfo[]> {
  return unwrap(await commands.dbGetTableColumns(connectionId, schema, table));
}

export async function listIndexes(
  connectionId: string,
  schema: string,
  table: string
): Promise<IndexInfo[]> {
  return unwrap(await commands.dbListIndexes(connectionId, schema, table));
}

export async function listConstraints(
  connectionId: string,
  schema: string,
  table: string
): Promise<ConstraintInfo[]> {
  return unwrap(await commands.dbListConstraints(connectionId, schema, table));
}

export async function getTableDdl(
  connectionId: string,
  schema: string,
  table: string
): Promise<string> {
  return unwrap(await commands.dbGetTableDdl(connectionId, schema, table));
}

export async function renderDdl(
  connectionId: string,
  schema: string,
  statements: DdlStatement[]
): Promise<DdlPreview[]> {
  return unwrap(await commands.dbRenderDdl(connectionId, schema, statements));
}

export async function executeDdl(
  connectionId: string,
  schema: string,
  statements: DdlStatement[]
): Promise<DdlBatchResult> {
  return unwrap(await commands.dbExecuteDdl(connectionId, schema, statements));
}

export async function fetchTableRows(
  connectionId: string,
  schema: string,
  table: string,
  limit: number,
  offset: number,
  filters: TableFilter[] = [],
  sort: TableSort[] = []
): Promise<TableRowsResult> {
  const raw: RawTableRowsResult = await unwrap(
    await commands.dbFetchTableRows(connectionId, schema, table, limit, offset, filters, sort)
  );
  return { ...raw, rows: decodeRows(raw.rows) };
}

export async function countTableRows(
  connectionId: string,
  schema: string,
  table: string,
  filters: TableFilter[] = []
): Promise<number> {
  return unwrap(await commands.dbCountTableRows(connectionId, schema, table, filters));
}

export async function updateJsonCell(
  connectionId: string,
  schema: string,
  table: string,
  row: Record<string, CellValue>,
  column: string,
  value: CellValue
): Promise<void> {
  await unwrap(
    await commands.dbUpdateJsonCell(connectionId, schema, table, encodeRow(row), column, JSON.stringify(value))
  );
}

export async function updateCellText(
  connectionId: string,
  schema: string,
  table: string,
  row: Record<string, CellValue>,
  column: string,
  value: string | null
): Promise<void> {
  await unwrap(
    await commands.dbUpdateCellText(connectionId, schema, table, encodeRow(row), column, value)
  );
}

export async function deleteRows(
  connectionId: string,
  schema: string,
  table: string,
  rows: Record<string, CellValue>[]
): Promise<number> {
  return unwrap(await commands.dbDeleteRows(connectionId, schema, table, rows.map(encodeRow)));
}

export async function insertRow(
  connectionId: string,
  schema: string,
  table: string,
  values: Record<string, CellValue>
): Promise<void> {
  await unwrap(await commands.dbInsertRow(connectionId, schema, table, encodeRow(values)));
}

export async function executeQuery(
  connectionId: string,
  sql: string,
  tabId?: string
): Promise<QueryResult> {
  const raw: RawQueryResult = await unwrap(await commands.dbExecuteQuery(connectionId, sql, tabId ?? null));
  if (raw.kind === "affected") {
    return raw;
  }
  return { ...raw, rows: decodeRows(raw.rows) };
}

export async function cancelQuery(connectionId: string, tabId: string): Promise<void> {
  await unwrap(await commands.dbCancelQuery(connectionId, tabId));
}

export async function fetchQueryResultPage(
  connectionId: string,
  tabId: string,
  offset: number,
  limit: number
): Promise<QueryResultPage> {
  const raw: RawQueryResultPage = await unwrap(
    await commands.dbFetchQueryResultPage(connectionId, tabId, offset, limit)
  );
  return { ...raw, rows: decodeRows(raw.rows) };
}

export async function clearQueryResultCache(tabId: string): Promise<void> {
  await commands.dbClearQueryResultCache(tabId);
}

export async function beginTransaction(connectionId: string, tabId: string): Promise<void> {
  await unwrap(await commands.dbBeginTransaction(connectionId, tabId));
}

export async function commitTransaction(connectionId: string, tabId: string): Promise<void> {
  await unwrap(await commands.dbCommitTransaction(connectionId, tabId));
}

export async function rollbackTransaction(connectionId: string, tabId: string): Promise<void> {
  await unwrap(await commands.dbRollbackTransaction(connectionId, tabId));
}

export async function hasActiveTransaction(connectionId: string, tabId: string): Promise<boolean> {
  return commands.dbHasActiveTransaction(connectionId, tabId);
}

export async function saveQuery(query: SavedQuery): Promise<void> {
  await unwrap(await commands.dbSaveQuery(query));
}

export async function listSavedQueries(connectionId: string): Promise<SavedQuery[]> {
  return unwrap(await commands.dbListSavedQueries(connectionId));
}

export async function deleteSavedQuery(queryId: string): Promise<void> {
  await unwrap(await commands.dbDeleteSavedQuery(queryId));
}

export async function listQueryHistory(connectionId: string): Promise<QueryHistoryEntry[]> {
  return unwrap(await commands.dbListQueryHistory(connectionId));
}

export async function clearQueryHistory(connectionId: string): Promise<void> {
  await unwrap(await commands.dbClearQueryHistory(connectionId));
}

export async function getExportDefaultDirectory(): Promise<string> {
  return unwrap(await commands.exportDefaultDirectory());
}

export async function pickExportDirectory(initialDirectory: string | null): Promise<string | null> {
  return unwrap(await commands.exportPickDirectory(initialDirectory));
}

export async function runTableExport(jobId: string, request: TableExportRequest): Promise<void> {
  await unwrap(await commands.exportRunTable(jobId, request));
}

export async function runRowsExport(jobId: string, request: RowsExportRequest): Promise<void> {
  await unwrap(await commands.exportRunRows(jobId, request));
}

export async function runQueryExport(jobId: string, request: QueryExportRequest): Promise<void> {
  await unwrap(await commands.exportRunQuery(jobId, request));
}

export async function cancelExport(jobId: string): Promise<void> {
  await commands.exportCancel(jobId);
}
