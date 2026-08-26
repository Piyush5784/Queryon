"use client"

import { useMemo, useState } from "react"
import { Terminal, X } from "lucide-react"

import { cn } from "@queryon/ui/lib/utils"

import { WindowChrome } from "../landing/mocks/WindowChrome"
import { ExamplesDataGrid } from "./ExamplesDataGrid"
import { QueryTab } from "./QueryTab"
import { buildOrdersRows, buildProductsRows, buildUsersRows } from "./exampleData"

const TABLES = [
  { id: "users", label: "users", rowCount: 10000 },
  { id: "orders", label: "orders", rowCount: 8000 },
  { id: "products", label: "products", rowCount: 2000 },
]

type TabId = "users" | "orders" | "products" | "query"

const TAB_ORDER: TabId[] = ["users", "orders", "products", "query"]

export function ExamplesWorkspace() {
  const [openTabs, setOpenTabs] = useState<TabId[]>(["users", "query"])
  const [activeTab, setActiveTab] = useState<TabId>("users")

  const usersRows = useMemo(() => buildUsersRows(10000), [])
  const ordersRows = useMemo(() => buildOrdersRows(8000), [])
  const productsRows = useMemo(() => buildProductsRows(2000), [])

  function openTable(id: TabId) {
    setOpenTabs((prev) => (prev.includes(id) ? prev : [...prev, id].sort((a, b) => TAB_ORDER.indexOf(a) - TAB_ORDER.indexOf(b))))
    setActiveTab(id)
  }

  function closeTab(id: TabId) {
    setOpenTabs((prev) => {
      const next = prev.filter((tab) => tab !== id)
      if (activeTab === id) setActiveTab(next[next.length - 1] ?? "users")
      return next
    })
  }

  const tabMeta: Record<TabId, { label: string; sub?: string }> = {
    users: { label: "users", sub: "10,000 rows" },
    orders: { label: "orders", sub: "8,000 rows" },
    products: { label: "products", sub: "2,000 rows" },
    query: { label: "query.sql" },
  }

  return (
    <WindowChrome title="devdb — Queryon">
      <div className="flex h-[640px]">
        <div className="hidden w-56 shrink-0 border-r bg-muted/10 p-3 sm:block">
          <p className="mb-2 px-1 text-[11px] font-medium tracking-wide text-muted-foreground uppercase">
            Connections
          </p>
          <div className="mb-4 flex items-center gap-2 rounded-md bg-foreground/10 px-2 py-1.5">
            <span className="size-2 rounded-full bg-emerald-500" />
            <span className="font-mono text-xs font-medium">devdb</span>
          </div>

          <p className="mb-2 px-1 text-[11px] font-medium tracking-wide text-muted-foreground uppercase">
            Tables
          </p>
          <div className="flex flex-col gap-0.5 pl-1">
            {TABLES.map((table) => (
              <button
                key={table.id}
                type="button"
                onClick={() => openTable(table.id as TabId)}
                className={cn(
                  "flex items-center justify-between rounded-md px-2 py-1 text-left font-mono text-xs",
                  activeTab === table.id
                    ? "bg-foreground text-background font-medium"
                    : "text-muted-foreground hover:bg-muted/50 hover:text-foreground"
                )}
              >
                {table.label}
                <span
                  className={cn(
                    "text-[10px]",
                    activeTab === table.id ? "text-background/70" : "text-muted-foreground/70"
                  )}
                >
                  {table.rowCount.toLocaleString()}
                </span>
              </button>
            ))}
          </div>

          <p className="mt-4 mb-2 px-1 text-[11px] font-medium tracking-wide text-muted-foreground uppercase">
            Query
          </p>
          <button
            type="button"
            onClick={() => openTable("query")}
            className={cn(
              "flex w-full items-center gap-1.5 rounded-md px-2 py-1 text-left font-mono text-xs",
              activeTab === "query"
                ? "bg-foreground text-background font-medium"
                : "text-muted-foreground hover:bg-muted/50 hover:text-foreground"
            )}
          >
            <Terminal className="size-3" />
            query.sql
          </button>
        </div>

        <div className="flex min-w-0 flex-1 flex-col">
          <div className="flex shrink-0 items-center gap-0.5 overflow-x-auto border-b bg-muted/20 px-1">
            {openTabs.map((tab) => (
              <div
                key={tab}
                onClick={() => setActiveTab(tab)}
                className={cn(
                  "group flex cursor-pointer items-center gap-2 border-r px-3 py-2 font-mono text-xs whitespace-nowrap",
                  activeTab === tab
                    ? "bg-background text-foreground"
                    : "text-muted-foreground hover:bg-muted/40"
                )}
              >
                {tabMeta[tab].label}
                <button
                  type="button"
                  onClick={(event) => {
                    event.stopPropagation()
                    closeTab(tab)
                  }}
                  className="rounded-sm p-0.5 opacity-0 transition-opacity group-hover:opacity-100 hover:bg-muted"
                >
                  <X className="size-3" />
                </button>
              </div>
            ))}
          </div>

          <div className="min-h-0 flex-1">
            {activeTab === "users" && <ExamplesDataGrid columns={USERS_COLUMNS} rows={usersRows} height={606} />}
            {activeTab === "orders" && (
              <ExamplesDataGrid columns={ORDERS_COLUMNS} rows={ordersRows} height={606} />
            )}
            {activeTab === "products" && (
              <ExamplesDataGrid columns={PRODUCTS_COLUMNS} rows={productsRows} height={606} />
            )}
            {activeTab === "query" && <QueryTab />}
            {openTabs.length === 0 && (
              <div className="flex h-full items-center justify-center text-sm text-muted-foreground">
                No tabs open — pick a table from the sidebar
              </div>
            )}
          </div>
        </div>
      </div>
    </WindowChrome>
  )
}

const USERS_COLUMNS = ["id", "email", "full_name", "plan", "country", "signed_up_via", "created_at"]
const ORDERS_COLUMNS = ["id", "user_id", "items", "total", "status", "created_at"]
const PRODUCTS_COLUMNS = ["id", "name", "category", "price", "stock", "status"]
