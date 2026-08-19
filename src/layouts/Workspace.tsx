import { DatabaseZap, Plus, Table2 } from "lucide-react";

import { Button } from "@/src/app/components/ui/button";
import {
  Empty,
  EmptyContent,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "@/src/app/components/ui/empty";
import { TabBar } from "@/src/features/tables/components/TableToolbar/TabBar";
import { TableView } from "@/src/features/tables/components/TableView";
import type { TableTab } from "@/src/features/tables/types";

interface WorkspaceProps {
  hasConnections: boolean;
  tabs: TableTab[];
  activeTabId: string | null;
  onSelectTab: (id: string) => void;
  onCloseTab: (id: string) => void;
  onNewConnection: () => void;
}

export function Workspace({
  hasConnections,
  tabs,
  activeTabId,
  onSelectTab,
  onCloseTab,
  onNewConnection,
}: WorkspaceProps) {
  const activeTab = tabs.find((t) => t.id === activeTabId) ?? null;

  if (!hasConnections) {
    return (
      <div className="flex flex-1 items-center justify-center p-6">
        <Empty className="max-w-md">
          <EmptyHeader>
            <EmptyMedia variant="icon">
              <DatabaseZap />
            </EmptyMedia>
            <EmptyTitle>No connection selected</EmptyTitle>
            <EmptyDescription>
              Create a database connection to browse schemas, run queries, and
              edit data.
            </EmptyDescription>
          </EmptyHeader>
          <EmptyContent>
            <Button size="sm" className="gap-2" onClick={onNewConnection}>
              <Plus className="size-4" />
              New Connection
            </Button>
          </EmptyContent>
        </Empty>
      </div>
    );
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <TabBar
        tabs={tabs}
        activeTabId={activeTabId}
        onSelectTab={onSelectTab}
        onCloseTab={onCloseTab}
      />

      <div className="min-h-0 flex-1">
        {activeTab ? (
          <TableView key={activeTab.id} tab={activeTab} />
        ) : (
          <div className="flex h-full items-center justify-center p-6">
            <Empty className="max-w-md">
              <EmptyHeader>
                <EmptyMedia variant="icon">
                  <Table2 />
                </EmptyMedia>
                <EmptyTitle>No table open</EmptyTitle>
                <EmptyDescription>
                  Select a table from the sidebar to browse its data.
                </EmptyDescription>
              </EmptyHeader>
            </Empty>
          </div>
        )}
      </div>
    </div>
  );
}
