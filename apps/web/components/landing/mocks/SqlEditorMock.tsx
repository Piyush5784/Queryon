import { WindowChrome } from "./WindowChrome"

export function SqlEditorMock() {
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
          <span className="size-1.5 rounded-lg bg-emerald-500" />4 rows · 12ms
        </div>
      </div>
    </WindowChrome>
  )
}
