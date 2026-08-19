import { useEffect, useState } from "react";
import type { SQLNamespace } from "@codemirror/lang-sql";

import { getTableColumns, listTables } from "@/src/features/tables/api";

export function useSchemaNamespace(connectionId: string): SQLNamespace | undefined {
  const [namespace, setNamespace] = useState<SQLNamespace | undefined>(undefined);

  useEffect(() => {
    let cancelled = false;

    async function load() {
      try {
        const tables = await listTables(connectionId);
        const entries = await Promise.all(
          tables.map(async (t) => {
            const columns = await getTableColumns(connectionId, t.schema, t.name);
            return [t.name, columns.map((c) => c.name)] as const;
          })
        );
        if (!cancelled) {
          setNamespace(Object.fromEntries(entries));
        }
      } catch {
        // Schema completion is a nice-to-have; if it fails to load,
        // the editor still works with plain SQL keyword completion.
        if (!cancelled) setNamespace(undefined);
      }
    }

    void load();
    return () => {
      cancelled = true;
    };
  }, [connectionId]);

  return namespace;
}
