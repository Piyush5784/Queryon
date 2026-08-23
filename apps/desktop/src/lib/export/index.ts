export type ExportFormat = "csv" | "json" | "sql";

export const EXPORT_FORMATS: { value: ExportFormat; label: string; extension: string }[] = [
  { value: "csv", label: "CSV", extension: "csv" },
  { value: "json", label: "JSON", extension: "json" },
  { value: "sql", label: "SQL (INSERT statements)", extension: "sql" },
];

export function suggestedFileName(baseName: string, format: ExportFormat): string {
  const extension = EXPORT_FORMATS.find((f) => f.value === format)?.extension ?? "txt";
  const safeName = baseName.replace(/[^a-zA-Z0-9_.-]+/g, "_");
  const timestamp = new Date().toISOString().replace(/[:T]/g, "-").slice(0, 19);
  return `${safeName}_${timestamp}.${extension}`;
}

// Hand-mirrored from `domain::export::models::ExportEvent` in
// src-tauri/src/domain/export/models.rs — not specta-generated because it
// only ever crosses the boundary via `app.emit`, which specta doesn't see.
export type ExportEvent =
  | { kind: "progress"; jobId: string; rowsWritten: number; totalRows: number | null }
  | { kind: "done"; jobId: string; rowsWritten: number; path: string }
  | { kind: "cancelled"; jobId: string }
  | { kind: "error"; jobId: string; message: string };
