"use client"

import { useState } from "react"
import { Check, Minus, X } from "lucide-react"

import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@queryon/ui/components/table"
import { cn } from "@queryon/ui/lib/utils"

type Cell = "yes" | "no" | "partial" | "unknown"

interface ComparisonRow {
  label: string
  queryon: Cell
  queryonNote?: string
  cells: Record<string, Cell>
  notes?: Partial<Record<string, string>>
}

const TOOLS = ["DBeaver", "DataGrip", "TablePlus", "pgAdmin", "Navicat", "Beekeeper Studio"]

const ROWS: ComparisonRow[] = [
  {
    label: "Cold start time",
    queryon: "yes",
    queryonNote: "~0.2s",
    cells: {
      DBeaver: "no",
      DataGrip: "no",
      TablePlus: "yes",
      pgAdmin: "no",
      Navicat: "unknown",
      "Beekeeper Studio": "yes",
    },
    notes: {
      DBeaver: "3–12s (Eclipse/JVM)",
      DataGrip: "10–30s (JetBrains indexing)",
      pgAdmin: "commonly 30s+",
    },
  },
  {
    label: "Free, no paid tier",
    queryon: "yes",
    cells: {
      DBeaver: "partial",
      DataGrip: "no",
      TablePlus: "partial",
      pgAdmin: "yes",
      Navicat: "no",
      "Beekeeper Studio": "partial",
    },
  },
  {
    label: "Built for writing queries, not DBA administration",
    queryon: "yes",
    cells: {
      DBeaver: "yes",
      DataGrip: "yes",
      TablePlus: "yes",
      pgAdmin: "no",
      Navicat: "no",
      "Beekeeper Studio": "yes",
    },
  },
  {
    label: "Native desktop app",
    queryon: "yes",
    cells: {
      DBeaver: "no",
      DataGrip: "no",
      TablePlus: "yes",
      pgAdmin: "yes",
      Navicat: "yes",
      "Beekeeper Studio": "no",
    },
  },
  {
    label: "Multiple SQL engines, one app",
    queryon: "yes",
    cells: {
      DBeaver: "yes",
      DataGrip: "yes",
      TablePlus: "yes",
      pgAdmin: "no",
      Navicat: "yes",
      "Beekeeper Studio": "yes",
    },
  },
  {
    label: "MongoDB / document browsing included free",
    queryon: "yes",
    cells: {
      DBeaver: "no",
      DataGrip: "no",
      TablePlus: "partial",
      pgAdmin: "no",
      Navicat: "partial",
      "Beekeeper Studio": "no",
    },
  },
]

const MORE_TOOLS = [
  "ArcType (RIP)",
  "Azure Data Studio",
  "DBVisualizer",
  "HeidiSQL",
  "MySQL Workbench",
  "Oracle SQL Developer",
  "phpMyAdmin",
  "PopSQL",
  "Postico",
  "PSQL",
  "RazorSQL",
  "Sequel Pro",
  "SQL Server Management Studio",
  "DB Browser for SQLite",
  "SQLPro Studio",
  "SQuirreL SQL",
  "Valentina Studio",
]

function CellIcon({ value, note }: { value: Cell; note?: string }) {
  const icon =
    value === "yes" ? (
      <Check className="mx-auto size-4 text-emerald-600 dark:text-emerald-400" />
    ) : value === "no" ? (
      <X className="mx-auto size-4 text-muted-foreground/40" />
    ) : value === "partial" ? (
      <Minus className="mx-auto size-4 text-amber-600 dark:text-amber-400" />
    ) : (
      <span className="block text-center text-muted-foreground/40">—</span>
    )

  if (!note) return icon

  return (
    <div className="flex flex-col items-center gap-0.5">
      {icon}
      <span className="text-[10px] whitespace-nowrap text-muted-foreground">{note}</span>
    </div>
  )
}

export function CompetitorComparison() {
  const [showMore, setShowMore] = useState(false)

  return (
    <section className="mx-auto max-w-7xl px-6 text-center">
      <h2 className="mb-3 text-3xl font-medium tracking-tight text-balance sm:text-4xl">
        How Queryon compares
      </h2>
      <p className="mx-auto max-w-xl text-lg text-muted-foreground text-balance">
        A honest look at where Queryon stands next to the tools you already know.
      </p>

      <div className="mt-10 overflow-x-auto rounded-xl border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead className="text-left">Capability</TableHead>
              <TableHead className="text-center font-semibold text-foreground">Queryon</TableHead>
              {TOOLS.map((tool) => (
                <TableHead key={tool} className="text-center">
                  {tool}
                </TableHead>
              ))}
            </TableRow>
          </TableHeader>
          <TableBody>
            {ROWS.map((row) => (
              <TableRow key={row.label}>
                <TableCell className="text-left font-medium">{row.label}</TableCell>
                <TableCell className="bg-foreground/5">
                  <CellIcon value={row.queryon} note={row.queryonNote} />
                </TableCell>
                {TOOLS.map((tool) => (
                  <TableCell key={tool}>
                    <CellIcon value={row.cells[tool] ?? "unknown"} note={row.notes?.[tool]} />
                  </TableCell>
                ))}
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>

      <p className="mt-3 text-center mx-20 text-xs text-muted-foreground">
        "—" means we couldn't verify a claim either way at the time this was written. Queryon's
        cold start was measured on an unoptimized debug build on Linux — a release build should
        only be faster. Competitor startup figures are commonly reported ranges, not numbers we
        measured ourselves.
      </p>
{/* 
      <button
        type="button"
        onClick={() => setShowMore((v) => !v)}
        className="mt-6 text-sm font-medium text-foreground underline underline-offset-4"
      >
        {showMore ? "Show less" : "Learn more →"}
      </button>

      {showMore && (
        <div className="mx-auto mt-6 max-w-2xl text-left">
          <p className="mb-3 text-sm text-muted-foreground">
            Queryon also aims to replace, or fits alongside, tools like:
          </p>
          <div className="flex flex-wrap gap-2">
            {MORE_TOOLS.map((tool) => (
              <span
                key={tool}
                className={cn(
                  "rounded-full border bg-muted/30 px-3 py-1 text-xs text-muted-foreground"
                )}
              >
                {tool}
              </span>
            ))}
          </div>
        </div>
      )} */}
    </section>
  )
}
