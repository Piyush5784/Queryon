import { useState } from "react";

import { ConnectionDialog } from "@/src/features/connections/components/ConnectionDialog";
import type { ConnectionProfile } from "@/src/features/connections/types";
import { tableTabId, type TableTab } from "@/src/features/tables/types";
import { AppLayout } from "@/src/layouts/AppLayout";
import "@/src/app/styles/globals.css";

function App() {
  const [connections, setConnections] = useState<ConnectionProfile[]>([]);
  const [activeConnectionId, setActiveConnectionId] = useState<string | null>(null);
  const [dialogOpen, setDialogOpen] = useState(false);
  const [tabs, setTabs] = useState<TableTab[]>([]);
  const [activeTabId, setActiveTabId] = useState<string | null>(null);

  const activeConnection = connections.find((c) => c.id === activeConnectionId) ?? null;

  function handleConnected(profile: ConnectionProfile) {
    setConnections((prev) => [...prev, profile]);
    setActiveConnectionId(profile.id);
  }

  function handleOpenTable(connectionId: string, schema: string, table: string) {
    const id = tableTabId(connectionId, schema, table);
    const connectionName = connections.find((c) => c.id === connectionId)?.name ?? "";

    setTabs((prev) =>
      prev.some((t) => t.id === id)
        ? prev
        : [...prev, { id, connectionId, connectionName, schema, table }]
    );
    setActiveTabId(id);
  }

  function handleCloseTab(id: string) {
    setTabs((prev) => {
      const next = prev.filter((t) => t.id !== id);
      if (activeTabId === id) {
        setActiveTabId(next.length > 0 ? next[next.length - 1].id : null);
      }
      return next;
    });
  }

  return (
    <>
      <AppLayout
        connections={connections}
        activeConnectionId={activeConnectionId}
        activeConnection={activeConnection}
        onSelectConnection={setActiveConnectionId}
        onOpenTable={handleOpenTable}
        onNewConnection={() => setDialogOpen(true)}
        tabs={tabs}
        activeTabId={activeTabId}
        onSelectTab={setActiveTabId}
        onCloseTab={handleCloseTab}
      />

      <ConnectionDialog
        open={dialogOpen}
        onOpenChange={setDialogOpen}
        onConnected={handleConnected}
      />
    </>
  );
}

export default App;
