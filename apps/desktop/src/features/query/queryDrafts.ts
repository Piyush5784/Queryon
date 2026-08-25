const drafts = new Map<string, string>();

export function getQueryDraft(tabId: string): string | undefined {
  return drafts.get(tabId);
}

export function setQueryDraft(tabId: string, sql: string): void {
  drafts.set(tabId, sql);
}

export function clearQueryDraft(tabId: string): void {
  drafts.delete(tabId);
}
