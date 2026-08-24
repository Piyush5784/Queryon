import Link from "next/link"
import { ArrowRight, Database, GitBranch } from "lucide-react"

import { Badge } from "@queryon/ui/components/badge"
import { buttonVariants } from "@queryon/ui/components/button"
import { cn } from "@queryon/ui/lib/utils"

const ENGINES = [
  "PostgreSQL",
  "MySQL",
  "SQL Server",
  "SQLite",
  "DuckDB",
  "ClickHouse",
  "StarRocks",
  "MariaDB",
  "TiDB",
  "CockroachDB",
  "GreengageDB",
  "Neon",
]

export default function LandingPage() {
  return (
    <div className="flex min-h-svh flex-col">
      <header className="sticky top-0 z-50 border-b bg-background/80 backdrop-blur-sm">
        <div className="mx-auto flex max-w-6xl items-center justify-between px-6 py-4">
          <Link href="/" className="flex items-center gap-2 font-medium">
            <div className="flex size-6 items-center justify-center rounded-md bg-foreground text-background">
              <Database className="size-3.5" />
            </div>
            Queryon
          </Link>
          <nav className="hidden items-center gap-6 text-sm text-muted-foreground sm:flex">
            <Link href="#features" className="transition-colors hover:text-foreground">
              Features
            </Link>
            <Link href="#engines" className="transition-colors hover:text-foreground">
              Engines
            </Link>
            <Link href="/docs" className="transition-colors hover:text-foreground">
              Docs
            </Link>
            <a
              href="https://github.com/Piyush5784/Queryon"
              className="transition-colors hover:text-foreground"
            >
              GitHub
            </a>
          </nav>
          <Link href="/docs/install" className={buttonVariants({ size: "sm" })}>
            Download
          </Link>
        </div>
      </header>

      <main className="flex-1">
        <section className="relative overflow-hidden">
          <div
            aria-hidden
            className="pointer-events-none absolute inset-x-0 top-0 -z-10 h-[560px] [mask-image:radial-gradient(closest-side,black,transparent)]"
          >
            <div className="absolute inset-0 bg-[linear-gradient(to_right,var(--border)_1px,transparent_1px),linear-gradient(to_bottom,var(--border)_1px,transparent_1px)] bg-[size:56px_56px] opacity-40" />
          </div>

          <div className="mx-auto flex max-w-4xl flex-col items-center gap-6 px-6 pt-20 pb-4 text-center sm:pt-28">
            <a
              href="https://github.com/Piyush5784/Queryon"
              className="group inline-flex items-center gap-2 rounded-full border bg-card px-3 py-1 text-xs text-muted-foreground shadow-sm transition-colors hover:border-foreground/20 hover:text-foreground"
            >
              <GitBranch className="size-3.5" />
              Free & open source on GitHub
              <ArrowRight className="size-3 transition-transform group-hover:translate-x-0.5" />
            </a>

            <h1 className="text-4xl font-medium tracking-tight text-balance sm:text-6xl">
              Every database.
              <br />
              One client.
            </h1>

            <p className="max-w-xl text-lg text-muted-foreground text-balance">
              A fast, native desktop client for Postgres, MySQL, SQL Server, ClickHouse, DuckDB,
              and a dozen more — browse schemas, run queries, and edit data without switching
              tools.
            </p>

            <div className="flex flex-wrap items-center justify-center gap-3 pt-2">
              <Link href="/docs/install" className={cn(buttonVariants({ size: "lg" }), "gap-2")}>
                Download for free
                <ArrowRight className="size-4" />
              </Link>
              <a
                href="https://github.com/Piyush5784/Queryon"
                className={buttonVariants({ size: "lg", variant: "outline" })}
              >
                View on GitHub
              </a>
            </div>
            <p className="text-xs text-muted-foreground">
              Windows, macOS, and Linux · No account required
            </p>
          </div>

          <div className="mx-auto max-w-5xl px-6 pt-14 pb-20 sm:pt-20">
            <DataGridMock />
          </div>
        </section>

        <section id="engines" className="border-y bg-muted/30">
          <div className="mx-auto max-w-5xl px-6 py-12">
            <p className="mb-6 text-center text-sm font-medium text-muted-foreground">
              Supported databases
            </p>
            <div className="flex flex-wrap items-center justify-center gap-2">
              {ENGINES.map((engine) => (
                <Badge key={engine} variant="secondary" className="px-3 py-1 text-sm">
                  {engine}
                </Badge>
              ))}
            </div>
          </div>
        </section>

        <section id="features" className="mx-auto max-w-6xl px-6 py-24">
          <div className="mx-auto mb-20 max-w-2xl text-center">
            <h2 className="text-3xl font-medium tracking-tight text-balance">
              Built for people who live in their database
            </h2>
            <p className="mt-3 text-muted-foreground text-balance">
              Not a scaled-down toy client, and not an enterprise tool with a decade of
              accumulated menus — just the parts you actually use, done well.
            </p>
          </div>

          <div className="flex flex-col gap-24 sm:gap-32">
            <FeatureRow
              index={1}
              title="Connect to anything, in seconds"
              description="Pick an engine, paste a connection string or fill in the fields, and go. Every engine gets its own real driver underneath — not a generic SQL layer papering over the differences — so schema editing, data types, and constraint behavior match what that engine actually supports."
              visual={<ConnectionMock />}
            />
            <FeatureRow
              index={2}
              title="A grid built for real data"
              description="Virtualized rendering handles tables with millions of rows without choking the UI. Filter, sort, hide columns, inspect JSON cells in place, and edit values inline."
              visual={<DataGridMock compact />}
              reverse
            />
            <FeatureRow
              index={3}
              title="Schema editing with a preview"
              description="Add tables, columns, indexes, and constraints through a visual editor. Review the exact DDL that will run before anything actually executes — no surprises on a live database."
              visual={<SchemaMock />}
            />
            <FeatureRow
              index={4}
              title="A real SQL editor"
              description="Syntax highlighting, query history, saved queries, and manual-commit transactions where the engine supports it. Built for people who still write their own SQL."
              visual={<SqlEditorMock />}
              reverse
            />
          </div>
        </section>

        <section className="border-t bg-muted/30">
          <div className="mx-auto flex max-w-4xl flex-col items-center gap-5 px-6 py-20 text-center">
            <h2 className="text-3xl font-medium tracking-tight text-balance">
              Get started in under a minute
            </h2>
            <p className="max-w-xl text-muted-foreground text-balance">
              Available for Windows, macOS, and Linux. No account required.
            </p>
            <Link href="/docs/install" className={cn(buttonVariants({ size: "lg" }), "gap-2")}>
              Download Queryon
              <ArrowRight className="size-4" />
            </Link>
          </div>
        </section>
      </main>

      <footer className="border-t">
        <div className="mx-auto flex max-w-6xl flex-col items-center justify-between gap-4 px-6 py-8 text-sm text-muted-foreground sm:flex-row">
          <span>© {new Date().getFullYear()} Queryon.</span>
          <div className="flex items-center gap-6">
            <Link href="/docs" className="transition-colors hover:text-foreground">
              Docs
            </Link>
            <a
              href="https://github.com/Piyush5784/Queryon"
              className="transition-colors hover:text-foreground"
            >
              GitHub
            </a>
          </div>
        </div>
      </footer>
    </div>
  )
}

function FeatureRow({
  index,
  title,
  description,
  visual,
  reverse,
}: {
  index: number
  title: string
  description: string
  visual: React.ReactNode
  reverse?: boolean
}) {
  return (
    <div
      className={cn(
        "grid grid-cols-1 items-center gap-10 lg:grid-cols-2 lg:gap-16",
        reverse && "lg:[&>*:first-child]:order-2"
      )}
    >
      <div>
        <span className="mb-3 inline-flex size-8 items-center justify-center rounded-full border font-mono text-sm text-muted-foreground">
          {String(index).padStart(2, "0")}
        </span>
        <h3 className="text-2xl font-medium tracking-tight text-balance">{title}</h3>
        <p className="mt-3 text-muted-foreground text-balance">{description}</p>
      </div>
      <div>{visual}</div>
    </div>
  )
}

function WindowChrome({
  title,
  children,
}: {
  title: string
  children: React.ReactNode
}) {
  return (
    <div className="overflow-hidden rounded-xl border bg-card shadow-2xl shadow-foreground/5 ring-1 ring-foreground/5">
      <div className="flex items-center gap-1.5 border-b bg-muted/40 px-4 py-2.5">
        <span className="size-2.5 rounded-full bg-foreground/15" />
        <span className="size-2.5 rounded-full bg-foreground/15" />
        <span className="size-2.5 rounded-full bg-foreground/15" />
        <span className="ml-3 font-mono text-xs text-muted-foreground">{title}</span>
      </div>
      {children}
    </div>
  )
}

function DataGridMock({ compact }: { compact?: boolean }) {
  const columns = ["id", "email", "full_name", "plan", "created_at"]
  const rows = [
    ["1", "alice@example.com", "Alice Nguyen", "pro", "2026-01-14"],
    ["2", "bob@example.com", "Bob Martinez", "free", "2026-01-15"],
    ["3", "carol@example.com", "Carol Singh", "free", "2026-01-18"],
    ["4", "dave@example.com", "Dave Okafor", "pro", "2026-02-02"],
  ]

  return (
    <WindowChrome title="users — devdb">
      <div className="flex">
        {!compact && (
          <div className="hidden w-44 shrink-0 border-r bg-muted/20 p-3 sm:block">
            <p className="mb-2 px-1 text-[11px] font-medium tracking-wide text-muted-foreground uppercase">
              devdb
            </p>
            <div className="flex flex-col gap-0.5">
              {["users", "categories", "products", "orders", "order_items"].map((table, i) => (
                <div
                  key={table}
                  className={cn(
                    "rounded-md px-2 py-1 text-left font-mono text-xs",
                    i === 0 ? "bg-foreground/10 font-medium" : "text-muted-foreground"
                  )}
                >
                  {table}
                </div>
              ))}
            </div>
          </div>
        )}
        <div className="min-w-0 flex-1 overflow-x-auto">
          <table className="w-full border-collapse text-left font-mono text-xs">
            <thead>
              <tr className="border-b bg-muted/20">
                {columns.map((col) => (
                  <th
                    key={col}
                    className="px-3 py-2 font-medium whitespace-nowrap text-muted-foreground"
                  >
                    {col}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {rows.map((row, i) => (
                <tr key={i} className="border-b last:border-0">
                  {row.map((cell, j) => (
                    <td key={j} className="px-3 py-2 whitespace-nowrap text-foreground/80">
                      {cell}
                    </td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </WindowChrome>
  )
}

function ConnectionMock() {
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
            <span className={cn("size-6 rounded-full", engine.color)} />
            <span className="text-[11px] font-medium">{engine.name}</span>
          </div>
        ))}
      </div>
    </WindowChrome>
  )
}

function SchemaMock() {
  return (
    <WindowChrome title="Add column — products">
      <div className="space-y-3 p-4 font-mono text-xs">
        <div className="flex items-center gap-2">
          <span className="w-20 shrink-0 text-muted-foreground">name</span>
          <span className="rounded border bg-muted/40 px-2 py-1">discount_pct</span>
        </div>
        <div className="flex items-center gap-2">
          <span className="w-20 shrink-0 text-muted-foreground">type</span>
          <span className="rounded border bg-muted/40 px-2 py-1">numeric(5,2)</span>
        </div>
        <div className="flex items-center gap-2">
          <span className="w-20 shrink-0 text-muted-foreground">default</span>
          <span className="rounded border bg-muted/40 px-2 py-1">0</span>
        </div>
        <div className="mt-4 rounded-lg border bg-muted/20 p-3">
          <p className="mb-1.5 text-[10px] font-medium tracking-wide text-muted-foreground uppercase">
            Preview
          </p>
          <code className="text-foreground/80">
            alter table products
            <br />
            &nbsp;&nbsp;add column discount_pct numeric(5,2) not null default 0;
          </code>
        </div>
      </div>
    </WindowChrome>
  )
}

function SqlEditorMock() {
  return (
    <WindowChrome title="query.sql">
      <div className="p-4 font-mono text-xs leading-relaxed">
        <div>
          <span className="text-sky-600 dark:text-sky-400">select</span>{" "}
          <span className="text-foreground/80">o.id, u.full_name, o.total_cents</span>
        </div>
        <div>
          <span className="text-sky-600 dark:text-sky-400">from</span>{" "}
          <span className="text-foreground/80">orders o</span>
        </div>
        <div>
          <span className="text-sky-600 dark:text-sky-400">join</span>{" "}
          <span className="text-foreground/80">users u </span>
          <span className="text-sky-600 dark:text-sky-400">on</span>{" "}
          <span className="text-foreground/80">u.id = o.user_id</span>
        </div>
        <div>
          <span className="text-sky-600 dark:text-sky-400">where</span>{" "}
          <span className="text-foreground/80">o.status = </span>
          <span className="text-emerald-600 dark:text-emerald-400">&apos;paid&apos;</span>
        </div>
        <div className="mt-3 flex items-center gap-2 text-muted-foreground">
          <span className="size-1.5 rounded-full bg-emerald-500" />4 rows · 12ms
        </div>
      </div>
    </WindowChrome>
  )
}
