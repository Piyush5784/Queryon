import { cn } from "@queryon/ui/lib/utils"

import { ConnectionMock } from "./mocks/ConnectionMock"
import { DataGridMock } from "./mocks/DataGridMock"
import { SchemaMock } from "./mocks/SchemaMock"
import { SqlEditorMock } from "./mocks/SqlEditorMock"

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
        <span className="mb-3 inline-flex size-8 items-center justify-center rounded-lg border font-mono text-sm text-muted-foreground">
          {String(index).padStart(2, "0")}
        </span>
        <h3 className="text-2xl font-medium tracking-tight text-balance">{title}</h3>
        <p className="mt-3 text-muted-foreground text-balance">{description}</p>
      </div>
      <div>{visual}</div>
    </div>
  )
}

export function FeatureDeepDives() {
  return (
    <section id="features" className="mx-auto max-w-6xl px-6">
      <div className="mx-auto mb-20 max-w-2xl text-center">
        <h2 className="text-3xl font-medium tracking-tight text-balance sm:text-4xl">
          Built for people who live in their database
        </h2>
        <p className="mt-3 text-muted-foreground text-balance">
          Not a scaled-down toy client, and not an enterprise tool with a decade of accumulated
          menus — just the parts you actually use, done well.
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
  )
}
