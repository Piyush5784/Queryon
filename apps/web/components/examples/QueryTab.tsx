"use client"

import { useState } from "react"
import { Play, Loader2 } from "lucide-react"

import { buttonVariants } from "@queryon/ui/components/button"
import { cn } from "@queryon/ui/lib/utils"

import { ExamplesDataGrid } from "./ExamplesDataGrid"

interface CannedQuery {
  label: string
  sql: string
  columns: string[]
  rows: (string | number)[][]
  ms: number
}

const CANNED_QUERIES: CannedQuery[] = [
  {
    label: "Top spenders",
    sql: "select u.full_name, u.plan, sum(o.total) as lifetime_value\nfrom orders o\njoin users u on u.id = o.user_id\nwhere o.status = 'paid'\ngroup by u.full_name, u.plan\norder by lifetime_value desc\nlimit 8;",
    columns: ["full_name", "plan", "lifetime_value"],
    rows: [
      ["Grace Kim", "enterprise", "$18,204.55"],
      ["Ivan Petrov", "team", "$14,932.10"],
      ["Alice Nguyen", "pro", "$12,884.40"],
      ["Kenji Sato", "enterprise", "$11,209.75"],
      ["Liu Wei", "pro", "$9,943.20"],
      ["Olivia Bergström", "team", "$8,712.00"],
      ["Dave Okafor", "pro", "$7,650.90"],
      ["Judy Alvarez", "pro", "$6,988.15"],
    ],
    ms: 41,
  },
  {
    label: "Low stock products",
    sql: "select name, category, stock, status\nfrom products\nwhere stock < 20\norder by stock asc\nlimit 8;",
    columns: ["name", "category", "stock", "status"],
    rows: [
      ["Compact Sensor", "Electronics", 2, "low_stock"],
      ["Industrial Bracket", "Hardware", 4, "low_stock"],
      ["Wireless Adapter", "Networking", 5, "low_stock"],
      ["Rugged Enclosure", "Tools", 7, "low_stock"],
      ["Precision Module", "Electronics", 9, "low_stock"],
      ["Slim Panel", "Furniture", 11, "low_stock"],
      ["Portable Cable", "Electronics", 14, "low_stock"],
      ["Universal Fixture", "Lighting", 18, "low_stock"],
    ],
    ms: 23,
  },
  {
    label: "Orders by status",
    sql: "select status, count(*) as orders, sum(total) as revenue\nfrom orders\ngroup by status\norder by orders desc;",
    columns: ["status", "orders", "revenue"],
    rows: [
      ["paid", 6842, "$412,908.20"],
      ["shipped", 5310, "$298,441.75"],
      ["pending", 1204, "$71,220.10"],
      ["refunded", 398, "$21,904.55"],
      ["cancelled", 246, "$13,880.40"],
    ],
    ms: 18,
  },
]

export function QueryTab() {
  const [activeIndex, setActiveIndex] = useState(0)
  const [sql, setSql] = useState(CANNED_QUERIES[0]!.sql)
  const [running, setRunning] = useState(false)
  const [result, setResult] = useState<CannedQuery | null>(null)

  const activeQuery = CANNED_QUERIES[activeIndex]!

  function selectPreset(index: number) {
    setActiveIndex(index)
    setSql(CANNED_QUERIES[index]!.sql)
    setResult(null)
  }

  function runQuery() {
    setRunning(true)
    setResult(null)
    const delay = 380 + Math.random() * 420
    setTimeout(() => {
      setRunning(false)
      setResult(activeQuery)
    }, delay)
  }

  return (
    <div className="flex h-full flex-col">
      <div className="flex flex-wrap items-center gap-2 border-b bg-muted/20 px-3 py-2">
        {CANNED_QUERIES.map((query, index) => (
          <button
            key={query.label}
            type="button"
            onClick={() => selectPreset(index)}
            className={cn(
              "rounded-md px-2.5 py-1 text-xs font-medium transition-colors",
              index === activeIndex
                ? "bg-foreground text-background"
                : "text-muted-foreground hover:bg-muted/60 hover:text-foreground"
            )}
          >
            {query.label}
          </button>
        ))}
        <button
          type="button"
          onClick={runQuery}
          disabled={running}
          className={cn(buttonVariants({ size: "sm" }), "ml-auto gap-1.5")}
        >
          {running ? (
            <Loader2 className="size-3.5 animate-spin" />
          ) : (
            <Play className="size-3.5" />
          )}
          Run
        </button>
      </div>

      <textarea
        value={sql}
        onChange={(event) => setSql(event.target.value)}
        spellCheck={false}
        rows={sql.split("\n").length + 1}
        className="w-full shrink-0 resize-none border-b bg-transparent p-4 font-mono text-xs leading-relaxed text-foreground/90 outline-none"
      />

      <div className="flex min-h-0 flex-1 flex-col">
        {result ? (
          <>
            <ExamplesDataGrid columns={result.columns} rows={result.rows} height={260} />
            <div className="flex shrink-0 items-center gap-2 border-t px-3 py-2 text-[11px] text-muted-foreground">
              <span className="size-1.5 rounded-full bg-emerald-500" />
              {result.rows.length} rows · {result.ms}ms
            </div>
          </>
        ) : (
          <div className="flex flex-1 items-center justify-center text-sm text-muted-foreground">
            {running ? "Running query…" : "Click Run to execute this query"}
          </div>
        )}
      </div>
    </div>
  )
}
