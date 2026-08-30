export interface CollectionTab {
  type: "collection";
  id: string;
  connectionId: string;
  connectionName: string;
  database: string;
  collection: string;
}

export interface DatabaseTab {
  type: "database";
  id: string;
  connectionId: string;
  connectionName: string;
  database: string;
}

export function collectionTabId(connectionId: string, database: string, collection: string): string {
  return `collection::${connectionId}::${database}.${collection}`;
}

export function databaseTabId(connectionId: string, database: string): string {
  return `database::${connectionId}::${database}`;
}
