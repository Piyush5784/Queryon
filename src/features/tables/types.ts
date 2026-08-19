export interface TableTab {
  type: "table";
  id: string;
  connectionId: string;
  connectionName: string;
  schema: string;
  table: string;
}

export function tableTabId(connectionId: string, schema: string, table: string): string {
  return `table::${connectionId}::${schema}.${table}`;
}
