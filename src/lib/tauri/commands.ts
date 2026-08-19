import { commands } from "@/src/lib/tauri/bindings";
import type {
  ColumnInfo,
  ConnectionInfo,
  ConnectionProfile,
  TableRef,
  TableRowsResult as RawTableRowsResult,
  QueryResult as RawQueryResult,
} from "@/src/lib/tauri/bindings";

export type { ColumnInfo, ConnectionInfo, ConnectionProfile, TableRef };

export type CellValue = string | number | boolean | null | Record<string, unknown> | unknown[];

export interface TableRowsResult {
  columns: string[];
  rows: CellValue[][];
  rowCount: number;
  hasMore: boolean;
}

export type QueryResult =
  | {
      kind: "rows";
      columns: string[];
      rows: CellValue[][];
      rowCount: number;
      truncated: boolean;
      durationMs: number;
    }
  | {
      kind: "affected";
      rowCount: number;
      durationMs: number;
    };

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

export async function fetchTableRows(
  connectionId: string,
  schema: string,
  table: string,
  limit: number,
  offset: number
): Promise<TableRowsResult> {
  const raw: RawTableRowsResult = await unwrap(
    await commands.dbFetchTableRows(connectionId, schema, table, limit, offset)
  );
  return { ...raw, rows: decodeRows(raw.rows) };
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

export async function executeQuery(connectionId: string, sql: string): Promise<QueryResult> {
  const raw: RawQueryResult = await unwrap(await commands.dbExecuteQuery(connectionId, sql));
  if (raw.kind === "affected") {
    return raw;
  }
  return { ...raw, rows: decodeRows(raw.rows) };
}
