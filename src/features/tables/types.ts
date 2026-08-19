export interface TableTab {
  id: string;
  connectionId: string;
  connectionName: string;
  schema: string;
  table: string;
}

export function tableTabId(connectionId: string, schema: string, table: string): string {
  return `${connectionId}::${schema}.${table}`;
}
