import { Database, History, Plus, Star } from "lucide-react";

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
import type { ConnectionProfile } from "@/src/features/connections/types";

interface SidebarProps {
  connections: ConnectionProfile[];
  activeConnectionId: string | null;
  onSelectConnection: (id: string) => void;
  onOpenTable: (connectionId: string, schema: string, table: string) => void;
  onNewConnection: () => void;
}

export function Sidebar({
  connections,
  activeConnectionId,
  onSelectConnection,
  onOpenTable,
  onNewConnection,
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
                    onSelect={() => onSelectConnection(conn.id)}
                    onOpenTable={(schema, table) =>
                      onOpenTable(conn.id, schema, table)
                    }
                  />
                ))}
              </SidebarMenu>
            )}
          </SidebarGroupContent>
        </SidebarGroup>

        <SidebarGroup>
          <SidebarGroupLabel>
            <Star className="size-3.5" />
            Saved Queries
          </SidebarGroupLabel>
          <SidebarGroupContent>
            <p className="px-2 py-1.5 text-xs text-muted-foreground group-data-[collapsible=icon]:hidden">
              No saved queries
            </p>
          </SidebarGroupContent>
        </SidebarGroup>

        <SidebarGroup>
          <SidebarGroupLabel>
            <History className="size-3.5" />
            Query History
          </SidebarGroupLabel>
          <SidebarGroupContent>
            <p className="px-2 py-1.5 text-xs text-muted-foreground group-data-[collapsible=icon]:hidden">
              No queries run yet
            </p>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>
    </SidebarPrimitive>
  );
}
