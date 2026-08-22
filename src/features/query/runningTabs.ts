const runningTabIds = new Set<string>();

export function markTabRunning(tabId: string): void {
  runningTabIds.add(tabId);
}

export function markTabIdle(tabId: string): void {
  runningTabIds.delete(tabId);
}

export function isTabRunning(tabId: string): boolean {
  return runningTabIds.has(tabId);
}
