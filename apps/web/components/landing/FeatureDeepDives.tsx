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
    <section id="features" className="mx-auto max-w-7xl px-6">
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
          title="Connect to any database, securely"
          description="SSL encryption, SSH tunnelling, and passwords stored in your OS keychain — never in plain text."
          visual={<ConnectionMock />}
        />
        <FeatureRow
          index={2}
          title="Write SQL"
          description="Syntax highlighting and autocomplete for your tables, so you can work quickly."
          visual={<SqlEditorMock />}
          reverse
        />
        <FeatureRow
          index={3}
          title="Easily view & edit data"
          description="Browse and edit tables in a spreadsheet-like grid. Edit JSON cells with syntax checking, even when stored as TEXT."
          visual={<DataGridMock compact />}
        />
        <FeatureRow
          index={4}
          title="Schema editing with a preview"
          description="Add tables, columns, indexes, and constraints through a visual editor. Review the exact DDL that will run before anything actually executes — no surprises on a live database."
          visual={<SchemaMock />}
          reverse
        />
      </div>
    </section>
  )
}
