import { useEffect, useState } from "react";
import { ChevronRight, Eye, History, Loader2, Plug, Plus, Star, Table2, Trash2, XCircle } from "lucide-react";

import {
  SidebarMenuAction,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarMenuSub,
  SidebarMenuSubButton,
  SidebarMenuSubItem,
} from "@/src/app/components/ui/sidebar";
import type { SavedConnectionProfile } from "@/src/features/connections/types";
import {
  clearQueryHistory,
  deleteSavedQuery,
  listQueryHistory,
  listSavedQueries,
  type QueryHistoryEntry,
  type SavedQuery,
} from "@/src/features/query/api";
import { listTables, type TableRef } from "@/src/features/tables/api";
import { toErrorMessage } from "@/src/lib/tauri/errors";

interface ConnectionTreeItemProps {
  connection: SavedConnectionProfile;
  isActive: boolean;
  isConnected: boolean;
  isConnecting: boolean;
  queryRefreshToken: number;
  onSelect: () => void;
  onOpenTable: (schema: string, table: string) => void;
  onOpenSavedQuery: (query: SavedQuery) => void;
  onOpenHistoryEntry: (sql: string) => void;
  onNewQuery: () => void;
}

export function ConnectionTreeItem({
  connection,
  isActive,
  isConnected,
  isConnecting,
  queryRefreshToken,
  onSelect,
  onOpenTable,
  onOpenSavedQuery,
  onOpenHistoryEntry,
  onNewQuery,
}: ConnectionTreeItemProps) {
  const [expanded, setExpanded] = useState(isActive);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [tables, setTables] = useState<TableRef[] | null>(null);

  useEffect(() => {
    if (isActive) setExpanded(true);
  }, [isActive]);

  useEffect(() => {
    if (!expanded || !isConnected || tables !== null) return;

    let cancelled = false;
    setLoading(true);
    setError(null);

    listTables(connection.id)
      .then((result) => {
        if (!cancelled) setTables(result);
      })
      .catch((err) => {
        if (!cancelled) setError(toErrorMessage(err));
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });

    return () => {
      cancelled = true;
    };
  }, [expanded, isConnected, tables, connection.id]);

  const schemas = groupBySchema(tables ?? []);

  return (
    <SidebarMenuItem>
      <SidebarMenuButton
        tooltip={connection.name}
        isActive={isActive}
        onClick={() => {
          onSelect();
          setExpanded((prev) => !prev);
        }}
      >
        <ChevronRight
          className={`size-3.5 shrink-0 transition-transform ${expanded ? "rotate-90" : ""}`}
        />
        {isConnecting ? (
          <Loader2 className="size-4 animate-spin" />
        ) : (
          <Plug className={`size-4 ${isConnected ? "" : "text-muted-foreground"}`} />
        )}
        <span className={isConnected ? "" : "text-muted-foreground"}>{connection.name}</span>
      </SidebarMenuButton>

      {expanded && (
        <SidebarMenuSub>
          {!isConnected && !isConnecting && (
            <SidebarMenuSubItem>
              <p className="px-2 py-1 text-xs text-muted-foreground">
                Not connected — click to connect
              </p>
            </SidebarMenuSubItem>
          )}

          {loading && (
            <SidebarMenuSubItem>
              <div className="flex items-center gap-2 px-2 py-1 text-xs text-muted-foreground">
                <Loader2 className="size-3 animate-spin" />
                Loading tables…
              </div>
            </SidebarMenuSubItem>
          )}

          {error && (
            <SidebarMenuSubItem>
              <p className="px-2 py-1 text-xs text-destructive">{error}</p>
            </SidebarMenuSubItem>
          )}

          {!loading && !error && isConnected && tables?.length === 0 && (
            <SidebarMenuSubItem>
              <p className="px-2 py-1 text-xs text-muted-foreground">
                No tables found
              </p>
            </SidebarMenuSubItem>
          )}

          {Object.entries(schemas).map(([schemaName, schemaTables]) => (
            <SchemaGroup
              key={schemaName}
              schema={schemaName}
              tables={schemaTables}
              onOpenTable={onOpenTable}
            />
          ))}

          <SavedQueriesGroup
            connectionId={connection.id}
            refreshToken={queryRefreshToken}
            onOpenQuery={onOpenSavedQuery}
            onNewQuery={onNewQuery}
          />

          <QueryHistoryGroup
            connectionId={connection.id}
            refreshToken={queryRefreshToken}
            onOpenQuery={onOpenHistoryEntry}
            onNewQuery={onNewQuery}
          />
        </SidebarMenuSub>
      )}
    </SidebarMenuItem>
  );
}

function SchemaGroup({
  schema,
  tables,
  onOpenTable,
}: {
  schema: string;
  tables: TableRef[];
  onOpenTable: (schema: string, table: string) => void;
}) {
  const [open, setOpen] = useState(schema === "public");

  return (
    <>
      <SidebarMenuSubItem>
        <button
          type="button"
          onClick={() => setOpen((prev) => !prev)}
          className="flex w-full items-center gap-1.5 rounded-md px-2 py-1 text-xs font-medium text-muted-foreground hover:bg-sidebar-accent"
        >
          <ChevronRight
            className={`size-3 shrink-0 transition-transform ${open ? "rotate-90" : ""}`}
          />
          {schema}
        </button>
      </SidebarMenuSubItem>
      {open &&
        tables.map((t) => (
          <SidebarMenuSubItem key={`${t.schema}.${t.name}`}>
            <SidebarMenuSubButton onClick={() => onOpenTable(t.schema, t.name)}>
              {t.kind === "table" ? (
                <Table2 className="size-3.5" />
              ) : (
                <Eye className="size-3.5" />
              )}
              <span>{t.name}</span>
            </SidebarMenuSubButton>
          </SidebarMenuSubItem>
        ))}
    </>
  );
}

function SavedQueriesGroup({
  connectionId,
  refreshToken,
  onOpenQuery,
  onNewQuery,
}: {
  connectionId: string;
  refreshToken: number;
  onOpenQuery: (query: SavedQuery) => void;
  onNewQuery: () => void;
}) {
  const [open, setOpen] = useState(false);
  const [queries, setQueries] = useState<SavedQuery[] | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!open) return;
    let cancelled = false;
    listSavedQueries(connectionId)
      .then((result) => {
        if (!cancelled) setQueries(result);
      })
      .catch((err) => {
        if (!cancelled) setError(toErrorMessage(err));
      });
    return () => {
      cancelled = true;
    };
  }, [open, connectionId, refreshToken]);

  async function handleDelete(query: SavedQuery) {
    setQueries((prev) => (prev ? prev.filter((q) => q.id !== query.id) : prev));
    try {
      await deleteSavedQuery(query.id);
    } catch (err) {
      setError(toErrorMessage(err));
    }
  }

  return (
    <>
      <SidebarMenuSubItem>
        <div className="group/saved relative flex items-center">
          <button
            type="button"
            onClick={() => setOpen((prev) => !prev)}
            className="flex w-full items-center gap-1.5 rounded-md px-2 py-1 text-xs font-medium text-muted-foreground hover:bg-sidebar-accent"
          >
            <ChevronRight className={`size-3 shrink-0 transition-transform ${open ? "rotate-90" : ""}`} />
            <Star className="size-3" />
            Saved Queries
          </button>
          <button
            type="button"
            title="New query"
            onClick={(e) => {
              e.stopPropagation();
              onNewQuery();
            }}
            className="absolute right-1 flex size-5 items-center justify-center rounded-md opacity-0 hover:bg-sidebar-accent group-hover/saved:opacity-100"
          >
            <Plus className="size-3.5" />
          </button>
        </div>
      </SidebarMenuSubItem>
      {open && (
        <>
          {error && (
            <SidebarMenuSubItem>
              <p className="px-2 py-1 text-xs text-destructive">{error}</p>
            </SidebarMenuSubItem>
          )}
          {!error && queries?.length === 0 && (
            <SidebarMenuSubItem>
              <p className="px-2 py-1 text-xs text-muted-foreground">No saved queries</p>
            </SidebarMenuSubItem>
          )}
          {queries?.map((query) => (
            <SidebarMenuSubItem key={query.id} className="group/menu-item relative">
              <SidebarMenuSubButton onClick={() => onOpenQuery(query)}>
                <Star className="size-3.5" />
                <span className="truncate">{query.title}</span>
              </SidebarMenuSubButton>
              <SidebarMenuAction showOnHover onClick={() => handleDelete(query)}>
                <Trash2 />
              </SidebarMenuAction>
            </SidebarMenuSubItem>
          ))}
        </>
      )}
    </>
  );
}

function QueryHistoryGroup({
  connectionId,
  refreshToken,
  onOpenQuery,
  onNewQuery,
}: {
  connectionId: string;
  refreshToken: number;
  onOpenQuery: (sql: string) => void;
  onNewQuery: () => void;
}) {
  const [open, setOpen] = useState(false);
  const [entries, setEntries] = useState<QueryHistoryEntry[] | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!open) return;
    let cancelled = false;
    listQueryHistory(connectionId)
      .then((result) => {
        if (!cancelled) setEntries(result);
      })
      .catch((err) => {
        if (!cancelled) setError(toErrorMessage(err));
      });
    return () => {
      cancelled = true;
    };
  }, [open, connectionId, refreshToken]);

  async function handleClear() {
    setEntries([]);
    try {
      await clearQueryHistory(connectionId);
    } catch (err) {
      setError(toErrorMessage(err));
    }
  }

  return (
    <>
      <SidebarMenuSubItem>
        <div className="group/history relative flex items-center">
          <button
            type="button"
            onClick={() => setOpen((prev) => !prev)}
            className="flex w-full items-center gap-1.5 rounded-md px-2 py-1 text-xs font-medium text-muted-foreground hover:bg-sidebar-accent"
          >
            <ChevronRight className={`size-3 shrink-0 transition-transform ${open ? "rotate-90" : ""}`} />
            <History className="size-3" />
            Query History
          </button>
          {open && entries && entries.length > 0 && (
            <button
              type="button"
              title="Clear history"
              onClick={handleClear}
              className="absolute right-6.5 flex size-5 items-center justify-center rounded-md opacity-0 hover:bg-sidebar-accent group-hover/history:opacity-100"
            >
              <Trash2 className="size-3.5" />
            </button>
          )}
          <button
            type="button"
            title="New query"
            onClick={(e) => {
              e.stopPropagation();
              onNewQuery();
            }}
            className="absolute right-1 flex size-5 items-center justify-center rounded-md opacity-0 hover:bg-sidebar-accent group-hover/history:opacity-100"
          >
            <Plus className="size-3.5" />
          </button>
        </div>
      </SidebarMenuSubItem>
      {open && (
        <>
          {error && (
            <SidebarMenuSubItem>
              <p className="px-2 py-1 text-xs text-destructive">{error}</p>
            </SidebarMenuSubItem>
          )}
          {!error && entries?.length === 0 && (
            <SidebarMenuSubItem>
              <p className="px-2 py-1 text-xs text-muted-foreground">No queries run yet</p>
            </SidebarMenuSubItem>
          )}
          {entries?.slice(0, 25).map((entry) => (
            <SidebarMenuSubItem key={entry.id}>
              <SidebarMenuSubButton onClick={() => onOpenQuery(entry.sql)} className="h-auto py-1">
                {entry.status === "error" ? (
                  <XCircle className="size-3.5 shrink-0 text-destructive" />
                ) : (
                  <span className="size-1.5 shrink-0 rounded-full bg-emerald-500" />
                )}
                <span className="truncate font-mono text-xs">{entry.sql}</span>
              </SidebarMenuSubButton>
            </SidebarMenuSubItem>
          ))}
        </>
      )}
    </>
  );
}

function groupBySchema(tables: TableRef[]): Record<string, TableRef[]> {
  const groups: Record<string, TableRef[]> = {};
  for (const t of tables) {
    (groups[t.schema] ??= []).push(t);
  }
  return groups;
}
