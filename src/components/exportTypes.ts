import type { TableFilter, TableSort } from "@/src/features/tables/api";

export type ExportTarget =
  | {
      kind: "table";
      connectionId: string;
      schema: string;
      table: string;
      /** Applied only when the user picks "Filtered view" in the dialog. */
      filters: TableFilter[];
      sort: TableSort[];
      /**
       * Already-loaded rows for the "This page" scope — no re-query
       * needed. Omitted where there's no loaded grid to export from
       * (e.g. the Structure tab), which hides that scope option.
       */
      page?: { columns: string[]; rows: string[][] };
    }
  | {
      kind: "query";
      connectionId: string;
      tabId: string;
      sql: string;
      /** Already-loaded rows for the "This page" scope — no re-query needed. */
      page: { columns: string[]; rows: string[][] };
    }
  | { kind: "rows"; columns: string[]; rows: string[][] };
