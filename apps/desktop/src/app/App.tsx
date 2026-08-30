import { useEffect, useState } from "react";

import {
  connectSaved,
  deleteSavedConnection,
  disconnect,
  listSavedConnections,
} from "@/src/features/connections/api";
import { ConnectionDialog } from "@/src/features/connections/components/ConnectionDialog";
import { engineOf, isDocumentEngine, type ConnectionProfile, type SavedConnectionProfile } from "@/src/features/connections/types";
import { collectionTabId, databaseTabId, docConnectSaved, docDisconnect } from "@/src/features/documents/api";
import type { SavedQuery } from "@/src/features/query/api";
import { clearQueryDraft } from "@/src/features/query/queryDrafts";
import { isTabRunning } from "@/src/features/query/runningTabs";
import { createQueryTabId } from "@/src/features/query/types";
import { tableTabId } from "@/src/features/tables/types";
import { AppLayout } from "@/src/layouts/AppLayout";
import { ThemeProvider } from "@/src/components/theme-provider";
import { UpdateChecker } from "@/src/components/UpdateChecker";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogMedia,
  AlertDialogTitle,
} from "@queryon/ui/components/alert-dialog";
import { Toaster, toast } from "@queryon/ui/components/toast";
import type { AppTab } from "@/src/app/tabs";
import { detachTab, onRedockTab } from "@/src/app/detachedWindow";
import { useAppKeyboardShortcuts } from "@/src/app/useAppKeyboardShortcuts";
import { toErrorMessage } from "@/src/lib/tauri/errors";
import { AlertTriangle } from "lucide-react";
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
  const [closeTabConfirmId, setCloseTabConfirmId] = useState<string | null>(null);

  const activeConnection = connections.find((c) => c.id === activeConnectionId) ?? null;

  useEffect(() => {
    refreshSavedConnections();
  }, []);

  useEffect(() => {
    const unlisten = onRedockTab((tab) => {
      setTabs((prev) => (prev.some((t) => t.id === tab.id) ? prev : [...prev, tab]));
      setActiveTabId(tab.id);
      setShowHome(false);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
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
      const connection = connections.find((c) => c.id === connectionId);
      if (isDocumentEngine(engineOf(connection ?? {}))) {
        await docConnectSaved(connectionId);
      } else {
        await connectSaved(connectionId);
      }
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
      const connection = connections.find((c) => c.id === connectionId);
      if (isDocumentEngine(engineOf(connection ?? {}))) {
        await docDisconnect(connectionId);
      } else {
        await disconnect(connectionId);
      }
      setConnectedIds((prev) => {
        const next = new Set(prev);
        next.delete(connectionId);
        return next;
      });
    }
    if (activeConnectionId === connectionId) {
      setActiveConnectionId(null);
    }
    setTabs((prev) => {
      const next = prev.filter((t) => t.connectionId !== connectionId);
      prev.forEach((t) => {
        if (t.connectionId === connectionId) clearQueryDraft(t.id);
      });
      if (activeTabId && !next.some((t) => t.id === activeTabId)) {
        setActiveTabId(next.length > 0 ? next[next.length - 1].id : null);
      }
      return next;
    });
    await deleteSavedConnection(connectionId);
    refreshSavedConnections();
  }

  async function handleDisconnect(connectionId: string) {
    if (!connectedIds.has(connectionId)) return;
    const connection = connections.find((c) => c.id === connectionId);
    if (isDocumentEngine(engineOf(connection ?? {}))) {
      await docDisconnect(connectionId);
    } else {
      await disconnect(connectionId);
    }
    setConnectedIds((prev) => {
      const next = new Set(prev);
      next.delete(connectionId);
      return next;
    });
    if (activeConnectionId === connectionId) {
      setActiveConnectionId(null);
    }
    setTabs((prev) => {
      const next = prev.filter((t) => t.connectionId !== connectionId);
      prev.forEach((t) => {
        if (t.connectionId === connectionId) clearQueryDraft(t.id);
      });
      return next;
    });
  }

  function handleOpenTable(connectionId: string, schema: string, table: string) {
    const id = tableTabId(connectionId, schema, table);
    const connection = connections.find((c) => c.id === connectionId);
    const connectionName = connection?.name ?? "";
    const engine = engineOf(connection ?? {});

    setTabs((prev) =>
      prev.some((t) => t.id === id)
        ? prev
        : [...prev, { type: "table", id, connectionId, connectionName, engine, schema, table }]
    );
    setActiveTabId(id);
    setShowHome(false);
  }

  function handleOpenDatabase(connectionId: string, database: string) {
    const id = databaseTabId(connectionId, database);
    const connection = connections.find((c) => c.id === connectionId);
    const connectionName = connection?.name ?? "";

    setTabs((prev) =>
      prev.some((t) => t.id === id)
        ? prev
        : [...prev, { type: "database", id, connectionId, connectionName, database }]
    );
    setActiveTabId(id);
    setShowHome(false);
  }

  function handleOpenCollection(connectionId: string, database: string, collection: string) {
    const id = collectionTabId(connectionId, database, collection);
    const connection = connections.find((c) => c.id === connectionId);
    const connectionName = connection?.name ?? "";

    setTabs((prev) =>
      prev.some((t) => t.id === id)
        ? prev
        : [...prev, { type: "collection", id, connectionId, connectionName, database, collection }]
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

  function closeTab(id: string) {
    clearQueryDraft(id);
    setTabs((prev) => {
      const next = prev.filter((t) => t.id !== id);
      if (activeTabId === id) {
        setActiveTabId(next.length > 0 ? next[next.length - 1].id : null);
      }
      return next;
    });
  }

  function handleCloseTab(id: string) {
    if (isTabRunning(id)) {
      setCloseTabConfirmId(id);
      return;
    }
    closeTab(id);
  }

  function handleReorderTabs(fromId: string, toId: string) {
    if (fromId === toId) return;
    setTabs((prev) => {
      const fromIndex = prev.findIndex((t) => t.id === fromId);
      const toIndex = prev.findIndex((t) => t.id === toId);
      if (fromIndex === -1 || toIndex === -1) return prev;
      const next = [...prev];
      [next[fromIndex], next[toIndex]] = [next[toIndex], next[fromIndex]];
      return next;
    });
  }

  async function handleDetachTab(id: string) {
    const tab = tabs.find((t) => t.id === id);
    if (!tab) return;
    try {
      await detachTab(tab);
      setTabs((prev) => {
        const next = prev.filter((t) => t.id !== id);
        if (activeTabId === id) {
          setActiveTabId(next.length > 0 ? next[next.length - 1].id : null);
        }
        return next;
      });
    } catch (err) {
      toast.add({ type: "error", title: "Failed to detach tab", description: toErrorMessage(err) });
    }
  }

  useAppKeyboardShortcuts({
    activeTabId,
    tabs,
    onCloseActiveTab: handleCloseTab,
    onNewQuery: () => handleNewQuery(),
    onNewConnection: () => setDialogOpen(true),
    onSelectTab: handleSelectTab,
  });

  return (
    <ThemeProvider defaultTheme="dark" storageKey="queryon-theme">
      <Toaster>
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
          onDisconnect={handleDisconnect}
          onOpenTable={handleOpenTable}
          onOpenDatabase={handleOpenDatabase}
          onOpenCollection={handleOpenCollection}
          onNewConnection={() => setDialogOpen(true)}
          onNewQuery={handleNewQuery}
          onOpenSavedQuery={handleOpenSavedQuery}
          onOpenHistoryEntry={handleOpenHistoryEntry}
          onConnectionRenamed={refreshSavedConnections}
          showHome={showHome}
          onGoHome={() => setShowHome(true)}
          tabs={tabs}
          activeTabId={activeTabId}
          onSelectTab={handleSelectTab}
          onCloseTab={handleCloseTab}
          onReorderTabs={handleReorderTabs}
          onDetachTab={handleDetachTab}
          onQueryActivity={handleQueryActivity}
        />

        <ConnectionDialog
          open={dialogOpen}
          onOpenChange={setDialogOpen}
          onConnected={handleConnected}
        />

        <AlertDialog
          open={closeTabConfirmId !== null}
          onOpenChange={(next) => !next && setCloseTabConfirmId(null)}
        >
          <AlertDialogContent>
            <AlertDialogHeader>
              <AlertDialogMedia className="bg-amber-500/10 text-amber-600 dark:text-amber-400">
                <AlertTriangle />
              </AlertDialogMedia>
              <AlertDialogTitle>Close tab while query is running?</AlertDialogTitle>
              <AlertDialogDescription>
                This tab has a query still running. Closing it now will cancel the query, or leave
                it half-performed if it can't be cancelled in time.
              </AlertDialogDescription>
            </AlertDialogHeader>

            <AlertDialogFooter>
              <AlertDialogCancel>Keep tab open</AlertDialogCancel>
              <AlertDialogAction
                variant="destructive"
                onClick={() => {
                  if (closeTabConfirmId) closeTab(closeTabConfirmId);
                  setCloseTabConfirmId(null);
                }}
              >
                Close anyway
              </AlertDialogAction>
            </AlertDialogFooter>
          </AlertDialogContent>
        </AlertDialog>

        <UpdateChecker />
      </div>
      </Toaster>
    </ThemeProvider>
  );
}

export default App;
