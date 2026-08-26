import { useEffect, useState } from "react";
import { ChevronLeft, ChevronRight, Loader2, RefreshCw, XCircle } from "lucide-react";

import { Button } from "@queryon/ui/components/button";
import { TooltipButton } from "@/src/components/TooltipButton";
import { JsonViewer } from "@/src/features/tables/components/JsonViewer";
import type { JsonValue } from "@/src/features/tables/components/JsonViewer/types";
import { docGetDocument, docListDocuments, type DocumentPage } from "@/src/features/documents/api";
import type { CollectionTab } from "@/src/features/documents/types";
import { toErrorMessage } from "@/src/lib/tauri/errors";

interface CollectionViewProps {
  tab: CollectionTab;
}

const PAGE_SIZE = 50;

export function CollectionView({ tab }: CollectionViewProps) {
  const [page, setPage] = useState(0);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<DocumentPage | null>(null);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [docLoading, setDocLoading] = useState(false);
  const [docError, setDocError] = useState<string | null>(null);
  const [docValue, setDocValue] = useState<JsonValue | null>(null);

  useEffect(() => {
    setPage(0);
    setSelectedId(null);
    setDocValue(null);
    setDocError(null);
  }, [tab.id]);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(null);

    docListDocuments(tab.connectionId, tab.database, tab.collection, PAGE_SIZE, page * PAGE_SIZE)
      .then((res) => {
        if (cancelled) return;
        setResult(res);
        if (res.documents.length > 0) {
          setSelectedId(res.documents[0].id);
        } else {
          setSelectedId(null);
        }
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
  }, [tab.connectionId, tab.database, tab.collection, page]);

  useEffect(() => {
    if (!selectedId) {
      setDocValue(null);
      return;
    }

    let cancelled = false;
    setDocLoading(true);
    setDocError(null);

    docGetDocument(tab.connectionId, tab.database, tab.collection, selectedId)
      .then((value) => {
        if (!cancelled) setDocValue((value ?? null) as JsonValue | null);
      })
      .catch((err) => {
        if (!cancelled) setDocError(toErrorMessage(err));
      })
      .finally(() => {
        if (!cancelled) setDocLoading(false);
      });

    return () => {
      cancelled = true;
    };
  }, [tab.connectionId, tab.database, tab.collection, selectedId]);

  function refresh() {
    setError(null);
    setLoading(true);
    docListDocuments(tab.connectionId, tab.database, tab.collection, PAGE_SIZE, page * PAGE_SIZE)
      .then(setResult)
      .catch((err) => setError(toErrorMessage(err)))
      .finally(() => setLoading(false));
  }

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="flex shrink-0 items-center justify-between border-b px-3 py-1.5">
        <span className="text-xs font-medium text-foreground">
          {tab.database}.{tab.collection}
        </span>
        <TooltipButton
          variant="ghost"
          size="icon-sm"
          onClick={refresh}
          disabled={loading}
          tooltip="Refresh"
          shortcut={["F5"]}
        >
          <RefreshCw className={`size-3.5 ${loading ? "animate-spin" : ""}`} />
        </TooltipButton>
      </div>

      <div className="flex min-h-0 flex-1">
        <div className="flex w-72 shrink-0 flex-col border-r">
          <div className="min-h-0 flex-1 overflow-auto">
            {loading && !result && (
              <div className="flex h-full items-center justify-center text-muted-foreground">
                <Loader2 className="size-5 animate-spin" />
              </div>
            )}

            {error && (
              <div className="flex flex-col items-center justify-center gap-2 p-6 text-center">
                <XCircle className="size-6 text-destructive" />
                <p className="max-w-xs text-sm text-destructive">{error}</p>
                <Button variant="outline" size="sm" onClick={refresh}>
                  Retry
                </Button>
              </div>
            )}

            {result && !error && result.documents.length === 0 && (
              <div className="flex h-full items-center justify-center p-6 text-center text-sm text-muted-foreground">
                This collection has no documents.
              </div>
            )}

            {result && !error && result.documents.length > 0 && (
              <ul>
                {result.documents.map((doc) => (
                  <li key={doc.id}>
                    <button
                      type="button"
                      onClick={() => setSelectedId(doc.id)}
                      className={`w-full truncate border-b px-3 py-2 text-left text-xs ${
                        selectedId === doc.id ? "bg-accent" : "hover:bg-accent/50"
                      }`}
                    >
                      <div className="truncate font-mono text-[11px] text-muted-foreground">{doc.id}</div>
                      <div className="truncate">{doc.preview}</div>
                    </button>
                  </li>
                ))}
              </ul>
            )}
          </div>

          <div className="flex shrink-0 items-center justify-between gap-2 border-t px-2 py-1.5">
            <Button
              variant="ghost"
              size="icon-sm"
              onClick={() => setPage((p) => Math.max(0, p - 1))}
              disabled={page === 0 || loading}
            >
              <ChevronLeft className="size-3.5" />
            </Button>
            <span className="text-xs text-muted-foreground">Page {page + 1}</span>
            <Button
              variant="ghost"
              size="icon-sm"
              onClick={() => setPage((p) => p + 1)}
              disabled={!result?.hasMore || loading}
            >
              <ChevronRight className="size-3.5" />
            </Button>
          </div>
        </div>

        <div className="min-h-0 flex-1">
          {!selectedId && (
            <div className="flex h-full items-center justify-center text-sm text-muted-foreground">
              Select a document to view it.
            </div>
          )}

          {selectedId && docLoading && (
            <div className="flex h-full items-center justify-center text-muted-foreground">
              <Loader2 className="size-5 animate-spin" />
            </div>
          )}

          {selectedId && docError && (
            <div className="flex h-full flex-col items-center justify-center gap-2 p-6 text-center">
              <XCircle className="size-6 text-destructive" />
              <p className="max-w-md text-sm text-destructive">{docError}</p>
            </div>
          )}

          {selectedId && !docLoading && !docError && docValue !== null && <JsonViewer value={docValue} />}
        </div>
      </div>
    </div>
  );
}
