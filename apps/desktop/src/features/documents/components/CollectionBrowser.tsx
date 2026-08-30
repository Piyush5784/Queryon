import { useEffect, useState } from "react";
import { ChevronRight, Database, Loader2 } from "lucide-react";

import {
  SidebarMenuSub,
  SidebarMenuSubButton,
  SidebarMenuSubItem,
} from "@queryon/ui/components/sidebar";
import { docListCollections, docListDatabases, type CollectionRef, type DatabaseRef } from "@/src/features/documents/api";
import { toErrorMessage } from "@/src/lib/tauri/errors";

interface CollectionBrowserProps {
  connectionId: string;
  defaultDatabase: string | null;
  onOpenDatabase: (database: string) => void;
  onOpenCollection: (database: string, collection: string) => void;
}

export function CollectionBrowser({
  connectionId,
  defaultDatabase,
  onOpenDatabase,
  onOpenCollection,
}: CollectionBrowserProps) {
  const [databases, setDatabases] = useState<DatabaseRef[] | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(null);

    docListDatabases(connectionId)
      .then((result) => {
        if (!cancelled) setDatabases(result);
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
  }, [connectionId]);

  if (loading) {
    return (
      <SidebarMenuSubItem>
        <div className="flex items-center gap-2 px-2 py-1 text-xs text-muted-foreground">
          <Loader2 className="size-3 animate-spin" />
          Loading databases…
        </div>
      </SidebarMenuSubItem>
    );
  }

  if (error) {
    return (
      <SidebarMenuSubItem>
        <p className="px-2 py-1 text-xs text-destructive">{error}</p>
      </SidebarMenuSubItem>
    );
  }

  if (!databases || databases.length === 0) {
    return (
      <SidebarMenuSubItem>
        <p className="px-2 py-1 text-xs text-muted-foreground">No databases found</p>
      </SidebarMenuSubItem>
    );
  }

  return (
    <>
      {databases.map((db) => (
        <DatabaseGroup
          key={db.name}
          connectionId={connectionId}
          database={db.name}
          defaultOpen={db.name === defaultDatabase}
          onOpenDatabase={onOpenDatabase}
          onOpenCollection={onOpenCollection}
        />
      ))}
    </>
  );
}

function DatabaseGroup({
  connectionId,
  database,
  defaultOpen,
  onOpenDatabase,
  onOpenCollection,
}: {
  connectionId: string;
  database: string;
  defaultOpen: boolean;
  onOpenDatabase: (database: string) => void;
  onOpenCollection: (database: string, collection: string) => void;
}) {
  const [open, setOpen] = useState(defaultOpen);
  const [collections, setCollections] = useState<CollectionRef[] | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!open || collections !== null) return;

    let cancelled = false;
    setLoading(true);
    setError(null);

    docListCollections(connectionId, database)
      .then((result) => {
        if (!cancelled) setCollections(result);
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
  }, [open, collections, connectionId, database]);

  return (
    <>
      <SidebarMenuSubItem>
        <button
          type="button"
          onClick={() => {
            setOpen((prev) => !prev);
            onOpenDatabase(database);
          }}
          className="flex w-full items-center gap-1.5 rounded-md px-2 py-1 text-xs font-medium text-muted-foreground hover:bg-sidebar-accent"
        >
          <ChevronRight className={`size-3 shrink-0 transition-transform ${open ? "rotate-90" : ""}`} />
          <Database className="size-3" />
          {database}
        </button>
      </SidebarMenuSubItem>

      {open && (
        <SidebarMenuSub className="mx-0 border-l-0 pl-3">
          {loading && (
            <SidebarMenuSubItem>
              <div className="flex items-center gap-2 px-2 py-1 text-xs text-muted-foreground">
                <Loader2 className="size-3 animate-spin" />
                Loading collections…
              </div>
            </SidebarMenuSubItem>
          )}

          {error && (
            <SidebarMenuSubItem>
              <p className="px-2 py-1 text-xs text-destructive">{error}</p>
            </SidebarMenuSubItem>
          )}

          {!loading && !error && collections?.length === 0 && (
            <SidebarMenuSubItem>
              <p className="px-2 py-1 text-xs text-muted-foreground">No collections</p>
            </SidebarMenuSubItem>
          )}

          {collections?.map((coll) => (
            <SidebarMenuSubItem key={coll.name}>
              <SidebarMenuSubButton onClick={() => onOpenCollection(database, coll.name)}>
                <span className="truncate">{coll.name}</span>
                <span className="ml-auto shrink-0 text-[0.65rem] text-muted-foreground">
                  {(coll.estimatedCount ?? 0).toLocaleString()}
                </span>
              </SidebarMenuSubButton>
            </SidebarMenuSubItem>
          ))}
        </SidebarMenuSub>
      )}
    </>
  );
}
