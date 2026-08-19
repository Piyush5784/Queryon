import { useEffect, useState } from "react";
import { ChevronRight, Eye, Loader2, Plug, Table2 } from "lucide-react";

import {
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarMenuSub,
  SidebarMenuSubButton,
  SidebarMenuSubItem,
} from "@/src/app/components/ui/sidebar";
import type { SavedConnectionProfile } from "@/src/features/connections/types";
import { listTables, type TableRef } from "@/src/features/tables/api";
import { toErrorMessage } from "@/src/lib/tauri/errors";

interface ConnectionTreeItemProps {
  connection: SavedConnectionProfile;
  isActive: boolean;
  isConnected: boolean;
  isConnecting: boolean;
  onSelect: () => void;
  onOpenTable: (schema: string, table: string) => void;
}

export function ConnectionTreeItem({
  connection,
  isActive,
  isConnected,
  isConnecting,
  onSelect,
  onOpenTable,
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

function groupBySchema(tables: TableRef[]): Record<string, TableRef[]> {
  const groups: Record<string, TableRef[]> = {};
  for (const t of tables) {
    (groups[t.schema] ??= []).push(t);
  }
  return groups;
}
