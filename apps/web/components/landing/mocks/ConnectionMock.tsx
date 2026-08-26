"use client"

import { useState } from "react"

import { cn } from "@queryon/ui/lib/utils"

import { ENGINE_LOGOS } from "../engineLogos"
import { WindowChrome } from "./WindowChrome"

const SHOWN_ENGINES = ["PostgreSQL", "MySQL", "ClickHouse", "DuckDB", "SQL Server", "SQLite"]
const REMAINING_ENGINE_COUNT = 15 - SHOWN_ENGINES.length

export function ConnectionMock() {
  const engines = ENGINE_LOGOS.filter((engine) => SHOWN_ENGINES.includes(engine.name))
  const [selected, setSelected] = useState("PostgreSQL")

  return (
    <WindowChrome title="New Connection">
      <div className="grid grid-cols-3 gap-2 p-4">
        {engines.map((engine) => (
          <button
            key={engine.name}
            type="button"
            onClick={() => setSelected(engine.name)}
            className={cn(
              "flex flex-col items-center gap-2 rounded-lg border p-3 text-center transition-colors",
              selected === engine.name
                ? "border-foreground/30 bg-foreground/5"
                : "hover:border-foreground/20 hover:bg-muted/40"
            )}
          >
            <div className="size-6">
              <engine.Logo />
            </div>
            <span className="text-[11px] font-medium">{engine.name}</span>
          </button>
        ))}
      </div>
      <p className="border-t px-4 py-2.5 text-center text-[11px] text-muted-foreground">
        +{REMAINING_ENGINE_COUNT} more databases supported
      </p>
    </WindowChrome>
  )
}
