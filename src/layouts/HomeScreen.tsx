import { Database, Plug, Plus } from "lucide-react";

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
import { toDisplayUrl, type ConnectionProfile } from "@/src/features/connections/types";

interface HomeScreenProps {
  connections: ConnectionProfile[];
  activeConnectionId: string | null;
  onSelectConnection: (id: string) => void;
  onNewConnection: () => void;
}

export function HomeScreen({
  connections,
  activeConnectionId,
  onSelectConnection,
  onNewConnection,
}: HomeScreenProps) {
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

        <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
          {connections.map((conn) => (
            <Card
              key={conn.id}
              onClick={() => onSelectConnection(conn.id)}
              className={`cursor-pointer transition-colors hover:border-primary/50 ${
                conn.id === activeConnectionId ? "border-primary/60 bg-primary/5" : ""
              }`}
            >
              <CardHeader>
                <div className="flex items-center gap-2">
                  <Plug className="size-4 shrink-0 text-muted-foreground" />
                  <CardTitle className="truncate">{conn.name}</CardTitle>
                </div>
                <CardDescription className="truncate font-mono text-xs">
                  {toDisplayUrl(conn)}
                </CardDescription>
              </CardHeader>
              <CardContent>
                {conn.id === activeConnectionId && (
                  <span className="text-xs font-medium text-primary">Active</span>
                )}
              </CardContent>
            </Card>
          ))}
        </div>
      </div>
    </div>
  );
}
