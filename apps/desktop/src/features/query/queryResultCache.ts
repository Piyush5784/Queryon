import type { QueryResult } from "@/src/features/query/api";

interface CachedResult {
  result: QueryResult;
  executedSql: string;
}

const cache = new Map<string, CachedResult>();

export function getCachedQueryResult(tabId: string): CachedResult | undefined {
  return cache.get(tabId);
}

export function setCachedQueryResult(tabId: string, result: QueryResult, executedSql: string): void {
  cache.set(tabId, { result, executedSql });
}

export function clearCachedQueryResult(tabId: string): void {
  cache.delete(tabId);
}
