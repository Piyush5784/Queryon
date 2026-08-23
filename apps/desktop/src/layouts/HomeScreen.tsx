import { useState } from "react";
import { AlertTriangle, Database, Loader2, Plus, Trash2, XCircle } from "lucide-react";

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
} from "@/src/app/components/ui/alert-dialog";
import { Button } from "@/src/app/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/src/app/components/ui/card";
import {
  Empty,
  EmptyContent,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "@/src/app/components/ui/empty";
import { EngineIcon } from "@/src/features/connections/components/EngineIcon";
import { engineOf, toDisplayUrl, type SavedConnectionProfile } from "@/src/features/connections/types";

interface HomeScreenProps {
  connections: SavedConnectionProfile[];
  connectedIds: Set<string>;
  connectingId: string | null;
  connectError: string | null;
  activeConnectionId: string | null;
  onSelectConnection: (id: string) => void;
  onDeleteConnection: (id: string) => void;
  onNewConnection: () => void;
}

export function HomeScreen({
  connections,
  connectedIds,
  connectingId,
  connectError,
  activeConnectionId,
  onSelectConnection,
  onDeleteConnection,
  onNewConnection,
}: HomeScreenProps) {
  const [deleteTarget, setDeleteTarget] = useState<SavedConnectionProfile | null>(null);

  function handleConfirmDelete() {
    if (!deleteTarget) return;
    onDeleteConnection(deleteTarget.id);
    setDeleteTarget(null);
  }

  if (connections.length === 0) {
    return (
      <div className="flex flex-1 items-center justify-center p-6">
        <Empty className="max-w-md">
          <EmptyHeader>
            <EmptyMedia variant="icon">
              <Database />
            </EmptyMedia>
            <EmptyTitle>No connections yet</EmptyTitle>
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
    <div className="min-h-0 flex-1 overflow-auto p-6">
      <div className="mx-auto flex max-w-3xl flex-col gap-4">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="font-heading text-lg font-medium text-foreground">Connections</h1>
            <p className="text-sm text-muted-foreground">
              {connections.length} saved connection{connections.length === 1 ? "" : "s"}
            </p>
          </div>
          <Button size="sm" className="gap-2" onClick={onNewConnection}>
            <Plus className="size-4" />
            New Connection
          </Button>
        </div>

        {connectError && (
          <div className="flex items-start gap-2 rounded-lg border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive">
            <XCircle className="size-4 shrink-0 translate-y-0.5" />
            <span className="wrap-break-word">{connectError}</span>
          </div>
        )}

        <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
          {connections.map((conn) => {
            const isConnected = connectedIds.has(conn.id);
            const isConnecting = connectingId === conn.id;
            return (
              <Card
                key={conn.id}
                onClick={() => onSelectConnection(conn.id)}
                className={`group cursor-pointer transition-colors hover:border-primary/50 ${
                  conn.id === activeConnectionId ? "border-primary/60 bg-primary/5" : ""
                }`}
              >
                <CardHeader>
                  <div className="flex items-center gap-2">
                    <EngineIcon engine={engineOf(conn)} className="size-4 shrink-0" />
                    <CardTitle className="truncate">{conn.name}</CardTitle>
                    <Button
                      variant="ghost"
                      size="icon-xs"
                      className="ml-auto shrink-0 opacity-0 group-hover:opacity-100"
                      onClick={(e) => {
                        e.stopPropagation();
                        setDeleteTarget(conn);
                      }}
                    >
                      <Trash2 className="size-3.5" />
                    </Button>
                  </div>
                  <CardDescription className="truncate font-mono text-xs">
                    {toDisplayUrl(conn)}
                  </CardDescription>
                </CardHeader>
                <CardContent>
                  <span className="flex items-center gap-1.5 text-xs font-medium">
                    {isConnecting ? (
                      <>
                        <Loader2 className="size-3 animate-spin text-muted-foreground" />
                        <span className="text-muted-foreground">Connecting…</span>
                      </>
                    ) : isConnected ? (
                      <>
                        <span className="size-1.5 rounded-full bg-emerald-500" />
                        <span className="text-primary">Connected</span>
                      </>
                    ) : (
                      <>
                        <span className="size-1.5 rounded-full bg-muted-foreground/40" />
                        <span className="text-muted-foreground">Offline</span>
                      </>
                    )}
                  </span>
                </CardContent>
              </Card>
            );
          })}
        </div>
      </div>

      <AlertDialog
        open={deleteTarget !== null}
        onOpenChange={(open) => !open && setDeleteTarget(null)}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogMedia className="bg-destructive/10 text-destructive">
              <AlertTriangle />
            </AlertDialogMedia>
            <AlertDialogTitle>Delete "{deleteTarget?.name}"?</AlertDialogTitle>
            <AlertDialogDescription>
              This removes the saved connection and its password from the system
              keychain. This cannot be undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction variant="destructive" onClick={handleConfirmDelete}>
              Delete
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}
