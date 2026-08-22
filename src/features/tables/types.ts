import type { Engine } from "@/src/features/connections/types";

export interface TableTab {
  type: "table";
  id: string;
  connectionId: string;
  connectionName: string;
  engine: Engine;
  schema: string;
  table: string;
}

export function tableTabId(connectionId: string, schema: string, table: string): string {
  return `table::${connectionId}::${schema}.${table}`;
}
