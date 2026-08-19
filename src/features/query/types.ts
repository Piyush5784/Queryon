export interface QueryTab {
  type: "query";
  id: string;
  connectionId: string;
  connectionName: string;
  title: string;
}

let queryTabCounter = 0;

export function createQueryTabId(): string {
  queryTabCounter += 1;
  return `query::${Date.now()}::${queryTabCounter}`;
}
