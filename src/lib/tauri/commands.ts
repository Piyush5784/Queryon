import { invoke } from "@tauri-apps/api/core";

import type { ConnectionProfile } from "@/src/features/connections/types";

export interface ConnectionInfo {
  id: string;
  serverVersion: string;
}

export interface TableRef {
  schema: string;
  name: string;
  kind: "table" | "view" | "materialized_view";
  estimatedRows: number;
}

export interface ColumnInfo {
  name: string;
  dataType: string;
  isNullable: boolean;
  default: string | null;
  isPrimaryKey: boolean;
  ordinalPosition: number;
}

export type CellValue = string | number | boolean | null | Record<string, unknown> | unknown[];

export interface TableRowsResult {
  columns: string[];
  rows: CellValue[][];
  rowCount: number;
  hasMore: boolean;
}

export function connect(profile: ConnectionProfile): Promise<ConnectionInfo> {
  return invoke<ConnectionInfo>("db_connect", { profile });
}

export function testConnection(profile: ConnectionProfile): Promise<ConnectionInfo> {
  return invoke<ConnectionInfo>("db_test_connection", { profile });
}

export function disconnect(connectionId: string): Promise<void> {
  return invoke<void>("db_disconnect", { connectionId });
}

export function listActiveConnections(): Promise<string[]> {
  return invoke<string[]>("db_list_active_connections");
}

export function listTables(connectionId: string): Promise<TableRef[]> {
  return invoke<TableRef[]>("db_list_tables", { connectionId });
}

export function getTableColumns(
  connectionId: string,
  schema: string,
  table: string
): Promise<ColumnInfo[]> {
  return invoke<ColumnInfo[]>("db_get_table_columns", { connectionId, schema, table });
}

export function fetchTableRows(
  connectionId: string,
  schema: string,
  table: string,
  limit: number,
  offset: number
): Promise<TableRowsResult> {
  return invoke<TableRowsResult>("db_fetch_table_rows", {
    connectionId,
    schema,
    table,
    limit,
    offset,
  });
}

export function updateJsonCell(
  connectionId: string,
  schema: string,
  table: string,
  row: Record<string, CellValue>,
  column: string,
  value: CellValue
): Promise<void> {
  return invoke<void>("db_update_json_cell", { connectionId, schema, table, row, column, value });
}

export function updateCellText(
  connectionId: string,
  schema: string,
  table: string,
  row: Record<string, CellValue>,
  column: string,
  value: string | null
): Promise<void> {
  return invoke<void>("db_update_cell_text", { connectionId, schema, table, row, column, value });
}

export function deleteRows(
  connectionId: string,
  schema: string,
  table: string,
  rows: Record<string, CellValue>[]
): Promise<number> {
  return invoke<number>("db_delete_rows", { connectionId, schema, table, rows });
}
