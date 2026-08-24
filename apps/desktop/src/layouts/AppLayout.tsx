import { PlugZap, TerminalSquare } from "lucide-react";

import { Separator } from "@queryon/ui/components/separator";
import {
  SidebarInset,
  SidebarProvider,
  SidebarTrigger,
} from "@queryon/ui/components/sidebar";
import { TooltipProvider } from "@queryon/ui/components/tooltip";
import { TooltipButton } from "@/src/components/TooltipButton";
import type { AppTab } from "@/src/app/tabs";
import type { SavedConnectionProfile } from "@/src/features/connections/types";
import type { SavedQuery } from "@/src/features/query/api";
import { Sidebar } from "@/src/layouts/Sidebar";
import { Workspace } from "@/src/layouts/Workspace";

interface AppLayoutProps {
  connections: SavedConnectionProfile[];
  connectedIds: Set<string>;
  connectingId: string | null;
  connectError: string | null;
  activeConnectionId: string | null;
  activeConnection: SavedConnectionProfile | null;
  queryRefreshToken: number;
  onSelectConnection: (id: string) => void;
  onDeleteConnection: (id: string) => void;
  onDisconnect: (id: string) => void;
  onOpenTable: (connectionId: string, schema: string, table: string) => void;
  onNewConnection: () => void;
  onNewQuery: (connectionId?: string) => void;
  onOpenSavedQuery: (query: SavedQuery) => void;
  onOpenHistoryEntry: (connectionId: string, sql: string) => void;
  onConnectionRenamed: () => void;
  showHome: boolean;
  onGoHome: () => void;
  tabs: AppTab[];
  activeTabId: string | null;
  onSelectTab: (id: string) => void;
  onCloseTab: (id: string) => void;
  onQueryActivity: () => void;
}

export function AppLayout({
  connections,
  connectedIds,
  connectingId,
  connectError,
  activeConnectionId,
  activeConnection,
  queryRefreshToken,
  onSelectConnection,
  onDeleteConnection,
  onDisconnect,
  onOpenTable,
  onNewConnection,
  onNewQuery,
  onOpenSavedQuery,
  onOpenHistoryEntry,
  onConnectionRenamed,
  showHome,
  onGoHome,
  tabs,
  activeTabId,
  onSelectTab,
  onCloseTab,
  onQueryActivity,
}: AppLayoutProps) {
  return (
    <TooltipProvider>
      <SidebarProvider className="h-full min-h-0 overflow-hidden">
        <Sidebar
          connections={connections}
          connectedIds={connectedIds}
          connectingId={connectingId}
          activeConnectionId={activeConnectionId}
          queryRefreshToken={queryRefreshToken}
          onSelectConnection={onSelectConnection}
          onDisconnect={onDisconnect}
          onOpenTable={onOpenTable}
          onNewConnection={onNewConnection}
          onOpenSavedQuery={onOpenSavedQuery}
          onOpenHistoryEntry={onOpenHistoryEntry}
          onNewQuery={onNewQuery}
          onConnectionRenamed={onConnectionRenamed}
        />
        <SidebarInset className="min-h-0">
          <header className="flex h-11 shrink-0 items-center justify-between gap-2 border-b px-2">
            <div className="flex items-center gap-2">
              <SidebarTrigger />
              <Separator orientation="vertical" className="h-4" />
              <div className="flex items-center gap-1.5 text-sm text-muted-foreground">
                <button
                  type="button"
                  onClick={onGoHome}
                  className="font-medium text-foreground hover:underline"
                >
                  Queryon
                </button>
                <span className="text-muted-foreground/50">/</span>
                <span className="flex items-center gap-1.5">
                  <PlugZap className="size-3.5" />
                  {activeConnection && !showHome ? (
                    <span className="text-foreground">
                      {activeConnection.name}
                    </span>
                  ) : (
                    "No connection"
                  )}
                </span>
              </div>
            </div>
            {activeConnection && !showHome && (
              <TooltipButton
                size="xs"
                variant="outline"
                className="gap-1.5"
                onClick={() => onNewQuery()}
                tooltip="New Query"
                shortcut={["⌘", "T"]}
              >
                <TerminalSquare className="size-3.5" />
                New Query
              </TooltipButton>
            )}
          </header>
          <Workspace
            showHome={showHome}
            connections={connections}
            connectedIds={connectedIds}
            connectingId={connectingId}
            connectError={connectError}
            activeConnectionId={activeConnectionId}
            onSelectConnection={onSelectConnection}
            onDeleteConnection={onDeleteConnection}
            tabs={tabs}
            activeTabId={activeTabId}
            onSelectTab={onSelectTab}
            onCloseTab={onCloseTab}
            onNewConnection={onNewConnection}
            onQueryActivity={onQueryActivity}
          />
        </SidebarInset>
      </SidebarProvider>
    </TooltipProvider>
  );
}
