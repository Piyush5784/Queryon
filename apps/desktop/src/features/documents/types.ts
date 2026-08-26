export interface CollectionTab {
  type: "collection";
  id: string;
  connectionId: string;
  connectionName: string;
  database: string;
  collection: string;
}

export function collectionTabId(connectionId: string, database: string, collection: string): string {
  return `collection::${connectionId}::${database}.${collection}`;
}
