import { Database, Plus } from "lucide-react";

import { Button } from "@/src/app/components/ui/button";
import {
  Sidebar as SidebarPrimitive,
  SidebarContent,
  SidebarGroup,
  SidebarGroupAction,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
} from "@/src/app/components/ui/sidebar";
import { ConnectionTreeItem } from "@/src/features/connections/components/ConnectionTreeItem";
import type { SavedConnectionProfile } from "@/src/features/connections/types";
import type { SavedQuery } from "@/src/features/query/api";

interface SidebarProps {
  connections: SavedConnectionProfile[];
  connectedIds: Set<string>;
  connectingId: string | null;
  activeConnectionId: string | null;
  queryRefreshToken: number;
  onSelectConnection: (id: string) => void;
  onOpenTable: (connectionId: string, schema: string, table: string) => void;
  onNewConnection: () => void;
  onOpenSavedQuery: (query: SavedQuery) => void;
  onOpenHistoryEntry: (connectionId: string, sql: string) => void;
}

export function Sidebar({
  connections,
  connectedIds,
  connectingId,
  activeConnectionId,
  queryRefreshToken,
  onSelectConnection,
  onOpenTable,
  onNewConnection,
  onOpenSavedQuery,
  onOpenHistoryEntry,
}: SidebarProps) {
  return (
    <SidebarPrimitive collapsible="icon">
      <SidebarHeader className="gap-2 px-2 py-2">
        <Button
          variant="outline"
          size="sm"
          className="justify-start gap-2"
          onClick={onNewConnection}
        >
          <Plus className="size-4" />
          <span className="group-data-[collapsible=icon]:hidden">
            New Connection
          </span>
        </Button>
      </SidebarHeader>

      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupLabel>
            <Database className="size-3.5" />
            Connections
          </SidebarGroupLabel>
          <SidebarGroupAction title="Add connection" onClick={onNewConnection}>
            <Plus />
          </SidebarGroupAction>
          <SidebarGroupContent>
            {connections.length === 0 ? (
              <p className="px-2 py-1.5 text-xs text-muted-foreground group-data-[collapsible=icon]:hidden">
                No connections yet
              </p>
            ) : (
              <SidebarMenu>
                {connections.map((conn) => (
                  <ConnectionTreeItem
                    key={conn.id}
                    connection={conn}
                    isActive={conn.id === activeConnectionId}
                    isConnected={connectedIds.has(conn.id)}
                    isConnecting={connectingId === conn.id}
                    queryRefreshToken={queryRefreshToken}
                    onSelect={() => onSelectConnection(conn.id)}
                    onOpenTable={(schema, table) =>
                      onOpenTable(conn.id, schema, table)
                    }
                    onOpenSavedQuery={onOpenSavedQuery}
                    onOpenHistoryEntry={(sql) => onOpenHistoryEntry(conn.id, sql)}
                  />
                ))}
              </SidebarMenu>
            )}
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>
    </SidebarPrimitive>
  );
}
