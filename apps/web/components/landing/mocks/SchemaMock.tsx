import { WindowChrome } from "./WindowChrome"

export function SchemaMock() {
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
