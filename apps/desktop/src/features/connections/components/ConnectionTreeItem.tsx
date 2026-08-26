import { useEffect, useState } from "react";
import {
  AlertTriangle,
  ChevronRight,
  Copy,
  Download,
  Eye,
  History,
  Loader2,
  Lock,
  MoreHorizontal,
  Pencil,
  Plug,
  PlugZap,
  Plus,
  Star,
  Table2,
  Trash2,
  XCircle,
} from "lucide-react";

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
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@queryon/ui/components/dropdown-menu";
import {
  SidebarMenuAction,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarMenuSub,
  SidebarMenuSubButton,
  SidebarMenuSubItem,
} from "@queryon/ui/components/sidebar";
import { RenameConnectionDialog } from "@/src/features/connections/components/RenameConnectionDialog";
import {
  engineOf,
  isDocumentEngine,
  toDisplayUrl,
  type Engine,
  type SavedConnectionProfile,
} from "@/src/features/connections/types";
import { CollectionBrowser } from "@/src/features/documents/components/CollectionBrowser";
import { docDefaultDatabase } from "@/src/features/documents/api";
import { MultiTableExportDialog } from "@/src/components/MultiTableExportDialog";
import { CreateTableDialog } from "@/src/features/schema/components/CreateTableDialog";
import { DropTableDialog } from "@/src/features/schema/components/DropTableDialog";
import { RenameTableDialog } from "@/src/features/schema/components/RenameTableDialog";
import { TableDdlDialog } from "@/src/features/schema/components/TableDdlDialog";
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
  collapseSignal: number;
  onSelect: () => void;
  onDisconnect: () => void;
  onDeleteConnection: () => void;
  onOpenTable: (schema: string, table: string) => void;
  onOpenCollection: (database: string, collection: string) => void;
  onOpenSavedQuery: (query: SavedQuery) => void;
  onOpenHistoryEntry: (sql: string) => void;
  onNewQuery: () => void;
  onConnectionRenamed: () => void;
}

export function ConnectionTreeItem({
  connection,
  isActive,
  isConnected,
  isConnecting,
  queryRefreshToken,
  collapseSignal,
  onSelect,
  onDisconnect,
  onDeleteConnection,
  onOpenTable,
  onOpenCollection,
  onOpenSavedQuery,
  onOpenHistoryEntry,
  onNewQuery,
  onConnectionRenamed,
}: ConnectionTreeItemProps) {
  const [expanded, setExpanded] = useState(isActive);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [renameOpen, setRenameOpen] = useState(false);
  const [exportOpen, setExportOpen] = useState(false);
  const [deleteOpen, setDeleteOpen] = useState(false);
  const [tables, setTables] = useState<TableRef[] | null>(null);
  const [defaultDatabase, setDefaultDatabase] = useState<string | null>(null);

  const isDocument = isDocumentEngine(engineOf(connection));

  useEffect(() => {
    if (isActive) setExpanded(true);
  }, [isActive]);

  useEffect(() => {
    if (!isConnected) {
      setExpanded(false);
      setTables(null);
      setError(null);
      setDefaultDatabase(null);
    }
  }, [isConnected]);

  useEffect(() => {
    if (collapseSignal > 0) setExpanded(false);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [collapseSignal]);

  useEffect(() => {
    if (!isDocument || !expanded || !isConnected) return;
    let cancelled = false;
    docDefaultDatabase(connection.id).then((db) => {
      if (!cancelled) setDefaultDatabase(db);
    });
    return () => {
      cancelled = true;
    };
  }, [isDocument, expanded, isConnected, connection.id]);

  useEffect(() => {
    if (isDocument || !expanded || !isConnected || tables !== null) return;

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
  }, [isDocument, expanded, isConnected, tables, connection.id]);

  function refetchTables() {
    listTables(connection.id)
      .then((result) => setTables(result))
      .catch((err) => setError(toErrorMessage(err)));
  }

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
        {connection.readOnly && (
          <Lock className="size-3 shrink-0 text-muted-foreground" aria-label="Read-only connection" />
        )}
      </SidebarMenuButton>

      <DropdownMenu>
        <DropdownMenuTrigger
          render={
            <SidebarMenuAction showOnHover title="More options">
              <MoreHorizontal />
            </SidebarMenuAction>
          }
        />
        <DropdownMenuContent align="start" side="right" className={"min-w-37.5"}>
          <DropdownMenuItem onClick={() => setRenameOpen(true)}>
            <Pencil className="size-3.5" />
            Rename
          </DropdownMenuItem>
          <DropdownMenuItem
            onClick={() => navigator.clipboard.writeText(toDisplayUrl(connection))}
          >
            <Copy className="size-3.5" />
            Copy Connection String
          </DropdownMenuItem>
          {isConnected && (
            <>
              {!isDocument && (
                <DropdownMenuItem onClick={() => setExportOpen(true)}>
                  <Download className="size-3.5" />
                  Export Tables…
                </DropdownMenuItem>
              )}
              <DropdownMenuItem variant="destructive" onClick={onDisconnect}>
                <PlugZap className="size-3.5" />
                Disconnect
              </DropdownMenuItem>
            </>
          )}
          <DropdownMenuItem variant="destructive" onClick={() => setDeleteOpen(true)}>
            <Trash2 className="size-3.5" />
            Delete Connection
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>

      <AlertDialog open={deleteOpen} onOpenChange={setDeleteOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogMedia className="bg-destructive/10 text-destructive">
              <AlertTriangle />
            </AlertDialogMedia>
            <AlertDialogTitle>Delete "{connection.name}"?</AlertDialogTitle>
            <AlertDialogDescription>
              This removes the saved connection and its password from the system keychain. This
              cannot be undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              variant="destructive"
              onClick={() => {
                setDeleteOpen(false);
                onDeleteConnection();
              }}
            >
              Delete
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      <MultiTableExportDialog
        connectionId={connection.id}
        open={exportOpen}
        onOpenChange={setExportOpen}
      />

      <RenameConnectionDialog
        connectionId={connection.id}
        currentName={connection.name}
        open={renameOpen}
        onOpenChange={setRenameOpen}
        onRenamed={onConnectionRenamed}
      />

      {expanded && (
        <SidebarMenuSub>
          {!isConnected && !isConnecting && (
            <SidebarMenuSubItem>
              <p className="px-2 py-1 text-xs text-muted-foreground">
                Not connected — click to connect
              </p>
            </SidebarMenuSubItem>
          )}

          {isDocument && isConnected && (
            <CollectionBrowser
              connectionId={connection.id}
              defaultDatabase={defaultDatabase}
              onOpenCollection={onOpenCollection}
            />
          )}

          {!isDocument && loading && (
            <SidebarMenuSubItem>
              <div className="flex items-center gap-2 px-2 py-1 text-xs text-muted-foreground">
                <Loader2 className="size-3 animate-spin" />
                Loading tables…
              </div>
            </SidebarMenuSubItem>
          )}

          {!isDocument && error && (
            <SidebarMenuSubItem>
              <p className="px-2 py-1 text-xs text-destructive">{error}</p>
            </SidebarMenuSubItem>
          )}

          {!isDocument && !loading && !error && isConnected && tables?.length === 0 && (
            <SidebarMenuSubItem>
              <p className="px-2 py-1 text-xs text-muted-foreground">
                No tables found
              </p>
            </SidebarMenuSubItem>
          )}

          {!isDocument &&
            Object.entries(schemas).map(([schemaName, schemaTables]) => (
              <SchemaGroup
                key={schemaName}
                connectionId={connection.id}
                engine={engineOf(connection)}
                schema={schemaName}
                tables={schemaTables}
                onOpenTable={onOpenTable}
                onTableChanged={refetchTables}
              />
            ))}

          {!isDocument && (
            <>
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
            </>
          )}
        </SidebarMenuSub>
      )}
    </SidebarMenuItem>
  );
}

function SchemaGroup({
  connectionId,
  engine,
  schema,
  tables,
  onOpenTable,
  onTableChanged,
}: {
  connectionId: string;
  engine: Engine;
  schema: string;
  tables: TableRef[];
  onOpenTable: (schema: string, table: string) => void;
  onTableChanged: () => void;
}) {
  const [open, setOpen] = useState(schema === "public");
  const [createOpen, setCreateOpen] = useState(false);

  return (
    <>
      <SidebarMenuSubItem>
        <div className="group/schema relative flex items-center">
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
          <button
            type="button"
            title="New table"
            onClick={(e) => {
              e.stopPropagation();
              setCreateOpen(true);
            }}
            className="absolute right-1 flex size-5 items-center justify-center rounded-md opacity-0 hover:bg-sidebar-accent group-hover/schema:opacity-100"
          >
            <Plus className="size-3.5" />
          </button>
        </div>
      </SidebarMenuSubItem>
      {open &&
        tables.map((t) => (
          <TableRow
            key={`${t.schema}.${t.name}`}
            connectionId={connectionId}
            table={t}
            onOpenTable={onOpenTable}
            onTableChanged={onTableChanged}
          />
        ))}

      <CreateTableDialog
        connectionId={connectionId}
        engine={engine}
        schema={schema}
        open={createOpen}
        onOpenChange={setCreateOpen}
        onCreated={onTableChanged}
      />
    </>
  );
}

function TableRow({
  connectionId,
  table,
  onOpenTable,
  onTableChanged,
}: {
  connectionId: string;
  table: TableRef;
  onOpenTable: (schema: string, table: string) => void;
  onTableChanged: () => void;
}) {
  const [ddlOpen, setDdlOpen] = useState(false);
  const [renameOpen, setRenameOpen] = useState(false);
  const [dropOpen, setDropOpen] = useState(false);

  return (
    <SidebarMenuSubItem className="group/table relative">
      <SidebarMenuSubButton onClick={() => onOpenTable(table.schema, table.name)}>
        {table.kind === "table" ? (
          <Table2 className="size-3.5" />
        ) : (
          <Eye className="size-3.5" />
        )}
        <span>{table.name}</span>
      </SidebarMenuSubButton>

      <DropdownMenu>
        <DropdownMenuTrigger
          render={
            <button
              type="button"
              title="More options"
              className="absolute top-1/2 right-1 flex size-5 -translate-y-1/2 items-center justify-center rounded-md opacity-0 hover:bg-sidebar-accent group-hover/table:opacity-100"
            >
              <MoreHorizontal className="size-3.5" />
            </button>
          }
        />
        <DropdownMenuContent align="start" side="right">
          <DropdownMenuItem onClick={() => setDdlOpen(true)}>View DDL</DropdownMenuItem>
          {table.kind === "table" && (
            <>
              <DropdownMenuItem onClick={() => setRenameOpen(true)}>
                <Pencil className="size-3.5" />
                Rename Table
              </DropdownMenuItem>
              <DropdownMenuItem variant="destructive" onClick={() => setDropOpen(true)}>
                <Trash2 className="size-3.5" />
                Drop Table
              </DropdownMenuItem>
            </>
          )}
        </DropdownMenuContent>
      </DropdownMenu>

      <TableDdlDialog
        connectionId={connectionId}
        schema={table.schema}
        table={table.name}
        open={ddlOpen}
        onOpenChange={setDdlOpen}
      />

      <RenameTableDialog
        connectionId={connectionId}
        schema={table.schema}
        table={table.name}
        open={renameOpen}
        onOpenChange={setRenameOpen}
        onRenamed={onTableChanged}
      />

      <DropTableDialog
        connectionId={connectionId}
        schema={table.schema}
        table={table.name}
        open={dropOpen}
        onOpenChange={setDropOpen}
        onDropped={onTableChanged}
      />
    </SidebarMenuSubItem>
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
