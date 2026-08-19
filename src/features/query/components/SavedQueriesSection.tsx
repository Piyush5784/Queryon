import { useEffect, useState } from "react";
import { Star, Trash2 } from "lucide-react";

import {
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarMenu,
  SidebarMenuAction,
  SidebarMenuButton,
  SidebarMenuItem,
} from "@/src/app/components/ui/sidebar";
import { deleteSavedQuery, listSavedQueries, type SavedQuery } from "@/src/features/query/api";
import { toErrorMessage } from "@/src/lib/tauri/errors";

interface SavedQueriesSectionProps {
  connectionId: string | null;
  refreshToken: number;
  onOpenQuery: (query: SavedQuery) => void;
}

export function SavedQueriesSection({
  connectionId,
  refreshToken,
  onOpenQuery,
}: SavedQueriesSectionProps) {
  const [queries, setQueries] = useState<SavedQuery[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!connectionId) {
      setQueries([]);
      return;
    }
    let cancelled = false;
    listSavedQueries(connectionId)
      .then((result) => {
        if (!cancelled) setQueries(result);
      })
      .catch((err) => {
        if (!cancelled) setError(toErrorMessage(err));
      });
    return () => {
      cancelled = true;
    };
  }, [connectionId, refreshToken]);

  async function handleDelete(query: SavedQuery) {
    setQueries((prev) => prev.filter((q) => q.id !== query.id));
    try {
      await deleteSavedQuery(query.id);
    } catch (err) {
      setError(toErrorMessage(err));
    }
  }

  return (
    <SidebarGroup>
      <SidebarGroupLabel>
        <Star className="size-3.5" />
        Saved Queries
      </SidebarGroupLabel>
      <SidebarGroupContent>
        {!connectionId ? (
          <p className="px-2 py-1.5 text-xs text-muted-foreground group-data-[collapsible=icon]:hidden">
            Select a connection
          </p>
        ) : error ? (
          <p className="px-2 py-1.5 text-xs text-destructive group-data-[collapsible=icon]:hidden">
            {error}
          </p>
        ) : queries.length === 0 ? (
          <p className="px-2 py-1.5 text-xs text-muted-foreground group-data-[collapsible=icon]:hidden">
            No saved queries
          </p>
        ) : (
          <SidebarMenu>
            {queries.map((query) => (
              <SidebarMenuItem key={query.id} className="group/menu-item relative">
                <SidebarMenuButton tooltip={query.title} onClick={() => onOpenQuery(query)}>
                  <Star className="size-3.5" />
                  <span className="truncate">{query.title}</span>
                </SidebarMenuButton>
                <SidebarMenuAction showOnHover onClick={() => handleDelete(query)}>
                  <Trash2 />
                </SidebarMenuAction>
              </SidebarMenuItem>
            ))}
          </SidebarMenu>
        )}
      </SidebarGroupContent>
    </SidebarGroup>
  );
}
