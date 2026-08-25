import { cn } from "@queryon/ui/lib/utils"

import { WindowChrome } from "./WindowChrome"

export function ConnectionMock() {
  const engines = [
    { name: "PostgreSQL", color: "bg-blue-500" },
    { name: "MySQL", color: "bg-orange-500" },
    { name: "ClickHouse", color: "bg-yellow-500" },
    { name: "DuckDB", color: "bg-yellow-600" },
    { name: "SQL Server", color: "bg-red-600" },
    { name: "SQLite", color: "bg-slate-500" },
  ]

  return (
    <WindowChrome title="New Connection">
      <div className="grid grid-cols-3 gap-2 p-4">
        {engines.map((engine) => (
          <div
            key={engine.name}
            className={cn(
              "flex flex-col items-center gap-2 rounded-lg border p-3 text-center",
              engine.name === "PostgreSQL" && "border-foreground/30 bg-foreground/5"
            )}
          >
            <span className={cn("size-6 rounded-lg", engine.color)} />
            <span className="text-[11px] font-medium">{engine.name}</span>
          </div>
        ))}
      </div>
    </WindowChrome>
  )
}
