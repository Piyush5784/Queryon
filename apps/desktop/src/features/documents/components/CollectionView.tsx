import { useEffect, useState } from "react";
import { AlertTriangle, ChevronLeft, ChevronRight, Loader2, Pencil, Plus, RefreshCw, Trash2, XCircle } from "lucide-react";

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
import { Button } from "@queryon/ui/components/button";
import { TooltipButton } from "@/src/components/TooltipButton";
import { JsonEditor } from "@/src/features/tables/components/JsonViewer/JsonEditor";
import { JsonViewer } from "@/src/features/tables/components/JsonViewer";
import type { JsonValue } from "@/src/features/tables/components/JsonViewer/types";
import {
  docDeleteDocument,
  docGetDocument,
  docInsertDocument,
  docListDocuments,
  docUpdateDocument,
  type DocumentPage,
} from "@/src/features/documents/api";
import type { CollectionTab } from "@/src/features/documents/types";
import { toErrorMessage } from "@/src/lib/tauri/errors";

interface CollectionViewProps {
  tab: CollectionTab;
}

const PAGE_SIZE = 50;
const NEW_DOCUMENT_TEMPLATE: JsonValue = {} as JsonValue;

export function CollectionView({ tab }: CollectionViewProps) {
  const [page, setPage] = useState(0);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<DocumentPage | null>(null);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [docLoading, setDocLoading] = useState(false);
  const [docError, setDocError] = useState<string | null>(null);
  const [docValue, setDocValue] = useState<JsonValue | null>(null);

  const [mode, setMode] = useState<"view" | "edit" | "new">("view");
  const [saving, setSaving] = useState(false);
  const [saveError, setSaveError] = useState<string | null>(null);

  const [confirmDeleteOpen, setConfirmDeleteOpen] = useState(false);
  const [deleting, setDeleting] = useState(false);
  const [deleteError, setDeleteError] = useState<string | null>(null);

  useEffect(() => {
    setPage(0);
    setSelectedId(null);
    setDocValue(null);
    setDocError(null);
    setMode("view");
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

  function selectDocument(id: string) {
    setSelectedId(id);
    setMode("view");
    setSaveError(null);
  }

  function startNewDocument() {
    setSelectedId(null);
    setDocValue(NEW_DOCUMENT_TEMPLATE);
    setMode("new");
    setSaveError(null);
  }

  async function handleSave(next: JsonValue) {
    setSaving(true);
    setSaveError(null);
    try {
      if (mode === "new") {
        const id = await docInsertDocument(tab.connectionId, tab.database, tab.collection, next);
        refresh();
        setMode("view");
        setSelectedId(id);
      } else if (selectedId) {
        await docUpdateDocument(tab.connectionId, tab.database, tab.collection, selectedId, next);
        refresh();
        setDocValue(next);
        setMode("view");
      }
    } catch (err) {
      setSaveError(toErrorMessage(err));
    } finally {
      setSaving(false);
    }
  }

  async function handleConfirmDelete() {
    if (!selectedId) return;
    setDeleting(true);
    setDeleteError(null);
    try {
      await docDeleteDocument(tab.connectionId, tab.database, tab.collection, selectedId);
      setConfirmDeleteOpen(false);
      setSelectedId(null);
      setDocValue(null);
      refresh();
    } catch (err) {
      setDeleteError(toErrorMessage(err));
    } finally {
      setDeleting(false);
    }
  }

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="flex shrink-0 items-center justify-between border-b px-3 py-1.5">
        <span className="text-xs font-medium text-foreground">
          {tab.database}.{tab.collection}
        </span>
        <div className="flex items-center gap-1">
          <TooltipButton
            variant="ghost"
            size="icon-sm"
            onClick={startNewDocument}
            tooltip="New document"
            aria-label="New document"
          >
            <Plus className="size-3.5" />
          </TooltipButton>
          <TooltipButton
            variant="ghost"
            size="icon-sm"
            onClick={refresh}
            disabled={loading}
            tooltip="Refresh"
            shortcut={["F5"]}
            aria-label="Refresh"
          >
            <RefreshCw className={`size-3.5 ${loading ? "animate-spin" : ""}`} />
          </TooltipButton>
        </div>
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
                      onClick={() => selectDocument(doc.id)}
                      className={`w-full truncate border-b px-3 py-2 text-left text-xs ${
                        selectedId === doc.id && mode !== "new" ? "bg-accent" : "hover:bg-accent/50"
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

        <div className="flex min-h-0 flex-1 flex-col">
          {mode !== "new" && (selectedId || docLoading) && (
            <div className="flex shrink-0 items-center justify-end gap-1.5 border-b px-3 py-1.5">
              {mode === "view" && selectedId && (
                <>
                  <Button
                    variant="outline"
                    size="sm"
                    className="gap-1.5"
                    onClick={() => setMode("edit")}
                  >
                    <Pencil className="size-3.5" />
                    Edit
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    className="gap-1.5 text-destructive hover:text-destructive"
                    onClick={() => setConfirmDeleteOpen(true)}
                  >
                    <Trash2 className="size-3.5" />
                    Delete
                  </Button>
                </>
              )}
            </div>
          )}

          <div className="min-h-0 flex-1">
            {mode === "new" && (
              <JsonEditor
                value={NEW_DOCUMENT_TEMPLATE}
                saving={saving}
                onSave={handleSave}
                onCancel={() => {
                  setMode("view");
                  setDocValue(null);
                }}
              />
            )}

            {mode !== "new" && !selectedId && (
              <div className="flex h-full items-center justify-center text-sm text-muted-foreground">
                Select a document to view it.
              </div>
            )}

            {mode !== "new" && selectedId && docLoading && (
              <div className="flex h-full items-center justify-center text-muted-foreground">
                <Loader2 className="size-5 animate-spin" />
              </div>
            )}

            {mode !== "new" && selectedId && docError && (
              <div className="flex h-full flex-col items-center justify-center gap-2 p-6 text-center">
                <XCircle className="size-6 text-destructive" />
                <p className="max-w-md text-sm text-destructive">{docError}</p>
              </div>
            )}

            {mode === "view" && selectedId && !docLoading && !docError && docValue !== null && (
              <JsonViewer value={docValue} />
            )}

            {mode === "edit" && selectedId && !docLoading && !docError && docValue !== null && (
              <JsonEditor
                value={docValue}
                saving={saving}
                onSave={handleSave}
                onCancel={() => setMode("view")}
              />
            )}
          </div>

          {saveError && (
            <div className="mx-3 mb-3 flex items-start gap-2 rounded-lg border border-destructive/30 bg-destructive/10 px-3 py-2 text-xs text-destructive">
              <XCircle className="size-3.5 shrink-0 translate-y-0.5" />
              <span className="wrap-break-word">{saveError}</span>
            </div>
          )}
        </div>
      </div>

      <AlertDialog
        open={confirmDeleteOpen}
        onOpenChange={(open) => !deleting && setConfirmDeleteOpen(open)}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogMedia className="bg-destructive/10 text-destructive">
              <AlertTriangle />
            </AlertDialogMedia>
            <AlertDialogTitle>Delete this document?</AlertDialogTitle>
            <AlertDialogDescription>
              This permanently deletes the selected document from{" "}
              <strong>
                {tab.database}.{tab.collection}
              </strong>
              . This cannot be undone.
            </AlertDialogDescription>
          </AlertDialogHeader>

          {deleteError && (
            <div className="flex items-start gap-2 rounded-lg border border-destructive/30 bg-destructive/10 px-3 py-2 text-xs text-destructive">
              <XCircle className="size-3.5 shrink-0 translate-y-0.5" />
              <span className="wrap-break-word">{deleteError}</span>
            </div>
          )}

          <AlertDialogFooter>
            <AlertDialogCancel disabled={deleting}>Cancel</AlertDialogCancel>
            <AlertDialogAction
              variant="destructive"
              onClick={handleConfirmDelete}
              disabled={deleting}
              className="gap-1.5"
            >
              {deleting && <Loader2 className="size-3.5 animate-spin" />}
              Delete
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}
