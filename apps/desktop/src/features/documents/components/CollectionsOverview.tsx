import { useEffect, useState } from "react";
import { Loader2, RefreshCw, XCircle } from "lucide-react";

import { Button } from "@queryon/ui/components/button";
import { TooltipButton } from "@/src/components/TooltipButton";
import { docListCollections, type CollectionRef } from "@/src/features/documents/api";
import type { DatabaseTab } from "@/src/features/documents/types";
import { toErrorMessage } from "@/src/lib/tauri/errors";

interface CollectionsOverviewProps {
  tab: DatabaseTab;
  onOpenCollection: (database: string, collection: string) => void;
}

function formatBytes(bytes: number | null | undefined): string {
  if (!bytes) return "0 B";
  const units = ["B", "kB", "MB", "GB", "TB"];
  let value = bytes;
  let unitIndex = 0;
  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024;
    unitIndex += 1;
  }
  return `${value.toFixed(unitIndex === 0 ? 0 : 2)} ${units[unitIndex]}`;
}

export function CollectionsOverview({ tab, onOpenCollection }: CollectionsOverviewProps) {
  const [collections, setCollections] = useState<CollectionRef[] | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  function load() {
    setLoading(true);
    setError(null);
    docListCollections(tab.connectionId, tab.database)
      .then(setCollections)
      .catch((err) => setError(toErrorMessage(err)))
      .finally(() => setLoading(false));
  }

  useEffect(() => {
    load();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [tab.connectionId, tab.database]);

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="flex shrink-0 items-center justify-between border-b px-3 py-1.5">
        <span className="text-xs font-medium text-foreground">{tab.database}</span>
        <TooltipButton
          variant="ghost"
          size="icon-sm"
          onClick={load}
          disabled={loading}
          tooltip="Refresh"
          shortcut={["F5"]}
        >
          <RefreshCw className={`size-3.5 ${loading ? "animate-spin" : ""}`} />
        </TooltipButton>
      </div>

      <div className="min-h-0 flex-1 overflow-auto p-4">
        {loading && !collections && (
          <div className="flex h-full items-center justify-center text-muted-foreground">
            <Loader2 className="size-5 animate-spin" />
          </div>
        )}

        {error && (
          <div className="flex h-full flex-col items-center justify-center gap-2 p-6 text-center">
            <XCircle className="size-6 text-destructive" />
            <p className="max-w-xs text-sm text-destructive">{error}</p>
            <Button variant="outline" size="sm" onClick={load}>
              Retry
            </Button>
          </div>
        )}

        {collections && !error && collections.length === 0 && (
          <div className="flex h-full items-center justify-center text-sm text-muted-foreground">
            This database has no collections.
          </div>
        )}

        {collections && !error && collections.length > 0 && (
          <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
            {collections.map((coll) => (
              <button
                key={coll.name}
                type="button"
                onClick={() => onOpenCollection(tab.database, coll.name)}
                className="flex flex-col gap-3 rounded-lg border p-4 text-left transition-colors hover:bg-accent/50"
              >
                <span className="truncate font-medium text-foreground">{coll.name}</span>
                <div className="grid grid-cols-2 gap-x-4 gap-y-1.5 text-xs text-muted-foreground">
                  <div>
                    <div className="text-[10px] tracking-wide uppercase">Storage size</div>
                    <div className="text-foreground">{formatBytes(coll.storageSizeBytes)}</div>
                  </div>
                  <div>
                    <div className="text-[10px] tracking-wide uppercase">Documents</div>
                    <div className="text-foreground">
                      {(coll.estimatedCount ?? 0).toLocaleString()}
                    </div>
                  </div>
                  <div>
                    <div className="text-[10px] tracking-wide uppercase">Avg. document size</div>
                    <div className="text-foreground">{formatBytes(coll.avgDocumentSizeBytes)}</div>
                  </div>
                  <div>
                    <div className="text-[10px] tracking-wide uppercase">Indexes</div>
                    <div className="text-foreground">{coll.indexCount}</div>
                  </div>
                  <div>
                    <div className="text-[10px] tracking-wide uppercase">Total index size</div>
                    <div className="text-foreground">{formatBytes(coll.totalIndexSizeBytes)}</div>
                  </div>
                </div>
              </button>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
