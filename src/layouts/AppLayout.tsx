import { PlugZap, TerminalSquare } from "lucide-react";

import { Button } from "@/src/app/components/ui/button";
import { Separator } from "@/src/app/components/ui/separator";
import {
  SidebarInset,
  SidebarProvider,
  SidebarTrigger,
} from "@/src/app/components/ui/sidebar";
import { TooltipProvider } from "@/src/app/components/ui/tooltip";
import type { AppTab } from "@/src/app/tabs";
import type { ConnectionProfile } from "@/src/features/connections/types";
import { Sidebar } from "@/src/layouts/Sidebar";
import { Workspace } from "@/src/layouts/Workspace";

interface AppLayoutProps {
  connections: ConnectionProfile[];
  activeConnectionId: string | null;
  activeConnection: ConnectionProfile | null;
  onSelectConnection: (id: string) => void;
  onOpenTable: (connectionId: string, schema: string, table: string) => void;
  onNewConnection: () => void;
  onNewQuery: () => void;
  showHome: boolean;
  onGoHome: () => void;
  tabs: AppTab[];
  activeTabId: string | null;
  onSelectTab: (id: string) => void;
  onCloseTab: (id: string) => void;
}

export function AppLayout({
  connections,
  activeConnectionId,
  activeConnection,
  onSelectConnection,
  onOpenTable,
  onNewConnection,
  onNewQuery,
  showHome,
  onGoHome,
  tabs,
  activeTabId,
  onSelectTab,
  onCloseTab,
}: AppLayoutProps) {
  return (
    <TooltipProvider>
      <SidebarProvider>
        <Sidebar
          connections={connections}
          activeConnectionId={activeConnectionId}
          onSelectConnection={onSelectConnection}
          onOpenTable={onOpenTable}
          onNewConnection={onNewConnection}
        />
        <SidebarInset>
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
              <Button size="xs" variant="outline" className="gap-1.5" onClick={onNewQuery}>
                <TerminalSquare className="size-3.5" />
                New Query
              </Button>
            )}
          </header>
          <Workspace
            showHome={showHome}
            connections={connections}
            activeConnectionId={activeConnectionId}
            onSelectConnection={onSelectConnection}
            tabs={tabs}
            activeTabId={activeTabId}
            onSelectTab={onSelectTab}
            onCloseTab={onCloseTab}
            onNewConnection={onNewConnection}
          />
        </SidebarInset>
      </SidebarProvider>
    </TooltipProvider>
  );
}
