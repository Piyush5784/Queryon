import { PlugZap } from "lucide-react";

import { Separator } from "@/src/app/components/ui/separator";
import {
  SidebarInset,
  SidebarProvider,
  SidebarTrigger,
} from "@/src/app/components/ui/sidebar";
import { TooltipProvider } from "@/src/app/components/ui/tooltip";
import type { ConnectionProfile } from "@/src/features/connections/types";
import type { TableTab } from "@/src/features/tables/types";
import { Sidebar } from "@/src/layouts/Sidebar";
import { Workspace } from "@/src/layouts/Workspace";

interface AppLayoutProps {
  connections: ConnectionProfile[];
  activeConnectionId: string | null;
  activeConnection: ConnectionProfile | null;
  onSelectConnection: (id: string) => void;
  onOpenTable: (connectionId: string, schema: string, table: string) => void;
  onNewConnection: () => void;
  tabs: TableTab[];
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
          <header className="flex h-11 shrink-0 items-center gap-2 border-b px-2">
            <SidebarTrigger />
            <Separator orientation="vertical" className="h-4" />
            <div className="flex items-center gap-1.5 text-sm text-muted-foreground">
              <span className="font-medium text-foreground">Queryon</span>
              <span className="text-muted-foreground/50">/</span>
              <span className="flex items-center gap-1.5">
                <PlugZap className="size-3.5" />
                {activeConnection ? (
                  <span className="text-foreground">
                    {activeConnection.name}
                  </span>
                ) : (
                  "No connection"
                )}
              </span>
            </div>
          </header>
          <Workspace
            hasConnections={connections.length > 0}
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
