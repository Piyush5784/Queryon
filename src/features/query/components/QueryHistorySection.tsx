import { useEffect, useState } from "react";
import { History, Trash2, XCircle } from "lucide-react";

import {
  SidebarGroup,
  SidebarGroupAction,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from "@/src/app/components/ui/sidebar";
import {
  clearQueryHistory,
  listQueryHistory,
  type QueryHistoryEntry,
} from "@/src/features/query/api";
import { toErrorMessage } from "@/src/lib/tauri/errors";

interface QueryHistorySectionProps {
  connectionId: string | null;
  refreshToken: number;
  onOpenQuery: (sql: string) => void;
}

export function QueryHistorySection({
  connectionId,
  refreshToken,
  onOpenQuery,
}: QueryHistorySectionProps) {
  const [entries, setEntries] = useState<QueryHistoryEntry[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!connectionId) {
      setEntries([]);
      return;
    }
    let cancelled = false;
    listQueryHistory(connectionId)
      .then((result) => {
        if (!cancelled) setEntries(result);
      })
      .catch((err) => {
        if (!cancelled) setError(toErrorMessage(err));
      });
    return () => {
      cancelled = true;
    };
  }, [connectionId, refreshToken]);

  async function handleClear() {
    if (!connectionId) return;
    setEntries([]);
    try {
      await clearQueryHistory(connectionId);
    } catch (err) {
      setError(toErrorMessage(err));
    }
  }

  return (
    <SidebarGroup>
      <SidebarGroupLabel>
        <History className="size-3.5" />
        Query History
      </SidebarGroupLabel>
      {entries.length > 0 && (
        <SidebarGroupAction title="Clear history" onClick={handleClear}>
          <Trash2 />
        </SidebarGroupAction>
      )}
      <SidebarGroupContent>
        {!connectionId ? (
          <p className="px-2 py-1.5 text-xs text-muted-foreground group-data-[collapsible=icon]:hidden">
            Select a connection
          </p>
        ) : error ? (
          <p className="px-2 py-1.5 text-xs text-destructive group-data-[collapsible=icon]:hidden">
            {error}
          </p>
        ) : entries.length === 0 ? (
          <p className="px-2 py-1.5 text-xs text-muted-foreground group-data-[collapsible=icon]:hidden">
            No queries run yet
          </p>
        ) : (
          <SidebarMenu>
            {entries.slice(0, 25).map((entry) => (
              <SidebarMenuItem key={entry.id}>
                <SidebarMenuButton
                  tooltip={entry.sql}
                  onClick={() => onOpenQuery(entry.sql)}
                  className="h-auto py-1"
                >
                  {entry.status === "error" ? (
                    <XCircle className="size-3.5 shrink-0 text-destructive" />
                  ) : (
                    <span className="size-1.5 shrink-0 rounded-full bg-emerald-500" />
                  )}
                  <span className="truncate font-mono text-xs">{entry.sql}</span>
                </SidebarMenuButton>
              </SidebarMenuItem>
            ))}
          </SidebarMenu>
        )}
      </SidebarGroupContent>
    </SidebarGroup>
  );
}
