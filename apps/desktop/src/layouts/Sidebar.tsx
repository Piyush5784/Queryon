import { useState } from "react";
import { Database, DownloadCloud, Moon, PanelTopClose, Plus, RotateCw, Sun } from "lucide-react";

import { Button } from "@queryon/ui/components/button";
import { TooltipButton } from "@/src/components/TooltipButton";
import {
  checkForUpdates,
  restartToUpdate,
  useUpdaterStatus,
  type UpdaterStatus,
} from "@/src/components/UpdateChecker";
import {
  Sidebar as SidebarPrimitive,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupAction,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
} from "@queryon/ui/components/sidebar";
import { useTheme } from "@/src/components/theme-provider";
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
  onDisconnect: (id: string) => void;
  onDeleteConnection: (id: string) => void;
  onOpenTable: (connectionId: string, schema: string, table: string) => void;
  onOpenDatabase: (connectionId: string, database: string) => void;
  onOpenCollection: (connectionId: string, database: string, collection: string) => void;
  onNewConnection: () => void;
  onOpenSavedQuery: (query: SavedQuery) => void;
  onOpenHistoryEntry: (connectionId: string, sql: string) => void;
  onNewQuery: (connectionId?: string) => void;
  onConnectionRenamed: () => void;
}

interface UpdaterButtonProps {
  label: string;
  icon: typeof DownloadCloud;
  disabled: boolean;
  onClick: () => void;
  tooltip: string;
}

function getUpdaterButton(status: UpdaterStatus): UpdaterButtonProps {
  switch (status.kind) {
    case "checking":
      return { label: "Checking for updates…", icon: DownloadCloud, disabled: true, onClick: checkForUpdates, tooltip: "Checking for updates" };
    case "up-to-date":
      return { label: "Up to date", icon: DownloadCloud, disabled: true, onClick: checkForUpdates, tooltip: "You're on the latest version" };
    case "updating":
      return { label: `Updating… ${status.progress}%`, icon: DownloadCloud, disabled: true, onClick: checkForUpdates, tooltip: "Downloading and installing the update" };
    case "ready":
      return { label: "Restart to update", icon: RotateCw, disabled: false, onClick: restartToUpdate, tooltip: "Restart to apply the update" };
    case "error":
      return { label: "Failed to update", icon: DownloadCloud, disabled: false, onClick: checkForUpdates, tooltip: status.message };
    default:
      return { label: "Check for updates", icon: DownloadCloud, disabled: false, onClick: checkForUpdates, tooltip: "Check for updates" };
  }
}

export function Sidebar({
  connections,
  connectedIds,
  connectingId,
  activeConnectionId,
  queryRefreshToken,
  onSelectConnection,
  onDisconnect,
  onDeleteConnection,
  onOpenTable,
  onOpenDatabase,
  onOpenCollection,
  onNewConnection,
  onOpenSavedQuery,
  onOpenHistoryEntry,
  onNewQuery,
  onConnectionRenamed,
}: SidebarProps) {
  const { theme, setTheme } = useTheme();
  const [collapseSignal, setCollapseSignal] = useState(0);
  const updaterStatus = useUpdaterStatus();
  const updaterButton = getUpdaterButton(updaterStatus);

  return (
    <SidebarPrimitive collapsible="icon">
      <SidebarHeader className="gap-2 px-2 py-2">
        <TooltipButton
          variant="outline"
          size="sm"
          className="justify-start gap-2"
          onClick={onNewConnection}
          tooltip="New Connection"
          shortcut={["⌘", "N"]}
        >
          <Plus className="size-4" />
          <span className="group-data-[collapsible=icon]:hidden">
            New Connection
          </span>
        </TooltipButton>
      </SidebarHeader>

      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupLabel>
            <Database className="size-3.5" />
            Connections
          </SidebarGroupLabel>
          <SidebarGroupAction
            title="Collapse all"
            className="right-9"
            onClick={() => setCollapseSignal((n) => n + 1)}
          >
            <PanelTopClose />
          </SidebarGroupAction>
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
                    collapseSignal={collapseSignal}
                    onSelect={() => onSelectConnection(conn.id)}
                    onDisconnect={() => onDisconnect(conn.id)}
                    onDeleteConnection={() => onDeleteConnection(conn.id)}
                    onOpenTable={(schema, table) =>
                      onOpenTable(conn.id, schema, table)
                    }
                    onOpenDatabase={(database) => onOpenDatabase(conn.id, database)}
                    onOpenCollection={(database, collection) =>
                      onOpenCollection(conn.id, database, collection)
                    }
                    onOpenSavedQuery={onOpenSavedQuery}
                    onOpenHistoryEntry={(sql) => onOpenHistoryEntry(conn.id, sql)}
                    onNewQuery={() => onNewQuery(conn.id)}
                    onConnectionRenamed={onConnectionRenamed}
                  />
                ))}
              </SidebarMenu>
            )}
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>

      <SidebarFooter className="gap-1.5 px-2 py-2">
        <TooltipButton
          tooltip={updaterButton.tooltip}
          variant="ghost"
          size="sm"
          className="justify-start gap-1.5"
          disabled={updaterButton.disabled}
          onClick={updaterButton.onClick}
        >
          <updaterButton.icon className="size-3.5" />
          <span className="group-data-[collapsible=icon]:hidden">{updaterButton.label}</span>
        </TooltipButton>
        <div className="flex items-center gap-1 rounded-lg border p-0.5 group-data-[collapsible=icon]:flex-col">
          <Button
            variant={theme === "light" ? "secondary" : "ghost"}
            size="sm"
            className="flex-1 justify-center gap-1.5"
            onClick={() => setTheme("light")}
          >
            <Sun className="size-3.5" />
            <span className="group-data-[collapsible=icon]:hidden">Light</span>
          </Button>
          <Button
            variant={theme === "dark" ? "secondary" : "ghost"}
            size="sm"
            className="flex-1 justify-center gap-1.5"
            onClick={() => setTheme("dark")}
          >
            <Moon className="size-3.5" />
            <span className="group-data-[collapsible=icon]:hidden">Dark</span>
          </Button>
        </div>
      </SidebarFooter>
    </SidebarPrimitive>
  );
}
