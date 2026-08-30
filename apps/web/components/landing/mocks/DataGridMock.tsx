import { cn } from "@queryon/ui/lib/utils"

import { WindowChrome } from "./WindowChrome"

const SAMPLE_ROWS = [
  ["alice@example.com", "Alice Nguyen", "pro", "US", "google", "2026-01-14"],
  ["bob@example.com", "Bob Martinez", "free", "MX", "email", "2026-01-15"],
  ["carol@example.com", "Carol Singh", "free", "IN", "github", "2026-01-18"],
  ["dave@example.com", "Dave Okafor", "pro", "NG", "google", "2026-02-02"],
  ["eve@example.com", "Eve Delacroix", "pro", "FR", "email", "2026-02-09"],
  ["frank@example.com", "Frank Boone", "free", "US", "github", "2026-02-11"],
  ["grace@example.com", "Grace Kim", "pro", "KR", "google", "2026-02-14"],
  ["heidi@example.com", "Heidi Larsen", "free", "DK", "email", "2026-02-20"],
  ["ivan@example.com", "Ivan Petrov", "pro", "RU", "github", "2026-03-01"],
  ["judy@example.com", "Judy Alvarez", "free", "ES", "google", "2026-03-04"],
  ["kenji@example.com", "Kenji Sato", "pro", "JP", "email", "2026-03-09"],
  ["liu@example.com", "Liu Wei", "free", "CN", "github", "2026-03-12"],
]

function buildRows(count: number) {
  return Array.from({ length: count }, (_, i) => {
    const sample = SAMPLE_ROWS[i % SAMPLE_ROWS.length]!
    const [emailName, domain] = sample[0]!.split("@")
    return [
      String(i + 1),
      `${emailName}${i + 1}@${domain}`,
      sample[1]!,
      sample[2]!,
      sample[3]!,
      sample[4]!,
      sample[5]!,
    ]
  })
}

export function DataGridMock({
  compact,
  fixedHeight,
}: {
  compact?: boolean
  fixedHeight?: boolean
}) {
  const columns = ["id", "email", "full_name", "plan", "country", "signed_up_via", "created_at"]
  const rows = buildRows(fixedHeight ? 500 : 12)

  return (
    <WindowChrome title="users — devdb">
      <div className={cn("flex", fixedHeight && "h-[420px] lg:h-[680px]")}>
        {!compact && (
          <div className="hidden w-48 shrink-0 border-r bg-muted/20 p-3 sm:block">
            <p className="mb-2 px-1 text-[11px] font-medium tracking-wide text-muted-foreground uppercase">
              Connections
            </p>
            <div className="mb-3 flex items-center gap-2 rounded-md bg-foreground/10 px-2 py-1.5">
              <span className="size-2 rounded-lg bg-emerald-500" />
              <span className="font-mono text-xs font-medium">devdb</span>
            </div>
            <p className="mb-2 px-1 text-[11px] font-medium tracking-wide text-muted-foreground uppercase">
              Tables
            </p>
            <div className="flex flex-col gap-0.5 pl-1">
              {["users", "categories", "products", "orders", "order_items"].map((table, i) => (
                <div
                  key={table}
                  className={cn(
                    "rounded-md px-2 py-1 text-left font-mono text-xs",
                    i === 0
                      ? "bg-foreground text-background font-medium"
                      : "text-muted-foreground"
                  )}
                >
                  {table}
                </div>
              ))}
            </div>
          </div>
        )}
        <div className="flex min-w-0 flex-1 flex-col">
          <div className={cn("min-w-0 flex-1 overflow-x-auto", fixedHeight && "overflow-y-auto")}>
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
                  <tr key={i} className="border-b last:border-0 hover:bg-muted/20">
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
          <div className="flex shrink-0 items-center justify-between border-t px-3 py-2 text-[11px] text-muted-foreground">
            <span>{rows.length} rows</span>
            <span>Page 1 of 1</span>
          </div>
        </div>
      </div>
    </WindowChrome>
  )
}
