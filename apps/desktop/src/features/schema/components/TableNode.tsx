import { Handle, Position, type NodeProps } from "@xyflow/react";
import { KeyRound, Link2, Table2 } from "lucide-react";

import type { TableNodeData } from "@/src/features/schema/graph";

export function TableNode({ data }: NodeProps & { data: TableNodeData }) {
  return (
    <div className="min-w-60 overflow-hidden rounded-lg border border-border bg-card shadow-sm">
      <div className="flex items-center gap-1.5 border-b bg-muted/40 px-2.5 py-1.5">
        <Table2 className="size-3.5 shrink-0 text-muted-foreground" />
        <span className="truncate text-xs font-semibold">{data.table}</span>
      </div>
      <div className="divide-y">
        {data.columns.map((column) => (
          <div key={column.name} className="relative flex items-center gap-1.5 px-2.5 py-1 text-xs">
            <Handle
              type="source"
              position={Position.Left}
              id={`${data.table}.${column.name}.source`}
              className="!size-1.5 !border-0 !bg-border"
            />
            <Handle
              type="target"
              position={Position.Left}
              id={`${data.table}.${column.name}.target`}
              className="!size-1.5 !border-0 !bg-border"
            />
            <span className="flex size-3.5 shrink-0 items-center justify-center" title={
              column.isPrimaryKey ? "Primary key" : column.isForeignKey ? "Foreign key" : undefined
            }>
              {column.isPrimaryKey && <KeyRound className="size-3 text-amber-500" />}
              {!column.isPrimaryKey && column.isForeignKey && (
                <Link2 className="size-3 text-primary" />
              )}
            </span>
            <span className="truncate font-mono">{column.name}</span>
            {column.isPrimaryKey && (
              <span className="shrink-0 rounded bg-amber-500/10 px-1 font-mono text-[0.6rem] font-semibold text-amber-600 dark:text-amber-400">
                PK
              </span>
            )}
            {!column.isPrimaryKey && column.isForeignKey && (
              <span className="shrink-0 rounded bg-primary/10 px-1 font-mono text-[0.6rem] font-semibold text-primary">
                FK
              </span>
            )}
            <span className="ml-auto shrink-0 truncate font-mono text-[0.65rem] text-muted-foreground">
              {column.dataType}
              {!column.isNullable && "!"}
            </span>
          </div>
        ))}
      </div>
    </div>
  );
}
