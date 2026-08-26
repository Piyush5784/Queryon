import { TerminalSquare } from "lucide-react";

import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "@queryon/ui/components/empty";
import type { AppTab } from "@/src/app/tabs";
import type { SavedConnectionProfile } from "@/src/features/connections/types";
import { CollectionView } from "@/src/features/documents/components/CollectionView";
import { QueryTabView } from "@/src/features/query/components/QueryTabView";
import { TabBar } from "@/src/features/tables/components/TableToolbar/TabBar";
import { TableView } from "@/src/features/tables/components/TableView";
import { HomeScreen } from "@/src/layouts/HomeScreen";

interface WorkspaceProps {
  showHome: boolean;
  connections: SavedConnectionProfile[];
  connectedIds: Set<string>;
  connectingId: string | null;
  connectError: string | null;
  activeConnectionId: string | null;
  onSelectConnection: (id: string) => void;
  onDeleteConnection: (id: string) => void;
  tabs: AppTab[];
  activeTabId: string | null;
  onSelectTab: (id: string) => void;
  onCloseTab: (id: string) => void;
  onReorderTabs: (fromId: string, toId: string) => void;
  onDetachTab: (id: string) => void;
  onNewConnection: () => void;
  onQueryActivity: () => void;
}

export function Workspace({
  showHome,
  connections,
  connectedIds,
  connectingId,
  connectError,
  activeConnectionId,
  onSelectConnection,
  onDeleteConnection,
  tabs,
  activeTabId,
  onSelectTab,
  onCloseTab,
  onReorderTabs,
  onDetachTab,
  onNewConnection,
  onQueryActivity,
}: WorkspaceProps) {
  const activeTab = tabs.find((t) => t.id === activeTabId) ?? null;

  if (showHome || connections.length === 0) {
    return (
      <HomeScreen
        connections={connections}
        connectedIds={connectedIds}
        connectingId={connectingId}
        connectError={connectError}
        activeConnectionId={activeConnectionId}
        onSelectConnection={onSelectConnection}
        onDeleteConnection={onDeleteConnection}
        onNewConnection={onNewConnection}
      />
    );
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <TabBar
        tabs={tabs}
        activeTabId={activeTabId}
        onSelectTab={onSelectTab}
        onCloseTab={onCloseTab}
        onReorderTabs={onReorderTabs}
        onDetachTab={onDetachTab}
      />

      <div className="min-h-0 flex-1">
        {activeTab ? (
          activeTab.type === "table" ? (
            <TableView key={activeTab.id} tab={activeTab} />
          ) : activeTab.type === "collection" ? (
            <CollectionView key={activeTab.id} tab={activeTab} />
          ) : (
            <QueryTabView key={activeTab.id} tab={activeTab} onQueryActivity={onQueryActivity} />
          )
        ) : (
          <div className="flex h-full items-center justify-center p-6">
            <Empty className="max-w-md">
              <EmptyHeader>
                <EmptyMedia variant="icon">
                  <TerminalSquare />
                </EmptyMedia>
                <EmptyTitle>No tab open</EmptyTitle>
                <EmptyDescription>
                  Select a table from the sidebar, or open a new SQL query.
                </EmptyDescription>
              </EmptyHeader>
            </Empty>
          </div>
        )}
      </div>
    </div>
  );
}
