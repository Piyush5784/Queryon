import { Badge } from "@queryon/ui/components/badge"

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

export function EngineMarquee() {
  return (
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
  )
}
