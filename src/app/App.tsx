import { useEffect, useState } from "react";

import {
  connectSaved,
  deleteSavedConnection,
  disconnect,
  listSavedConnections,
} from "@/src/features/connections/api";
import { ConnectionDialog } from "@/src/features/connections/components/ConnectionDialog";
import type { ConnectionProfile, SavedConnectionProfile } from "@/src/features/connections/types";
import type { SavedQuery } from "@/src/features/query/api";
import { createQueryTabId } from "@/src/features/query/types";
import { tableTabId } from "@/src/features/tables/types";
import { AppLayout } from "@/src/layouts/AppLayout";
import { ThemeProvider } from "@/src/app/components/theme-provider";
import type { AppTab } from "@/src/app/tabs";
import { toErrorMessage } from "@/src/lib/tauri/errors";
import "@/src/app/styles/globals.css";

function hashSql(sql: string): string {
  let hash = 0;
  for (let i = 0; i < sql.length; i++) {
    hash = (hash * 31 + sql.charCodeAt(i)) | 0;
  }
  return (hash >>> 0).toString(36);
}

function App() {
  const [connections, setConnections] = useState<SavedConnectionProfile[]>([]);
  const [connectedIds, setConnectedIds] = useState<Set<string>>(new Set());
  const [connectingId, setConnectingId] = useState<string | null>(null);
  const [connectError, setConnectError] = useState<string | null>(null);
  const [activeConnectionId, setActiveConnectionId] = useState<string | null>(null);
  const [dialogOpen, setDialogOpen] = useState(false);
  const [tabs, setTabs] = useState<AppTab[]>([]);
  const [activeTabId, setActiveTabId] = useState<string | null>(null);
  const [showHome, setShowHome] = useState(true);
  const [queryRefreshToken, setQueryRefreshToken] = useState(0);

  const activeConnection = connections.find((c) => c.id === activeConnectionId) ?? null;

  useEffect(() => {
    listSavedConnections()
      .then(setConnections)
      .catch((err) => setConnectError(toErrorMessage(err)));
  }, []);

  function refreshSavedConnections() {
    listSavedConnections()
      .then(setConnections)
      .catch((err) => setConnectError(toErrorMessage(err)));
  }

  function handleConnected(profile: ConnectionProfile) {
    setConnectedIds((prev) => new Set(prev).add(profile.id));
    setActiveConnectionId(profile.id);
    refreshSavedConnections();
  }

  async function handleSelectConnection(connectionId: string) {
    setConnectError(null);
    if (connectedIds.has(connectionId)) {
      setActiveConnectionId(connectionId);
      return;
    }
    setConnectingId(connectionId);
    try {
      await connectSaved(connectionId);
      setConnectedIds((prev) => new Set(prev).add(connectionId));
      setActiveConnectionId(connectionId);
    } catch (err) {
      setConnectError(toErrorMessage(err));
    } finally {
      setConnectingId(null);
    }
  }

  async function handleDeleteConnection(connectionId: string) {
    if (connectedIds.has(connectionId)) {
      await disconnect(connectionId);
      setConnectedIds((prev) => {
        const next = new Set(prev);
        next.delete(connectionId);
        return next;
      });
    }
    if (activeConnectionId === connectionId) {
      setActiveConnectionId(null);
    }
    await deleteSavedConnection(connectionId);
    refreshSavedConnections();
  }

  function handleOpenTable(connectionId: string, schema: string, table: string) {
    const id = tableTabId(connectionId, schema, table);
    const connectionName = connections.find((c) => c.id === connectionId)?.name ?? "";

    setTabs((prev) =>
      prev.some((t) => t.id === id)
        ? prev
        : [...prev, { type: "table", id, connectionId, connectionName, schema, table }]
    );
    setActiveTabId(id);
    setShowHome(false);
  }

  function openQueryTab(id: string, connectionId: string, title: string, initialSql?: string) {
    const connectionName = connections.find((c) => c.id === connectionId)?.name ?? "";

    setTabs((prev) =>
      prev.some((t) => t.id === id)
        ? prev
        : [...prev, { type: "query", id, connectionId, connectionName, title, initialSql }]
    );
    setActiveTabId(id);
    setShowHome(false);
  }

  function handleNewQuery(connectionId?: string) {
    const targetConnectionId = connectionId ?? activeConnectionId;
    if (!targetConnectionId) return;
    const queryNumber = tabs.filter((t) => t.type === "query").length + 1;
    openQueryTab(createQueryTabId(), targetConnectionId, `Query ${queryNumber}`);
  }

  function handleOpenSavedQuery(query: SavedQuery) {
    openQueryTab(`query::saved::${query.id}`, query.connectionId, query.title, query.sql);
  }

  function handleOpenHistoryEntry(connectionId: string, sql: string) {
    const id = `query::history::${connectionId}::${hashSql(sql)}`;
    openQueryTab(id, connectionId, "History", sql);
  }

  function handleQueryActivity() {
    setQueryRefreshToken((prev) => prev + 1);
  }

  function handleSelectTab(id: string) {
    setActiveTabId(id);
    setShowHome(false);
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
    <ThemeProvider defaultTheme="dark" storageKey="queryon-theme">
      <div className="h-full">
        <AppLayout
          connections={connections}
          connectedIds={connectedIds}
          connectingId={connectingId}
          connectError={connectError}
          activeConnectionId={activeConnectionId}
          activeConnection={activeConnection}
          queryRefreshToken={queryRefreshToken}
          onSelectConnection={handleSelectConnection}
          onDeleteConnection={handleDeleteConnection}
          onOpenTable={handleOpenTable}
          onNewConnection={() => setDialogOpen(true)}
          onNewQuery={handleNewQuery}
          onOpenSavedQuery={handleOpenSavedQuery}
          onOpenHistoryEntry={handleOpenHistoryEntry}
          showHome={showHome}
          onGoHome={() => setShowHome(true)}
          tabs={tabs}
          activeTabId={activeTabId}
          onSelectTab={handleSelectTab}
          onCloseTab={handleCloseTab}
          onQueryActivity={handleQueryActivity}
        />

        <ConnectionDialog
          open={dialogOpen}
          onOpenChange={setDialogOpen}
          onConnected={handleConnected}
        />
      </div>
    </ThemeProvider>
  );
}

export default App;
